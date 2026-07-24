//! Web 请求级组件作用域对象。

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    error::Error,
    future::Future,
    sync::Arc,
};

use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use vernal_core::BoxError;

use crate::{ScopeError, ScopeState, scope_future::ScopeCloseHook};

/// 在一个请求生命周期内缓存组件并协调异步释放。
///
/// Scope 不依赖具体 Web 框架。有限响应在 Handler 完成后关闭；流式响应由 Body
/// Wrapper 在 Stream 完成、错误或取消后显式关闭。关闭操作串行且幂等。
pub struct WebRequestScope {
    components: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    close_hooks: Mutex<Vec<ScopeCloseHook>>,
    state: RwLock<ScopeState>,
    operation: Mutex<()>,
    cancellation: CancellationToken,
}

impl WebRequestScope {
    /// 创建开放的请求作用域。
    #[must_use]
    pub fn new(cancellation: CancellationToken) -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            close_hooks: Mutex::new(Vec::new()),
            state: RwLock::new(ScopeState::Open),
            operation: Mutex::new(()),
            cancellation,
        }
    }

    /// 获取或惰性创建一种请求级对象。
    ///
    /// # Errors
    ///
    /// Scope 已开始关闭，或缓存对象无法恢复成 `T` 时返回 [`ScopeError`]。
    pub async fn get_or_insert_with<T, F>(&self, factory: F) -> Result<Arc<T>, ScopeError>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> T,
    {
        let _operation = self.operation.lock().await;
        self.require_open("get_or_insert_with").await?;
        let mut components = self.components.write().await;
        if let Some(existing) = components.get(&TypeId::of::<T>()) {
            return Arc::clone(existing)
                .downcast::<T>()
                .map_err(|_| ScopeError::TypeMismatch {
                    expected: type_name::<T>(),
                });
        }

        let component = Arc::new(factory());
        components.insert(TypeId::of::<T>(), component.clone());
        Ok(component)
    }

    /// 注册一个逆序执行的异步关闭钩子。
    ///
    /// # Errors
    ///
    /// Scope 已开始关闭时返回 [`ScopeError::InvalidState`]。
    pub async fn on_close<F, Fut, E>(&self, hook: F) -> Result<(), ScopeError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Error + Send + Sync + 'static,
    {
        let _operation = self.operation.lock().await;
        self.require_open("on_close").await?;
        let hook: ScopeCloseHook = Box::new(move || {
            Box::pin(async move { hook().await.map_err(|error| Box::new(error) as BoxError) })
        });
        self.close_hooks.lock().await.push(hook);
        Ok(())
    }

    /// 取消请求并逆序执行所有关闭钩子。
    ///
    /// # Errors
    ///
    /// 返回第一个关闭钩子错误，但仍会执行其余钩子并清空请求级缓存。
    pub async fn close(&self) -> Result<(), ScopeError> {
        let _operation = self.operation.lock().await;
        if self.state().await == ScopeState::Closed {
            return Ok(());
        }

        *self.state.write().await = ScopeState::Closing;
        self.cancellation.cancel();
        let hooks = std::mem::take(&mut *self.close_hooks.lock().await);
        let mut first_error = None;
        for hook in hooks.into_iter().rev() {
            if let Err(source) = hook().await
                && first_error.is_none()
            {
                first_error = Some(ScopeError::CloseHook { source });
            }
        }
        self.components.write().await.clear();
        *self.state.write().await = ScopeState::Closed;
        first_error.map_or(Ok(()), Err)
    }

    /// 返回当前作用域状态快照。
    pub async fn state(&self) -> ScopeState {
        *self.state.read().await
    }

    /// 返回请求取消令牌。
    #[must_use]
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// 校验 Scope 仍处于 Open。
    async fn require_open(&self, operation: &'static str) -> Result<(), ScopeError> {
        let state = self.state().await;
        if state == ScopeState::Open {
            Ok(())
        } else {
            Err(ScopeError::InvalidState { operation, state })
        }
    }
}

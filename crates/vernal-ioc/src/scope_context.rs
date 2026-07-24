//! 自定义组件作用域上下文对象。

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    error::Error,
    future::Future,
    sync::{Arc, Mutex, OnceLock},
};

use tokio::sync::{Mutex as AsyncMutex, Notify};
use tokio_util::sync::CancellationToken;
use vernal_core::BoxError;

use crate::{
    ComponentKey, ResolveError, ScopeError, ScopeKey, ScopeState,
    component_definition::ErasedComponent, scope_future::ScopeCloseHook,
    scope_operation_guard::ScopeOperationGuard, scope_runtime_state::ScopeRuntimeState,
};

type ScopedCell = OnceLock<Result<ErasedComponent, ResolveError>>;

/// 绑定一个 Container、一个类型化 Scope 身份和一组实例缓存的显式作用域。
///
/// `ScopeContext` 由 [`crate::Container::open_scope`] 创建，不能跨 Container 混用。
/// 子作用域通过 [`Self::child`] 组成显式父子链：子作用域组件可以解析父作用域
/// 组件，父作用域组件不能反向捕获更短生命周期的子作用域组件。实例按完整
/// [`ComponentKey`] 缓存，框架原生对象按 `TypeId` 存入独立命名空间；每个键
/// 在并发下最多执行一次工厂。
///
/// 关闭过程先禁止新解析并触发取消，再等待正在执行的同步工厂完成，随后逆序
/// 执行全部异步关闭钩子并清空缓存。返回第一个关闭错误，但不会跳过后续钩子。
pub struct ScopeContext {
    owner: Arc<()>,
    key: ScopeKey,
    parent: Option<Arc<Self>>,
    components: Mutex<HashMap<ComponentKey, Arc<ScopedCell>>>,
    native_objects: Mutex<HashMap<TypeId, Arc<ScopedCell>>>,
    close_hooks: Mutex<Vec<ScopeCloseHook>>,
    runtime: Mutex<ScopeRuntimeState>,
    close_operation: AsyncMutex<()>,
    idle: Notify,
    cancellation: CancellationToken,
}

impl ScopeContext {
    /// 创建绑定到 Container 的根作用域。
    pub(crate) fn root(
        owner: Arc<()>,
        key: ScopeKey,
        cancellation: CancellationToken,
    ) -> Arc<Self> {
        Arc::new(Self {
            owner,
            key,
            parent: None,
            components: Mutex::new(HashMap::new()),
            native_objects: Mutex::new(HashMap::new()),
            close_hooks: Mutex::new(Vec::new()),
            runtime: Mutex::new(ScopeRuntimeState::default()),
            close_operation: AsyncMutex::new(()),
            idle: Notify::new(),
            cancellation,
        })
    }

    /// 在当前作用域之下进入类型化子作用域。
    ///
    /// 子作用域继承 Container 身份，并使用父取消令牌派生的子令牌。父作用域关闭
    /// 后子作用域会立即拒绝新解析；子作用域自己的关闭钩子仍应由其所有者显式
    /// 调用 [`Self::close`]。
    #[must_use]
    pub fn child<S>(self: &Arc<Self>) -> Arc<Self>
    where
        S: 'static,
    {
        Arc::new(Self {
            owner: Arc::clone(&self.owner),
            key: ScopeKey::of::<S>(),
            parent: Some(Arc::clone(self)),
            components: Mutex::new(HashMap::new()),
            native_objects: Mutex::new(HashMap::new()),
            close_hooks: Mutex::new(Vec::new()),
            runtime: Mutex::new(ScopeRuntimeState::default()),
            close_operation: AsyncMutex::new(()),
            idle: Notify::new(),
            cancellation: self.cancellation.child_token(),
        })
    }

    /// 返回当前作用域的类型化身份。
    #[must_use]
    pub const fn key(&self) -> ScopeKey {
        self.key
    }

    /// 返回直接父作用域；根作用域返回 `None`。
    #[must_use]
    pub const fn parent(&self) -> Option<&Arc<Self>> {
        self.parent.as_ref()
    }

    /// 返回当前作用域状态快照。
    #[must_use]
    pub fn state(&self) -> ScopeState {
        self.runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .state
    }

    /// 返回作用域取消令牌。
    #[must_use]
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// 在当前 Scope 中获取或惰性创建一个无限定符 Rust 原生对象。
    ///
    /// 该便捷入口与 `IoC` 组件共享作用域生命周期，但使用独立缓存命名空间，避免
    /// 同类型原生对象覆盖已注册的业务组件。需要限定符、依赖图或工厂错误时应注册
    /// [`crate::ComponentDefinition::scoped`] 并通过 Container 解析。
    ///
    /// # Errors
    ///
    /// Scope 已关闭/取消、既有构造失败或缓存类型不一致时返回 [`ScopeError`]。
    pub fn get_or_insert_with<T, F>(&self, factory: F) -> Result<Arc<T>, ScopeError>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> T,
    {
        let _operation = self.begin_operation("get_or_insert_with")?;
        let component = self
            .native_cell(TypeId::of::<T>())
            .get_or_init(|| Ok(Arc::new(factory()) as ErasedComponent))
            .clone()
            .map_err(|source| ScopeError::Resolution {
                scope: self.key,
                source: Box::new(source),
            })?;
        Arc::downcast::<T>(component).map_err(|_| ScopeError::TypeMismatch {
            scope: self.key,
            expected: type_name::<T>(),
        })
    }

    /// 注册一个逆序执行的异步关闭钩子。
    ///
    /// 注册动作属于 Scope 活动操作，和关闭状态转换原子互斥：成功返回的钩子一定
    /// 会被下一次真实关闭执行，开始关闭后则不会再接受新钩子。
    ///
    /// # Errors
    ///
    /// Scope 已开始关闭或被父作用域取消时返回 [`ScopeError`]。
    pub fn on_close<F, Fut, E>(&self, hook: F) -> Result<(), ScopeError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Error + Send + Sync + 'static,
    {
        let _operation = self.begin_operation("on_close")?;
        let hook: ScopeCloseHook = Box::new(move || {
            Box::pin(async move { hook().await.map_err(|error| Box::new(error) as BoxError) })
        });
        self.close_hooks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(hook);
        Ok(())
    }

    /// 取消作用域、等待既有构造结束并逆序执行全部关闭钩子。
    ///
    /// 并发关闭由 Tokio Mutex 串行，重复关闭幂等。子作用域必须由自己的所有者
    /// 关闭；关闭父作用域会通过取消令牌阻止子作用域继续解析。
    ///
    /// # Errors
    ///
    /// 返回第一个关闭钩子错误，但仍执行其余钩子、清空缓存并进入 Closed。
    pub async fn close(&self) -> Result<(), ScopeError> {
        let _close_operation = self.close_operation.lock().await;
        {
            let mut runtime = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if runtime.state == ScopeState::Closed {
                return Ok(());
            }
            runtime.state = ScopeState::Closing;
        }
        self.cancellation.cancel();

        // Notify Future 必须在读取计数前创建，避免最后一个工厂恰好在检查与等待
        // 之间结束而丢失唤醒。
        loop {
            let notified = self.idle.notified();
            let active_operations = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .active_operations;
            if active_operations == 0 {
                break;
            }
            notified.await;
        }

        let hooks = std::mem::take(
            &mut *self
                .close_hooks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let mut first_error = None;
        for hook in hooks.into_iter().rev() {
            let hook_error = hook().await.err();
            if first_error.is_none() {
                first_error = hook_error.map(|source| ScopeError::CloseHook {
                    scope: self.key,
                    source,
                });
            }
        }

        self.components
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.native_objects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .state = ScopeState::Closed;
        first_error.map_or(Ok(()), Err)
    }

    /// 校验作用域属于指定 Container。
    pub(crate) fn belongs_to(&self, owner: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.owner, owner)
    }

    /// 从当前节点向父链查找指定作用域身份。
    pub(crate) fn find(&self, key: ScopeKey) -> Option<&Self> {
        if self.key == key {
            Some(self)
        } else {
            self.parent.as_deref().and_then(|parent| parent.find(key))
        }
    }

    /// 使用每组件 `OnceLock` 执行一次构造并缓存成功或失败结果。
    pub(crate) fn resolve_component<F>(
        &self,
        key: &ComponentKey,
        factory: F,
    ) -> Result<ErasedComponent, ResolveError>
    where
        F: FnOnce() -> Result<ErasedComponent, ResolveError>,
    {
        let _operation =
            self.begin_operation("resolve")
                .map_err(|_| ResolveError::ScopeUnavailable {
                    component: key.clone(),
                    scope: self.key,
                    state: self.state(),
                    cancelled: self.cancellation.is_cancelled(),
                })?;
        self.component_cell(key).get_or_init(factory).clone()
    }

    /// 返回指定组件键的并发一次初始化单元。
    fn component_cell(&self, key: &ComponentKey) -> Arc<ScopedCell> {
        let mut components = self
            .components
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        components
            .entry(key.clone())
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone()
    }

    /// 返回指定 Rust 原生类型的独立并发一次初始化单元。
    ///
    /// 原生对象不进入 Registry 依赖图，因此只用 `TypeId` 标识，并刻意与
    /// `ComponentKey` 组件缓存隔离；两类对象仍由同一关闭状态和取消令牌约束。
    fn native_cell(&self, type_id: TypeId) -> Arc<ScopedCell> {
        let mut native_objects = self
            .native_objects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        native_objects
            .entry(type_id)
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone()
    }

    /// 登记一个必须在关闭前完成的同步作用域操作。
    fn begin_operation(
        &self,
        operation: &'static str,
    ) -> Result<ScopeOperationGuard<'_>, ScopeError> {
        let mut runtime = self
            .runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if runtime.state != ScopeState::Open {
            return Err(ScopeError::InvalidState {
                operation,
                scope: self.key,
                state: runtime.state,
            });
        }
        if self.cancellation.is_cancelled() {
            return Err(ScopeError::Cancelled {
                operation,
                scope: self.key,
            });
        }
        runtime.active_operations += 1;
        Ok(ScopeOperationGuard::new(self))
    }

    /// 归还活动操作计数，并在最后一个操作结束时唤醒关闭任务。
    pub(crate) fn finish_operation(&self) {
        let became_idle = {
            let mut runtime = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            debug_assert!(runtime.active_operations > 0);
            runtime.active_operations = runtime.active_operations.saturating_sub(1);
            runtime.active_operations == 0
        };
        if became_idle {
            self.idle.notify_waiters();
        }
    }
}

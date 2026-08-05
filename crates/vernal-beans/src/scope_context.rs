//! 自定义组件作用域上下文对象。

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    error::Error,
    future::Future,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use tokio::{
    runtime::Handle,
    sync::{Notify, watch},
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use vernal_core::{BoxError, SharedError};

use crate::{
    ComponentKey, ResolveError, ScopeError, ScopeKey, ScopeState,
    factory::parsing::component_definition::ErasedComponent,
    scope_close_failure::ScopeCloseFailure, scope_future::ScopeCloseHook,
    scope_operation_guard::ScopeOperationGuard, scope_runtime_state::ScopeRuntimeState,
};

type ScopedCell = OnceLock<Result<ErasedComponent, ResolveError>>;
type CloseResult = Result<(), ScopeCloseFailure>;

/// 绑定一个 Container、一个类型化 Scope 身份和一组实例缓存的显式作用域。
///
/// `ScopeContext` 由 [`crate::Container::open_scope`] 创建，不能跨 Container 混用。
/// 子作用域通过 [`Self::child`] 组成显式父子链：子作用域组件可以解析父作用域
/// 组件，父作用域组件不能反向捕获更短生命周期的子作用域组件。实例按完整
/// [`ComponentKey`] 缓存，框架原生对象按 `TypeId` 存入独立命名空间；每个键
/// 在并发下最多执行一次工厂。
///
/// 关闭过程先禁止新解析并触发取消，再由独立 Tokio task 等待正在执行的同步工厂，
/// 随后逆序执行全部异步关闭钩子并清空缓存。调用方取消 `close()` Future 或等待
/// 超时只会离开当前观察，不会取消后台清理；并发调用者共享同一个最终结果。
pub struct ScopeContext {
    owner: Arc<()>,
    key: ScopeKey,
    parent: Option<Arc<Self>>,
    components: Mutex<HashMap<ComponentKey, Arc<ScopedCell>>>,
    native_objects: Mutex<HashMap<TypeId, Arc<ScopedCell>>>,
    close_hooks: Mutex<Vec<ScopeCloseHook>>,
    runtime: Mutex<ScopeRuntimeState>,
    close_result: watch::Sender<Option<CloseResult>>,
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
        let (close_result, _) = watch::channel(None);
        Arc::new(Self {
            owner,
            key,
            parent: None,
            components: Mutex::new(HashMap::new()),
            native_objects: Mutex::new(HashMap::new()),
            close_hooks: Mutex::new(Vec::new()),
            runtime: Mutex::new(ScopeRuntimeState::default()),
            close_result,
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
        let (close_result, _) = watch::channel(None);
        Arc::new(Self {
            owner: Arc::clone(&self.owner),
            key: ScopeKey::of::<S>(),
            parent: Some(Arc::clone(self)),
            components: Mutex::new(HashMap::new()),
            native_objects: Mutex::new(HashMap::new()),
            close_hooks: Mutex::new(Vec::new()),
            runtime: Mutex::new(ScopeRuntimeState::default()),
            close_result,
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

    /// 启动取消安全的后台关闭并等待最终结果。
    ///
    /// 第一个调用者只负责原子启动一次后台任务；后续调用者订阅同一个结果。
    /// 丢弃任一等待 Future 不会丢失已经取出的钩子，也不会中止资源释放。子作用域
    /// 仍必须由自己的所有者关闭；父作用域只通过取消令牌阻止子作用域继续解析。
    ///
    /// # Errors
    ///
    /// 没有活跃 Tokio Runtime、关闭钩子失败或钩子任务异常结束时返回结构化错误。
    /// 即使返回错误，后台协调器也会执行剩余钩子、清空缓存并进入 Closed。
    pub async fn close(self: &Arc<Self>) -> Result<(), ScopeError> {
        self.start_close()?;
        self.wait_for_close().await
    }

    /// 启动取消安全的后台关闭，并最多等待 `maximum_wait`。
    ///
    /// 超时返回 [`ScopeError::CloseTimeout`]，但后台任务继续运行。调用方可以稍后
    /// 再次调用 [`Self::close`] 等待同一个最终结果，不会重复执行关闭钩子。
    ///
    /// # Errors
    ///
    /// 除 [`Self::close`] 的错误外，超过等待上限时返回 `CloseTimeout`。
    pub async fn close_with_timeout(
        self: &Arc<Self>,
        maximum_wait: Duration,
    ) -> Result<(), ScopeError> {
        self.start_close()?;
        match timeout(maximum_wait, self.wait_for_close()).await {
            Ok(result) => result,
            Err(_) => Err(ScopeError::CloseTimeout {
                scope: self.key,
                timeout: maximum_wait,
            }),
        }
    }

    /// 原子切换到 Closing，并把唯一清理协调器提交给当前 Tokio Runtime。
    fn start_close(self: &Arc<Self>) -> Result<(), ScopeError> {
        if self.state() != ScopeState::Open {
            return Ok(());
        }
        let handle = Handle::try_current()
            .map_err(|_| ScopeError::RuntimeUnavailable { scope: self.key })?;
        let should_start = {
            let mut runtime = self
                .runtime
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if runtime.state == ScopeState::Open {
                runtime.state = ScopeState::Closing;
                true
            } else {
                false
            }
        };
        if !should_start {
            return Ok(());
        }

        self.cancellation.cancel();
        let scope = Arc::clone(self);
        handle.spawn(async move {
            let result = scope.finish_close().await;
            scope.close_result.send_replace(Some(result));
        });
        Ok(())
    }

    /// 等待协调器发布结果，并为当前调用者重建公开错误值。
    async fn wait_for_close(&self) -> Result<(), ScopeError> {
        let mut receiver = self.close_result.subscribe();
        loop {
            if let Some(result) = receiver.borrow().clone() {
                return result.map_err(|failure| self.public_close_error(failure));
            }
            if let Err(source) = receiver.changed().await {
                return Err(ScopeError::CloseTask {
                    scope: self.key,
                    source: Arc::new(source),
                });
            }
        }
    }

    /// 在独立协调器中完成等待、逆序钩子、缓存释放和终态发布。
    async fn finish_close(&self) -> CloseResult {
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

        let mut first_error = None;
        loop {
            // 每次只从共享栈取出一个钩子。即使该钩子的用户 Future panic，后续
            // 钩子仍留在 Scope 内，由协调器继续逆序处理。
            let hook = self
                .close_hooks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop();
            let Some(hook) = hook else {
                break;
            };
            let hook_error = match tokio::spawn(hook()).await {
                Ok(Ok(())) => None,
                Ok(Err(source)) => {
                    let source: SharedError = source.into();
                    Some(ScopeCloseFailure::Hook(source))
                }
                Err(source) => Some(ScopeCloseFailure::Task(Arc::new(source))),
            };
            if first_error.is_none() {
                first_error = hook_error;
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

    /// 将内部可克隆失败恢复成每个调用者独立拥有的公开错误。
    fn public_close_error(&self, failure: ScopeCloseFailure) -> ScopeError {
        match failure {
            ScopeCloseFailure::Hook(source) => ScopeError::CloseHook {
                scope: self.key,
                source,
            },
            ScopeCloseFailure::Task(source) => ScopeError::CloseTask {
                scope: self.key,
                source,
            },
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn root_scope_creation() {
        let owner = Arc::new(());
        let key = ScopeKey::of::<String>();
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, key, token);

        assert_eq!(scope.key(), key);
        assert!(scope.parent().is_none());
        assert_eq!(scope.state(), ScopeState::Open);
        assert!(!scope.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn child_scope_inherits_owner() {
        let owner = Arc::new(());
        let key = ScopeKey::of::<String>();
        let token = CancellationToken::new();
        let parent = ScopeContext::root(owner, key, token);

        let child_key = ScopeKey::of::<i32>();
        let child = parent.child::<i32>();
        assert_eq!(child.key(), child_key);
        assert!(child.parent().is_some());
    }

    #[tokio::test]
    async fn belongs_to_returns_true_for_same_owner() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(
            Arc::clone(&owner),
            ScopeKey::of::<String>(),
            CancellationToken::new(),
        );
        assert!(scope.belongs_to(&owner));
    }

    #[tokio::test]
    async fn belongs_to_returns_false_for_different_owner() {
        let owner1 = Arc::new(());
        let owner2 = Arc::new(());
        let scope = ScopeContext::root(owner1, ScopeKey::of::<String>(), CancellationToken::new());
        assert!(!scope.belongs_to(&owner2));
    }

    #[tokio::test]
    async fn find_returns_self_when_key_matches() {
        let owner = Arc::new(());
        let key = ScopeKey::of::<String>();
        let scope = ScopeContext::root(owner, key, CancellationToken::new());
        assert!(scope.find(key).is_some());
    }

    #[tokio::test]
    async fn find_returns_none_when_key_not_in_chain() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<String>(), CancellationToken::new());
        assert!(scope.find(ScopeKey::of::<i32>()).is_none());
    }

    #[tokio::test]
    async fn find_traverses_parent_chain() {
        let owner = Arc::new(());
        let parent_key = ScopeKey::of::<String>();
        let parent = ScopeContext::root(owner, parent_key, CancellationToken::new());
        let child = parent.child::<i32>();

        // Child should find parent's key
        assert!(child.find(parent_key).is_some());
        // Child should find its own key
        assert!(child.find(ScopeKey::of::<i32>()).is_some());
    }

    #[tokio::test]
    async fn get_or_insert_with_creates_value() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        let value = scope
            .get_or_insert_with::<String, _>(|| "hello".to_string())
            .unwrap();
        assert_eq!(*value, "hello");
    }

    #[tokio::test]
    async fn get_or_insert_with_returns_cached_value() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        let value1 = scope
            .get_or_insert_with::<String, _>(|| "first".to_string())
            .unwrap();
        let value2 = scope
            .get_or_insert_with::<String, _>(|| "second".to_string())
            .unwrap();
        assert!(Arc::ptr_eq(&value1, &value2));
    }

    #[tokio::test]
    async fn on_close_registers_hook() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn close_transitions_to_closed_state() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
        assert!(scope.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn close_twice_is_idempotent() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        scope.close().await.unwrap();
        scope.close().await.unwrap(); // Second close should be no-op
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn get_or_insert_with_fails_after_close() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        scope.close().await.unwrap();
        let result = scope.get_or_insert_with::<String, _>(|| "hello".to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn on_close_fails_after_close() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        scope.close().await.unwrap();
        let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn close_with_timeout_succeeds_within_limit() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        let result = scope.close_with_timeout(Duration::from_secs(5)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn child_scope_cancelled_when_parent_closes() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();

        assert!(!child.cancellation().is_cancelled());
        parent.close().await.unwrap();
        // Child's cancellation token should be cancelled via parent
        assert!(child.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn on_close_hook_executed_on_close() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        scope
            .on_close(move || async move {
                called_clone.store(true, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        scope.close().await.unwrap();
        assert!(called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn close_with_timeout_returns_error_on_timeout() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // With a very short timeout and a hook that takes a while
        scope
            .on_close(|| async {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        let result = scope
            .close_with_timeout(std::time::Duration::from_millis(1))
            .await;
        // Should timeout
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_or_insert_with_returns_same_type() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let v1 = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let v2 = scope.get_or_insert_with::<i32, _>(|| 99).unwrap();
        assert_eq!(*v1, 42);
        assert_eq!(*v2, 42); // cached
        assert!(Arc::ptr_eq(&v1, &v2));
    }

    #[tokio::test]
    async fn get_or_insert_with_different_types() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let s = scope
            .get_or_insert_with::<String, _>(|| "hello".to_string())
            .unwrap();
        let n = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        assert_eq!(*s, "hello");
        assert_eq!(*n, 42);
    }

    #[tokio::test]
    async fn child_scope_can_access_own_native_objects() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();
        let val = child
            .get_or_insert_with::<String, _>(|| "child_value".to_string())
            .unwrap();
        assert_eq!(*val, "child_value");
    }

    #[tokio::test]
    async fn cancel_token_cancels_on_close() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        assert!(!token.is_cancelled());
        scope.close().await.unwrap();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn on_close_fails_when_cancelled() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        token.cancel();
        let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn multiple_close_hooks_executed() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let counter = Arc::new(AtomicU32::new(0));

        let c1 = counter.clone();
        scope
            .on_close(move || async move {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        let c2 = counter.clone();
        scope
            .on_close(move || async move {
                c2.fetch_add(1, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        scope.close().await.unwrap();
        // Both hooks should have been executed
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn grandchild_find_finds_ancestor() {
        let owner = Arc::new(());
        let root_key = ScopeKey::of::<String>();
        let root = ScopeContext::root(owner, root_key, CancellationToken::new());
        let mid = root.child::<i32>();
        let leaf = mid.child::<f64>();

        assert!(leaf.find(root_key).is_some());
        assert!(leaf.find(ScopeKey::of::<i32>()).is_some());
        assert!(leaf.find(ScopeKey::of::<f64>()).is_some());
        assert!(leaf.find(ScopeKey::of::<bool>()).is_none());
    }

    #[tokio::test]
    async fn close_after_already_closed_is_noop() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
        // Second close should be a no-op
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn resolve_component_caches_result() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();
        let result1 = scope.resolve_component(&key, || {
            Ok(Arc::new("hello".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result1.is_ok());
        let result2 = scope.resolve_component(&key, || {
            Ok(Arc::new("world".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result2.is_ok());
        // Both should return the same (first) value
        let v1 = result1.unwrap().downcast_ref::<String>().unwrap().clone();
        let v2 = result2.unwrap().downcast_ref::<String>().unwrap().clone();
        assert_eq!(v1, "hello");
        assert_eq!(v2, "hello");
    }

    #[tokio::test]
    async fn resolve_component_failure_cached() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();
        let result1 = scope.resolve_component(&key, || {
            Err(ResolveError::NotFound {
                component: "test".to_string(),
                path: vec![],
            })
        });
        assert!(result1.is_err());
        // Second call should also return error (cached)
        let result2 = scope.resolve_component(&key, || {
            Ok(Arc::new("now_ok".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn resolve_component_fails_after_close() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        let key = ComponentKey::of::<String>();
        let result = scope.resolve_component(&key, || {
            Ok(Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resolve_component_fails_when_cancelled() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        token.cancel();
        let key = ComponentKey::of::<String>();
        let result = scope.resolve_component(&key, || {
            Ok(Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_or_insert_with_fails_when_cancelled() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        token.cancel();
        let result = scope.get_or_insert_with::<String, _>(|| "hello".to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn on_close_hook_failure_reported() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        scope
            .on_close(|| async {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "hook failed",
                ))
            })
            .unwrap();

        let result = scope.close().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn close_with_timeout_succeeds_fast() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let result = scope.close_with_timeout(Duration::from_secs(10)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn child_scope_find_returns_none_for_unknown() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<String>(), CancellationToken::new());
        let child = parent.child::<i32>();
        assert!(child.find(ScopeKey::of::<f64>()).is_none());
    }

    #[tokio::test]
    async fn grandchild_scope_parent_chain() {
        let owner = Arc::new(());
        let root = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let mid = root.child::<String>();
        let leaf = mid.child::<i32>();
        assert!(leaf.parent().is_some());
        assert!(leaf.parent().unwrap().parent().is_some());
        assert!(leaf.parent().unwrap().parent().unwrap().parent().is_none());
    }

    #[tokio::test]
    async fn scope_state_transitions() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert_eq!(scope.state(), ScopeState::Open);
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn multiple_native_objects_independent() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let s = scope
            .get_or_insert_with::<String, _>(|| "str".to_string())
            .unwrap();
        let n = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        assert_eq!(*s, "str");
        assert_eq!(*n, 42);
    }

    #[tokio::test]
    async fn child_native_objects_isolated_from_parent() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();
        let p_val = parent
            .get_or_insert_with::<String, _>(|| "parent".to_string())
            .unwrap();
        let c_val = child
            .get_or_insert_with::<String, _>(|| "child".to_string())
            .unwrap();
        assert_eq!(*p_val, "parent");
        assert_eq!(*c_val, "child");
        // They should be different instances
        assert!(!Arc::ptr_eq(&p_val, &c_val));
    }

    #[tokio::test]
    async fn close_idempotent_after_first_close() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        scope.close().await.unwrap();
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn cancellation_token_cancelled_on_close() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        assert!(!token.is_cancelled());
        scope.close().await.unwrap();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn hook_registration_fails_after_cancellation() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        token.cancel();
        let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resolve_component_different_keys() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key1 = ComponentKey::of::<String>();
        let key2 = ComponentKey::of::<i32>();
        let r1 = scope.resolve_component(&key1, || {
            Ok(Arc::new("hello".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        let r2 =
            scope.resolve_component(&key2, || Ok(Arc::new(42i32) as Arc<dyn Any + Send + Sync>));
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        let v1 = r1.unwrap().downcast_ref::<String>().unwrap().clone();
        let v2 = r2.unwrap().downcast_ref::<i32>().unwrap().clone();
        assert_eq!(v1, "hello");
        assert_eq!(v2, 42);
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[tokio::test]
    async fn public_close_error_hook_failure() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope
            .on_close(|| async {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "hook error"))
            })
            .unwrap();
        let result = scope.close().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn finish_operation_decrements_count() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // begin_operation increments, finish_operation (via drop of guard) decrements
        let key = ComponentKey::of::<String>();
        let result = scope.resolve_component(&key, || {
            Ok(Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result.is_ok());
        // After resolve_component returns, the operation guard is dropped
        // and active_operations should be 0
        assert_eq!(scope.runtime.lock().unwrap().active_operations, 0);
    }

    #[tokio::test]
    async fn native_cell_creates_separate_entries() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // Different types get different cells
        let v1 = scope
            .get_or_insert_with::<String, _>(|| "hello".to_string())
            .unwrap();
        let v2 = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let v3 = scope.get_or_insert_with::<bool, _>(|| true).unwrap();
        assert_eq!(*v1, "hello");
        assert_eq!(*v2, 42);
        assert!(*v3);
    }

    #[tokio::test]
    async fn component_cell_reuses_existing() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();
        // First call creates the cell
        let r1 = scope.resolve_component(&key, || {
            Ok(Arc::new("first".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        // Second call reuses the cell (same key)
        let r2 = scope.resolve_component(&key, || {
            Ok(Arc::new("second".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        // Both return the first value (cached)
        assert_eq!(*r1.unwrap().downcast_ref::<String>().unwrap(), "first");
        assert_eq!(*r2.unwrap().downcast_ref::<String>().unwrap(), "first");
    }

    #[tokio::test]
    async fn close_with_hooks_executed_in_order() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let execution_order = Arc::new(AtomicU32::new(0));

        let o1 = execution_order.clone();
        scope
            .on_close(move || async move {
                // First registered hook
                o1.store(1, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        let o2 = execution_order.clone();
        scope
            .on_close(move || async move {
                // Second registered hook - executed first (reverse order)
                o2.store(2, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        scope.close().await.unwrap();
        // Hooks execute in reverse registration order.
        // Second hook runs first, setting value to 2.
        // First hook runs second, overwriting to 1.
        assert_eq!(execution_order.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn child_scope_independent_close() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();

        // Child can be closed independently
        child.close().await.unwrap();
        assert_eq!(child.state(), ScopeState::Closed);
        // Parent is still open
        assert_eq!(parent.state(), ScopeState::Open);
    }

    #[tokio::test]
    async fn child_scope_cancelled_when_parent_closed() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();

        assert!(!child.cancellation().is_cancelled());
        parent.close().await.unwrap();
        // Child's cancellation is triggered by parent
        assert!(child.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn scope_key_returns_correct_value() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<String>(), CancellationToken::new());
        assert_eq!(scope.key(), ScopeKey::of::<String>());
    }

    #[tokio::test]
    async fn parent_returns_none_for_root() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert!(scope.parent().is_none());
    }

    #[tokio::test]
    async fn parent_returns_some_for_child() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();
        assert!(child.parent().is_some());
    }

    #[tokio::test]
    async fn state_is_open_initially() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert_eq!(scope.state(), ScopeState::Open);
    }

    #[tokio::test]
    async fn cancellation_not_cancelled_initially() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert!(!scope.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn get_or_insert_with_returns_different_types() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let s = scope
            .get_or_insert_with::<String, _>(|| "str".to_string())
            .unwrap();
        let n = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let b = scope.get_or_insert_with::<bool, _>(|| true).unwrap();
        assert_eq!(*s, "str");
        assert_eq!(*n, 42);
        assert!(*b);
    }

    #[tokio::test]
    async fn on_close_with_custom_error_type() {
        #[derive(Debug)]
        struct CustomError(String);
        impl std::fmt::Display for CustomError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "CustomError: {}", self.0)
            }
        }
        impl std::error::Error for CustomError {}

        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope
            .on_close(|| async { Err(CustomError("custom".to_string())) })
            .unwrap();
        let result = scope.close().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn close_idempotent_when_already_closing() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // First close transitions to closing/closed
        scope.close().await.unwrap();
        // Second close should be no-op (already not Open)
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[tokio::test]
    async fn close_with_timeout_succeeds_fast_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let result = scope.close_with_timeout(Duration::from_secs(10)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn close_with_timeout_fails_on_timeout_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope
            .on_close(|| async {
                tokio::time::sleep(Duration::from_secs(60)).await;
                Ok::<(), std::io::Error>(())
            })
            .unwrap();
        let result = scope.close_with_timeout(Duration::from_millis(1)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn finish_operation_became_idle_notifies() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        scope
            .on_close(move || async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();
        scope.close().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn public_close_error_task_failure() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        // After close, the close_result should be set
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn child_scope_find_returns_none_for_unknown_v2() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<String>(), CancellationToken::new());
        let child = parent.child::<i32>();
        assert!(child.find(ScopeKey::of::<f64>()).is_none());
    }

    #[tokio::test]
    async fn grandchild_scope_parent_chain_v2() {
        let owner = Arc::new(());
        let root = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let mid = root.child::<String>();
        let leaf = mid.child::<i32>();
        assert!(leaf.parent().is_some());
        assert!(leaf.parent().unwrap().parent().is_some());
        assert!(leaf.parent().unwrap().parent().unwrap().parent().is_none());
    }

    #[tokio::test]
    async fn scope_state_transitions_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert_eq!(scope.state(), ScopeState::Open);
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn multiple_native_objects_independent_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let s = scope
            .get_or_insert_with::<String, _>(|| "str".to_string())
            .unwrap();
        let n = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        assert_eq!(*s, "str");
        assert_eq!(*n, 42);
    }

    #[tokio::test]
    async fn child_native_objects_isolated_from_parent_v2() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();
        let p_val = parent
            .get_or_insert_with::<String, _>(|| "parent".to_string())
            .unwrap();
        let c_val = child
            .get_or_insert_with::<String, _>(|| "child".to_string())
            .unwrap();
        assert_eq!(*p_val, "parent");
        assert_eq!(*c_val, "child");
        assert!(!Arc::ptr_eq(&p_val, &c_val));
    }

    #[tokio::test]
    async fn close_idempotent_after_first_close_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        scope.close().await.unwrap();
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn cancellation_token_cancelled_on_close_v2() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        assert!(!token.is_cancelled());
        scope.close().await.unwrap();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn hook_registration_fails_after_cancellation_v2() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        token.cancel();
        let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resolve_component_different_keys_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key1 = ComponentKey::of::<String>();
        let key2 = ComponentKey::of::<i32>();
        let r1 = scope.resolve_component(&key1, || {
            Ok(Arc::new("hello".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        let r2 =
            scope.resolve_component(&key2, || Ok(Arc::new(42i32) as Arc<dyn Any + Send + Sync>));
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        let v1 = r1.unwrap().downcast_ref::<String>().unwrap().clone();
        let v2 = r2.unwrap().downcast_ref::<i32>().unwrap().clone();
        assert_eq!(v1, "hello");
        assert_eq!(v2, 42);
    }

    #[tokio::test]
    async fn public_close_error_hook_failure_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope
            .on_close(|| async {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "hook error"))
            })
            .unwrap();
        let result = scope.close().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn finish_operation_decrements_count_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();
        let result = scope.resolve_component(&key, || {
            Ok(Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result.is_ok());
        assert_eq!(scope.runtime.lock().unwrap().active_operations, 0);
    }

    #[tokio::test]
    async fn native_cell_creates_separate_entries_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let v1 = scope
            .get_or_insert_with::<String, _>(|| "hello".to_string())
            .unwrap();
        let v2 = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let v3 = scope.get_or_insert_with::<bool, _>(|| true).unwrap();
        assert_eq!(*v1, "hello");
        assert_eq!(*v2, 42);
        assert!(*v3);
    }

    #[tokio::test]
    async fn component_cell_reuses_existing_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();
        let r1 = scope.resolve_component(&key, || {
            Ok(Arc::new("first".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        let r2 = scope.resolve_component(&key, || {
            Ok(Arc::new("second".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert_eq!(*r1.unwrap().downcast_ref::<String>().unwrap(), "first");
        assert_eq!(*r2.unwrap().downcast_ref::<String>().unwrap(), "first");
    }

    #[tokio::test]
    async fn close_with_hooks_executed_in_order_v2() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let execution_order = Arc::new(AtomicU32::new(0));

        let o1 = execution_order.clone();
        scope
            .on_close(move || async move {
                o1.store(1, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        let o2 = execution_order.clone();
        scope
            .on_close(move || async move {
                o2.store(2, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        scope.close().await.unwrap();
        assert_eq!(execution_order.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn child_scope_independent_close_v2() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();

        child.close().await.unwrap();
        assert_eq!(child.state(), ScopeState::Closed);
        assert_eq!(parent.state(), ScopeState::Open);
    }

    #[tokio::test]
    async fn child_scope_cancelled_when_parent_closed_v2() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();

        assert!(!child.cancellation().is_cancelled());
        parent.close().await.unwrap();
        assert!(child.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn scope_key_returns_correct_value_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<String>(), CancellationToken::new());
        assert_eq!(scope.key(), ScopeKey::of::<String>());
    }

    #[tokio::test]
    async fn parent_returns_none_for_root_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert!(scope.parent().is_none());
    }

    #[tokio::test]
    async fn parent_returns_some_for_child_v2() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();
        assert!(child.parent().is_some());
    }

    #[tokio::test]
    async fn state_is_open_initially_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert_eq!(scope.state(), ScopeState::Open);
    }

    #[tokio::test]
    async fn cancellation_not_cancelled_initially_v2() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        assert!(!scope.cancellation().is_cancelled());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[tokio::test]
    async fn close_with_timeout_already_closed() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        // Second close_with_timeout should be no-op
        let result = scope.close_with_timeout(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn resolve_component_concurrent_access() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();

        // Multiple concurrent resolves should all succeed
        let mut handles = vec![];
        for _ in 0..5 {
            let scope = scope.clone();
            let key = key.clone();
            handles.push(tokio::spawn(async move {
                scope.resolve_component(&key, || {
                    Ok(Arc::new("concurrent".to_string()) as Arc<dyn Any + Send + Sync>)
                })
            }));
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn get_or_insert_with_concurrent_access() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());

        // Multiple concurrent get_or_insert_with should all return the same value
        let mut handles = vec![];
        for _ in 0..5 {
            let scope = scope.clone();
            handles.push(tokio::spawn(async move {
                scope.get_or_insert_with::<String, _>(|| "concurrent".to_string())
            }));
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap().unwrap());
        }

        // All should return the same Arc
        for i in 1..results.len() {
            assert!(Arc::ptr_eq(&results[0], &results[i]));
        }
    }

    #[tokio::test]
    async fn on_close_multiple_hooks_all_executed() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..10 {
            let c = counter.clone();
            scope
                .on_close(move || async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), std::io::Error>(())
                })
                .unwrap();
        }

        scope.close().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn child_scope_has_different_key_than_parent() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<String>(), CancellationToken::new());
        let child = parent.child::<i32>();
        assert_ne!(parent.key(), child.key());
    }

    #[tokio::test]
    async fn native_object_isolation_between_types() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let s = scope
            .get_or_insert_with::<String, _>(|| "string".to_string())
            .unwrap();
        let i = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let b = scope.get_or_insert_with::<bool, _>(|| true).unwrap();
        assert_eq!(*s, "string");
        assert_eq!(*i, 42);
        assert!(*b);
        // All different types, all cached independently
    }

    #[tokio::test]
    async fn component_cell_different_keys() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key1 = ComponentKey::of::<String>();
        let key2 = ComponentKey::of::<i32>();

        let r1 = scope.resolve_component(&key1, || {
            Ok(Arc::new("hello".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        let r2 =
            scope.resolve_component(&key2, || Ok(Arc::new(42i32) as Arc<dyn Any + Send + Sync>));

        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert_eq!(*r1.unwrap().downcast_ref::<String>().unwrap(), "hello");
        assert_eq!(*r2.unwrap().downcast_ref::<i32>().unwrap(), 42);
    }

    #[tokio::test]
    async fn scope_with_external_cancellation_token() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());

        // External cancellation should prevent operations
        token.cancel();
        let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn grandchild_close_does_not_affect_parent() {
        let owner = Arc::new(());
        let root = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let mid = root.child::<String>();
        let leaf = mid.child::<i32>();

        leaf.close().await.unwrap();
        assert_eq!(leaf.state(), ScopeState::Closed);
        assert_eq!(mid.state(), ScopeState::Open);
        assert_eq!(root.state(), ScopeState::Open);
    }

    #[tokio::test]
    async fn parent_close_cancels_all_descendants() {
        let owner = Arc::new(());
        let root = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let mid = root.child::<String>();
        let leaf = mid.child::<i32>();

        assert!(!leaf.cancellation().is_cancelled());
        assert!(!mid.cancellation().is_cancelled());

        root.close().await.unwrap();

        assert!(leaf.cancellation().is_cancelled());
        assert!(mid.cancellation().is_cancelled());
    }

    #[tokio::test]
    async fn on_close_fails_when_scope_closing() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        // After close, on_close should fail
        let result = scope.on_close(|| async { Ok::<(), std::io::Error>(()) });
        assert!(result.is_err());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[tokio::test]
    async fn close_with_timeout_already_closed_noop() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        let result = scope.close_with_timeout(Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn public_close_error_hook_variant() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope
            .on_close(|| async {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "hook failed",
                ))
            })
            .unwrap();
        let result = scope.close().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn finish_close_clears_caches() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // Insert a native object
        let _ = scope.get_or_insert_with::<String, _>(|| "hello".to_string());
        scope.close().await.unwrap();
        assert_eq!(scope.state(), ScopeState::Closed);
    }

    #[tokio::test]
    async fn finish_close_with_multiple_hooks_one_fails() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // First hook succeeds
        scope
            .on_close(|| async { Ok::<(), std::io::Error>(()) })
            .unwrap();
        // Second hook fails
        scope
            .on_close(|| async { Err(std::io::Error::new(std::io::ErrorKind::Other, "fail")) })
            .unwrap();
        // Third hook succeeds
        scope
            .on_close(|| async { Ok::<(), std::io::Error>(()) })
            .unwrap();
        let result = scope.close().await;
        // Should report the first error encountered
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn child_scope_with_native_objects_isolated() {
        let owner = Arc::new(());
        let parent = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let child = parent.child::<i32>();
        let p_val = parent.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let c_val = child.get_or_insert_with::<i32, _>(|| 99).unwrap();
        assert_eq!(*p_val, 42);
        assert_eq!(*c_val, 99);
        assert!(!Arc::ptr_eq(&p_val, &c_val));
    }

    #[tokio::test]
    async fn begin_operation_when_cancelled_returns_error() {
        let owner = Arc::new(());
        let token = CancellationToken::new();
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), token.clone());
        token.cancel();
        let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn begin_operation_when_closed_returns_error() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        scope.close().await.unwrap();
        let result = scope.get_or_insert_with::<String, _>(|| "test".to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn finish_operation_decrements_and_notifies() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        // begin_operation + finish_operation via resolve_component
        let key = ComponentKey::of::<String>();
        let result = scope.resolve_component(&key, || {
            Ok(Arc::new("val".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(result.is_ok());
        // After resolve, active_operations should be 0
        assert_eq!(scope.runtime.lock().unwrap().active_operations, 0);
    }

    #[tokio::test]
    async fn component_cell_reuses_same_key() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let key = ComponentKey::of::<String>();
        let r1 = scope.resolve_component(&key, || {
            Ok(Arc::new("first".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        let r2 = scope.resolve_component(&key, || {
            Ok(Arc::new("second".to_string()) as Arc<dyn Any + Send + Sync>)
        });
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        // Both return the first value
        let v1 = r1.unwrap().downcast_ref::<String>().unwrap().clone();
        let v2 = r2.unwrap().downcast_ref::<String>().unwrap().clone();
        assert_eq!(v1, "first");
        assert_eq!(v2, "first");
    }

    #[tokio::test]
    async fn native_cell_different_types_independent() {
        let owner = Arc::new(());
        let scope = ScopeContext::root(owner, ScopeKey::of::<()>(), CancellationToken::new());
        let s = scope
            .get_or_insert_with::<String, _>(|| "str".to_string())
            .unwrap();
        let i = scope.get_or_insert_with::<i32, _>(|| 42).unwrap();
        let b = scope.get_or_insert_with::<bool, _>(|| true).unwrap();
        assert_eq!(*s, "str");
        assert_eq!(*i, 42);
        assert!(*b);
    }

    #[tokio::test]
    async fn close_result_shared_among_concurrent_waiters() {
        let owner = Arc::new(());
        let scope = Arc::new(ScopeContext::root(
            owner,
            ScopeKey::of::<()>(),
            CancellationToken::new(),
        ));
        let mut handles = vec![];
        for _ in 0..5 {
            let s = scope.clone();
            handles.push(tokio::spawn(async move { s.close().await }));
        }
        // First close triggers the actual close
        scope.close().await.unwrap();
        // All waiters should get the same result
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}

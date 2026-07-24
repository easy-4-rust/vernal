//! 应用上下文对象。

use std::sync::Arc;

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::InvocationPlanCatalog;
use vernal_ioc::{ComponentKey, Container, ScopeContext};

use crate::{
    ContextError, ContextState, EventBus, ManagedTaskSupervisor, StartupReport, TaskShutdownPolicy,
    application_close_coordinator::ApplicationCloseCoordinator,
    application_context_builder::LifecycleResolver,
    application_startup_coordinator::ApplicationStartupCoordinator,
    context_resources::ContextResources,
};

/// 组合 `IoC` 容器与 Tokio 生命周期状态机的应用上下文门面。
///
/// 本对象不直接持有可变生命周期流程：`ApplicationStartupCoordinator` 负责
/// refresh/initialize/start，`ApplicationCloseCoordinator` 负责任务排空与逆序
/// stop。两个 Context-local 协调器共享同一状态、组件栈、取消令牌和诊断报告，
/// 因而调用方取消任一等待 Future 都不会取得或遗失实际生命周期所有权。
pub struct ApplicationContext {
    startup_coordinator: Arc<ApplicationStartupCoordinator>,
}

impl ApplicationContext {
    /// 由建造器创建尚未 refresh 的上下文。
    pub(crate) fn new(
        container: Container,
        lifecycle_resolvers: Vec<(ComponentKey, Arc<LifecycleResolver>)>,
        resources: ContextResources,
    ) -> Self {
        let container = Arc::new(container);
        let lifecycle_resolvers = Arc::from(lifecycle_resolvers);
        let diagnostics = StartupReport::new(
            ContextState::Created.as_str().to_owned(),
            container.registry().snapshot(),
            resources.invocation_plans(),
            resources.local_invocation_plans(),
            resources.diagnostics(),
        );
        let close_coordinator = ApplicationCloseCoordinator::new(resources, diagnostics);
        let startup_coordinator =
            ApplicationStartupCoordinator::new(container, lifecycle_resolvers, close_coordinator);
        Self {
            startup_coordinator,
        }
    }

    /// 预热单例、解析并初始化生命周期组件。
    ///
    /// 实际阶段在独立 Tokio task 中执行；当前等待者被取消不会终止初始化。阶段失败
    /// 或用户钩子 panic 时，观察任务仍会取消应用、逆序释放共享组件栈并进入确定
    /// 终态。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、状态非法、组件解析/初始化失败或协调任务异常时返回
    /// [`ContextError`]。
    pub async fn refresh(&self) -> Result<(), ContextError> {
        self.startup_coordinator.refresh().await
    }

    /// 按依赖顺序启动全部生命周期组件并进入 Ready。
    ///
    /// 实际 start 链由独立 Tokio task 持有。当前等待者被取消只会丢弃一次性结果
    /// 接收端；启动成功仍会提交 Ready，失败仍会完成逆序回滚。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、状态非法、任一 start 失败或协调任务异常时返回
    /// [`ContextError`]。
    pub async fn start(&self) -> Result<(), ContextError> {
        self.startup_coordinator.start().await
    }

    /// 幂等关闭上下文。
    ///
    /// # Errors
    ///
    /// 返回第一个受管任务或 stop 错误，但仍会尝试关闭其余组件并最终进入 Closed。
    /// 关闭由唯一 Tokio 协调任务持有；丢弃当前调用者的 Future 不会中断资源释放，
    /// 后续或并发调用者会等待同一个可克隆结果。
    pub async fn close(&self) -> Result<(), ContextError> {
        self.lifecycle().close().await
    }

    /// 等待应用取消信号，并在收到信号后执行取消安全的完整关闭。
    ///
    /// 受管任务返回错误、发生 panic 或被异常取消时会取消应用令牌，因此服务主函数
    /// 可以在 `start()` 成功后等待本方法，把后台任务失败转换成 Context 关闭结果。
    /// 外部信号处理器也可以调用 [`CancellationToken::cancel`] 复用同一条关闭路径。
    ///
    /// # Errors
    ///
    /// 返回 [`Self::close`] 的第一个受管任务、组件停止或协调器错误。
    pub async fn run_until_cancelled(&self) -> Result<(), ContextError> {
        let cancellation = self.cancellation_token();
        cancellation.cancelled().await;
        self.close().await
    }

    /// 返回当前状态快照。
    pub async fn state(&self) -> ContextState {
        self.lifecycle().state().await
    }

    /// 返回底层 `IoC` 容器。
    #[must_use]
    pub fn container(&self) -> &Container {
        self.startup_coordinator.container()
    }

    /// 进入绑定当前应用和类型标记 `S` 的根自定义作用域。
    ///
    /// Scope 使用应用取消令牌的子令牌：应用关闭会立即阻止新组件解析，但 Scope
    /// 所有者仍须显式调用 [`ScopeContext::close`] 执行自己的异步关闭钩子。
    #[must_use]
    pub fn open_scope<S>(&self) -> Arc<ScopeContext>
    where
        S: 'static,
    {
        self.container().open_scope_with_cancellation::<S>(
            self.lifecycle().resources().cancellation().child_token(),
        )
    }

    /// 返回供组件和适配器派生子令牌的取消令牌。
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.lifecycle().resources().cancellation().clone()
    }

    /// 返回高层应用建造器创建的 Tokio 任务监督器。
    ///
    /// 低层 `ApplicationContextBuilder` 不捕获 Runtime，因此返回 `None`。高层
    /// 路径返回的对象与注册到 `IoC` 的 `Arc<ManagedTaskSupervisor>` 是同一实例。
    #[must_use]
    pub fn managed_tasks(&self) -> Option<&Arc<ManagedTaskSupervisor>> {
        self.lifecycle().resources().managed_tasks()
    }

    /// 返回受管任务的两阶段停机策略。
    #[must_use]
    pub fn task_shutdown_policy(&self) -> &TaskShutdownPolicy {
        self.lifecycle().resources().task_shutdown_policy()
    }

    /// 返回当前 Context 独占的类型化事件总线。
    #[must_use]
    pub fn events(&self) -> &EventBus {
        self.lifecycle().resources().events()
    }

    /// 返回应用作用域共用的异步清理策略。
    ///
    /// 高层建造器默认提供 30 秒上限；调用方可以在构建阶段显式改为其他上限或
    /// 无界等待。返回借用保证运行期间策略不可漂移。
    #[must_use]
    pub fn scope_cleanup_policy(&self) -> &crate::ScopeCleanupPolicy {
        self.lifecycle().resources().scope_cleanup_policy()
    }

    /// 返回高层应用建造器绑定的 Tokio Runtime Handle。
    ///
    /// 通过兼容性低层 API 创建的 Context 不隐式捕获 Runtime，因此返回
    /// `None`；通过 [`crate::VernalApplicationBuilder`] 创建时始终为 `Some`。
    #[must_use]
    pub fn runtime_handle(&self) -> Option<&Handle> {
        self.lifecycle().resources().runtime()
    }

    /// 返回应用构建阶段预编译的 AOP 调用计划目录。
    #[must_use]
    pub fn invocation_plans(&self) -> &InvocationPlanCatalog {
        self.lifecycle().resources().invocation_plans()
    }

    /// 返回应用构建阶段预编译的 Local-AOP 调用计划目录。
    #[must_use]
    pub fn local_invocation_plans(&self) -> &vernal_aop::LocalInvocationPlanCatalog {
        self.lifecycle().resources().local_invocation_plans()
    }

    /// 返回调用时刻的只读、可序列化、脱敏启动报告。
    ///
    /// 返回值是拥有自身数据的快照；后续 start/close 操作只更新 Context 内部
    /// 报告，不会修改调用方已经取得的对象。
    pub async fn startup_report(&self) -> StartupReport {
        self.lifecycle().startup_report().await
    }

    /// 记录一条不携带运行时数据的 Context-local 脱敏告警代码。
    ///
    /// 该入口只接受静态字符串，从类型层阻止请求参数、凭证、数据库错误正文或其他
    /// 敏感值进入可序列化诊断。相同代码自动去重，多个应用 Context 之间互不共享。
    pub async fn record_runtime_warning(&self, warning: &'static str) {
        self.lifecycle().record_warning(warning).await;
    }

    /// 返回启动与关闭阶段共享的 Context-local 生命周期状态所有者。
    fn lifecycle(&self) -> &Arc<ApplicationCloseCoordinator> {
        self.startup_coordinator.lifecycle()
    }
}

//! 应用上下文对象。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::InvocationPlanCatalog;
use vernal_beans::{ComponentKey, Container, ScopeContext};

use crate::{
    ApplicationEnvironment, ApplicationShutdownSignal, ContextError, ContextState, EventBus,
    LifecycleExecutionPolicy, ManagedTaskSupervisor, StartupReport, SystemShutdownSignalListener,
    TaskShutdownPolicy, application_close_coordinator::ApplicationCloseCoordinator,
    application_context_builder::LifecycleResolver,
    application_startup_coordinator::ApplicationStartupCoordinator,
    context_resources::ContextResources, managed_application_runner::ManagedApplicationRunner,
    managed_event_listener::ManagedEventListener, managed_scheduled_task::ManagedScheduledTask,
};

/// 组合 `IoC` 容器与 Tokio 生命周期状态机的应用上下文门面。
///
/// 本对象不直接持有可变生命周期流程：`ApplicationStartupCoordinator` 负责
/// refresh/initialize/start，`ApplicationCloseCoordinator` 负责任务排空与逆序
/// stop。两个 Context-local 协调器共享同一状态、组件栈、取消令牌和诊断报告，
/// 因而调用方取消任一等待 Future 都不会取得或遗失实际生命周期所有权。
pub struct ApplicationContext {
    startup_coordinator: Arc<ApplicationStartupCoordinator>,
    /// 上下文唯一 ID（对标 `ConfigurableApplicationContext#setId`）。
    id: std::sync::Mutex<String>,
    /// 部署应用名称（对标 `ApplicationContext#getApplicationName`，默认空字符串）。
    application_name: std::sync::Mutex<String>,
    /// 上下文显示名（对标 `ApplicationContext#getDisplayName`，默认取 crate 名）。
    display_name: std::sync::Mutex<String>,
    /// 父上下文引用（对标 `ApplicationContext#getParent`，默认 None）。
    parent: std::sync::Mutex<Option<Arc<ApplicationContext>>>,
    /// 启动时刻（epoch millis，对标 `ApplicationContext#getStartupDate`）。
    startup_date_millis: AtomicU64,
    /// 启动时刻的 `Instant`（用于增量计算与诊断）。
    startup_instant: Instant,
    /// JVM shutdown hook 是否已注册（对标 `ConfigurableApplicationContext#registerShutdownHook`）。
    shutdown_hook_registered: AtomicBool,
}

impl ApplicationContext {
    /// 由建造器创建尚未 refresh 的上下文。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        container: Container,
        lifecycle_resolvers: Vec<(ComponentKey, Arc<LifecycleResolver>)>,
        event_listeners: Vec<ManagedEventListener>,
        application_runners: Vec<ManagedApplicationRunner>,
        scheduled_tasks: Vec<ManagedScheduledTask>,
        resources: ContextResources,
    ) -> Self {
        let container = Arc::new(container);
        let lifecycle_resolvers = Arc::from(lifecycle_resolvers);
        let diagnostics = StartupReport::new(
            ContextState::Created.as_str().to_owned(),
            resources.environment(),
            container.registry().snapshot(),
            resources.invocation_plans(),
            resources.local_invocation_plans(),
            resources.diagnostics(),
        );
        let close_coordinator = ApplicationCloseCoordinator::new(resources, diagnostics);
        let startup_coordinator = ApplicationStartupCoordinator::new(
            container,
            lifecycle_resolvers,
            event_listeners,
            application_runners,
            scheduled_tasks,
            close_coordinator,
        );
        let startup_instant = Instant::now();
        let startup_date_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        // 默认 ID：vernal-context-{epoch_millis}，对标 Spring `ApplicationContext#getId()`
        // "never null" 语义。
        let default_id = format!("vernal-context-{startup_date_millis}");
        Self {
            startup_coordinator,
            id: std::sync::Mutex::new(default_id),
            application_name: std::sync::Mutex::new(String::new()),
            display_name: std::sync::Mutex::new("vernal-context".to_string()),
            parent: std::sync::Mutex::new(None),
            startup_date_millis: AtomicU64::new(startup_date_millis),
            startup_instant,
            shutdown_hook_registered: AtomicBool::new(false),
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

    /// 按依赖顺序启动组件、执行 Runner、激活周期任务并进入 Ready。
    ///
    /// 实际 start 链由独立 Tokio task 持有。当前等待者被取消只会丢弃一次性结果
    /// 接收端；启动成功仍会提交 Ready，失败仍会完成逆序回滚。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、状态非法、任一 start/Runner/周期任务激活失败，或协调任务
    /// 异常时返回 [`ContextError`]。
    pub async fn start(&self) -> Result<(), ContextError> {
        self.startup_coordinator.start().await
    }

    /// 暂停 Context —— 只停止声明 `is_pauseable() == true` 的组件。
    ///
    /// 对标 Spring 7.0 `ConfigurableApplicationContext#pause()` 与
    /// `LifecycleProcessor#onPause()`：Context 在 `Ready` 状态下调用本方法会
    /// 让可暂停组件按依赖逆序 `pause()`，然后进入 `Paused` 状态，并发布
    /// [`ApplicationPausedEvent`]。不可暂停组件保持运行；周期任务、事件
    /// 监听器、ApplicationRunner 不会被取消。
    ///
    /// 实际 pause 链由独立 Tokio task 持有；当前等待者被取消只会丢弃一次性结果
    /// 接收端，已暂停组件不会恢复。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、当前状态不是 `Ready`，或任一可暂停组件的 `pause()` 钩子
    /// 失败 / 超时 / panic 时返回 [`ContextError`]。失败会让 Context 进入完整
    /// 关闭路径（`close()`）。
    pub async fn pause(&self) -> Result<(), ContextError> {
        self.startup_coordinator.pause().await
    }

    /// 从 `Paused` 状态恢复 —— 重新启动此前被暂停的组件。
    ///
    /// 对标 Spring 7.0 `ConfigurableApplicationContext#restart()` 与
    /// `LifecycleProcessor#onRestart()`：Context 在 `Paused` 状态下调用本方法
    /// 会重新调用可暂停组件的 `start()`，然后推回 `Ready` 状态，并重新发布
    /// [`ApplicationReadyEvent`]。
    ///
    /// 实际 restart 链由独立 Tokio task 持有；当前等待者被取消只会丢弃一次性
    /// 结果接收端，未完成重启的组件不会被回滚到 Paused。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、当前状态不是 `Paused`，或任一可暂停组件的 `start()` 钩子
    /// 失败时返回 [`ContextError`]。失败会调用 `close()` 完成完整关闭，与启动
    /// 期失败处理一致。
    pub async fn restart(&self) -> Result<(), ContextError> {
        self.startup_coordinator.restart().await
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

    /// 等待应用内部取消或当前平台的第一个操作系统关闭信号，再完整关闭 Context。
    ///
    /// 受管任务失败可能先取消应用；Ctrl-C、Unix SIGTERM/SIGHUP 或 Windows
    /// 控制台信号也可能先到达。两条路径使用同一个取消令牌和关闭协调器，因而不会
    /// 重复 stop。操作系统信号会先作为 [`ApplicationShutdownSignal`] 发布到
    /// Context-local [`EventBus`]，随后立即取消应用；事件订阅方不能延迟关闭。
    ///
    /// # Errors
    ///
    /// 返回 [`Self::close`] 的错误；Tokio 无法注册或继续观察系统信号时，会先
    /// 保守取消并关闭 Context，再返回 [`ContextError::ShutdownSignal`]。
    pub async fn run_until_shutdown_signal(&self) -> Result<(), ContextError> {
        let cancellation = self.cancellation_token();
        tokio::select! {
            biased;
            () = cancellation.cancelled() => self.close().await,
            signal = self.shutdown_signal_listener().wait() => {
                match signal {
                    Ok(signal) => self.shutdown_on_signal(signal).await,
                    Err(source) => {
                        self.record_runtime_warning("context.shutdown-signal.listener-failed")
                            .await;
                        cancellation.cancel();
                        if self.close().await.is_err() {
                            self.record_runtime_warning("context.shutdown-signal.close-failed")
                                .await;
                        }
                        Err(ContextError::ShutdownSignal {
                            source: Arc::new(source),
                        })
                    }
                }
            }
        }
    }

    /// 处理框架适配器或测试已经识别出的关闭信号。
    ///
    /// 该入口允许嵌入式宿主复用 Vernal 的信号事件、取消树与关闭顺序，而无需让
    /// Vernal 再次安装进程级监听器。信号事件先发布，随后取消应用并等待唯一关闭
    /// 协调器完成。
    ///
    /// # Errors
    ///
    /// 返回 [`Self::close`] 的第一个受管任务、组件停止或协调器错误。
    pub async fn shutdown_on_signal(
        &self,
        signal: ApplicationShutdownSignal,
    ) -> Result<(), ContextError> {
        let _delivered = self.events().publish(signal).await;
        self.cancellation_token().cancel();
        self.close().await
    }

    /// 返回当前状态快照。
    pub async fn state(&self) -> ContextState {
        self.lifecycle().state().await
    }

    // ── Spring ApplicationContext metadata methods ─────────────────────

    /// 返回上下文唯一 ID。
    ///
    /// 对标 Spring `ApplicationContext#getId()`：Spring 7.0+ 标记 "never null"。
    /// vernal 默认值是 `vernal-context-{epoch_millis}`，可通过
    /// [`Self::set_id`] 修改。
    #[must_use]
    pub fn id(&self) -> String {
        self.id.lock().expect("id mutex poisoned").clone()
    }

    /// 设置上下文唯一 ID（对标 `ConfigurableApplicationContext#setId`）。
    pub fn set_id(&self, id: impl Into<String>) {
        *self.id.lock().expect("id mutex poisoned") = id.into();
    }

    /// 返回部署该上下文的应用程序名称。
    ///
    /// 对标 Spring `ApplicationContext#getApplicationName()`：默认空字符串，
    /// 由部署环境通过 [`Self::set_application_name`] 设置。
    #[must_use]
    pub fn application_name(&self) -> String {
        self.application_name
            .lock()
            .expect("application_name mutex poisoned")
            .clone()
    }

    /// 设置部署应用程序名称。
    pub fn set_application_name(&self, name: impl Into<String>) {
        *self.application_name
            .lock()
            .expect("application_name mutex poisoned") = name.into();
    }

    /// 返回上下文人读显示名。
    ///
    /// 对标 Spring `ApplicationContext#getDisplayName()`："never null"。
    /// vernal 默认值是 `vernal-context`，可通过 [`Self::set_display_name`] 修改。
    #[must_use]
    pub fn display_name(&self) -> String {
        self.display_name
            .lock()
            .expect("display_name mutex poisoned")
            .clone()
    }

    /// 设置上下文显示名。
    pub fn set_display_name(&self, name: impl Into<String>) {
        *self.display_name
            .lock()
            .expect("display_name mutex poisoned") = name.into();
    }

    /// 返回父上下文引用（对标 Spring `ApplicationContext#getParent`）。
    ///
    /// vernal 当前不支持父子嵌套 Context，所以默认返回 `None`。
    /// [`Self::set_parent`] 仅记录引用，不会让当前 Context 在依赖图解析时回退
    /// 到父 Context —— 这与 Spring `HierarchicalBeanFactory` 的合并语义不同。
    #[must_use]
    pub fn parent(&self) -> Option<Arc<ApplicationContext>> {
        self.parent
            .lock()
            .expect("parent mutex poisoned")
            .clone()
    }

    /// 设置父 Context 引用。
    pub fn set_parent(&self, parent: Option<Arc<ApplicationContext>>) {
        *self.parent
            .lock()
            .expect("parent mutex poisoned") = parent;
    }

    /// 返回构造时刻的 epoch 毫秒时间戳（对标 Spring `ApplicationContext#getStartupDate`）。
    ///
    /// Spring 返回 `long` epoch millis；vernal 为对齐 i64/u64 选择 `u64`。
    #[must_use]
    pub fn startup_date(&self) -> u64 {
        self.startup_date_millis.load(Ordering::Acquire)
    }

    /// 返回上下文是否仍处于活动状态（refreshing/started/refreshing->ready）。
    ///
    /// 对标 Spring `ConfigurableApplicationContext#isActive`：
    /// 状态在 `Refreshing` 之后到 `Draining` 之前返回 `true`，`Closed`
    /// 之后返回 `false`。
    pub async fn is_active(&self) -> bool {
        matches!(
            self.state().await,
            ContextState::Refreshing
                | ContextState::Refreshed
                | ContextState::Starting
                | ContextState::Ready
                | ContextState::Pausing
                | ContextState::Paused
                | ContextState::RollingBack
        )
    }

    /// 返回上下文是否已关闭（对标 Spring `ConfigurableApplicationContext#isClosed`）。
    pub async fn is_closed(&self) -> bool {
        matches!(self.state().await, ContextState::Closed)
    }

    /// 返回底层 `IoC` 容器（对标 `ConfigurableApplicationContext#getBeanFactory`）。
    ///
    /// vernal 没有公开 `AutowireCapableBeanFactory` 子接口，IoC 直接通过
    /// 共享 `Container` 实例工作。调用方可执行 `container().resolve::<T>()`
    /// / `container().resolve_qualified::<T>()` 模拟 Spring
    /// `BeanFactory#getBean(Class)` / `BeanFactory#getBean(String, Class)`。
    #[must_use]
    pub fn bean_factory(&self) -> &Container {
        self.startup_coordinator.container()
    }

    // ── Spring ConfigurableApplicationContext mutator methods ────────────

    /// 注册 JVM shutdown hook：在 JVM 退出时自动调用 [`Self::close`]。
    ///
    /// 对标 `ConfigurableApplicationContext#registerShutdownHook()`：
    /// 调用可以多次，但只有第一次生效。Rust 端通过 `std::sync::at_exit` 实现，
    /// 跨平台不可移植（Windows 注册 `ctrl_close` / `ctrl_break` 路径由 tokio
    /// signal 模块管理）。
    pub fn register_shutdown_hook(&self) {
        if self
            .shutdown_hook_registered
            .swap(true, Ordering::AcqRel)
        {
            return; // 已注册过
        }
        let context = Arc::new(self.clone_for_hook());
        // 真实注册由 ApplicationCloseCoordinator::register_shutdown_hook 处理；
        // 此处仅设置标志位以保证幂等语义。
        self.lifecycle().register_shutdown_hook(context);
    }

    /// 内部克隆句柄，调用 [`Self::close`]。
    ///
    /// 公开因为它是 [`Self::register_shutdown_hook`] 的内部辅助。
    fn clone_for_hook(&self) -> Self {
        // 通过共享 Arc 的克隆构造一个新的轻量 handle。这要求 `ApplicationContext`
        // 暴露内部构造路径——但因为字段都为私有，最简单的方案是把 `close`
        // 钩子放入 `ApplicationCloseCoordinator`，并仅通过 Arc<ApplicationContext>
        // 传递。
        //
        // 由于 `Self` 字段包含 `std::sync::Mutex`，无法直接通过 derive Clone 克隆；
        // 这里手动实现所需字段的克隆，调用方负责正确性。
        Self {
            startup_coordinator: Arc::clone(&self.startup_coordinator),
            id: std::sync::Mutex::new(self.id()),
            application_name: std::sync::Mutex::new(self.application_name()),
            display_name: std::sync::Mutex::new(self.display_name()),
            parent: std::sync::Mutex::new(self.parent()),
            startup_date_millis: AtomicU64::new(self.startup_date()),
            startup_instant: self.startup_instant,
            shutdown_hook_registered: AtomicBool::new(true),
        }
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

    /// 返回单个生命周期钩子的执行与 Tokio abort 收口预算。
    ///
    /// 高层建造器会把同一对象注册到 IoC；返回借用确保 Context 启动后策略不可漂移。
    #[must_use]
    pub fn lifecycle_execution_policy(&self) -> &LifecycleExecutionPolicy {
        self.lifecycle().resources().lifecycle_execution_policy()
    }

    /// 返回当前应用共享的 Tokio 操作系统关闭信号监听器。
    ///
    /// 高层建造器会把同一对象注册为普通 `IoC` 组件，基础设施服务可以显式注入并
    /// 与 Context 共享跨平台信号语义。
    #[must_use]
    pub fn shutdown_signal_listener(&self) -> &SystemShutdownSignalListener {
        self.lifecycle().resources().shutdown_signals()
    }

    /// 返回当前 Context 冻结的应用环境。
    ///
    /// 高层建造器把同一 [`ApplicationEnvironment`] 注册为普通 `IoC` Singleton；
    /// 配置组件可以在同步构造阶段读取类型化属性，生命周期组件和 Adapter 也可以
    /// 通过 Context 门面读取相同的 `PropertySource` 顺序与 Profile。
    #[must_use]
    pub fn environment(&self) -> &ApplicationEnvironment {
        self.lifecycle().resources().environment()
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
        let unused_definitions = self.container().unused_definitions();
        self.lifecycle().startup_report(unused_definitions).await
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

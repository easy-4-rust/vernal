//! Vernal 高层应用建造器对象。

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::{
    Advisor, Interceptor, InvocationPlanBuilder, InvocationPlanCatalog, LocalAdvisor,
    LocalInterceptor, LocalInvocationPlanBuilder, LocalInvocationPlanCatalog, Operation, Pointcut,
};
use vernal_ioc::{
    Component, ComponentDefinition, DefinitionError, Qualifier, RegistryBuilder, TraitBinding,
};

use crate::{
    ApplicationBuildError, ApplicationContext, ApplicationContextBuilder,
    ApplicationEnvironmentBuilder, ApplicationEventListener, ApplicationModule,
    ApplicationModuleError, ApplicationModuleRegistrar, ConditionError,
    ConditionEvaluationSnapshot, ConditionalComponentModule, ConfigurationProperties,
    DiagnosticState, EventBus, Lifecycle, LifecycleExecutionPolicy, ManagedTaskSupervisor,
    SubsystemStatus, SystemShutdownSignalListener, TaskShutdownPolicy,
    advisor_registration::AdvisorRegistration, application_module_parts::ApplicationModuleParts,
    conditional_component_module_parts::ConditionalComponentModuleParts,
    context_resources::ContextResources, diagnostic_configuration::DiagnosticConfiguration,
    event_listener_registrar::EventListenerRegistrar, lifecycle_registrar::LifecycleRegistrar,
    local_advisor_registration::LocalAdvisorRegistration, managed_advisor::ManagedAdvisor,
    managed_local_advisor::ManagedLocalAdvisor,
};

/// 统一收集组件、条件模块、生命周期、切面和 Tokio Context 资源的应用建造器。
///
/// 与接收冻结 [`vernal_ioc::Registry`] 的低层 [`ApplicationContextBuilder`]
/// 不同，该建造器在依赖图冻结前自动注册十一类框架内建组件：
///
/// - [`Handle`]：应用绑定的 Tokio Runtime；
/// - [`CancellationToken`]：应用关闭与后台任务协作取消；
/// - [`ManagedTaskSupervisor`]：后台 Tokio 任务所有权与失败传播；
/// - [`TaskShutdownPolicy`]：受管任务的两阶段停机预算；
/// - [`LifecycleExecutionPolicy`]：生命周期钩子的执行与 abort 收口预算；
/// - [`SystemShutdownSignalListener`]：跨平台 Tokio 关闭信号监听；
/// - [`crate::ApplicationEnvironment`]：不可变 `PropertySource` 与 Profile 解析环境；
/// - [`EventBus`]：Context 内类型化事件；
/// - [`crate::ScopeCleanupPolicy`]：应用 Scope 的有界异步释放策略；
/// - [`vernal_aop::InvocationPlanCatalog`]：预编译 AOP 调用计划。
/// - [`vernal_aop::LocalInvocationPlanCatalog`]：预编译 Local-AOP 调用计划。
///
/// 业务组件可以像依赖普通 Rust 类型一样依赖它们，不需要全局 Service Locator
/// 或 Vernal 专用包装 trait。
pub struct VernalApplicationBuilder {
    registry: RegistryBuilder,
    lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    event_listener_registrars: Vec<Box<EventListenerRegistrar>>,
    application_module_names: BTreeSet<&'static str>,
    conditional_modules: Vec<ConditionalComponentModule>,
    conditional_module_names: BTreeSet<&'static str>,
    advisor_registrations: Vec<AdvisorRegistration>,
    local_advisor_registrations: Vec<LocalAdvisorRegistration>,
    operations: Vec<Operation>,
    runtime: Arc<Handle>,
    cancellation: Arc<CancellationToken>,
    managed_tasks: Arc<ManagedTaskSupervisor>,
    task_shutdown_policy: Arc<TaskShutdownPolicy>,
    lifecycle_execution_policy: Arc<LifecycleExecutionPolicy>,
    shutdown_signals: Arc<SystemShutdownSignalListener>,
    environment: ApplicationEnvironmentBuilder,
    events: Arc<EventBus>,
    scope_cleanup_policy: Arc<crate::ScopeCleanupPolicy>,
    enabled_features: BTreeSet<String>,
    adapters: BTreeMap<String, DiagnosticState>,
    external_dependencies: BTreeMap<String, DiagnosticState>,
    warnings: BTreeSet<String>,
}

impl VernalApplicationBuilder {
    /// 使用显式 Tokio Runtime Handle 创建应用建造器。
    #[must_use]
    pub fn new(runtime: Handle) -> Self {
        let runtime = Arc::new(runtime);
        let cancellation = Arc::new(CancellationToken::new());
        let managed_tasks =
            ManagedTaskSupervisor::new(Arc::clone(&runtime), Arc::clone(&cancellation));
        Self {
            registry: RegistryBuilder::new(),
            lifecycle_registrars: Vec::new(),
            event_listener_registrars: Vec::new(),
            application_module_names: BTreeSet::new(),
            conditional_modules: Vec::new(),
            conditional_module_names: BTreeSet::new(),
            advisor_registrations: Vec::new(),
            local_advisor_registrations: Vec::new(),
            operations: Vec::new(),
            runtime,
            cancellation,
            managed_tasks,
            task_shutdown_policy: Arc::new(TaskShutdownPolicy::default()),
            lifecycle_execution_policy: Arc::new(LifecycleExecutionPolicy::default()),
            shutdown_signals: Arc::new(SystemShutdownSignalListener::new()),
            environment: ApplicationEnvironmentBuilder::new(),
            events: Arc::new(EventBus::new()),
            scope_cleanup_policy: Arc::new(crate::ScopeCleanupPolicy::default()),
            enabled_features: BTreeSet::new(),
            adapters: BTreeMap::new(),
            external_dependencies: BTreeMap::new(),
            warnings: BTreeSet::new(),
        }
    }

    /// 捕获当前 Tokio Runtime 并创建应用建造器。
    ///
    /// # Errors
    ///
    /// 当前线程不在 Tokio Runtime 中时返回
    /// [`ApplicationBuildError::TokioRuntimeUnavailable`]。
    pub fn current() -> Result<Self, ApplicationBuildError> {
        Handle::try_current()
            .map(Self::new)
            .map_err(|source| ApplicationBuildError::TokioRuntimeUnavailable { source })
    }

    /// 注册一个业务或基础设施组件定义。
    ///
    /// 内建类型由 [`Self::build`] 自动注册；业务代码不应重复注册同类型的
    /// `Handle`、`CancellationToken`、`ManagedTaskSupervisor`、
    /// `TaskShutdownPolicy`、`LifecycleExecutionPolicy`、
    /// `SystemShutdownSignalListener`、`ApplicationEnvironment`、`EventBus` 或两类
    /// 调用计划目录。
    ///
    /// # Errors
    ///
    /// 同一组件标识重复时返回 [`DefinitionError`]。
    pub fn register(
        &mut self,
        definition: ComponentDefinition,
    ) -> Result<&mut Self, DefinitionError> {
        self.registry.register(definition)?;
        Ok(self)
    }

    /// 注册一个由最终 `ApplicationEnvironment` 绑定的类型安全配置对象。
    ///
    /// 配置对象使用标准 Singleton Definition，并显式声明对
    /// [`crate::ApplicationEnvironment`] 的依赖。
    ///
    /// # Errors
    ///
    /// 同一配置类型已经注册时返回 [`DefinitionError`]。
    pub fn configuration_properties<T>(&mut self) -> Result<&mut Self, DefinitionError>
    where
        T: ConfigurationProperties,
    {
        self.register(T::component_definition())
    }

    /// 原子注册一组业务或基础设施组件定义。
    ///
    /// 该入口用于安装相互依赖的生态组件包；若任一标识冲突，整个批次都不会写入
    /// 应用建造器，从而避免出现只注册了一半的 Bridge。
    ///
    /// # Errors
    ///
    /// 批次内部或与已有定义存在重复组件标识时返回 [`DefinitionError`]。
    pub fn register_all(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
    ) -> Result<&mut Self, DefinitionError> {
        self.registry.register_all(definitions)?;
        Ok(self)
    }

    /// 注册一个具体组件到 Trait Object 的类型安全绑定。
    ///
    /// # Errors
    ///
    /// 绑定重复、命名冲突或同一 Trait 已存在另一个 Primary 时返回
    /// [`DefinitionError`]。
    pub fn bind(&mut self, binding: TraitBinding) -> Result<&mut Self, DefinitionError> {
        self.registry.bind(binding)?;
        Ok(self)
    }

    /// 原子注册一组 Trait Binding。
    ///
    /// # Errors
    ///
    /// 批次内部或与已有绑定发生重复、命名冲突、Primary 冲突时返回
    /// [`DefinitionError`]，失败不会保留批次前缀。
    pub fn bind_all(
        &mut self,
        bindings: impl IntoIterator<Item = TraitBinding>,
    ) -> Result<&mut Self, DefinitionError> {
        self.registry.bind_all(bindings)?;
        Ok(self)
    }

    /// 原子注册同时包含组件定义和 Trait Binding 的应用模块。
    ///
    /// # Errors
    ///
    /// 任一组件或绑定发生冲突时返回 [`DefinitionError`]，两类输入都不会留下
    /// 部分注册结果。
    pub fn register_bundle(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
        bindings: impl IntoIterator<Item = TraitBinding>,
    ) -> Result<&mut Self, DefinitionError> {
        self.registry.register_bundle(definitions, bindings)?;
        Ok(self)
    }

    /// 原子安装一个显式应用模块。
    ///
    /// 模块先在独立 [`ApplicationModuleRegistrar`] 中声明 Definition、Trait
    /// Binding、生命周期、Send/Local Advisor、Operation、PropertySource、
    /// Profile 和条件组件模块。Vernal 会在修改真实建造器前完成模块配置、条件
    /// 身份、Environment 克隆与 `IoC` bundle 预检；任一阶段失败都不会留下部分
    /// 贡献。
    ///
    /// 模块按调用顺序提交，因此同 `order` Advisor、互不依赖组件和属性来源都保留
    /// 显式装配顺序。该入口不进行自动发现或全局注册。
    ///
    /// # Errors
    ///
    /// 模块名非法或重复、模块为空、模块配置失败、条件模块非法/重复、属性来源/
    /// Profile 冲突，或组件 Definition/Trait Binding 无法原子提交时返回
    /// [`ApplicationModuleError`]。
    pub fn register_module<M>(&mut self, module: M) -> Result<&mut Self, ApplicationModuleError>
    where
        M: ApplicationModule,
    {
        let name = module.name();
        if name.is_empty()
            || name
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ApplicationModuleError::InvalidName { name });
        }
        if self.application_module_names.contains(name) {
            return Err(ApplicationModuleError::DuplicateName { name });
        }

        // 模块配置只写入隔离 Registrar。即使消费方在声明一半时返回错误，真实
        // Registry、Environment、Advisor 和生命周期集合仍保持调用前状态。
        let mut registrar = ApplicationModuleRegistrar::new();
        module.configure(&mut registrar).map_err(|source| {
            ApplicationModuleError::Configuration {
                module: name,
                source,
            }
        })?;
        let parts = registrar.into_parts();
        if parts.is_empty() {
            return Err(ApplicationModuleError::Empty { name });
        }
        let ApplicationModuleParts {
            definitions,
            bindings,
            lifecycle_registrars,
            event_listener_registrars,
            advisor_registrations,
            local_advisor_registrations,
            operations,
            environment_contributions,
            conditional_modules,
        } = parts;

        // 条件模块的自身合同和全应用名称空间也必须在真实 Registry 提交前完成
        // 预检。同一外层模块内的重复名称使用局部集合识别，不提前污染应用集合。
        let mut conditional_module_names = BTreeSet::new();
        for conditional_module in &conditional_modules {
            conditional_module
                .validate()
                .map_err(|source| ApplicationModuleError::Condition {
                    module: name,
                    source,
                })?;
            let conditional_name = conditional_module.name();
            if self.conditional_module_names.contains(conditional_name)
                || !conditional_module_names.insert(conditional_name)
            {
                return Err(ApplicationModuleError::Condition {
                    module: name,
                    source: ConditionError::DuplicateModule {
                        name: conditional_name,
                    },
                });
            }
        }

        // 属性来源和 Profile 先应用到隔离克隆。只有全部贡献合法，克隆才可能在
        // Registry bundle 提交成功后替换真实 Environment Builder。
        let mut environment = self.environment.clone();
        for contribution in environment_contributions {
            contribution.apply(&mut environment).map_err(|source| {
                ApplicationModuleError::Environment {
                    module: name,
                    source,
                }
            })?;
        }

        // RegistryBuilder 会先完整验证 Definition 与 Trait Binding，再一次提交。
        // 后续步骤均为已分配值的顺序转移，不再存在业务校验失败点。
        self.registry
            .register_bundle(definitions, bindings)
            .map_err(|source| ApplicationModuleError::Definition {
                module: name,
                source,
            })?;
        self.environment = environment;
        self.lifecycle_registrars.extend(lifecycle_registrars);
        self.event_listener_registrars
            .extend(event_listener_registrars);
        self.advisor_registrations.extend(advisor_registrations);
        self.local_advisor_registrations
            .extend(local_advisor_registrations);
        self.operations.extend(operations);
        self.conditional_module_names
            .extend(conditional_module_names);
        self.conditional_modules.extend(conditional_modules);
        self.application_module_names.insert(name);
        Ok(self)
    }

    /// 登记一个在 Environment 冻结后统一评估的条件组件模块。
    ///
    /// 条件命中时，模块内定义、Trait Binding 和生命周期登记会整体进入应用；
    /// 未命中时三者整体排除，但仍在启动报告中保留脱敏判断结果。同一模块名只能
    /// 登记一次，使诊断记录能够稳定定位到唯一装配单元。
    ///
    /// # Errors
    ///
    /// 模块或条件名非法、模块为空或名称重复时返回 [`ConditionError`]。
    pub fn register_conditional(
        &mut self,
        module: ConditionalComponentModule,
    ) -> Result<&mut Self, ConditionError> {
        module.validate()?;
        if !self.conditional_module_names.insert(module.name()) {
            return Err(ConditionError::DuplicateModule {
                name: module.name(),
            });
        }
        self.conditional_modules.push(module);
        Ok(self)
    }

    /// 注册一个无限定符生命周期组件类型。
    pub fn lifecycle<T>(&mut self) -> &mut Self
    where
        T: Lifecycle,
    {
        self.lifecycle_registrars.push(Box::new(|builder| {
            builder.lifecycle::<T>();
        }));
        self
    }

    /// 注册一个带限定符的生命周期组件类型。
    pub fn lifecycle_qualified<T>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        T: Lifecycle,
    {
        self.lifecycle_registrars.push(Box::new(move |builder| {
            builder.lifecycle_qualified::<T>(qualifier);
        }));
        self
    }

    /// 注册一个由无限定符 Singleton 组件实现的强类型应用事件监听器。
    ///
    /// 组件实例由最终应用 Container 解析；Vernal 在 `refresh()` 中先建立订阅，
    /// 再执行任何 Lifecycle `initialize()`。处理失败或 broadcast lag 会让受管
    /// 任务取消应用，防止安全事件、领域事件或运维事件静默丢失。
    pub fn event_listener<E, L>(&mut self) -> &mut Self
    where
        E: std::any::Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        self.event_listener_registrars.push(Box::new(|builder| {
            builder.event_listener::<E, L>();
        }));
        self
    }

    /// 注册一个由带限定符 Singleton 组件实现的强类型应用事件监听器。
    pub fn event_listener_qualified<E, L>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        E: std::any::Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        self.event_listener_registrars
            .push(Box::new(move |builder| {
                builder.event_listener_qualified::<E, L>(qualifier);
            }));
        self
    }

    /// 同时注册派生监听器组件定义及其强类型监听声明。
    ///
    /// # Errors
    ///
    /// 同一组件身份已经登记时返回 [`DefinitionError`]；失败不会追加监听声明。
    pub fn register_event_listener_component<E, L>(&mut self) -> Result<&mut Self, DefinitionError>
    where
        E: std::any::Any + Send + Sync + 'static,
        L: Component + ApplicationEventListener<E>,
    {
        self.registry.register(L::definition())?;
        Ok(self.event_listener::<E, L>())
    }

    /// 注册一个 AOP 顾问。
    pub fn advisor(&mut self, advisor: Advisor) -> &mut Self {
        self.advisor_registrations
            .push(AdvisorRegistration::Instance(advisor));
        self
    }

    /// 注册一个由无限定符 `IoC` 组件实现的线程安全 AOP 顾问。
    ///
    /// 拦截器定义必须已经通过 [`Self::register`]、[`Self::register_all`] 或条件模块
    /// 进入应用。Vernal 会在依赖图冻结后使用最终应用 Container 解析该组件，所以
    /// 拦截器可以注入其他组件，并与业务代码观察到同一个 Singleton。解析只发生
    /// 一次，运行期计划直接持有 `Arc<dyn Interceptor>`，不会退化为 Service Locator。
    /// 组件必须声明为 Singleton；Transient 或 Custom Scope 会在构建阶段被拒绝，
    /// 避免其生命周期被应用级计划静默改变。
    pub fn advisor_component<I, P>(&mut self, pointcut: P, order: i32) -> &mut Self
    where
        I: Interceptor,
        P: Pointcut,
    {
        self.advisor_registrations
            .push(AdvisorRegistration::Component(ManagedAdvisor::new::<I, P>(
                pointcut, order,
            )));
        self
    }

    /// 注册一个由精确限定符 `IoC` 组件实现的线程安全 AOP 顾问。
    ///
    /// 限定符同时参与依赖图身份和 Container 解析，不进行按名称字符串查找或
    /// 运行期候选回退。
    pub fn advisor_component_qualified<I, P>(
        &mut self,
        qualifier: Qualifier,
        pointcut: P,
        order: i32,
    ) -> &mut Self
    where
        I: Interceptor,
        P: Pointcut,
    {
        let managed = ManagedAdvisor::qualified::<I, P>(qualifier, pointcut, order);
        self.advisor_registrations
            .push(AdvisorRegistration::Component(managed));
        self
    }

    /// 原子注册可派生拦截器组件定义及其 AOP 顾问声明。
    ///
    /// 该便利入口适合一个拦截器只声明一个切点的常见场景。需要让同一个组件实例
    /// 服务多个切点时，应先注册一次 [`Component::definition`]，再多次调用
    /// [`Self::advisor_component`]。
    ///
    /// # Errors
    ///
    /// 拦截器组件身份已存在时返回 [`DefinitionError`]；失败时不会留下 Advisor
    /// 登记。
    pub fn register_advisor_component<I, P>(
        &mut self,
        pointcut: P,
        order: i32,
    ) -> Result<&mut Self, DefinitionError>
    where
        I: Component + Interceptor,
        P: Pointcut,
    {
        self.registry.register(I::definition())?;
        Ok(self.advisor_component::<I, P>(pointcut, order))
    }

    /// 注册一个面向 `!Send` 目标 Future 的 Local-AOP 顾问。
    ///
    /// 该入口保留调用方直接提供实例的低层组合方式；组件化入口见
    /// [`Self::local_advisor_component`]。
    pub fn local_advisor(&mut self, advisor: LocalAdvisor) -> &mut Self {
        self.local_advisor_registrations
            .push(LocalAdvisorRegistration::Instance(advisor));
        self
    }

    /// 注册一个由无限定符 `IoC` 组件实现的 Local-AOP 顾问。
    ///
    /// `LocalInterceptor` 对象本身仍满足 `Send + Sync`，因此可以由 Container
    /// 构造并注入依赖；只有调用时 Future、目标和返回值保持 Worker-local。
    /// 与 Send Advisor 一样，组件必须使用 Singleton 作用域。
    pub fn local_advisor_component<I, P>(&mut self, pointcut: P, order: i32) -> &mut Self
    where
        I: LocalInterceptor,
        P: Pointcut,
    {
        let managed = ManagedLocalAdvisor::new::<I, P>(pointcut, order);
        self.local_advisor_registrations
            .push(LocalAdvisorRegistration::Component(managed));
        self
    }

    /// 注册一个由精确限定符 `IoC` 组件实现的 Local-AOP 顾问。
    pub fn local_advisor_component_qualified<I, P>(
        &mut self,
        qualifier: Qualifier,
        pointcut: P,
        order: i32,
    ) -> &mut Self
    where
        I: LocalInterceptor,
        P: Pointcut,
    {
        let managed = ManagedLocalAdvisor::qualified::<I, P>(qualifier, pointcut, order);
        self.local_advisor_registrations
            .push(LocalAdvisorRegistration::Component(managed));
        self
    }

    /// 原子注册可派生本地拦截器组件定义及其 Local Advisor 声明。
    ///
    /// # Errors
    ///
    /// 拦截器组件身份已存在时返回 [`DefinitionError`]；失败时不会留下 Local
    /// Advisor 登记。
    pub fn register_local_advisor_component<I, P>(
        &mut self,
        pointcut: P,
        order: i32,
    ) -> Result<&mut Self, DefinitionError>
    where
        I: Component + LocalInterceptor,
        P: Pointcut,
    {
        self.registry.register(I::definition())?;
        Ok(self.local_advisor_component::<I, P>(pointcut, order))
    }

    /// 设置应用拥有的 Scope 清理等待策略。
    ///
    /// `WebRequestScope` 和其他通过 `ApplicationContext` 创建的作用域可以读取同一个
    /// 策略；策略本身也作为 `IoC` 内建组件注册，基础设施组件可以显式注入并复用。
    pub fn scope_cleanup_policy(&mut self, policy: crate::ScopeCleanupPolicy) -> &mut Self {
        self.scope_cleanup_policy = Arc::new(policy);
        self
    }

    /// 设置应用受管 Tokio 任务的两阶段停机预算。
    ///
    /// 该策略与 [`ManagedTaskSupervisor`] 一同注册为 `IoC` 内建组件，长期 Worker
    /// 和基础设施适配器可以注入同一只读策略；构建完成后不会运行期漂移。
    pub fn task_shutdown_policy(&mut self, policy: TaskShutdownPolicy) -> &mut Self {
        self.task_shutdown_policy = Arc::new(policy);
        self
    }

    /// 设置单个组件生命周期钩子的执行与 Tokio abort 收口预算。
    ///
    /// 策略会作为 `IoC` 内建组件注册；Context 与基础设施组件解析到同一个不可变
    /// 实例，运行期间不会因配置刷新产生不同的超时语义。
    pub fn lifecycle_execution_policy(&mut self, policy: LifecycleExecutionPolicy) -> &mut Self {
        self.lifecycle_execution_policy = Arc::new(policy);
        self
    }

    /// 返回应用环境建造器，供装配代码显式声明来源优先级和 Profile。
    ///
    /// 具体 TOML/YAML、Hutool `.setting`、进程环境变量或配置中心适配器应先实现
    /// [`crate::PropertySource`]，再通过该建造器加入。Vernal 不隐式读取进程全局
    /// 配置，因而同进程的多个 Context 可以拥有完全不同的环境。
    pub fn environment(&mut self) -> &mut ApplicationEnvironmentBuilder {
        &mut self.environment
    }

    /// 声明一个需要在应用构建阶段预编译调用计划的组件操作。
    pub fn operation(&mut self, operation: Operation) -> &mut Self {
        self.operations.push(operation);
        self
    }

    /// 登记一个需要出现在启动报告中的应用 feature。
    ///
    /// 参数要求静态字符串，避免把运行时配置值或密钥误当作 feature 写入诊断
    /// 输出；重复名称会被确定性去重。
    pub fn diagnostic_feature(&mut self, feature: &'static str) -> &mut Self {
        self.enabled_features.insert(feature.to_owned());
        self
    }

    /// 登记一个 Web/RPC Adapter 的脱敏状态。
    ///
    /// 相同名称后一次登记覆盖前一次，最终报告按名称排序，确保快照和测试稳定。
    pub fn adapter_status(&mut self, name: &'static str, state: DiagnosticState) -> &mut Self {
        self.adapters.insert(name.to_owned(), state);
        self
    }

    /// 登记一个外部依赖的脱敏状态。
    ///
    /// API 不接收任意详情或错误字符串，只允许固定状态枚举，避免连接串、令牌或
    /// 下游响应进入可公开序列化的启动报告。
    pub fn external_dependency_status(
        &mut self,
        name: &'static str,
        state: DiagnosticState,
    ) -> &mut Self {
        self.external_dependencies.insert(name.to_owned(), state);
        self
    }

    /// 登记一个静态、脱敏的告警代码。
    ///
    /// 建议使用 `deprecated.adapter-api` 一类稳定代码；该入口不接受运行时
    /// `String`，从 API 层阻止拼接业务错误正文。
    pub fn warning_code(&mut self, warning: &'static str) -> &mut Self {
        self.warnings.insert(warning.to_owned());
        self
    }

    /// 在图冻结前注册 Context 拥有的十一类原生框架对象。
    ///
    /// 该步骤集中维护高层建造器与组件图之间的身份合同，避免主构建流程被重复的
    /// `shared_arc` 细节淹没。任一身份冲突都会立即返回，RegistryBuilder 保留此前
    /// 已明确登记的业务内容供错误链诊断。
    fn register_builtin_components(
        &mut self,
        environment: &Arc<crate::ApplicationEnvironment>,
        invocation_plans: &Arc<InvocationPlanCatalog>,
        local_invocation_plans: &Arc<LocalInvocationPlanCatalog>,
    ) -> Result<(), DefinitionError> {
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(&self.runtime)))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.cancellation,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.managed_tasks,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.task_shutdown_policy,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.lifecycle_execution_policy,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.shutdown_signals,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(environment)))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(&self.events)))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.scope_cleanup_policy,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                invocation_plans,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                local_invocation_plans,
            )))?;
        Ok(())
    }

    /// 在冻结 Environment 上求值并原子提交全部条件模块。
    fn apply_conditional_modules(
        &mut self,
        environment: &Arc<crate::ApplicationEnvironment>,
    ) -> Result<Vec<ConditionEvaluationSnapshot>, ApplicationBuildError> {
        let conditional_modules = std::mem::take(&mut self.conditional_modules);
        let mut evaluations = Vec::with_capacity(conditional_modules.len());
        for module in conditional_modules {
            let matched = module.matches(environment)?;
            evaluations.push(module.snapshot(matched));
            if matched {
                let ConditionalComponentModuleParts {
                    definitions,
                    bindings,
                    lifecycle_registrars,
                    event_listener_registrars,
                } = module.into_parts();
                self.registry.register_bundle(definitions, bindings)?;
                self.lifecycle_registrars.extend(lifecycle_registrars);
                self.event_listener_registrars
                    .extend(event_listener_registrars);
            }
        }
        Ok(evaluations)
    }

    /// 冻结依赖图和 AOP 计划并创建尚未 refresh 的应用上下文。
    ///
    /// # Errors
    ///
    /// 条件评估失败、内建组件冲突、依赖图无效或生命周期绑定无效时返回
    /// [`ApplicationBuildError`]。
    pub fn build(mut self) -> Result<ApplicationContext, ApplicationBuildError> {
        // Environment 必须先冻结，所有条件模块才能对同一个不可变快照执行一次判断。
        // 命中模块通过 RegistryBuilder 的原子 bundle API 提交，未命中模块不会留下
        // Definition、Trait Binding 或生命周期登记中的任一残片。
        let environment = Arc::new(std::mem::take(&mut self.environment).build());
        let condition_evaluations = self.apply_conditional_modules(&environment)?;

        // 两类 AOP 目录先以待封存原生对象进入依赖图：这样由 IoC 管理的 Send 与
        // Local 拦截器都可以注入目录、Tokio 或其他业务组件，随后仍由同一个
        // Container 构造。`!Send` 边界只存在于 Local 调用 Future 和返回值。
        let operations = std::mem::take(&mut self.operations);
        let invocation_plans = Arc::new(InvocationPlanCatalog::deferred());
        let local_invocation_plans = Arc::new(LocalInvocationPlanCatalog::deferred());

        // 内建原生对象必须在图冻结前进入注册表，业务组件对它们的依赖才会被
        // GraphPlanner 与其他依赖完全一致地校验。
        self.register_builtin_components(&environment, &invocation_plans, &local_invocation_plans)?;

        // 依赖图只冻结一次；随后创建的 Container 就是最终 ApplicationContext 持有
        // 的实例。IoC 拦截器在这里解析，其 Singleton 身份、依赖和失败追踪不会因
        // AOP 装配再创建第二套 Store 或 bootstrap Container。
        let registry = self.registry.build()?;
        let container = registry.into_container();
        let mut invocation_plan_builder = InvocationPlanBuilder::new();
        for registration in self.advisor_registrations {
            match registration {
                AdvisorRegistration::Instance(advisor) => {
                    invocation_plan_builder.register(advisor);
                }
                AdvisorRegistration::Component(managed) => {
                    let component = managed.component().clone();
                    if let Some(scope) = managed.invalid_scope(&container) {
                        return Err(ApplicationBuildError::AdvisorScope { component, scope });
                    }
                    let advisor = managed.resolve(&container).map_err(|source| {
                        ApplicationBuildError::AdvisorResolution {
                            component,
                            source: Box::new(source),
                        }
                    })?;
                    invocation_plan_builder.register(advisor);
                }
            }
        }

        // Pointcut 仍只在应用构建阶段匹配。封存成功后目录没有修改入口，业务方法
        // 与 Web Adapter 的热路径只执行 Operation 查找和预排序拦截器链。
        let compiled_invocation_plans =
            invocation_plan_builder.build_catalog(operations.iter().cloned())?;
        invocation_plans.initialize_from(&compiled_invocation_plans)?;

        let mut local_invocation_plan_builder = LocalInvocationPlanBuilder::new();
        for registration in self.local_advisor_registrations {
            match registration {
                LocalAdvisorRegistration::Instance(advisor) => {
                    local_invocation_plan_builder.register(advisor);
                }
                LocalAdvisorRegistration::Component(managed) => {
                    let component = managed.component().clone();
                    if let Some(scope) = managed.invalid_scope(&container) {
                        return Err(ApplicationBuildError::AdvisorScope { component, scope });
                    }
                    let advisor = managed.resolve(&container).map_err(|source| {
                        ApplicationBuildError::LocalAdvisorResolution {
                            component,
                            source: Box::new(source),
                        }
                    })?;
                    local_invocation_plan_builder.register(advisor);
                }
            }
        }
        let compiled_local_invocation_plans =
            local_invocation_plan_builder.build_catalog(operations)?;
        local_invocation_plans.initialize_from(&compiled_local_invocation_plans)?;

        let resources = ContextResources {
            runtime: Some(self.runtime),
            cancellation: self.cancellation,
            managed_tasks: Some(self.managed_tasks),
            task_shutdown_policy: self.task_shutdown_policy,
            lifecycle_execution_policy: self.lifecycle_execution_policy,
            shutdown_signals: self.shutdown_signals,
            environment,
            events: self.events,
            scope_cleanup_policy: self.scope_cleanup_policy,
            invocation_plans,
            local_invocation_plans,
            diagnostics: DiagnosticConfiguration::new(
                self.enabled_features.into_iter().collect(),
                self.adapters
                    .into_iter()
                    .map(|(name, state)| SubsystemStatus::new(name, state))
                    .collect(),
                self.external_dependencies
                    .into_iter()
                    .map(|(name, state)| SubsystemStatus::new(name, state))
                    .collect(),
                condition_evaluations,
                self.warnings.into_iter().collect(),
            ),
        };
        let mut context = ApplicationContextBuilder::managed(container, resources);
        for registrar in self.lifecycle_registrars {
            registrar(&mut context);
        }
        for registrar in self.event_listener_registrars {
            registrar(&mut context);
        }
        context.build().map_err(Into::into)
    }
}

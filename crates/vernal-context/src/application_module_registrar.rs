//! 应用模块隔离登记器对象。

use std::sync::Arc;

use vernal_aop::{Advisor, Interceptor, LocalAdvisor, LocalInterceptor, Operation, Pointcut};
use vernal_ioc::{Component, ComponentDefinition, Qualifier, TraitBinding};

use crate::{
    ApplicationEventListener, ApplicationRunner, ConditionalComponentModule,
    ConfigurationProperties, Lifecycle, PropertySource, advisor_registration::AdvisorRegistration,
    application_module_parts::ApplicationModuleParts,
    application_runner_registrar::ApplicationRunnerRegistrar,
    event_listener_registrar::EventListenerRegistrar, lifecycle_registrar::LifecycleRegistrar,
    local_advisor_registration::LocalAdvisorRegistration, managed_advisor::ManagedAdvisor,
    managed_local_advisor::ManagedLocalAdvisor,
    module_environment_contribution::ModuleEnvironmentContribution,
};

/// 在应用建造器之外暂存一个模块的全部装配贡献。
///
/// 所有方法只修改当前 Registrar，不触碰真实 `VernalApplicationBuilder`。只有
/// [`crate::VernalApplicationBuilder::register_module`] 完成模块配置、环境预检和
/// `IoC` bundle 校验后，组件、监听器、Runner 及其他贡献才按原声明顺序一次提交。
#[derive(Default)]
pub struct ApplicationModuleRegistrar {
    definitions: Vec<ComponentDefinition>,
    bindings: Vec<TraitBinding>,
    lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    event_listener_registrars: Vec<Box<EventListenerRegistrar>>,
    application_runner_registrars: Vec<Box<ApplicationRunnerRegistrar>>,
    advisor_registrations: Vec<AdvisorRegistration>,
    local_advisor_registrations: Vec<LocalAdvisorRegistration>,
    operations: Vec<Operation>,
    environment_contributions: Vec<ModuleEnvironmentContribution>,
    conditional_modules: Vec<ConditionalComponentModule>,
}

impl ApplicationModuleRegistrar {
    /// 创建没有任何贡献的隔离 Registrar。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 暂存一个组件定义。
    pub fn register(&mut self, definition: ComponentDefinition) -> &mut Self {
        self.definitions.push(definition);
        self
    }

    /// 暂存一组组件定义并保留输入顺序。
    pub fn register_all(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
    ) -> &mut Self {
        self.definitions.extend(definitions);
        self
    }

    /// 暂存一个派生组件定义。
    pub fn component<T>(&mut self) -> &mut Self
    where
        T: Component,
    {
        self.register(T::definition())
    }

    /// 暂存一个由当前应用 Environment 绑定的类型安全配置对象。
    ///
    /// 配置对象以标准 Singleton Definition 进入同一原子模块；模块提交失败时，
    /// 该定义不会残留。具体配置格式仍由模块贡献的 `PropertySource` 决定。
    pub fn configuration_properties<T>(&mut self) -> &mut Self
    where
        T: ConfigurationProperties,
    {
        self.register(T::component_definition())
    }

    /// 暂存一个类型安全 Trait Object 绑定。
    pub fn bind(&mut self, binding: TraitBinding) -> &mut Self {
        self.bindings.push(binding);
        self
    }

    /// 暂存一组 Trait Object 绑定并保留输入顺序。
    pub fn bind_all(&mut self, bindings: impl IntoIterator<Item = TraitBinding>) -> &mut Self {
        self.bindings.extend(bindings);
        self
    }

    /// 声明一个无限定符生命周期组件。
    pub fn lifecycle<T>(&mut self) -> &mut Self
    where
        T: Lifecycle,
    {
        self.lifecycle_registrars.push(Box::new(|builder| {
            builder.lifecycle::<T>();
        }));
        self
    }

    /// 声明一个带精确限定符的生命周期组件。
    pub fn lifecycle_qualified<T>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        T: Lifecycle,
    {
        self.lifecycle_registrars.push(Box::new(move |builder| {
            builder.lifecycle_qualified::<T>(qualifier);
        }));
        self
    }

    /// 声明一个由无限定符 Singleton 组件实现的强类型应用事件监听器。
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

    /// 声明一个由带限定符 Singleton 组件实现的强类型应用事件监听器。
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

    /// 同时暂存派生监听器组件定义及其强类型监听声明。
    pub fn register_event_listener_component<E, L>(&mut self) -> &mut Self
    where
        E: std::any::Any + Send + Sync + 'static,
        L: Component + ApplicationEventListener<E>,
    {
        self.component::<L>().event_listener::<E, L>()
    }

    /// 声明一个由无限定符 Singleton 组件实现的一次性应用 Runner。
    pub fn application_runner<R>(&mut self) -> &mut Self
    where
        R: ApplicationRunner,
    {
        self.application_runner_registrars.push(Box::new(|builder| {
            builder.application_runner::<R>();
        }));
        self
    }

    /// 声明一个由带精确限定符 Singleton 组件实现的一次性应用 Runner。
    pub fn application_runner_qualified<R>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        R: ApplicationRunner,
    {
        self.application_runner_registrars
            .push(Box::new(move |builder| {
                builder.application_runner_qualified::<R>(qualifier);
            }));
        self
    }

    /// 同时暂存派生 Runner 组件定义及其执行声明。
    pub fn register_application_runner<R>(&mut self) -> &mut Self
    where
        R: Component + ApplicationRunner,
    {
        self.component::<R>().application_runner::<R>()
    }

    /// 暂存一个已经构造完成的线程安全 Advisor。
    pub fn advisor(&mut self, advisor: Advisor) -> &mut Self {
        self.advisor_registrations
            .push(AdvisorRegistration::Instance(advisor));
        self
    }

    /// 声明一个由无限定符 Singleton 组件实现的线程安全 Advisor。
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

    /// 声明一个由带限定符 Singleton 组件实现的线程安全 Advisor。
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
        self.advisor_registrations
            .push(AdvisorRegistration::Component(ManagedAdvisor::qualified::<
                I,
                P,
            >(
                qualifier, pointcut, order
            )));
        self
    }

    /// 同时暂存派生拦截器组件定义及其线程安全 Advisor。
    pub fn register_advisor_component<I, P>(&mut self, pointcut: P, order: i32) -> &mut Self
    where
        I: Component + Interceptor,
        P: Pointcut,
    {
        self.component::<I>()
            .advisor_component::<I, P>(pointcut, order)
    }

    /// 暂存一个已经构造完成的 Local Advisor。
    pub fn local_advisor(&mut self, advisor: LocalAdvisor) -> &mut Self {
        self.local_advisor_registrations
            .push(LocalAdvisorRegistration::Instance(advisor));
        self
    }

    /// 声明一个由无限定符 Singleton 组件实现的 Local Advisor。
    pub fn local_advisor_component<I, P>(&mut self, pointcut: P, order: i32) -> &mut Self
    where
        I: LocalInterceptor,
        P: Pointcut,
    {
        self.local_advisor_registrations
            .push(LocalAdvisorRegistration::Component(
                ManagedLocalAdvisor::new::<I, P>(pointcut, order),
            ));
        self
    }

    /// 声明一个由带限定符 Singleton 组件实现的 Local Advisor。
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
        self.local_advisor_registrations
            .push(LocalAdvisorRegistration::Component(
                ManagedLocalAdvisor::qualified::<I, P>(qualifier, pointcut, order),
            ));
        self
    }

    /// 同时暂存派生拦截器组件定义及其 Local Advisor。
    pub fn register_local_advisor_component<I, P>(&mut self, pointcut: P, order: i32) -> &mut Self
    where
        I: Component + LocalInterceptor,
        P: Pointcut,
    {
        self.component::<I>()
            .local_advisor_component::<I, P>(pointcut, order)
    }

    /// 暂存一个需要预编译 AOP 调用计划的操作。
    pub fn operation(&mut self, operation: Operation) -> &mut Self {
        self.operations.push(operation);
        self
    }

    /// 暂存一组操作并保留输入顺序。
    pub fn operations(&mut self, operations: impl IntoIterator<Item = Operation>) -> &mut Self {
        self.operations.extend(operations);
        self
    }

    /// 在应用 Environment 的最高优先级位置暂存属性来源。
    pub fn property_source_first(&mut self, source: Arc<dyn PropertySource>) -> &mut Self {
        self.environment_contributions
            .push(ModuleEnvironmentContribution::First(source));
        self
    }

    /// 在应用 Environment 的最低优先级位置暂存属性来源。
    pub fn property_source_last(&mut self, source: Arc<dyn PropertySource>) -> &mut Self {
        self.environment_contributions
            .push(ModuleEnvironmentContribution::Last(source));
        self
    }

    /// 暂存一个 Active Profile。
    pub fn active_profile(&mut self, profile: impl Into<String>) -> &mut Self {
        self.environment_contributions
            .push(ModuleEnvironmentContribution::ActiveProfile(profile.into()));
        self
    }

    /// 暂存一个 Default Profile。
    pub fn default_profile(&mut self, profile: impl Into<String>) -> &mut Self {
        self.environment_contributions
            .push(ModuleEnvironmentContribution::DefaultProfile(
                profile.into(),
            ));
        self
    }

    /// 暂存一个在最终 Environment 冻结后求值的条件组件模块。
    ///
    /// 条件模块的声明身份会与当前应用以及同一外层模块内的其他条件模块统一预检。
    /// 外层模块注册失败时，条件定义、Binding 和生命周期登记都不会进入真实建造器。
    pub fn conditional(&mut self, module: ConditionalComponentModule) -> &mut Self {
        self.conditional_modules.push(module);
        self
    }

    /// 暂存一组条件组件模块并保留输入顺序。
    pub fn conditionals(
        &mut self,
        modules: impl IntoIterator<Item = ConditionalComponentModule>,
    ) -> &mut Self {
        self.conditional_modules.extend(modules);
        self
    }

    /// 消费 Registrar 并返回等待原子提交的命名贡献集合。
    pub(crate) fn into_parts(self) -> ApplicationModuleParts {
        ApplicationModuleParts {
            definitions: self.definitions,
            bindings: self.bindings,
            lifecycle_registrars: self.lifecycle_registrars,
            event_listener_registrars: self.event_listener_registrars,
            application_runner_registrars: self.application_runner_registrars,
            advisor_registrations: self.advisor_registrations,
            local_advisor_registrations: self.local_advisor_registrations,
            operations: self.operations,
            environment_contributions: self.environment_contributions,
            conditional_modules: self.conditional_modules,
        }
    }
}

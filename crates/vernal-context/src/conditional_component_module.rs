//! 条件组件模块对象。

use std::{fmt, sync::Arc};

use vernal_beans::{ComponentDefinition, Qualifier, TraitBinding};

use crate::{
    ApplicationEnvironment, ApplicationEventListener, ApplicationRunner, ComponentCondition,
    ConditionError, ConditionEvaluationSnapshot, ConfigurationPhase, ConfigurationProperties,
    Lifecycle, ScheduledTask, application_runner_registrar::ApplicationRunnerRegistrar,
    condition_contribution_counts::ConditionContributionCounts,
    conditional_component_module_parts::ConditionalComponentModuleParts,
    event_listener_registrar::EventListenerRegistrar, lifecycle_registrar::LifecycleRegistrar,
    scheduled_task_registrar::ScheduledTaskRegistrar,
};

/// 把同一装配条件下的组件定义、Trait Binding、生命周期、监听器、Runner 与任务
/// 登记组成原子模块。
///
/// 模块只有在条件命中时才整体提交到 `RegistryBuilder`。这避免组件定义被排除、
/// Trait Binding 却残留，或生命周期仍尝试解析不存在组件的半装配状态。模块名和
/// 条件名进入启动报告，条件捕获的属性键和值不会进入诊断对象。
///
/// `phase` 字段对标 Spring `ConfigurationCondition#getConfigurationPhase`：
/// 默认 `ParseConfiguration` 与 vernal 当前在 builder 阶段评估条件的行为一致；
/// 设置为 `RegisterBean` 让模块语义与 Spring `@Conditional` 在注册普通 bean 阶段评估。
pub struct ConditionalComponentModule {
    name: &'static str,
    condition: Arc<dyn ComponentCondition>,
    phase: ConfigurationPhase,
    definitions: Vec<ComponentDefinition>,
    bindings: Vec<TraitBinding>,
    lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    event_listener_registrars: Vec<Box<EventListenerRegistrar>>,
    application_runner_registrars: Vec<Box<ApplicationRunnerRegistrar>>,
    scheduled_task_registrars: Vec<Box<ScheduledTaskRegistrar>>,
}

impl ConditionalComponentModule {
    /// 使用具体条件创建空模块。
    #[must_use]
    pub fn new<C>(name: &'static str, condition: C) -> Self
    where
        C: ComponentCondition,
    {
        Self::shared(name, Arc::new(condition))
    }

    /// 使用已经共享的条件创建空模块。
    #[must_use]
    pub const fn shared(name: &'static str, condition: Arc<dyn ComponentCondition>) -> Self {
        Self {
            name,
            condition,
            phase: ConfigurationPhase::ParseConfiguration,
            definitions: Vec::new(),
            bindings: Vec::new(),
            lifecycle_registrars: Vec::new(),
            event_listener_registrars: Vec::new(),
            application_runner_registrars: Vec::new(),
            scheduled_task_registrars: Vec::new(),
        }
    }

    /// 设置模块的评估阶段（对标 Spring `ConfigurationCondition#getConfigurationPhase`）。
    ///
    /// 默认为 `ConfigurationPhase::ParseConfiguration`：在 builder 阶段立即评估，
    /// 不命中时整个模块都不会进入依赖图。设置为 `RegisterBean` 时模块与 Spring
    /// 的 `@Conditional(REGISTER_BEAN)` 等价 —— 此时 vernal 由调用方负责延后评估
    /// 时机（当前架构下两者效果相同，因为 vernal 评估时机固定在 builder 阶段）。
    #[must_use]
    pub const fn with_phase(mut self, phase: ConfigurationPhase) -> Self {
        self.phase = phase;
        self
    }

    /// 返回模块的当前评估阶段。
    #[must_use]
    pub const fn phase(&self) -> ConfigurationPhase {
        self.phase
    }

    /// 向模块追加一个组件定义。
    pub fn register(&mut self, definition: ComponentDefinition) -> &mut Self {
        self.definitions.push(definition);
        self
    }

    /// 向模块追加一组组件定义，并保留输入顺序。
    pub fn register_all(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
    ) -> &mut Self {
        self.definitions.extend(definitions);
        self
    }

    /// 向条件模块追加一个类型安全配置对象定义。
    ///
    /// 只有条件命中时该配置对象才进入依赖图，绑定时读取最终冻结的同一份
    /// [`ApplicationEnvironment`]。
    pub fn configuration_properties<T>(&mut self) -> &mut Self
    where
        T: ConfigurationProperties,
    {
        self.register(T::component_definition())
    }

    /// 向模块追加一个 Trait Object 绑定。
    pub fn bind(&mut self, binding: TraitBinding) -> &mut Self {
        self.bindings.push(binding);
        self
    }

    /// 向模块追加一组 Trait Object 绑定，并保留输入顺序。
    pub fn bind_all(&mut self, bindings: impl IntoIterator<Item = TraitBinding>) -> &mut Self {
        self.bindings.extend(bindings);
        self
    }

    /// 声明模块内一个无限定符生命周期组件。
    ///
    /// 该登记与组件定义使用同一条件：模块未命中时，两者都会被排除。
    pub fn lifecycle<T>(&mut self) -> &mut Self
    where
        T: Lifecycle,
    {
        self.lifecycle_registrars.push(Box::new(|builder| {
            builder.lifecycle::<T>();
        }));
        self
    }

    /// 声明模块内一个带限定符的生命周期组件。
    pub fn lifecycle_qualified<T>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        T: Lifecycle,
    {
        self.lifecycle_registrars.push(Box::new(move |builder| {
            builder.lifecycle_qualified::<T>(qualifier);
        }));
        self
    }

    /// 声明模块内一个无限定符 Singleton 事件监听器。
    ///
    /// 条件未命中时，监听声明与组件定义会一起排除，不会留下后台任务。
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

    /// 声明模块内一个带限定符的 Singleton 事件监听器。
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

    /// 声明模块内一个无限定符 Singleton 应用 Runner。
    ///
    /// 条件未命中时，Runner 声明与组件定义一起排除，不会在启动阶段留下解析计划。
    pub fn application_runner<R>(&mut self) -> &mut Self
    where
        R: ApplicationRunner,
    {
        self.application_runner_registrars.push(Box::new(|builder| {
            builder.application_runner::<R>();
        }));
        self
    }

    /// 声明模块内一个带精确限定符的 Singleton 应用 Runner。
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

    /// 声明模块内一个无限定符 Singleton 周期任务。
    ///
    /// 条件未命中时，任务声明与组件定义一起排除，不会创建后台 Tokio 任务。
    pub fn scheduled_task<T>(&mut self) -> &mut Self
    where
        T: ScheduledTask,
    {
        self.scheduled_task_registrars.push(Box::new(|builder| {
            builder.scheduled_task::<T>();
        }));
        self
    }

    /// 声明模块内一个带精确限定符的 Singleton 周期任务。
    pub fn scheduled_task_qualified<T>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        T: ScheduledTask,
    {
        self.scheduled_task_registrars
            .push(Box::new(move |builder| {
                builder.scheduled_task_qualified::<T>(qualifier);
            }));
        self
    }

    /// 返回条件模块静态名称。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// 返回稳定条件类型名。
    #[must_use]
    pub fn condition_name(&self) -> &'static str {
        self.condition.name()
    }

    /// 校验静态诊断身份和模块内容。
    pub fn validate(&self) -> Result<(), ConditionError> {
        if self.name.is_empty()
            || self
                .name
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ConditionError::InvalidModuleName { name: self.name });
        }
        let condition = self.condition.name();
        if condition.is_empty()
            || condition
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ConditionError::InvalidConditionName {
                module: self.name,
                condition,
            });
        }
        if self.definitions.is_empty()
            && self.bindings.is_empty()
            && self.lifecycle_registrars.is_empty()
            && self.event_listener_registrars.is_empty()
            && self.application_runner_registrars.is_empty()
            && self.scheduled_task_registrars.is_empty()
        {
            return Err(ConditionError::EmptyModule { name: self.name });
        }
        Ok(())
    }

    /// 在冻结环境中执行一次条件判断。
    pub fn matches(
        &self,
        environment: &ApplicationEnvironment,
    ) -> Result<bool, ConditionError> {
        self.condition.matches(environment).map_err(|source| {
            ConditionError::evaluation_failed(self.name, self.condition.name(), source)
        })
    }

    /// 生成不包含配置键和值的评估快照。
    pub fn snapshot(&self, matched: bool) -> ConditionEvaluationSnapshot {
        ConditionEvaluationSnapshot::new(
            self.name,
            self.condition.name(),
            matched,
            self.definitions
                .iter()
                .map(|definition| definition.key().to_string())
                .collect(),
            ConditionContributionCounts::new(
                self.bindings.len(),
                self.lifecycle_registrars.len(),
                self.event_listener_registrars.len(),
                self.application_runner_registrars.len(),
                self.scheduled_task_registrars.len(),
            ),
        )
    }

    /// 消费模块并返回可原子提交的六类注册项。
    pub fn into_parts(self) -> ConditionalComponentModuleParts {
        ConditionalComponentModuleParts {
            definitions: self.definitions,
            bindings: self.bindings,
            lifecycle_registrars: self.lifecycle_registrars,
            event_listener_registrars: self.event_listener_registrars,
            application_runner_registrars: self.application_runner_registrars,
            scheduled_task_registrars: self.scheduled_task_registrars,
        }
    }
}

impl fmt::Debug for ConditionalComponentModule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConditionalComponentModule")
            .field("name", &self.name)
            .field("condition", &self.condition.name())
            .field(
                "components",
                &self
                    .definitions
                    .iter()
                    .map(|definition| definition.key().to_string())
                    .collect::<Vec<_>>(),
            )
            .field("trait_binding_count", &self.bindings.len())
            .field("lifecycle_count", &self.lifecycle_registrars.len())
            .field(
                "event_listener_count",
                &self.event_listener_registrars.len(),
            )
            .field(
                "application_runner_count",
                &self.application_runner_registrars.len(),
            )
            .field(
                "scheduled_task_count",
                &self.scheduled_task_registrars.len(),
            )
            .finish()
    }
}

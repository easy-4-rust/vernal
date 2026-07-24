//! Vernal 高层应用建造器对象。

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::{
    Advisor, InvocationPlanBuilder, LocalAdvisor, LocalInvocationPlanBuilder, Operation,
};
use vernal_ioc::{ComponentDefinition, DefinitionError, Qualifier, RegistryBuilder, TraitBinding};

use crate::{
    ApplicationBuildError, ApplicationContext, ApplicationContextBuilder, DiagnosticState,
    EventBus, Lifecycle, SubsystemStatus, context_resources::ContextResources,
    diagnostic_configuration::DiagnosticConfiguration,
};

type LifecycleRegistrar = dyn FnOnce(&mut ApplicationContextBuilder) + Send + Sync + 'static;

/// 统一收集组件、生命周期、切面和 Tokio Context 资源的应用建造器。
///
/// 与接收冻结 [`vernal_ioc::Registry`] 的低层 [`ApplicationContextBuilder`]
/// 不同，该建造器在依赖图冻结前自动注册五类框架内建组件：
///
/// - [`Handle`]：应用绑定的 Tokio Runtime；
/// - [`CancellationToken`]：应用关闭与后台任务协作取消；
/// - [`EventBus`]：Context 内类型化事件；
/// - [`vernal_aop::InvocationPlanCatalog`]：预编译 AOP 调用计划。
/// - [`vernal_aop::LocalInvocationPlanCatalog`]：预编译 Local-AOP 调用计划。
///
/// 业务组件可以像依赖普通 Rust 类型一样依赖它们，不需要全局 Service Locator
/// 或 Vernal 专用包装 trait。
pub struct VernalApplicationBuilder {
    registry: RegistryBuilder,
    lifecycle_registrars: Vec<Box<LifecycleRegistrar>>,
    invocation_plans: InvocationPlanBuilder,
    local_invocation_plans: LocalInvocationPlanBuilder,
    operations: Vec<Operation>,
    runtime: Arc<Handle>,
    cancellation: Arc<CancellationToken>,
    events: Arc<EventBus>,
    enabled_features: BTreeSet<String>,
    adapters: BTreeMap<String, DiagnosticState>,
    external_dependencies: BTreeMap<String, DiagnosticState>,
    warnings: BTreeSet<String>,
}

impl VernalApplicationBuilder {
    /// 使用显式 Tokio Runtime Handle 创建应用建造器。
    #[must_use]
    pub fn new(runtime: Handle) -> Self {
        Self {
            registry: RegistryBuilder::new(),
            lifecycle_registrars: Vec::new(),
            invocation_plans: InvocationPlanBuilder::new(),
            local_invocation_plans: LocalInvocationPlanBuilder::new(),
            operations: Vec::new(),
            runtime: Arc::new(runtime),
            cancellation: Arc::new(CancellationToken::new()),
            events: Arc::new(EventBus::new()),
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
    /// `Handle`、`CancellationToken`、`EventBus` 或两类调用计划目录。
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

    /// 注册一个 AOP 顾问。
    pub fn advisor(&mut self, advisor: Advisor) -> &mut Self {
        self.invocation_plans.register(advisor);
        self
    }

    /// 注册一个面向 `!Send` 目标 Future 的 Local-AOP 顾问。
    pub fn local_advisor(&mut self, advisor: LocalAdvisor) -> &mut Self {
        self.local_invocation_plans.register(advisor);
        self
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

    /// 冻结依赖图和 AOP 计划并创建尚未 refresh 的应用上下文。
    ///
    /// # Errors
    ///
    /// 内建组件冲突、依赖图无效或生命周期绑定无效时返回
    /// [`ApplicationBuildError`]。
    pub fn build(mut self) -> Result<ApplicationContext, ApplicationBuildError> {
        // Pointcut 只在启动阶段匹配；运行期目录保持不可变。
        let invocation_plans = Arc::new(
            self.invocation_plans
                .build_catalog(self.operations.iter().cloned()),
        );
        let local_invocation_plans =
            Arc::new(self.local_invocation_plans.build_catalog(self.operations));

        // 内建原生对象必须在图冻结前进入注册表，业务组件对它们的依赖才会被
        // GraphPlanner 与其他依赖完全一致地校验。
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(&self.runtime)))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &self.cancellation,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(&self.events)))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &invocation_plans,
            )))?;
        self.registry
            .register(ComponentDefinition::shared_arc(Arc::clone(
                &local_invocation_plans,
            )))?;

        let resources = ContextResources::managed(
            self.runtime,
            self.cancellation,
            self.events,
            invocation_plans,
            local_invocation_plans,
            DiagnosticConfiguration::new(
                self.enabled_features.into_iter().collect(),
                self.adapters
                    .into_iter()
                    .map(|(name, state)| SubsystemStatus::new(name, state))
                    .collect(),
                self.external_dependencies
                    .into_iter()
                    .map(|(name, state)| SubsystemStatus::new(name, state))
                    .collect(),
                self.warnings.into_iter().collect(),
            ),
        );
        let registry = self.registry.build()?;
        let mut context = ApplicationContextBuilder::managed(registry, resources);
        for registrar in self.lifecycle_registrars {
            registrar(&mut context);
        }
        context.build().map_err(Into::into)
    }
}

//! 应用启动诊断报告对象。

use serde::Serialize;
use vernal_aop::{InvocationPlanCatalog, LocalInvocationPlanCatalog};
use vernal_ioc::RegistrySnapshot;

use crate::{
    StartupObservation, SubsystemStatus, diagnostic_configuration::DiagnosticConfiguration,
};

/// `ApplicationContext` 的可序列化、只读、脱敏诊断快照。
///
/// 报告持有值对象而不是容器、组件实例或错误源。调用
/// [`crate::ApplicationContext::startup_report`] 时会克隆当前快照，因此获得的
/// 报告不会被后续生命周期操作修改，也不能反向控制 Context。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StartupReport {
    framework_version: String,
    minimum_rust_version: String,
    project_status: String,
    context_state: String,
    registry: RegistrySnapshot,
    aop_plan_count: usize,
    aop_interceptor_count: usize,
    local_aop_plan_count: usize,
    local_aop_interceptor_count: usize,
    enabled_features: Vec<String>,
    adapters: Vec<SubsystemStatus>,
    external_dependencies: Vec<SubsystemStatus>,
    observations: Vec<StartupObservation>,
    warnings: Vec<String>,
    unused_definitions: Vec<String>,
}

impl StartupReport {
    /// 创建尚未执行 refresh 的初始报告。
    pub(crate) fn new(
        context_state: String,
        registry: RegistrySnapshot,
        invocation_plans: &InvocationPlanCatalog,
        local_invocation_plans: &LocalInvocationPlanCatalog,
        diagnostics: &DiagnosticConfiguration,
    ) -> Self {
        Self {
            framework_version: vernal_core::FRAMEWORK_VERSION.to_owned(),
            minimum_rust_version: vernal_core::MINIMUM_RUST_VERSION.to_owned(),
            project_status: vernal_core::PROJECT_STATUS.to_owned(),
            context_state,
            registry,
            aop_plan_count: invocation_plans.len(),
            aop_interceptor_count: invocation_plans.interceptor_count(),
            local_aop_plan_count: local_invocation_plans.len(),
            local_aop_interceptor_count: local_invocation_plans.interceptor_count(),
            enabled_features: diagnostics.enabled_features().to_vec(),
            adapters: diagnostics.adapters().to_vec(),
            external_dependencies: diagnostics.external_dependencies().to_vec(),
            observations: Vec::new(),
            warnings: diagnostics.warnings().to_vec(),
            // 仅凭“没有入边”不能判断组件未使用；在引入准确的解析追踪前保持空集，
            // 避免把合法入口服务误报为死定义。
            unused_definitions: Vec::new(),
        }
    }

    /// 返回 Vernal Workspace 版本。
    #[must_use]
    pub fn framework_version(&self) -> &str {
        &self.framework_version
    }

    /// 返回当前承诺的最低 Rust 工具链版本。
    #[must_use]
    pub fn minimum_rust_version(&self) -> &str {
        &self.minimum_rust_version
    }

    /// 返回框架成熟度状态。
    #[must_use]
    pub fn project_status(&self) -> &str {
        &self.project_status
    }

    /// 返回生成快照时的 Context 状态。
    #[must_use]
    pub fn context_state(&self) -> &str {
        &self.context_state
    }

    /// 返回 `IoC` 注册表诊断快照。
    #[must_use]
    pub const fn registry(&self) -> &RegistrySnapshot {
        &self.registry
    }

    /// 返回唯一 AOP 调用计划数量。
    #[must_use]
    pub const fn aop_plan_count(&self) -> usize {
        self.aop_plan_count
    }

    /// 返回全部调用计划匹配的拦截器槽位总数。
    #[must_use]
    pub const fn aop_interceptor_count(&self) -> usize {
        self.aop_interceptor_count
    }

    /// 返回唯一 Local-AOP 调用计划数量。
    #[must_use]
    pub const fn local_aop_plan_count(&self) -> usize {
        self.local_aop_plan_count
    }

    /// 返回全部 Local-AOP 计划匹配的拦截器槽位总数。
    #[must_use]
    pub const fn local_aop_interceptor_count(&self) -> usize {
        self.local_aop_interceptor_count
    }

    /// 返回应用显式声明的 feature 名称。
    #[must_use]
    pub fn enabled_features(&self) -> &[String] {
        &self.enabled_features
    }

    /// 返回 Web/RPC Adapter 状态。
    #[must_use]
    pub fn adapters(&self) -> &[SubsystemStatus] {
        &self.adapters
    }

    /// 返回外部依赖状态。
    #[must_use]
    pub fn external_dependencies(&self) -> &[SubsystemStatus] {
        &self.external_dependencies
    }

    /// 返回按实际完成顺序记录的启动与关闭观察。
    #[must_use]
    pub fn observations(&self) -> &[StartupObservation] {
        &self.observations
    }

    /// 返回应用构建阶段显式登记的脱敏警告。
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// 返回经过可靠解析追踪确认的未使用定义。
    ///
    /// 当前内核尚未启用解析追踪，因此该集合保持为空，不使用“没有入边”等不可靠
    /// 启发式规则制造误报。
    #[must_use]
    pub fn unused_definitions(&self) -> &[String] {
        &self.unused_definitions
    }

    /// 更新 Context 状态；只由 Context 状态机调用。
    pub(crate) fn set_context_state(&mut self, state: String) {
        self.context_state = state;
    }

    /// 追加一条完成后的脱敏观察记录。
    pub(crate) fn record(&mut self, observation: StartupObservation) {
        self.observations.push(observation);
    }
}

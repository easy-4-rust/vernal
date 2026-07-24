//! 应用上下文内建资源集合对象。

use std::sync::Arc;

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::{InvocationPlanCatalog, LocalInvocationPlanCatalog};

use crate::{EventBus, ScopeCleanupPolicy, diagnostic_configuration::DiagnosticConfiguration};

/// 聚合一个 `ApplicationContext` 独占或共享的基础运行资源。
///
/// 该对象只在 Context 内部传递，公开 API 仍直接暴露 Rust 原生类型。高层应用
/// 建造器会把其中的 Tokio Handle、取消令牌、事件总线，以及线程安全与本地
/// AOP 计划目录同时注册到 `IoC` 容器，使业务组件与 Context 使用相同实例。
pub(crate) struct ContextResources {
    runtime: Option<Arc<Handle>>,
    cancellation: Arc<CancellationToken>,
    events: Arc<EventBus>,
    scope_cleanup_policy: Arc<ScopeCleanupPolicy>,
    invocation_plans: Arc<InvocationPlanCatalog>,
    local_invocation_plans: Arc<LocalInvocationPlanCatalog>,
    diagnostics: DiagnosticConfiguration,
}

impl ContextResources {
    /// 为保留兼容性的低层 Context 构建路径创建默认资源。
    ///
    /// 低层 API 可以在 Tokio Runtime 外完成组装，所以这里不隐式捕获运行时。
    pub(crate) fn standalone() -> Self {
        Self {
            runtime: None,
            cancellation: Arc::new(CancellationToken::new()),
            events: Arc::new(EventBus::new()),
            scope_cleanup_policy: Arc::new(ScopeCleanupPolicy::default()),
            invocation_plans: Arc::new(InvocationPlanCatalog::default()),
            local_invocation_plans: Arc::new(LocalInvocationPlanCatalog::default()),
            diagnostics: DiagnosticConfiguration::default(),
        }
    }

    /// 使用高层建造器已经注册到 `IoC` 的同一组资源创建集合。
    pub(crate) fn managed(
        runtime: Arc<Handle>,
        cancellation: Arc<CancellationToken>,
        events: Arc<EventBus>,
        scope_cleanup_policy: Arc<ScopeCleanupPolicy>,
        invocation_plans: Arc<InvocationPlanCatalog>,
        local_invocation_plans: Arc<LocalInvocationPlanCatalog>,
        diagnostics: DiagnosticConfiguration,
    ) -> Self {
        Self {
            runtime: Some(runtime),
            cancellation,
            events,
            scope_cleanup_policy,
            invocation_plans,
            local_invocation_plans,
            diagnostics,
        }
    }

    /// 返回高层构建路径绑定的 Tokio Runtime Handle。
    pub(crate) fn runtime(&self) -> Option<&Handle> {
        self.runtime.as_deref()
    }

    /// 返回应用级共享取消令牌。
    pub(crate) fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// 返回 Context 独占的类型化事件总线。
    pub(crate) fn events(&self) -> &EventBus {
        &self.events
    }

    /// 返回应用作用域共享的清理等待策略。
    pub(crate) fn scope_cleanup_policy(&self) -> &ScopeCleanupPolicy {
        &self.scope_cleanup_policy
    }

    /// 返回应用构建阶段预编译的 AOP 调用计划目录。
    pub(crate) fn invocation_plans(&self) -> &InvocationPlanCatalog {
        &self.invocation_plans
    }

    /// 返回应用构建阶段预编译的 Local-AOP 调用计划目录。
    pub(crate) fn local_invocation_plans(&self) -> &LocalInvocationPlanCatalog {
        &self.local_invocation_plans
    }

    /// 返回应用构建阶段冻结的静态诊断配置。
    pub(crate) const fn diagnostics(&self) -> &DiagnosticConfiguration {
        &self.diagnostics
    }
}

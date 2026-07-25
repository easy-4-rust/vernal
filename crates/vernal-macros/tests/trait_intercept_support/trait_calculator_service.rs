//! Trait 默认方法织入合同测试组件对象。

use std::sync::Arc;

use vernal_aop::{CancellationToken, InvocationPlanCatalog};

use super::TraitCalculatorPort;

/// 只负责提供 Context-local AOP 资源、复用 Trait 默认业务实现的组件。
#[derive(vernal_macros::Component)]
#[component(aop)]
pub(crate) struct TraitCalculatorService {
    invocation_plans: Arc<InvocationPlanCatalog>,
    cancellation: Arc<CancellationToken>,
}

impl TraitCalculatorPort for TraitCalculatorService {}

//! 关联类型参数由宏投影为具名调用传输合同。

mod intercept_associated_output_support;

use vernal_aop::{
    AopComponent, CancellationToken, InvocationError, InvocationPlanCatalog,
};

use crate::intercept_associated_output_support::AssociatedProjection;

struct AssociatedOutputService {
    plans: InvocationPlanCatalog,
    cancellation: CancellationToken,
}

impl AopComponent for AssociatedOutputService {
    fn invocation_plans(&self) -> &InvocationPlanCatalog {
        &self.plans
    }

    fn invocation_cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl AssociatedOutputService {
    #[vernal_macros::intercept]
    async fn project<P>(&self, value: P) -> Result<P::Output, InvocationError>
    where
        P: AssociatedProjection,
    {
        Ok(value.project())
    }
}

fn compile_contract(service: &AssociatedOutputService) {
    let _future = service.project(String::from("vernal"));
}

fn main() {}

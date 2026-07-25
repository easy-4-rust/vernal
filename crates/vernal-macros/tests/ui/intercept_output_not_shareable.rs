//! 泛型成功返回值必须满足动态调用值合同。

use std::rc::Rc;

use vernal_aop::{
    AopComponent, CancellationToken, InvocationError, InvocationPlanCatalog,
};

struct OutputService {
    plans: InvocationPlanCatalog,
    cancellation: CancellationToken,
}

impl AopComponent for OutputService {
    fn invocation_plans(&self) -> &InvocationPlanCatalog {
        &self.plans
    }

    fn invocation_cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl OutputService {
    #[vernal_macros::intercept]
    async fn create<T>(&self) -> Result<T, InvocationError>
    where
        T: Default,
    {
        Ok(T::default())
    }
}

fn invoke(service: &OutputService) {
    let _future = service.create::<Rc<String>>();
}

fn main() {}

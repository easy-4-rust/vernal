//! 共享借用参数的目标类型必须满足 Sync。

use std::rc::Rc;

use vernal_aop::{
    AopComponent, CancellationToken, InvocationError, InvocationPlanCatalog,
};

struct SharedArgumentService {
    plans: InvocationPlanCatalog,
    cancellation: CancellationToken,
}

impl AopComponent for SharedArgumentService {
    fn invocation_plans(&self) -> &InvocationPlanCatalog {
        &self.plans
    }

    fn invocation_cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl SharedArgumentService {
    #[vernal_macros::intercept]
    async fn inspect<T>(&self, _value: &T) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn invoke(service: &SharedArgumentService) {
    let value = Rc::new(String::from("not-sync"));
    let _future = service.inspect(&value);
}

fn main() {}

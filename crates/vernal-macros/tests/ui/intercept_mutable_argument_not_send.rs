//! 可变借用参数的目标类型必须满足 Send。

use std::rc::Rc;

use vernal_aop::{
    AopComponent, CancellationToken, InvocationError, InvocationPlanCatalog,
};

struct MutableArgumentService {
    plans: InvocationPlanCatalog,
    cancellation: CancellationToken,
}

impl AopComponent for MutableArgumentService {
    fn invocation_plans(&self) -> &InvocationPlanCatalog {
        &self.plans
    }

    fn invocation_cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl MutableArgumentService {
    #[vernal_macros::intercept]
    async fn update<T>(&self, _value: &mut T) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn invoke(service: &MutableArgumentService) {
    let mut value = Rc::new(String::from("not-send"));
    let _future = service.update(&mut value);
}

fn main() {}

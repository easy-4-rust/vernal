//! owned 泛型参数不能进入 Send-AOP 静态目标。

use std::{
    rc::Rc,
    sync::Arc,
};

use vernal_aop::{
    AopComponent, CancellationToken, InvocationError, InvocationPlanCatalog,
};

struct OwnedArgumentService {
    plans: InvocationPlanCatalog,
    cancellation: CancellationToken,
}

impl AopComponent for OwnedArgumentService {
    fn invocation_plans(&self) -> &InvocationPlanCatalog {
        &self.plans
    }

    fn invocation_cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl OwnedArgumentService {
    #[vernal_macros::intercept]
    async fn accept<T>(self: Arc<Self>, _value: T) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn invoke(service: Arc<OwnedArgumentService>) {
    let _future = service.accept(Rc::new(String::from("not-send")));
}

fn main() {}

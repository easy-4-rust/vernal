//! 只有真正调用默认方法的实现类型需要提供 Context-local AOP 资源。
#![allow(async_fn_in_trait)]

use vernal_aop::InvocationError;

trait DefaultPort {
    #[vernal_macros::intercept]
    async fn execute(&self) -> Result<(), InvocationError> {
        Ok(())
    }
}

struct PlainService;

impl DefaultPort for PlainService {}

async fn invoke(service: &PlainService) {
    let _ = service.execute().await;
}

fn main() {}

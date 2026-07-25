//! 可变 receiver 暂不进入共享 AOP 计划。
#![allow(unused_imports)]

use vernal_aop::InvocationError;

struct MutableService;

impl MutableService {
    #[vernal_macros::intercept]
    async fn execute(&mut self) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn main() {}

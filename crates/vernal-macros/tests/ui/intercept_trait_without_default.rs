//! 抽象 Trait 方法没有可供 Around 链推进的最终业务目标。
#![allow(async_fn_in_trait, unused_imports)]

use vernal_aop::InvocationError;

trait AbstractPort {
    #[vernal_macros::intercept]
    async fn execute(&self) -> Result<(), InvocationError>;
}

fn main() {}

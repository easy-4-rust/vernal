//! 非异步方法不能进入 Tokio AOP 调用链。
#![allow(unused_imports)]

use std::sync::Arc;
use vernal_aop::InvocationError;

struct InvalidService;

impl InvalidService {
    #[vernal_macros::intercept]
    fn execute(self: Arc<Self>) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn main() {}

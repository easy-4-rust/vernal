//! 按值消费裸 Self 无法建立 Arc 所有权或当前调用期借用。
#![allow(unused_imports)]

use vernal_aop::InvocationError;

struct ValueService;

impl ValueService {
    #[vernal_macros::intercept]
    async fn execute(self) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn main() {}

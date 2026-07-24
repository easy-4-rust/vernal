//! 借用 receiver 不能跨越 `'static` InvocationTarget Future。
#![allow(unused_imports)]

use vernal_aop::InvocationError;

struct BorrowedService;

impl BorrowedService {
    #[vernal_macros::intercept]
    async fn execute(&self) -> Result<(), InvocationError> {
        Ok(())
    }
}

fn main() {}

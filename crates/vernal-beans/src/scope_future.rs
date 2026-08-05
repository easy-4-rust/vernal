//! 作用域关闭钩子类型。

use std::future::Future;
use std::pin::Pin;

/// 作用域关闭时执行的异步钩子工厂。
///
/// 返回一个 Future，在作用域关闭时执行。
pub type ScopeCloseHook = Box<
    dyn FnOnce() -> Pin<
            Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
        > + Send,
>;

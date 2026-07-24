//! 自定义作用域后台关闭失败对象。

use vernal_core::SharedError;

/// 在取消安全的后台关闭任务中保存可克隆失败原因。
///
/// 多个并发 `close()` 调用者会等待同一个关闭结果，因此内部错误必须能够安全
/// 克隆。该对象只在 `IoC` 内核中传播，公开边界仍返回结构化 [`crate::ScopeError`]。
#[derive(Clone)]
pub(crate) enum ScopeCloseFailure {
    /// 用户注册的异步关闭钩子返回错误。
    Hook(SharedError),
    /// 承载关闭钩子的 Tokio 子任务发生 panic 或被 Runtime 取消。
    Task(SharedError),
}

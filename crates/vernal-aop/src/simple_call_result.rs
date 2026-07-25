//! 简化拦截器的调用结果。
//!
//! 对标 tx_di 的 `CallResult`：标记方法是否成功执行。
//! 不携带返回值（与完整 AOP 的 `InvocationOutput` 不同），保持零开销。

use vernal_core::BoxError;

/// 简化拦截器的调用结果。
///
/// 标记被拦截方法是否成功执行。`before` 钩子返回 `Err` 将阻止方法执行；
/// `after` 钩子可以检查结果状态。
///
/// # 与完整 AOP 的区别
///
/// 完整 AOP 的 `InvocationOutput` 携带 `Box<dyn Any + Send + Sync>` 类型的返回值。
/// `SimpleCallResult` 只标记成功/失败状态，适用于不需要修改返回值的场景。
#[derive(Debug)]
pub struct SimpleCallResult {
    /// 是否成功执行
    success: bool,
    /// 失败时的错误（成功时为 None）
    error: Option<BoxError>,
}

impl SimpleCallResult {
    /// 创建成功结果。
    #[must_use]
    pub fn ok() -> Self {
        Self {
            success: true,
            error: None,
        }
    }

    /// 创建失败结果。
    #[must_use]
    pub fn err(error: BoxError) -> Self {
        Self {
            success: false,
            error: Some(error),
        }
    }

    /// 是否成功执行。
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.success
    }

    /// 是否执行失败。
    #[must_use]
    pub fn is_err(&self) -> bool {
        !self.success
    }

    /// 获取失败原因（如果有）。
    #[must_use]
    pub fn error(&self) -> Option<&BoxError> {
        self.error.as_ref()
    }
}

//! Gotham 原生 Handler 错误标记对象。

use std::{error::Error, fmt};

/// 标记 AOP 目标已经产生 Gotham 原生 `HandlerError`。
///
/// 实际错误连同 `State` 保留在借用型目标中；该零数据标记只用于让调用计划的
/// After/Exception 拦截器观察失败，并在链结束后恢复原生错误。
#[derive(Debug)]
pub(crate) struct GothamUpstreamError;

impl fmt::Display for GothamUpstreamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Gotham handler returned a native error")
    }
}

impl Error for GothamUpstreamError {}

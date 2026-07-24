//! 测试用 Tower 错误映射策略对象。

use std::io;

use http::{Response, StatusCode};
use vernal_tower::TowerErrorMapper;

/// 在“恢复成响应”和“转换成新错误”两种模式间切换的测试 Mapper。
#[derive(Clone, Copy)]
pub struct TestErrorMapper {
    recover: bool,
}

impl TestErrorMapper {
    /// 创建把错误恢复成响应的 Mapper。
    pub const fn recovering() -> Self {
        Self { recover: true }
    }

    /// 创建把错误转换成新错误的 Mapper。
    pub const fn transforming() -> Self {
        Self { recover: false }
    }
}

impl TowerErrorMapper for TestErrorMapper {
    type Source = io::Error;
    type Response = Response<u8>;
    type Error = io::Error;

    fn map_error(&self, error: Self::Source) -> Result<Self::Response, Self::Error> {
        if self.recover {
            Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header("x-vernal-mapped", "true")
                .body(42)
                .expect("valid test response"))
        } else {
            Err(io::Error::other(format!("mapped: {error}")))
        }
    }
}

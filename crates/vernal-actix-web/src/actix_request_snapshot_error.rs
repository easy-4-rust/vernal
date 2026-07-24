//! Actix 请求快照转换错误对象。

use std::{error::Error, fmt};

/// Actix HTTP 0.2 元数据转换到 Vernal HTTP 1.x 合同时的失败。
#[derive(Debug)]
pub struct ActixRequestSnapshotError {
    field: &'static str,
    source: Box<dyn Error + Send + Sync>,
}

impl ActixRequestSnapshotError {
    /// 创建带稳定字段名的转换错误。
    pub(crate) fn new(field: &'static str, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            field,
            source: Box::new(source),
        }
    }
}

impl fmt::Display for ActixRequestSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid Actix request {} metadata", self.field)
    }
}

impl Error for ActixRequestSnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

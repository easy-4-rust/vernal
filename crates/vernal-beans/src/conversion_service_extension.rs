//! ConversionServiceExtension — 转换服务扩展。
use std::any::TypeId;

/// 转换服务扩展。
#[derive(Clone, Debug, Default)]
pub struct ConversionServiceExtension;
impl ConversionServiceExtension {
    pub fn new() -> Self { Self }
    pub fn can_convert(&self, _source: TypeId, _target: TypeId) -> bool { false }
}

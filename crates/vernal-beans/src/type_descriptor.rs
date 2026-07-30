//! TypeDescriptor — 类型描述符。
use std::any::TypeId;

/// 类型描述符。
#[derive(Clone, Debug)]
pub struct TypeDescriptor {
    pub type_name: String,
    pub type_id: TypeId,
}
impl TypeDescriptor {
    pub fn new(type_name: impl Into<String>, type_id: TypeId) -> Self {
        Self { type_name: type_name.into(), type_id }
    }
    pub fn type_name(&self) -> &str { &self.type_name }
    pub fn type_id(&self) -> TypeId { self.type_id }
}

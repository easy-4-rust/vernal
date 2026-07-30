//! SerializableTypeWrapper — 可序列化类型包装器。
#[derive(Clone, Debug)]
pub struct SerializableTypeWrapper {
    pub type_name: String,
}
impl SerializableTypeWrapper {
    pub fn new(type_name: impl Into<String>) -> Self { Self { type_name: type_name.into() } }
    pub fn type_name(&self) -> &str { &self.type_name }
}

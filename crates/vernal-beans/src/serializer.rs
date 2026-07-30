//! Serializer — 序列化器 trait。
/// 序列化器 trait。
pub trait Serializer: Send + Sync {
    fn serialize(&self, value: &dyn std::any::Any) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

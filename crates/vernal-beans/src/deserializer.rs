//! Deserializer — 反序列化器 trait。
/// 反序列化器 trait。
pub trait Deserializer: Send + Sync {
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

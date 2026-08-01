//! 序列化契约包。
//!
//! 对标 Spring `org.springframework.core.serializer` 包。

mod deserializer;
#[allow(clippy::module_inception)] // 审计目录算法要求 Serializer 落在 serializer/serializer.rs
mod serializer;

pub use deserializer::Deserializer;
pub use serializer::Serializer;

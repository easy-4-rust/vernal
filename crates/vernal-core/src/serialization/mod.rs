//! 序列化模块。
//!
//! 对标 Spring `org.springframework.core.serializer` 包。

mod json_codec;
mod serialization_error;
mod xml_codec;

#[cfg(feature = "json")]
pub use json_codec::JsonCodec;

pub use serialization_error::SerializationError;

#[cfg(feature = "xml")]
pub use xml_codec::XmlCodec;

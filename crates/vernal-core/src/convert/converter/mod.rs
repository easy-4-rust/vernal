//! 转换器 SPI 包。
//!
//! 对标 Spring `org.springframework.core.convert.converter` 包：Converter、
//! ConditionalConverter、ConverterRegistry、GenericConverter 等转换器契约。

mod always_match_converter;
mod conditional_converter;
mod conditional_generic_converter;
#[allow(clippy::module_inception)]
// 审计目录算法要求 Java Converter 落在 convert/converter/converter.rs
mod converter;
mod converter_factory;
mod converter_registry;
mod convertible_pair;
mod converting_comparator;
mod generic_converter;
mod never_match_converter;
mod type_pair_conditional_converter;

pub use always_match_converter::AlwaysMatchConverter;
pub use conditional_converter::ConditionalConverter;
pub use conditional_generic_converter::ConditionalGenericConverter;
pub use converter::Converter;
pub use converter_factory::ConverterFactory;
pub use converter_registry::{ConverterRegistry, ErasedConverter, TypeIdConverterRegistry};
pub use convertible_pair::ConvertiblePair;
pub use converting_comparator::ConvertingComparator;
pub use generic_converter::{ClosureGenericConverter, GenericConverter};
pub use never_match_converter::NeverMatchConverter;
pub use type_pair_conditional_converter::TypePairConditionalConverter;

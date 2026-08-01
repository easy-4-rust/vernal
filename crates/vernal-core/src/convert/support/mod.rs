//! 转换器支持实现包。
//!
//! 对标 Spring `org.springframework.core.convert.support` 包：`DefaultConversionService`
//! 及其内置的 `StringTo*Converter` / `*To*Converter` 系列。实现以 vernal-core 的
//! [`crate::convert::Convertible`] 静态分发与 [`crate::convert::converter::ConverterRegistry`]
//! 运行时注册为基础，保留 Spring 类名与转换语义。

mod abstract_conditional_enum_converter;
mod array_to_array_converter;
mod array_to_collection_converter;
mod array_to_object_converter;
mod array_to_string_converter;
mod character_to_number_factory;
mod collection_to_array_converter;
mod collection_to_collection_converter;
mod collection_to_object_converter;
mod collection_to_string_converter;
mod configurable_conversion_service;
mod conversion_service_factory;
mod conversion_utils;
mod default_conversion_service;
mod enum_to_integer_converter;
mod enum_to_string_converter;
mod fallback_object_to_string_converter;
mod generic_conversion_service;
mod integer_to_enum_converter_factory;
mod map_to_map_converter;
mod number_to_character_converter;
mod number_to_number_converter_factory;
mod object_to_array_converter;
mod object_to_collection_converter;
mod object_to_object_converter;
mod object_to_optional_converter;
mod object_to_string_converter;
mod optional_to_object_converter;
mod properties_to_string_converter;
mod stream_converter;
mod string_to_array_converter;
mod string_to_boolean_converter;
mod string_to_character_converter;
mod string_to_collection_converter;
mod string_to_enum_converter_factory;
mod string_to_number_converter_factory;
mod string_to_properties_converter;

#[cfg(feature = "convert-regex")]
mod string_to_pattern_converter;
#[cfg(feature = "convert-regex")]
mod string_to_regex_converter;
#[cfg(feature = "convert-uuid")]
mod string_to_uuid_converter;

pub use abstract_conditional_enum_converter::AbstractConditionalEnumConverter;
pub use array_to_array_converter::ArrayToArrayConverter;
pub use array_to_collection_converter::ArrayToCollectionConverter;
pub use array_to_object_converter::ArrayToObjectConverter;
pub use array_to_string_converter::ArrayToStringConverter;
pub use character_to_number_factory::CharacterToNumberFactory;
pub use collection_to_array_converter::CollectionToArrayConverter;
pub use collection_to_collection_converter::CollectionToCollectionConverter;
pub use collection_to_object_converter::CollectionToObjectConverter;
pub use collection_to_string_converter::CollectionToStringConverter;
pub use configurable_conversion_service::ConfigurableConversionService;
pub use conversion_service_factory::ConversionServiceFactory;
pub use conversion_utils::ConversionUtils;
pub use default_conversion_service::DefaultConversionService;
pub use enum_to_integer_converter::EnumToIntegerConverter;
pub use enum_to_string_converter::EnumToStringConverter;
pub use fallback_object_to_string_converter::FallbackObjectToStringConverter;
pub use generic_conversion_service::GenericConversionService;
pub use integer_to_enum_converter_factory::IntegerToEnumConverterFactory;
pub use map_to_map_converter::MapToMapConverter;
pub use number_to_character_converter::NumberToCharacterConverter;
pub use number_to_number_converter_factory::NumberToNumberConverterFactory;
pub use object_to_array_converter::ObjectToArrayConverter;
pub use object_to_collection_converter::ObjectToCollectionConverter;
pub use object_to_object_converter::ObjectToObjectConverter;
pub use object_to_optional_converter::ObjectToOptionalConverter;
pub use object_to_string_converter::ObjectToStringConverter;
pub use optional_to_object_converter::OptionalToObjectConverter;
pub use properties_to_string_converter::PropertiesToStringConverter;
pub use stream_converter::StreamConverter;
pub use string_to_array_converter::StringToArrayConverter;
pub use string_to_boolean_converter::StringToBooleanConverter;
pub use string_to_character_converter::StringToCharacterConverter;
pub use string_to_collection_converter::StringToCollectionConverter;
pub use string_to_enum_converter_factory::StringToEnumConverterFactory;
pub use string_to_number_converter_factory::StringToNumberConverterFactory;
pub use string_to_properties_converter::StringToPropertiesConverter;

#[cfg(feature = "convert-regex")]
pub use string_to_pattern_converter::StringToPatternConverter;
#[cfg(feature = "convert-regex")]
pub use string_to_regex_converter::StringToRegexConverter;
#[cfg(feature = "convert-uuid")]
pub use string_to_uuid_converter::StringToUUIDConverter;

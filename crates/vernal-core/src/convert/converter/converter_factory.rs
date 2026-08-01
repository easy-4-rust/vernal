//! 转换器工厂 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.ConverterFactory`。

use super::Converter;

/// 转换器工厂 trait。
///
/// 对应 Java: org.springframework.core.convert.converter.ConverterFactory
///
/// Spring 语义：一个工厂为同一源类型的多个目标类型生成转换器
/// （`<S, R> Converter<S, T> getConverter(Class<T>)`，其中 `T extends R`）。
/// Rust 中目标类型族以 [`crate::convert::Convertible`] 表达（对标 Spring 的
/// `Number`/枚举等目标类型族），编译期检查。
pub trait ConverterFactory<S> {
    /// 为目标类型 `T` 生成转换器。
    ///
    /// 对标 Spring `getConverter(Class<T> targetType)`。
    fn get_converter<T: crate::convert::Convertible>(&self) -> impl Converter<S, T>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::ConversionError;

    struct IdentityFactory;

    struct DelegateConverter<T>(std::marker::PhantomData<T>);

    impl<T: crate::convert::Convertible> Converter<&str, T> for DelegateConverter<T> {
        fn convert(&self, source: &str) -> Result<T, ConversionError> {
            T::from_str_value(source)
        }
    }

    impl<'a> ConverterFactory<&'a str> for IdentityFactory {
        fn get_converter<T: crate::convert::Convertible>(&self) -> impl Converter<&'a str, T> {
            DelegateConverter(std::marker::PhantomData)
        }
    }

    #[test]
    fn converter_factory_is_implementable() {
        // D 类（重构安全）：trait 契约可被自定义工厂实现并产出可用转换器
        let factory = IdentityFactory;
        let converter = factory.get_converter::<i32>();
        assert_eq!(converter.convert("7").unwrap(), 7);
    }
}

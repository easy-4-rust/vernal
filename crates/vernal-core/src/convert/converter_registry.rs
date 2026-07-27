//! 转换器注册表 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.ConverterRegistry`。
//!
//! # 设计来源
//!
//! Spring 的 `ConverterRegistry` 是一个运行时可变的转换器注册中心,允许:
//! - `addConverter(converter)`:根据泛型参数自动推断 source/target 类型
//! - `addConverter(sourceType, targetType, converter)`:显式指定类型对
//! - `addConverterFactory(factory)`:添加类型族工厂
//! - `removeConvertible(sourceType, targetType)`:移除指定类型对
//!
//! # Rust 适配
//!
//! Rust 的 `Convertible` trait 默认通过静态分发工作(零运行时开销),
//! 但有些场景需要运行时注册(如插件系统、动态配置)。
//! vernal-core 通过 `ConverterRegistry` trait 提供运行时注册能力,
//! 用 `TypeId` 作为类型键。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

use super::ConversionError;

/// 转换器注册表 trait。
///
/// 对标 Spring `ConverterRegistry` 接口。
/// 允许在运行时注册类型擦除的转换器。
///
/// # 示例
///
/// ```rust
/// use vernal_core::convert::{ConverterRegistry, TypeIdConverterRegistry, Converter, ConversionError};
///
/// let registry = TypeIdConverterRegistry::new();
/// registry.add_converter::<String, i64>(Box::new(|s: &str| s.parse::<i64>().map_err(|e| ConversionError {
///     value: s.to_string(),
///     target_type: "i64",
///     reason: e.to_string(),
/// })));
/// ```
pub trait ConverterRegistry: Send + Sync {
    /// 注册一个类型擦除的转换器。
    ///
    /// 对标 Spring `<S, T> void addConverter(Class<S> sourceType, Class<T> targetType, Converter<? super S, ? extends T> converter)`。
    fn add_converter(
        &self,
        source_type: TypeId,
        target_type: TypeId,
        converter: Box<dyn Fn(&str) -> Result<String, ConversionError> + Send + Sync>,
    );

    /// 移除指定类型对的转换器。
    ///
    /// 对标 Spring `void removeConvertible(Class<?> sourceType, Class<?> targetType)`。
    fn remove_convertible(&self, source_type: TypeId, target_type: TypeId);

    /// 判断指定类型对是否有注册的转换器。
    ///
    /// 对标 Spring `boolean canConvert(Class<?> sourceType, Class<?> targetType)`(简版)。
    fn can_convert(&self, source_type: TypeId, target_type: TypeId) -> bool;
}

/// 基于 `TypeId` 的运行时转换器注册表实现。
///
/// 内部使用 `Mutex<HashMap<(TypeId, TypeId), Box<dyn Fn>>>` 存储类型擦除的转换器。
///
/// # 线程安全
///
/// 通过 `Mutex` 保护,可跨线程共享(`Arc<TypeIdConverterRegistry>`)。
pub struct TypeIdConverterRegistry {
    converters: Mutex<
        HashMap<
            (TypeId, TypeId),
            Box<dyn Fn(&str) -> Result<String, ConversionError> + Send + Sync>,
        >,
    >,
}

impl TypeIdConverterRegistry {
    /// 创建空的注册表。
    #[must_use]
    pub fn new() -> Self {
        Self {
            converters: Mutex::new(HashMap::new()),
        }
    }

    /// 调用注册的转换器把字符串转换为目标类型(返回字符串形式的结果)。
    ///
    /// 对标 Spring `ConversionService.convert(source, targetType)`。
    ///
    /// # 错误
    ///
    /// 如果类型对未注册,返回 `ConversionError`。
    pub fn convert(
        &self,
        source: &str,
        source_type: TypeId,
        target_type: TypeId,
    ) -> Result<String, ConversionError> {
        let map = self.converters.lock().unwrap();
        match map.get(&(source_type, target_type)) {
            Some(converter) => converter(source),
            None => Err(ConversionError {
                value: source.to_string(),
                target_type: "dynamic",
                reason: format!("no converter registered for {source_type:?} -> {target_type:?}"),
            }),
        }
    }
}

impl Default for TypeIdConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ConverterRegistry for TypeIdConverterRegistry {
    fn add_converter(
        &self,
        source_type: TypeId,
        target_type: TypeId,
        converter: Box<dyn Fn(&str) -> Result<String, ConversionError> + Send + Sync>,
    ) {
        let mut map = self.converters.lock().unwrap();
        map.insert((source_type, target_type), converter);
    }

    fn remove_convertible(&self, source_type: TypeId, target_type: TypeId) {
        let mut map = self.converters.lock().unwrap();
        map.remove(&(source_type, target_type));
    }

    fn can_convert(&self, source_type: TypeId, target_type: TypeId) -> bool {
        let map = self.converters.lock().unwrap();
        map.contains_key(&(source_type, target_type))
    }
}

impl std::fmt::Debug for TypeIdConverterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = self.converters.lock().unwrap();
        f.debug_struct("TypeIdConverterRegistry")
            .field("registered_count", &map.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_registry() {
        let registry = TypeIdConverterRegistry::new();
        let map = registry.converters.lock().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn add_and_query_converter() {
        let registry = TypeIdConverterRegistry::new();
        let src = TypeId::of::<String>();
        let dst = TypeId::of::<i64>();

        assert!(!registry.can_convert(src, dst));

        registry.add_converter(
            src,
            dst,
            Box::new(|s: &str| Ok(s.parse::<i64>().unwrap_or(0).to_string())),
        );

        assert!(registry.can_convert(src, dst));
    }

    #[test]
    fn convert_uses_registered_converter() {
        let registry = TypeIdConverterRegistry::new();
        registry.add_converter(
            TypeId::of::<String>(),
            TypeId::of::<i64>(),
            Box::new(|s: &str| Ok(s.parse::<i64>().unwrap_or(0).to_string())),
        );

        let result = registry
            .convert("42", TypeId::of::<String>(), TypeId::of::<i64>())
            .unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn convert_returns_error_for_unregistered() {
        let registry = TypeIdConverterRegistry::new();
        let err = registry
            .convert("x", TypeId::of::<String>(), TypeId::of::<bool>())
            .unwrap_err();
        assert!(err.reason.contains("no converter"));
    }

    #[test]
    fn remove_converter_clears_entry() {
        let registry = TypeIdConverterRegistry::new();
        let src = TypeId::of::<String>();
        let dst = TypeId::of::<i64>();

        registry.add_converter(src, dst, Box::new(|_| Ok("0".to_string())));
        assert!(registry.can_convert(src, dst));

        registry.remove_convertible(src, dst);
        assert!(!registry.can_convert(src, dst));
    }

    #[test]
    fn registry_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TypeIdConverterRegistry>();
    }

    #[test]
    fn registry_can_be_shared_via_arc() {
        use std::sync::Arc;
        let registry = Arc::new(TypeIdConverterRegistry::new());
        let r1 = Arc::clone(&registry);
        let r2 = Arc::clone(&registry);

        let h1 = std::thread::spawn(move || {
            r1.add_converter(
                TypeId::of::<String>(),
                TypeId::of::<i64>(),
                Box::new(|s: &str| Ok(s.to_string())),
            );
        });
        let h2 = std::thread::spawn(move || {
            r2.add_converter(
                TypeId::of::<String>(),
                TypeId::of::<bool>(),
                Box::new(|s: &str| Ok(s.to_string())),
            );
        });
        h1.join().unwrap();
        h2.join().unwrap();

        assert!(registry.can_convert(TypeId::of::<String>(), TypeId::of::<i64>()));
        assert!(registry.can_convert(TypeId::of::<String>(), TypeId::of::<bool>()));
    }

    #[test]
    fn debug_format_shows_count() {
        let registry = TypeIdConverterRegistry::new();
        registry.add_converter(
            TypeId::of::<String>(),
            TypeId::of::<i64>(),
            Box::new(|_| Ok("0".to_string())),
        );
        let s = format!("{registry:?}");
        assert!(s.contains("TypeIdConverterRegistry"));
        assert!(s.contains("registered_count"));
    }
}

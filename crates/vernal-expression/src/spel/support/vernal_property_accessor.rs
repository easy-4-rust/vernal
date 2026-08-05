//! Vernal 属性访问器（对标 Spring BeanFactory 属性集成）。
//!
//! `VernalPropertyAccessor` 通过可注入的闭包实现属性读写，
//! 允许在运行时从 IoC 容器（如 `vernal-beans::ComponentRegistry`）解析属性。
//!
//! 对标 Spring 中 `BeanFactory` 与 `PropertyAccessor` 的集成：
//! 当表达式根对象是 Bean 时，通过 BeanFactory 解析其属性。
//!
//! 对应 Java 类：Spring 没有直接等价类，概念上对标 BeanFactory 属性注入。

use std::sync::{Arc, RwLock};

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::property_accessor::PropertyAccessor;
use crate::typed_value::TypedValue;

/// 属性读取闭包类型。
type PropertyReadFn = Arc<dyn Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync>;

/// 属性写入闭包类型。
type PropertyWriteFn = Arc<dyn Fn(&str, &TypedValue) -> Result<(), AccessException> + Send + Sync>;

/// Vernal 属性访问器（对标 Spring BeanFactory 属性集成）。
///
/// 通过可注入的闭包实现属性读写。用户可以在构造时提供读写闭包，
/// 该闭包内部可以调用 `vernal-beans::ComponentRegistry`
/// 或任何其他 IoC 容器来解析/设置属性。
///
/// # 与 Spring 的关系
///
/// Spring 中 `PropertyAccessor` 可以通过 `BeanFactory` 解析 Bean 属性。
/// Rust 中通过闭包实现：用户构造 `VernalPropertyAccessor` 时提供
/// 读写闭包，闭包内部可以调用任何 IoC 容器。
///
/// # 使用场景
///
/// 在 SpEL 表达式中访问 IoC 容器管理的 Bean 的属性。
///
/// # 示例
///
/// ```rust
/// use vernal_expression::spel::support::vernal_property_accessor::VernalPropertyAccessor;
/// use vernal_expression::PropertyAccessor;
/// use vernal_expression::EvaluationContext;
/// use vernal_expression::{TypedValue, ExpressionValue, TypeDescriptor};
/// use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
///
/// let accessor = VernalPropertyAccessor::new(
///     |name| match name {
///         "greeting" => Ok(TypedValue::new(
///             ExpressionValue::String("Hello!".into()),
///             TypeDescriptor::STRING,
///         )),
///         _ => Err(vernal_expression::AccessException::new(
///             format!("Property '{}' not found", name),
///         )),
///     },
/// );
///
/// let ctx = StandardEvaluationContext::new(TypedValue::null());
/// let target = TypedValue::null();
/// let result = accessor.read(&ctx, &target, "greeting").unwrap();
/// assert_eq!(*result.value(), ExpressionValue::String("Hello!".into()));
/// ```
pub struct VernalPropertyAccessor {
    /// 属性读取闭包。
    read_fn: RwLock<Option<PropertyReadFn>>,
    /// 属性写入闭包。
    write_fn: RwLock<Option<PropertyWriteFn>>,
}

impl VernalPropertyAccessor {
    /// 创建新的 Vernal 属性访问器（只读）。
    ///
    /// # 参数
    ///
    /// - `reader` — 属性读取闭包
    pub fn new<F>(reader: F) -> Self
    where
        F: Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        Self {
            read_fn: RwLock::new(Some(Arc::new(reader))),
            write_fn: RwLock::new(None),
        }
    }

    /// 创建新的 Vernal 属性访问器（读写）。
    ///
    /// # 参数
    ///
    /// - `reader` — 属性读取闭包
    /// - `writer` — 属性写入闭包
    pub fn with_write<F, G>(reader: F, writer: G) -> Self
    where
        F: Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
        G: Fn(&str, &TypedValue) -> Result<(), AccessException> + Send + Sync + 'static,
    {
        Self {
            read_fn: RwLock::new(Some(Arc::new(reader))),
            write_fn: RwLock::new(Some(Arc::new(writer))),
        }
    }

    /// 创建空的 Vernal 属性访问器（未配置闭包）。
    pub fn empty() -> Self {
        Self {
            read_fn: RwLock::new(None),
            write_fn: RwLock::new(None),
        }
    }

    /// 设置属性读取闭包。
    pub fn set_reader<F>(&self, reader: F)
    where
        F: Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        let mut guard = self.read_fn.write().unwrap();
        *guard = Some(Arc::new(reader));
    }

    /// 设置属性写入闭包。
    pub fn set_writer<F>(&self, writer: F)
    where
        F: Fn(&str, &TypedValue) -> Result<(), AccessException> + Send + Sync + 'static,
    {
        let mut guard = self.write_fn.write().unwrap();
        *guard = Some(Arc::new(writer));
    }
}

impl PropertyAccessor for VernalPropertyAccessor {
    /// 检查是否可以读取属性。
    ///
    /// 如果配置了读取闭包，则总是返回 `true`（由闭包决定是否能实际读取）。
    fn can_read(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
    ) -> bool {
        let guard = self.read_fn.read().unwrap();
        guard.is_some()
    }

    /// 读取属性。
    fn read(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException> {
        let guard = self.read_fn.read().unwrap();
        if let Some(reader) = guard.as_ref() {
            reader(name)
        } else {
            Err(AccessException::new(&format!(
                "No property reader configured: cannot read property '{}'",
                name
            )))
        }
    }

    /// 检查是否可以写入属性。
    fn can_write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
    ) -> bool {
        let guard = self.write_fn.read().unwrap();
        guard.is_some()
    }

    /// 写入属性。
    fn write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        name: &str,
        value: &TypedValue,
    ) -> Result<(), AccessException> {
        let guard = self.write_fn.read().unwrap();
        if let Some(writer) = guard.as_ref() {
            writer(name, value)
        } else {
            Err(AccessException::new(&format!(
                "No property writer configured: cannot write property '{}'",
                name
            )))
        }
    }

    fn specific_target_classes(&self) -> &[&str] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::{ExpressionValue, TypeDescriptor};

    #[test]
    fn new_read_only() {
        let accessor = VernalPropertyAccessor::new(|name| {
            Ok(TypedValue::new(
                ExpressionValue::String(format!("val:{}", name)),
                TypeDescriptor::STRING,
            ))
        });
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::null();

        assert!(accessor.can_read(&ctx, &target, "prop"));
        assert!(!accessor.can_write(&ctx, &target, "prop"));

        let result = accessor.read(&ctx, &target, "prop").unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("val:prop".into()));
    }

    #[test]
    fn new_read_write() {
        let accessor = VernalPropertyAccessor::with_write(
            |name| {
                Ok(TypedValue::new(
                    ExpressionValue::String(format!("read:{}", name)),
                    TypeDescriptor::STRING,
                ))
            },
            |_name, _value| Ok(()),
        );
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::null();

        assert!(accessor.can_read(&ctx, &target, "prop"));
        assert!(accessor.can_write(&ctx, &target, "prop"));

        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        assert!(accessor.write(&ctx, &target, "prop", &value).is_ok());
    }

    #[test]
    fn empty_accessor_returns_error() {
        let accessor = VernalPropertyAccessor::empty();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::null();

        assert!(!accessor.can_read(&ctx, &target, "prop"));
        assert!(!accessor.can_write(&ctx, &target, "prop"));
        assert!(accessor.read(&ctx, &target, "prop").is_err());
        assert!(
            accessor
                .write(&ctx, &target, "prop", &TypedValue::null())
                .is_err()
        );
    }

    #[test]
    fn set_reader_dynamically() {
        let accessor = VernalPropertyAccessor::empty();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::null();

        assert!(!accessor.can_read(&ctx, &target, "prop"));

        accessor.set_reader(|name| {
            Ok(TypedValue::new(
                ExpressionValue::String(format!("dynamic:{}", name)),
                TypeDescriptor::STRING,
            ))
        });

        assert!(accessor.can_read(&ctx, &target, "prop"));
        let result = accessor.read(&ctx, &target, "prop").unwrap();
        assert_eq!(
            *result.value(),
            ExpressionValue::String("dynamic:prop".into())
        );
    }

    #[test]
    fn set_writer_dynamically() {
        let accessor = VernalPropertyAccessor::empty();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::null();

        assert!(!accessor.can_write(&ctx, &target, "prop"));

        accessor.set_writer(|_name, _value| Ok(()));

        assert!(accessor.can_write(&ctx, &target, "prop"));
        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        assert!(accessor.write(&ctx, &target, "prop", &value).is_ok());
    }

    #[test]
    fn specific_target_classes_empty() {
        let accessor = VernalPropertyAccessor::empty();
        assert!(accessor.specific_target_classes().is_empty());
    }
}

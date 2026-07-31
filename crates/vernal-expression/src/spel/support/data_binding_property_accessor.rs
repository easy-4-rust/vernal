//! 数据绑定属性访问器（对标 Spring `DataBindingPropertyAccessor`）。
//!
//! `DataBindingPropertyAccessor` 是 `ReflectivePropertyAccessor` 的特化版本，
//! 专为数据绑定场景设计。与 `ReflectivePropertyAccessor` 不同，它：
//!
//! - 不解析 `Object`、`Class`、`ClassLoader` 上的技术属性
//! - 仅允许访问用户声明的属性
//!
//! 对应 Java 类：`org.springframework.expression.spel.support.DataBindingPropertyAccessor`。

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::property_accessor::PropertyAccessor;
use crate::typed_value::TypedValue;

use super::reflective_property_accessor::ReflectivePropertyAccessor;

/// 数据绑定属性访问器（对标 Spring `DataBindingPropertyAccessor`）。
///
/// 仅解析用户声明的属性，排除 `Object`、`Class`、`ClassLoader` 上的技术属性。
///
/// # 与 Spring 的关系
///
/// Spring `DataBindingPropertyAccessor` 继承 `ReflectivePropertyAccessor`，
/// 重写 `isCandidateForProperty()` 以过滤技术方法。
///
/// Rust 中通过组合（持有 `ReflectivePropertyAccessor`）实现，
/// 在 `read()`/`write()` 中添加类型过滤。
///
/// # 使用场景
///
/// 适用于 `SimpleEvaluationContext` 等数据绑定场景，
/// 不允许访问 `Object.getClass()`、`Class.getName()` 等技术属性。
pub struct DataBindingPropertyAccessor {
    /// 内部委托的反射属性访问器。
    delegate: ReflectivePropertyAccessor,
}

impl DataBindingPropertyAccessor {
    /// 创建新的数据绑定属性访问器。
    ///
    /// # 参数
    ///
    /// - `allow_write` — 是否允许写操作
    fn new(_allow_write: bool) -> Self {
        Self {
            delegate: ReflectivePropertyAccessor::new(),
        }
    }

    /// 创建只读数据绑定属性访问器。
    ///
    /// 对标 Spring `DataBindingPropertyAccessor.forReadOnlyAccess()`。
    pub fn for_read_only_access() -> Self {
        Self::new(false)
    }

    /// 创建读写数据绑定属性访问器。
    ///
    /// 对标 Spring `DataBindingPropertyAccessor.forReadWriteAccess()`。
    pub fn for_read_write_access() -> Self {
        Self::new(true)
    }

    /// 检查类型名是否为技术类型（Object、Class、ClassLoader）。
    ///
    /// 对标 Spring `DataBindingPropertyAccessor.isCandidateForProperty()` 中的
    /// `clazz != Object.class && clazz != Class.class && !ClassLoader.class.isAssignableFrom(targetClass)`。
    fn is_technical_type(type_name: &str) -> bool {
        matches!(
            type_name,
            "Object"
                | "java.lang.Object"
                | "Class"
                | "java.lang.Class"
                | "ClassLoader"
                | "java.lang.ClassLoader"
        )
    }
}

impl PropertyAccessor for DataBindingPropertyAccessor {
    /// 检查是否可以读取目标对象的指定属性。
    ///
    /// 对标 Spring `DataBindingPropertyAccessor.canRead()`（继承自 `ReflectivePropertyAccessor`）。
    fn can_read(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> bool {
        let type_name = target.type_descriptor().name();
        if Self::is_technical_type(&type_name) {
            return false;
        }
        self.delegate.can_read(context, target, name)
    }

    /// 读取目标对象的指定属性。
    ///
    /// 对标 Spring `DataBindingPropertyAccessor.read()`（继承自 `ReflectivePropertyAccessor`）。
    fn read(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException> {
        let type_name = target.type_descriptor().name();
        if Self::is_technical_type(&type_name) {
            return Err(AccessException::new(&format!(
                "DataBindingPropertyAccessor does not support properties on type '{}'",
                type_name
            )));
        }
        self.delegate.read(context, target, name)
    }

    /// 检查是否可以写入目标对象的指定属性。
    fn can_write(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> bool {
        let type_name = target.type_descriptor().name();
        if Self::is_technical_type(&type_name) {
            return false;
        }
        self.delegate.can_write(context, target, name)
    }

    /// 写入目标对象的指定属性。
    fn write(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
        value: &TypedValue,
    ) -> Result<(), AccessException> {
        let type_name = target.type_descriptor().name();
        if Self::is_technical_type(&type_name) {
            return Err(AccessException::new(&format!(
                "DataBindingPropertyAccessor does not support properties on type '{}'",
                type_name
            )));
        }
        self.delegate.write(context, target, name, value)
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
    fn for_read_only_access() {
        let accessor = DataBindingPropertyAccessor::for_read_only_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        assert!(!accessor.can_write(&ctx, &target, "value"));
    }

    #[test]
    fn for_read_write_access() {
        let accessor = DataBindingPropertyAccessor::for_read_write_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        assert!(!accessor.can_write(&ctx, &target, "nonexistent"));
    }

    #[test]
    fn read_on_object_type_returns_error() {
        let accessor = DataBindingPropertyAccessor::for_read_only_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("Object"),
        );
        let result = accessor.read(&ctx, &target, "class");
        assert!(result.is_err());
    }

    #[test]
    fn read_on_class_type_returns_error() {
        let accessor = DataBindingPropertyAccessor::for_read_only_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("Class"),
        );
        let result = accessor.read(&ctx, &target, "name");
        assert!(result.is_err());
    }

    #[test]
    fn can_read_on_object_type_returns_false() {
        let accessor = DataBindingPropertyAccessor::for_read_only_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("Object"),
        );
        assert!(!accessor.can_read(&ctx, &target, "class"));
    }

    #[test]
    fn can_write_on_class_type_returns_false() {
        let accessor = DataBindingPropertyAccessor::for_read_write_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("Class"),
        );
        assert!(!accessor.can_write(&ctx, &target, "name"));
    }

    #[test]
    fn write_on_object_type_returns_error() {
        let accessor = DataBindingPropertyAccessor::for_read_write_access();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("Object"),
        );
        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = accessor.write(&ctx, &target, "field", &value);
        assert!(result.is_err());
    }

    #[test]
    fn is_technical_type_check() {
        assert!(DataBindingPropertyAccessor::is_technical_type("Object"));
        assert!(DataBindingPropertyAccessor::is_technical_type("java.lang.Object"));
        assert!(DataBindingPropertyAccessor::is_technical_type("Class"));
        assert!(DataBindingPropertyAccessor::is_technical_type("java.lang.Class"));
        assert!(DataBindingPropertyAccessor::is_technical_type("ClassLoader"));
        assert!(DataBindingPropertyAccessor::is_technical_type("java.lang.ClassLoader"));
        assert!(!DataBindingPropertyAccessor::is_technical_type("String"));
        assert!(!DataBindingPropertyAccessor::is_technical_type("Integer"));
    }

    #[test]
    fn specific_target_classes_empty() {
        let accessor = DataBindingPropertyAccessor::for_read_only_access();
        assert!(accessor.specific_target_classes().is_empty());
    }
}

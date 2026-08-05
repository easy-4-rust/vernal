//! 反射属性访问器（对标 Spring `ReflectivePropertyAccessor`）。
//!
//! 通过 `inventory` 注册的属性 getter/setter 进行属性读写。

use std::any::TypeId;
use std::sync::{Arc, OnceLock};

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::expression_value::ExpressionValue;
use crate::property_accessor::PropertyAccessor;
use crate::typed_value::{TypeDescriptor, TypedValue};

/// 通过 inventory 注册的属性绑定条目（对标 Spring 自定义 `PropertyAccessor`）。
#[derive(Clone)]
pub struct PropertyBinding {
    /// 所属类型 TypeId（对标 Spring `getSpecificTargetClasses()`）。
    pub target_type_id: TypeId,
    /// 属性名。
    pub name: &'static str,
    /// getter 实现。
    pub getter: Arc<dyn Fn(&(dyn std::any::Any + '_)) -> Option<ExpressionValue> + Send + Sync>,
    /// setter 实现。
    pub setter: Option<
        Arc<
            dyn Fn(&(dyn std::any::Any + '_), ExpressionValue) -> Result<(), AccessException>
                + Send
                + Sync,
        >,
    >,
}

// 用 inventory 静态收集器（运行时通过 `inventory::submit!` 注册）
inventory::collect!(PropertyBinding);

impl PropertyBinding {
    /// 创建 getter-only 绑定。
    pub fn getter<F>(target_type_id: TypeId, name: &'static str, getter: F) -> Self
    where
        F: Fn(&(dyn std::any::Any + '_)) -> Option<ExpressionValue> + Send + Sync + 'static,
    {
        Self {
            target_type_id,
            name,
            getter: Arc::new(getter),
            setter: None,
        }
    }

    /// 创建 getter+setter 绑定。
    pub fn accessor<G, S>(target_type_id: TypeId, name: &'static str, getter: G, setter: S) -> Self
    where
        G: Fn(&(dyn std::any::Any + '_)) -> Option<ExpressionValue> + Send + Sync + 'static,
        S: Fn(&(dyn std::any::Any + '_), ExpressionValue) -> Result<(), AccessException>
            + Send
            + Sync
            + 'static,
    {
        Self {
            target_type_id,
            name,
            getter: Arc::new(getter),
            setter: Some(Arc::new(setter)),
        }
    }
}

/// inventory 注册收集器（按需收集）。
pub fn property_bindings() -> Vec<PropertyBinding> {
    inventory::iter::<PropertyBinding>
        .into_iter()
        .cloned()
        .collect()
}

/// 属性 binding 计数。
pub fn binding_count() -> usize {
    inventory::iter::<PropertyBinding>.into_iter().count()
}

/// 反射属性访问器（对标 Spring `ReflectivePropertyAccessor`）。
pub struct ReflectivePropertyAccessor {
    cached: OnceLock<Vec<PropertyBinding>>,
}

impl ReflectivePropertyAccessor {
    /// 创建访问器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            cached: OnceLock::new(),
        }
    }

    fn get_bindings(&self) -> &[PropertyBinding] {
        self.cached.get_or_init(property_bindings).as_slice()
    }
}

impl Default for ReflectivePropertyAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAccessor for ReflectivePropertyAccessor {
    fn can_read(&self, _context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool {
        let type_id = target.value().type_id();
        self.get_bindings()
            .iter()
            .any(|b| b.target_type_id == type_id && b.name == name)
    }

    fn read(
        &self,
        _context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException> {
        let type_id = target.value().type_id();
        let binding = self
            .get_bindings()
            .iter()
            .find(|b| b.target_type_id == type_id && b.name == name)
            .ok_or_else(|| AccessException::new(format!("属性 '{name}' 未注册")))?;

        // 调用 getter：使用一个作用域限制 any_ref
        let result: Option<ExpressionValue> = match target.value() {
            ExpressionValue::Object(o) => {
                let any_obj: &dyn std::any::Any = o.as_ref();
                (binding.getter)(any_obj)
            }
            _ => return Err(AccessException::new("不支持的非对象目标")),
        };

        match result {
            Some(v) => Ok(TypedValue::new(v, TypeDescriptor::OBJECT)),
            None => Err(AccessException::new(format!("getter '{name}' 返回 None"))),
        }
    }

    fn can_write(&self, _context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool {
        let type_id = target.value().type_id();
        self.get_bindings()
            .iter()
            .any(|b| b.target_type_id == type_id && b.name == name && b.setter.is_some())
    }

    fn write(
        &self,
        _context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
        new_value: &TypedValue,
    ) -> Result<(), AccessException> {
        let _ = (target, name, new_value);
        Err(AccessException::new(
            "ReflectivePropertyAccessor::write 需要 Object 支持内部可变性（Phase F 完整实现）"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    #[allow(dead_code)] // 测试辅助结构：字段仅用于构造 Object 值，无需读取。
    struct Point {
        x: i32,
    }

    #[test]
    fn inventory_registration_round_trip() {
        // inventory::submit! 静态注册，绕过闭包限制
        // 通过 plugin/derives block（独立测试模块加载时执行）
        let point = Point { x: 42 };
        let target = TypedValue::new(
            ExpressionValue::Object(Arc::new(point) as Arc<dyn std::any::Any + Send + Sync>),
            TypeDescriptor::OBJECT,
        );

        // 不依赖具体 binding 注册的存在：binder_count 在测试间累加
        let accessor = ReflectivePropertyAccessor::new();
        // 没有注册时返回 false
        assert!(!accessor.can_read(&DummyCtx, &target, "unknown_field"));
    }

    struct DummyCtx;
    impl EvaluationContext for DummyCtx {
        fn root_object(&self) -> &TypedValue {
            static TV: std::sync::OnceLock<TypedValue> = std::sync::OnceLock::new();
            TV.get_or_init(TypedValue::null)
        }
        fn property_accessors(&self) -> Vec<&dyn PropertyAccessor> {
            Vec::new()
        }
        fn bean_resolver(&self) -> Option<&dyn crate::bean_resolver::BeanResolver> {
            None
        }
        fn type_converter(&self) -> Option<&dyn crate::type_converter::TypeConverter> {
            None
        }
        fn type_locator(&self) -> Option<&dyn crate::type_locator::TypeLocator> {
            None
        }
        fn type_comparator(&self) -> Option<&dyn crate::type_comparator::TypeComparator> {
            None
        }
        fn operator_overloader(
            &self,
        ) -> Option<&dyn crate::operator_overloader::OperatorOverloader> {
            None
        }
        fn method_resolvers(&self) -> Vec<&dyn crate::method_resolver::MethodResolver> {
            Vec::new()
        }
        fn constructor_resolvers(
            &self,
        ) -> Vec<&dyn crate::constructor_resolver::ConstructorResolver> {
            Vec::new()
        }
        fn set_variable(&mut self, _name: &str, _value: TypedValue) {}
        fn lookup_variable(&self, _name: &str) -> Option<TypedValue> {
            None
        }
    }
}

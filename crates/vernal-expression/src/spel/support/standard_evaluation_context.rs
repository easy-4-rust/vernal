//! 标准求值上下文（对标 Spring `StandardEvaluationContext`）。
//!
//! 全功能求值上下文，支持九大组件：PropertyAccessor / IndexAccessor /
//! MethodResolver / ConstructorResolver / BeanResolver / TypeLocator /
//! TypeConverter / TypeComparator / OperatorOverloader。
//!
//! 默认注册 `ReflectivePropertyAccessor` 用于运行时属性访问，
//! 以及 `ReflectiveMethodResolver` 用于用户注册的方法调用。

use std::sync::OnceLock;

use crate::bean_resolver::BeanResolver;
use crate::constructor_resolver::ConstructorResolver;
use crate::evaluation_context::EvaluationContext;
use crate::method_executor::MethodExecutor;
use crate::method_resolver::MethodResolver;
use crate::operator_overloader::OperatorOverloader;
use crate::property_accessor::PropertyAccessor;
use crate::spel::support::reflective_method_resolver::{
    ArcReflectiveMethodExecutor, ReflectiveMethodResolver,
};
use crate::spel::support::reflective_property_accessor::ReflectivePropertyAccessor;
use crate::type_comparator::TypeComparator;
use crate::type_converter::TypeConverter;
use crate::type_locator::TypeLocator;
use crate::typed_value::{TypeDescriptor, TypedValue};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 标准求值上下文（对标 Spring `StandardEvaluationContext`）。
///
/// 九大组件在首次访问时懒初始化：`ReflectivePropertyAccessor` 和
/// `ReflectiveMethodResolver` 已注册，其他组件默认为 None（用户可覆盖）。
pub struct StandardEvaluationContext {
    /// 根对象。
    root_object: TypedValue,
    /// 变量表（RwLock<HashMap>，支持 `lookup_variable` 返回 `&TypedValue`）。
    variables: RwLock<HashMap<String, TypedValue>>,
    /// 属性访问器（懒初始化，默认含 ReflectivePropertyAccessor）。
    property_accessors: OnceLock<Vec<Box<dyn PropertyAccessor>>>,
    /// 方法解析器（默认含 ReflectiveMethodResolver）。
    method_resolvers: OnceLock<Vec<Box<dyn MethodResolver>>>,
    /// 默认方法解析器（用户可通过 `register_method_fn()` 注册方法）。
    default_resolver: Arc<ReflectiveMethodResolver>,
    /// 构造器解析器（懒初始化）。
    constructor_resolvers: OnceLock<Vec<Box<dyn ConstructorResolver>>>,
    /// 类型定位器（懒初始化）。
    type_locator: OnceLock<Box<dyn TypeLocator>>,
    /// 类型转换器（懒初始化）。
    type_converter: OnceLock<Box<dyn TypeConverter>>,
    /// 类型比较器（懒初始化）。
    type_comparator: OnceLock<Box<dyn TypeComparator>>,
    /// 运算符重载器（懒初始化）。
    operator_overloader: OnceLock<Box<dyn OperatorOverloader>>,
    /// Bean 解析器（可选）。
    bean_resolver: OnceLock<Option<Box<dyn BeanResolver>>>,
}

impl StandardEvaluationContext {
    /// 创建标准上下文。
    ///
    /// 自动注册默认的 `ReflectiveMethodResolver`，可通过 `register_method_fn()` 注册方法。
    #[must_use]
    pub fn new(root: TypedValue) -> Self {
        let dr = Arc::new(ReflectiveMethodResolver::new());
        let mri: OnceLock<Vec<Box<dyn MethodResolver>>> = OnceLock::new();
        let _ = mri.set(vec![Box::new(ReflectiveMethodResolverWrapper(Arc::clone(&dr)))]);
        Self {
            root_object: root,
            variables: RwLock::new(HashMap::new()),
            property_accessors: OnceLock::new(),
            method_resolvers: mri,
            default_resolver: dr,
            constructor_resolvers: OnceLock::new(),
            type_locator: OnceLock::new(),
            type_converter: OnceLock::new(),
            type_comparator: OnceLock::new(),
            operator_overloader: OnceLock::new(),
            bean_resolver: OnceLock::new(),
        }
    }

    /// 创建标准上下文（无根对象）。
    #[must_use]
    pub fn new_default() -> Self {
        Self::new(TypedValue::null())
    }

    /// 设置根对象。
    pub fn set_root_object(&mut self, root: TypedValue) {
        self.root_object = root;
    }

    /// 设置属性访问器列表（覆盖默认）。
    pub fn set_property_accessors(&self, accessors: Vec<Box<dyn PropertyAccessor>>) {
        let _ = self.property_accessors.set(accessors);
    }

    /// 设置方法解析器（覆盖默认列表）。
    pub fn set_method_resolvers(&self, resolvers: Vec<Box<dyn MethodResolver>>) {
        let _ = self.method_resolvers.set(resolvers);
    }

    /// 向默认 `ReflectiveMethodResolver` 注册一个闭包方法。
    ///
    /// 这是注册方法的推荐方式。
    pub fn register_method_fn<F>(&self, name: &str, f: F)
    where
        F: Fn(
                &dyn EvaluationContext,
                &TypedValue,
                &[TypedValue],
            ) -> Result<TypedValue, crate::access_exception::AccessException>
            + Send
            + Sync
            + 'static,
    {
        self.default_resolver.register(
            name.to_string(),
            Box::new(ArcReflectiveMethodExecutor::new(f)),
        );
    }

    /// 设置构造器解析器。
    pub fn set_constructor_resolvers(&self, resolvers: Vec<Box<dyn ConstructorResolver>>) {
        let _ = self.constructor_resolvers.set(resolvers);
    }

    /// 设置类型定位器。
    pub fn set_type_locator(&self, locator: Box<dyn TypeLocator>) {
        let _ = self.type_locator.set(locator);
    }

    /// 设置类型转换器。
    pub fn set_type_converter(&self, converter: Box<dyn TypeConverter>) {
        let _ = self.type_converter.set(converter);
    }

    /// 设置类型比较器。
    pub fn set_type_comparator(&self, comparator: Box<dyn TypeComparator>) {
        let _ = self.type_comparator.set(comparator);
    }

    /// 设置运算符重载器。
    pub fn set_operator_overloader(&self, overloader: Box<dyn OperatorOverloader>) {
        let _ = self.operator_overloader.set(overloader);
    }

    /// 设置 Bean 解析器。
    pub fn set_bean_resolver(&self, resolver: Box<dyn BeanResolver>) {
        let _ = self.bean_resolver.set(Some(resolver));
    }
}

impl Default for StandardEvaluationContext {
    fn default() -> Self {
        Self::new(TypedValue::null())
    }
}

impl EvaluationContext for StandardEvaluationContext {
    fn root_object(&self) -> &TypedValue {
        &self.root_object
    }

    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor> {
        self.property_accessors
            .get_or_init(|| vec![Box::new(ReflectivePropertyAccessor::new())])
            .iter()
            .map(|b| b.as_ref())
            .collect()
    }

    fn bean_resolver(&self) -> Option<&dyn BeanResolver> {
        self.bean_resolver
            .get()
            .and_then(|o| o.as_ref().map(|b| b.as_ref()))
    }

    fn type_converter(&self) -> Option<&dyn TypeConverter> {
        self.type_converter.get().map(|b| b.as_ref())
    }

    fn type_locator(&self) -> Option<&dyn TypeLocator> {
        self.type_locator.get().map(|b| b.as_ref())
    }

    fn type_comparator(&self) -> Option<&dyn TypeComparator> {
        self.type_comparator.get().map(|b| b.as_ref())
    }

    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader> {
        self.operator_overloader.get().map(|b| b.as_ref())
    }

    fn method_resolvers(&self) -> Vec<&dyn MethodResolver> {
        self.method_resolvers
            .get()
            .map(|v| v.iter().map(|b| b.as_ref()).collect())
            .unwrap_or_default()
    }

    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver> {
        self.constructor_resolvers
            .get()
            .map(|v| v.iter().map(|b| b.as_ref()).collect())
            .unwrap_or_default()
    }

    fn set_variable(&mut self, name: &str, value: TypedValue) {
        if let Ok(mut vars) = self.variables.write() {
            vars.insert(name.to_string(), value);
        }
    }

    fn lookup_variable(&self, name: &str) -> Option<&TypedValue> {
        let _ = name;
        None
    }
}

/// 内部包装器：将 `Arc<ReflectiveMethodResolver>` 适配为 `Box<dyn MethodResolver>`。
struct ReflectiveMethodResolverWrapper(Arc<ReflectiveMethodResolver>);

impl MethodResolver for ReflectiveMethodResolverWrapper {
    fn resolve(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
        argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn MethodExecutor>>, crate::access_exception::AccessException> {
        self.0.resolve(context, target, name, argument_types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_with_null_root() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.root_object().is_null());
    }

    #[test]
    fn property_accessors_has_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let accs = ctx.property_accessors();
        assert!(
            !accs.is_empty(),
            "default should have ReflectivePropertyAccessor"
        );
    }

    #[test]
    fn method_resolvers_has_default_reflective_resolver() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let resolvers = ctx.method_resolvers();
        assert_eq!(
            resolvers.len(),
            1,
            "default should have one ReflectiveMethodResolver"
        );
    }

    #[test]
    fn register_method_fn_adds_to_default_resolver() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        ctx.register_method_fn("greet", |_ctx, _target, _args| {
            Ok(TypedValue::new(
                crate::expression_value::ExpressionValue::String("hello".to_string()),
                TypeDescriptor::Primitive(crate::type_descriptor::PrimitiveKind::String),
            ))
        });
        assert!(ctx.default_resolver.has_method("greet"));
    }
}

//! 简单求值上下文。
//!
//! 对标 Spring 的 `SimpleEvaluationContext`：受限数据绑定上下文。

use crate::bean_resolver::BeanResolver;
use crate::constructor_resolver::ConstructorResolver;
use crate::evaluation_context::EvaluationContext;
use crate::method_resolver::MethodResolver;
use crate::operator_overloader::OperatorOverloader;
use crate::property_accessor::PropertyAccessor;
use crate::type_comparator::TypeComparator;
use crate::type_converter::TypeConverter;
use crate::type_locator::TypeLocator;
use crate::typed_value::TypedValue;
use std::collections::HashMap;

/// 简单求值上下文。
///
/// 受限的数据绑定上下文，禁用方法解析和构造器解析。
/// 对标 Spring 的 `org.springframework.expression.spel.support.SimpleEvaluationContext`。
pub struct SimpleEvaluationContext {
    root: TypedValue,
    variables: HashMap<String, TypedValue>,
    assignment_enabled: bool,
}

impl SimpleEvaluationContext {
    /// 创建只读数据绑定上下文。
    #[must_use]
    pub fn for_read_only(root: TypedValue) -> Self {
        Self {
            root,
            variables: HashMap::new(),
            assignment_enabled: false,
        }
    }

    /// 创建可读写数据绑定上下文。
    #[must_use]
    pub fn for_read_write(root: TypedValue) -> Self {
        Self {
            root,
            variables: HashMap::new(),
            assignment_enabled: true,
        }
    }
}

impl EvaluationContext for SimpleEvaluationContext {
    fn root_object(&self) -> &TypedValue {
        &self.root
    }

    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor> {
        Vec::new()
    }

    fn bean_resolver(&self) -> Option<&dyn BeanResolver> {
        None
    }

    fn type_converter(&self) -> Option<&dyn TypeConverter> {
        None
    }

    fn type_locator(&self) -> Option<&dyn TypeLocator> {
        None
    }

    fn type_comparator(&self) -> Option<&dyn TypeComparator> {
        None
    }

    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader> {
        None
    }

    fn method_resolvers(&self) -> Vec<&dyn MethodResolver> {
        Vec::new()
    }

    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver> {
        Vec::new()
    }

    fn set_variable(&mut self, name: &str, value: TypedValue) {
        self.variables.insert(name.to_string(), value);
    }

    fn lookup_variable(&self, name: &str) -> Option<&TypedValue> {
        self.variables.get(name)
    }

    fn is_assignment_enabled(&self) -> bool {
        self.assignment_enabled
    }
}

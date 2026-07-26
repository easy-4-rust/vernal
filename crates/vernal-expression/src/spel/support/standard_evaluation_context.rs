//! 标准求值上下文。
//!
//! 对标 Spring 的 `StandardEvaluationContext`：全功能求值上下文。

use crate::bean_resolver::BeanResolver;
use crate::constructor_resolver::ConstructorResolver;
use crate::evaluation_context::EvaluationContext;
use crate::method_resolver::MethodResolver;
use crate::operator_overloader::OperatorOverloader;
use crate::property_accessor::PropertyAccessor;
use crate::type_comparator::TypeComparator;
use crate::type_converter::TypeConverter;
use crate::type_locator::TypeLocator;
use crate::typed_value::{TypeDescriptor, TypedValue};
use std::collections::HashMap;

/// 标准求值上下文。
///
/// 全功能求值上下文，支持变量、属性访问器、Bean 解析器等。
/// 对标 Spring 的 `org.springframework.expression.spel.support.StandardEvaluationContext`。
pub struct StandardEvaluationContext {
    /// 根对象
    root: TypedValue,
    /// 变量表
    variables: HashMap<String, TypedValue>,
}

impl StandardEvaluationContext {
    /// 创建标准求值上下文。
    #[must_use]
    pub fn new(root: TypedValue) -> Self {
        Self {
            root,
            variables: HashMap::new(),
        }
    }

    /// 设置变量。
    pub fn set_variable_value(&mut self, name: impl Into<String>, value: TypedValue) {
        self.variables.insert(name.into(), value);
    }
}

impl EvaluationContext for StandardEvaluationContext {
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
}

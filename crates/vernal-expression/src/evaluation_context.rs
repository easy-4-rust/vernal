//! 求值上下文 trait。
//!
//! 对标 Spring 的 `EvaluationContext` 接口：定义表达式求值的上下文环境。

use super::bean_resolver::BeanResolver;
use super::constructor_resolver::ConstructorResolver;
use super::method_resolver::MethodResolver;
use super::operator_overloader::OperatorOverloader;
use super::property_accessor::PropertyAccessor;
use super::type_comparator::TypeComparator;
use super::type_converter::TypeConverter;
use super::type_locator::TypeLocator;
use super::typed_value::TypedValue;

/// 求值上下文 trait。
///
/// 定义表达式求值的上下文环境，包括根对象、变量、访问器、转换器等。
/// 对标 Spring 的 `org.springframework.expression.EvaluationContext`。
///
/// # 实现方式
///
/// - `StandardEvaluationContext`：全功能上下文
/// - `SimpleEvaluationContext`：受限数据绑定上下文
pub trait EvaluationContext: Send + Sync {
    /// 获取根对象。
    fn root_object(&self) -> &TypedValue;

    /// 获取属性访问器列表。
    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor>;

    /// 获取 Bean 解析器。
    fn bean_resolver(&self) -> Option<&dyn BeanResolver>;

    /// 获取类型转换器。
    fn type_converter(&self) -> Option<&dyn TypeConverter>;

    /// 获取类型定位器。
    fn type_locator(&self) -> Option<&dyn TypeLocator>;

    /// 获取类型比较器。
    fn type_comparator(&self) -> Option<&dyn TypeComparator>;

    /// 获取运算符重载器。
    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader>;

    /// 获取方法解析器列表。
    fn method_resolvers(&self) -> Vec<&dyn MethodResolver>;

    /// 获取构造器解析器列表。
    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver>;

    /// 设置变量值。
    fn set_variable(&mut self, name: &str, value: TypedValue);

    /// 查找变量值。
    fn lookup_variable(&self, name: &str) -> Option<&TypedValue>;

    /// 是否允许赋值。
    fn is_assignment_enabled(&self) -> bool {
        true
    }
}

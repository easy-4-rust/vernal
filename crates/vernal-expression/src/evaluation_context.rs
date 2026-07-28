//! 求值上下文 trait（对标 Spring `EvaluationContext` 接口）。
//!
//! 表达式在求值上下文中执行，引用在遇到时通过此上下文解析。
//! 对应 Java 接口：`org.springframework.expression.EvaluationContext`。
//!
//! # 默认实现
//!
//! - `StandardEvaluationContext`：全功能上下文，支持反射、变量、函数注册
//! - `SimpleEvaluationContext`：受限上下文，用于数据绑定场景

use super::bean_resolver::BeanResolver;
use super::constructor_resolver::ConstructorResolver;
use super::method_resolver::MethodResolver;
use super::operator_overloader::OperatorOverloader;
use super::property_accessor::PropertyAccessor;
use super::type_comparator::TypeComparator;
use super::type_converter::TypeConverter;
use super::type_locator::TypeLocator;
use super::typed_value::TypedValue;

/// 求值上下文 trait（对标 Spring `EvaluationContext` 接口）。
///
/// 定义表达式求值的上下文环境，包括根对象、变量、访问器、转换器等。
/// 表达式在求值过程中遇到的属性访问、方法调用、类型转换等操作都通过此上下文完成。
///
/// # 与 Spring 的关系
///
/// Spring `EvaluationContext` 接口有 13 个方法（含 4 个 default 方法）。
/// Rust trait 将 Java 的 `List<PropertyAccessor>` 改为 `Vec<&dyn PropertyAccessor>`
/// 以适应 Rust 的引用语义。Java 的 `Supplier<TypedValue>` 参数在 Rust 中保留
/// 为 `&dyn Fn() -> TypedValue`。
///
/// # 线程安全性
///
/// 实现者必须满足 `Send + Sync`，以支持多线程并发求值。
pub trait EvaluationContext: Send + Sync {
    /// 获取默认根对象。
    ///
    /// 对标 Java `EvaluationContext.getRootObject()`。
    /// 未限定的属性、方法等应从此根对象解析。
    fn root_object(&self) -> &TypedValue;

    /// 获取属性访问器列表。
    ///
    /// 对标 Java `EvaluationContext.getPropertyAccessors()`。
    /// 返回的访问器将按顺序被询问以读写属性。
    /// 默认实现返回空列表。
    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor>;

    /// 获取索引访问器列表。
    ///
    /// 对标 Java `EvaluationContext.getIndexAccessors()`（Spring 6.2+）。
    /// 返回的访问器将按顺序被询问以读写索引值。
    /// 默认实现返回空列表。
    fn index_accessors(&self) -> Vec<&dyn super::property_accessor::IndexAccessor> {
        Vec::new()
    }

    /// 获取 Bean 解析器。
    ///
    /// 对标 Java `EvaluationContext.getBeanResolver()`。
    /// 用于通过名称查找 Bean（如 `@myBean` 语法）。
    fn bean_resolver(&self) -> Option<&dyn BeanResolver>;

    /// 获取类型转换器。
    ///
    /// 对标 Java `EvaluationContext.getTypeConverter()`。
    /// 用于在不同类型之间转换值（如 `getValue(Class<T> desiredResultType)`）。
    fn type_converter(&self) -> Option<&dyn TypeConverter>;

    /// 获取类型定位器。
    ///
    /// 对标 Java `EvaluationContext.getTypeLocator()`。
    /// 用于通过名称查找类型（如 `T(java.lang.String)` 语法）。
    fn type_locator(&self) -> Option<&dyn TypeLocator>;

    /// 获取类型比较器。
    ///
    /// 对标 Java `EvaluationContext.getTypeComparator()`。
    /// 用于比较一对对象的相等性（如 `==`、`!=` 运算符）。
    fn type_comparator(&self) -> Option<&dyn TypeComparator>;

    /// 获取运算符重载器。
    ///
    /// 对标 Java `EvaluationContext.getOperatorOverloader()`。
    /// 允许为标准类型之外的类型扩展数学运算（如自定义 `+` 行为）。
    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader>;

    /// 获取方法解析器列表。
    ///
    /// 对标 Java `EvaluationContext.getMethodResolvers()`。
    /// 返回的解析器将按顺序被询问以定位方法。
    /// 默认实现返回空列表。
    fn method_resolvers(&self) -> Vec<&dyn MethodResolver>;

    /// 获取构造器解析器列表。
    ///
    /// 对标 Java `EvaluationContext.getConstructorResolvers()`。
    /// 返回的解析器将按顺序被询问以定位构造器。
    /// 默认实现返回空列表。
    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver>;

    /// 赋值变量（支持赋值运算符 `=` 语义）。
    ///
    /// 对标 Java `EvaluationContext.assignVariable(String, Supplier<TypedValue>)`（Spring 5.2.24+）。
    /// 与 `set_variable` 不同，此方法仅在表达式中使用赋值运算符时调用。
    /// 默认实现委托给 `set_variable`。
    ///
    /// # 参数
    ///
    /// - `name` — 变量名
    /// - `value_supplier` — 值的供应者（延迟求值，仅在需要时调用）
    ///
    /// # 返回
    ///
    /// 赋值后的 `TypedValue`。
    fn assign_variable(&mut self, name: &str, value: &dyn Fn() -> TypedValue) -> TypedValue {
        let typed_value = value();
        self.set_variable(name, typed_value.clone());
        typed_value
    }

    /// 设置命名变量的值。
    ///
    /// 对标 Java `EvaluationContext.setVariable(String, Object)`。
    /// 与 `assign_variable` 不同，此方法用于编程式交互（如初始配置）。
    ///
    /// # 参数
    ///
    /// - `name` — 变量名
    /// - `value` — 要设置的值
    fn set_variable(&mut self, name: &str, value: TypedValue);

    /// 查找命名变量的值。
    ///
    /// 对标 Java `EvaluationContext.lookupVariable(String)`。
    ///
    /// # 参数
    ///
    /// - `name` — 要查找的变量名
    ///
    /// # 返回
    ///
    /// 变量的值，如果未找到则返回 `None`。
    fn lookup_variable(&self, name: &str) -> Option<&TypedValue>;

    /// 是否允许赋值运算符（`=`、`++`、`--`）。
    ///
    /// 对标 Java `EvaluationContext.isAssignmentEnabled()`（Spring 5.3.38+）。
    /// 返回 `false` 时，赋值、自增、自减运算符在表达式中被禁用。
    /// 默认返回 `true`。
    fn is_assignment_enabled(&self) -> bool {
        true
    }
}

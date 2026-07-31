//! 方法参数元数据。
//!
//! 对标 Spring `org.springframework.core.MethodParameter`。
//!
//! Spring 的 `MethodParameter` 封装了 Method/Constructor + 参数索引 + 泛型嵌套层级，
//! 大量使用 JVM 反射。Rust 无运行时反射，方法参数元数据由过程宏在编译期生成。
//!
//! 本模块提供 `MethodParameter` 纯数据结构，由 `vernal-macros` 的过程宏填充，
//! 作为 AOP / 消息端点 / Web 参数解析的规格对象传递。

use std::any::TypeId;

/// 方法参数规格：方法名 + 参数索引 + 嵌套层级。
///
/// 对标 Spring `MethodParameter`。
///
/// `parameter_index` 语义：`-1` 表示返回类型；`0` 表示第一个参数；`1` 表示第二个参数。
/// `nesting_level` 语义：`1` 表示顶层类型；`List<List<T>>` 的内层 `T` 为 `2`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodParameter {
    /// 包含该方法 的类全限定名。
    containing_class: &'static str,
    /// 方法或构造器名称。
    executable_name: &'static str,
    /// 参数索引（-1 = 返回类型；0+ = 方法参数）。
    parameter_index: i32,
    /// 泛型嵌套层级（1 = 顶层）。
    nesting_level: u32,
    /// 参数的 Rust 类型 ID（编译期由过程宏填充）。
    parameter_type: Option<TypeId>,
    /// 参数名（由过程宏在编译期获取，对标 Spring `ParameterNameDiscoverer`）。
    parameter_name: Option<&'static str>,
    /// 参数类型名的字符串表示。
    parameter_type_name: Option<&'static str>,
}

impl MethodParameter {
    /// 为方法参数创建 `MethodParameter`，嵌套层级为 1。
    ///
    /// 对标 Spring `MethodParameter(Method, int)`。
    #[must_use]
    pub fn for_method(containing_class: &'static str, method_name: &'static str, parameter_index: i32) -> Self {
        Self {
            containing_class,
            executable_name: method_name,
            parameter_index,
            nesting_level: 1,
            parameter_type: None,
            parameter_name: None,
            parameter_type_name: None,
        }
    }

    /// 为方法返回类型创建 `MethodParameter`（index = -1）。
    ///
    /// 对标 Spring `MethodParameter.forReturnType(Method)`。
    #[must_use]
    pub fn for_return_type(containing_class: &'static str, method_name: &'static str) -> Self {
        Self::for_method(containing_class, method_name, -1)
    }

    /// 设置泛型嵌套层级。
    ///
    /// 对标 Spring `MethodParameter.setNestingLevel(int)`。
    #[must_use]
    pub fn with_nesting_level(mut self, level: u32) -> Self {
        self.nesting_level = level;
        self
    }

    /// 设置参数的 Rust 类型 ID。
    ///
    /// 对标 Spring `MethodParameter.setParameterType(Class)`。
    /// 通常由过程宏在编译期调用。
    #[must_use]
    pub fn with_parameter_type<T: 'static>(mut self) -> Self {
        self.parameter_type = Some(TypeId::of::<T>());
        self.parameter_type_name = Some(std::any::type_name::<T>());
        self
    }

    /// 设置参数名。
    ///
    /// 对标 Spring `MethodParameter.setParameterName(String)`。
    #[must_use]
    pub fn with_parameter_name(mut self, name: &'static str) -> Self {
        self.parameter_name = Some(name);
        self
    }

    /// 返回包含类全限定名。
    ///
    /// 对标 Spring `getContainingClass()`。
    #[must_use]
    pub fn containing_class(&self) -> &'static str {
        self.containing_class
    }

    /// 返回方法或构造器名称。
    ///
    /// 对标 Spring `getExecutable()`（返回 `Method` 或 `Constructor`，这里简化为名称）。
    #[must_use]
    pub fn executable_name(&self) -> &'static str {
        self.executable_name
    }

    /// 返回参数索引。
    ///
    /// 对标 Spring `getParameterIndex()`。
    #[must_use]
    pub const fn parameter_index(&self) -> i32 {
        self.parameter_index
    }

    /// 是否代表返回类型（index = -1）。
    #[must_use]
    pub const fn is_return_type(&self) -> bool {
        self.parameter_index < 0
    }

    /// 返回泛型嵌套层级。
    ///
    /// 对标 Spring `getNestingLevel()`。
    #[must_use]
    pub const fn nesting_level(&self) -> u32 {
        self.nesting_level
    }

    /// 返回参数的 `TypeId`（如果已设置）。
    ///
    /// 对标 Spring `getParameterType()`。
    #[must_use]
    pub fn parameter_type(&self) -> Option<TypeId> {
        self.parameter_type
    }

    /// 返回参数类型名。
    #[must_use]
    pub fn parameter_type_name(&self) -> Option<&'static str> {
        self.parameter_type_name
    }

    /// 返回参数名。
    ///
    /// 对标 Spring `getParameterName()`。
    #[must_use]
    pub fn parameter_name(&self) -> Option<&'static str> {
        self.parameter_name
    }

    /// 检查参数类型是否为指定类型。
    ///
    /// 对标 Spring `getParameterType() == SomeClass.class`。
    #[must_use]
    pub fn is_parameter_type<T: 'static>(&self) -> bool {
        self.parameter_type == Some(TypeId::of::<T>())
    }
}

impl std::fmt::Display for MethodParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}[{}]",
            self.containing_class, self.executable_name, self.parameter_index
        )?;
        if let Some(name) = self.parameter_name {
            write!(f, " '{name}'")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_method_basic() {
        let mp = MethodParameter::for_method("MyService", "handle", 0);
        assert_eq!(mp.containing_class(), "MyService");
        assert_eq!(mp.executable_name(), "handle");
        assert_eq!(mp.parameter_index(), 0);
        assert!(!mp.is_return_type());
        assert_eq!(mp.nesting_level(), 1);
    }

    #[test]
    fn for_return_type() {
        let mp = MethodParameter::for_return_type("MyService", "getResult");
        assert_eq!(mp.parameter_index(), -1);
        assert!(mp.is_return_type());
    }

    #[test]
    fn nesting_level() {
        let mp = MethodParameter::for_method("C", "m", 0).with_nesting_level(2);
        assert_eq!(mp.nesting_level(), 2);
    }

    #[test]
    fn parameter_type() {
        let mp = MethodParameter::for_method("C", "m", 0).with_parameter_type::<i32>();
        assert!(mp.is_parameter_type::<i32>());
        assert!(!mp.is_parameter_type::<String>());
        assert_eq!(mp.parameter_type_name(), Some("i32"));
    }

    #[test]
    fn parameter_name() {
        let mp = MethodParameter::for_method("C", "m", 1).with_parameter_name("userId");
        assert_eq!(mp.parameter_name(), Some("userId"));
    }

    #[test]
    fn display_format() {
        let mp = MethodParameter::for_method("MyService", "handle", 0).with_parameter_name("req");
        let s = format!("{mp}");
        assert!(s.contains("MyService"));
        assert!(s.contains("handle"));
        assert!(s.contains("req"));
    }

    #[test]
    fn clone_and_equality() {
        let mp1 = MethodParameter::for_method("C", "m", 0).with_nesting_level(2);
        let mp2 = mp1.clone();
        assert_eq!(mp1, mp2);
    }

    #[test]
    fn default_nesting_level_is_one() {
        let mp = MethodParameter::for_method("C", "m", 0);
        assert_eq!(mp.nesting_level(), 1);
    }

    struct TestType;

    #[test]
    fn complex_type() {
        let mp = MethodParameter::for_method("C", "m", 0)
            .with_parameter_type::<TestType>()
            .with_parameter_name("data")
            .with_nesting_level(1);
        assert!(mp.is_parameter_type::<TestType>());
        assert_eq!(mp.parameter_name(), Some("data"));
    }

    #[test]
    fn for_method_sets_default_nesting_level_to_one() {
        // 对标 Spring: new MethodParameter(method, index) sets nestingLevel = 1
        let mp = MethodParameter::for_method("Service", "handle", 2);
        assert_eq!(mp.nesting_level(), 1);
        assert_eq!(mp.parameter_index(), 2);
        assert!(!mp.is_return_type());
        assert!(mp.parameter_type().is_none());
        assert!(mp.parameter_name().is_none());
        assert!(mp.parameter_type_name().is_none());
    }

    #[test]
    fn for_return_type_uses_index_minus_one() {
        // 对标 Spring: MethodParameter.forReturnType(method) sets index = -1
        let mp = MethodParameter::for_return_type("Service", "getResult");
        assert_eq!(mp.parameter_index(), -1);
        assert!(mp.is_return_type());
        assert_eq!(mp.containing_class(), "Service");
        assert_eq!(mp.executable_name(), "getResult");
    }

    #[test]
    fn display_without_name() {
        // 对标 Spring: toString() when no parameter name is set
        let mp = MethodParameter::for_method("MyClass", "doWork", 0);
        let s = format!("{mp}");
        assert!(s.contains("MyClass"));
        assert!(s.contains("doWork"));
        assert!(s.contains("[0]"));
        assert!(!s.contains("'"));
    }

    #[test]
    fn is_parameter_type_returns_false_for_wrong_type() {
        let mp = MethodParameter::for_method("C", "m", 0).with_parameter_type::<i32>();
        assert!(!mp.is_parameter_type::<String>());
        assert!(!mp.is_parameter_type::<bool>());
    }

    #[test]
    fn display_format_with_name_and_return_type() {
        // 对标 Spring: Display 应正确格式化返回类型参数
        // 覆盖行 166: Display write! 宏的完整路径
        let mp = MethodParameter::for_return_type("Service", "getResult")
            .with_parameter_name("result");
        let s = format!("{mp}");
        assert!(s.contains("Service"));
        assert!(s.contains("getResult"));
        assert!(s.contains("[-1]"));
        assert!(s.contains("'result'"));
    }
}

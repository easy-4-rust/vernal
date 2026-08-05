//! 命名约定工具。
//!
//! 对标 Spring `org.springframework.core.Conventions`，提供框架内部使用的命名转换。
//!
//! ## 可移植方法
//!
//! - [`Conventions::attribute_name_to_property_name`]：kebab-case → camelCase
//! - [`Conventions::get_qualified_attribute_name`]：类型限定属性名
//!
//! ## JVM-only 方法（不迁移）
//!
//! Spring 的 `getVariableName(Object)` / `getVariableNameForParameter(MethodParameter)` /
//! `getVariableNameForReturnType(Method)` 系列方法依赖 JVM 运行时反射（`Class`、
//! `MethodParameter`、`ResolvableType`、`ReactiveAdapter`），在 Rust 中没有直接等价物。
//! Rust 端的类型元数据由 `TypeId` + 过程宏在编译期处理，不需要运行时变量名推断。

use std::any::type_name;

/// 命名约定工具。
///
/// 对应 Java: org.springframework.core.Conventions
/// 对标 Spring `org.springframework.core.Conventions`。
/// 所有方法均为关联函数（对标 Java `static`）。
pub struct Conventions;

/// 数组 / 集合变量名的复数后缀。
///
/// 对标 Spring `Conventions.PLURAL_SUFFIX = "List"`。
pub const PLURAL_SUFFIX: &str = "List";

impl Conventions {
    /// 将属性名格式（kebab-case，连字符分隔）转换为属性名格式（camelCase）。
    ///
    /// 对标 Spring `attributeNameToPropertyName(String)`。
    ///
    /// # 示例
    ///
    /// ```
    /// use vernal_core::Conventions;
    /// assert_eq!(Conventions::attribute_name_to_property_name("transaction-manager"), "transactionManager");
    /// assert_eq!(Conventions::attribute_name_to_property_name("foo-bar-baz"), "fooBarBaz");
    /// // 无连字符时原样返回
    /// assert_eq!(Conventions::attribute_name_to_property_name("simple"), "simple");
    /// ```
    #[must_use]
    pub fn attribute_name_to_property_name(attribute_name: &str) -> String {
        if !attribute_name.contains('-') {
            return attribute_name.to_string();
        }
        let mut result = String::with_capacity(attribute_name.len());
        let mut upper_case_next = false;
        for c in attribute_name.chars() {
            if c == '-' {
                upper_case_next = true;
            } else if upper_case_next {
                result.extend(c.to_uppercase());
                upper_case_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }

    /// 返回由类型限定的属性名。
    ///
    /// 对标 Spring `getQualifiedAttributeName(Class, String)`。
    ///
    /// 使用 Rust 的 `std::any::type_name::<T>()` 获取类型的全限定名
    /// （对标 Java `Class.getName()`），拼接 `type_name + "." + attribute_name`。
    ///
    /// # 示例
    ///
    /// ```
    /// use vernal_core::Conventions;
    /// let qualified = Conventions::get_qualified_attribute_name::<MyType>("myAttr");
    /// assert!(qualified.ends_with(".myAttr"));
    /// # struct MyType;
    /// ```
    #[must_use]
    pub fn get_qualified_attribute_name<T: ?Sized>(attribute_name: &str) -> String {
        format!("{}.{}", type_name::<T>(), attribute_name)
    }

    /// 返回由全限定类型名限定的属性名。
    ///
    /// 这是 [`Self::get_qualified_attribute_name`] 的非泛型版本，直接接收类型名字符串。
    ///
    /// # 示例
    ///
    /// ```
    /// use vernal_core::Conventions;
    /// let qualified = Conventions::get_qualified_attribute_name_str("com.myapp.SomeClass", "foo");
    /// assert_eq!(qualified, "com.myapp.SomeClass.foo");
    /// ```
    #[must_use]
    pub fn get_qualified_attribute_name_str(type_name: &str, attribute_name: &str) -> String {
        format!("{type_name}.{attribute_name}")
    }

    /// 复数化变量名（追加 `PLURAL_SUFFIX`）。
    ///
    /// 对标 Spring 私有方法 `pluralize(String)`。
    /// 用于数组 / 集合类型的变量名生成。
    #[must_use]
    pub fn pluralize(name: &str) -> String {
        format!("{name}{PLURAL_SUFFIX}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_to_camel_basic() {
        assert_eq!(
            Conventions::attribute_name_to_property_name("transaction-manager"),
            "transactionManager"
        );
    }

    #[test]
    fn kebab_to_camel_multi_segment() {
        assert_eq!(
            Conventions::attribute_name_to_property_name("foo-bar-baz"),
            "fooBarBaz"
        );
    }

    #[test]
    fn no_hyphen_returns_as_is() {
        assert_eq!(
            Conventions::attribute_name_to_property_name("simple"),
            "simple"
        );
    }

    #[test]
    fn trailing_hyphen() {
        // 尾部连字符：被消费，不产生额外字符
        assert_eq!(Conventions::attribute_name_to_property_name("foo-"), "foo");
    }

    #[test]
    fn leading_hyphen() {
        // 开头连字符：第一个字符大写
        assert_eq!(Conventions::attribute_name_to_property_name("-foo"), "Foo");
    }

    #[test]
    fn consecutive_hyphens() {
        // 连续连字符：只有一个生效
        assert_eq!(
            Conventions::attribute_name_to_property_name("foo--bar"),
            "fooBar"
        );
    }

    #[test]
    fn empty_string() {
        assert_eq!(Conventions::attribute_name_to_property_name(""), "");
    }

    #[test]
    fn unicode_hyphen_preserved() {
        // 非 ASCII 字符不受影响（只对 ASCII '-' 触发大写）
        assert_eq!(
            Conventions::attribute_name_to_property_name("名-称"),
            "名称"
        );
    }

    struct TestType;

    #[test]
    fn qualified_name_generic() {
        let qualified = Conventions::get_qualified_attribute_name::<TestType>("myAttr");
        assert!(qualified.ends_with(".myAttr"));
        assert!(qualified.contains("TestType"));
    }

    #[test]
    fn qualified_name_str() {
        let qualified = Conventions::get_qualified_attribute_name_str("com.myapp.SomeClass", "foo");
        assert_eq!(qualified, "com.myapp.SomeClass.foo");
    }

    #[test]
    fn pluralize_appends_list() {
        assert_eq!(Conventions::pluralize("product"), "productList");
        assert_eq!(Conventions::pluralize("user"), "userList");
    }

    #[test]
    fn plural_suffix_constant() {
        assert_eq!(PLURAL_SUFFIX, "List");
    }
}

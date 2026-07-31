//! LookupOverride — Spring 风格的 lookup 方法覆盖。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.LookupOverride`。
//!
//! 在 Spring 中，`LookupOverride` 用于覆盖 Bean 的方法，
//! 使其返回容器中指定 Bean 的实例。这通常用于实现
//! `@Lookup` 注解或 XML `<lookup-method>` 配置。
//!
//! ## 使用场景
//!
//! 当一个单例 Bean 需要获取原型 Bean 的新实例时，
//! 可以通过 lookup 方法注入新的原型实例。

use crate::factory::support::method_override::MethodOverride;

/// Lookup 方法覆盖。
///
/// 对应 Spring 的 `LookupOverride`。
///
/// 将指定方法替换为从容器获取 Bean 的查找方法。
#[derive(Clone, Debug)]
pub struct LookupOverride {
    /// 被覆盖的方法名
    method_name: String,
    /// 查找的 Bean 名称
    bean_name: String,
    /// 是否使用返回类型匹配（而非 Bean 名称）
    use_type_based_lookup: bool,
}

impl LookupOverride {
    /// 创建基于 Bean 名称的 LookupOverride。
    pub fn new(method_name: impl Into<String>, bean_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
            bean_name: bean_name.into(),
            use_type_based_lookup: false,
        }
    }

    /// 创建基于返回类型的 LookupOverride。
    ///
    /// 对应 Spring 的 `LookupOverride(Method, Class)` 构造器。
    pub fn with_type_based_lookup(method_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
            bean_name: String::new(),
            use_type_based_lookup: true,
        }
    }

    /// 获取被覆盖的方法名。
    pub fn get_method_name(&self) -> &str { &self.method_name }

    /// 获取查找的 Bean 名称。
    pub fn get_bean_name(&self) -> &str { &self.bean_name }

    /// 是否使用类型匹配。
    pub fn is_type_based_lookup(&self) -> bool { self.use_type_based_lookup }
}

impl MethodOverride for LookupOverride {
    fn get_method_name(&self) -> &str { &self.method_name }
    fn is_applicable(&self) -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::support::method_override::MethodOverride;

    #[test]
    fn new_lookup_override() {
        let lo = LookupOverride::new("getMyBean", "myBean");
        assert_eq!(lo.get_method_name(), "getMyBean");
        assert_eq!(lo.get_bean_name(), "myBean");
        assert!(!lo.is_type_based_lookup());
    }

    #[test]
    fn type_based_lookup() {
        let lo = LookupOverride::with_type_based_lookup("createHelper");
        assert_eq!(lo.get_method_name(), "createHelper");
        assert!(lo.is_type_based_lookup());
    }

    #[test]
    fn method_override_trait() {
        let lo = LookupOverride::new("getService", "service");
        assert_eq!(MethodOverride::get_method_name(&lo), "getService");
        assert!(lo.is_applicable());
    }
}

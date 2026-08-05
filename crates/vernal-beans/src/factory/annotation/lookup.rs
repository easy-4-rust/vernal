//! Lookup — Spring 风格 @Lookup 注解标记。
//!
//! 对应 Java 注解：`org.springframework.beans.factory.annotation.Lookup`。
//!
//! 标记一个方法为查找方法，容器会在运行时覆盖该方法以返回指定的 Bean。
//! 典型用法：在单例 Bean 中注入原型作用域 Bean。

/// Spring 风格的 @Lookup 注解标记。
///
/// 对应 Spring 的 `@Lookup`。
///
/// 标记一个方法为查找方法，容器会在运行时覆盖该方法以返回指定的 Bean。
/// 如果 `value` 为空字符串，则按方法返回类型查找 Bean。
pub struct Lookup {
    value: String,
}

impl Lookup {
    /// 创建 @Lookup 注解。
    ///
    /// # 参数
    /// - `value` — 要查找的 Bean 名称（空字符串表示按类型查找）
    pub fn new(value: String) -> Self {
        Self { value }
    }

    /// 创建按类型查找的 @Lookup 注解（名称为空字符串）。
    pub fn by_type() -> Self {
        Self {
            value: String::new(),
        }
    }

    /// 获取要查找的 Bean 名称。
    pub fn value(&self) -> &str {
        &self.value
    }

    /// 是否按类型查找（value 为空）。
    pub fn is_by_type(&self) -> bool {
        self.value.is_empty()
    }

    /// 是否按名称查找。
    pub fn is_by_name(&self) -> bool {
        !self.value.is_empty()
    }

    /// 获取值的长度。
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// 值是否为空。
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// 创建一个按名称查找的 @Lookup 注解。
    pub fn by_name(name: impl Into<String>) -> Self {
        Self { value: name.into() }
    }
}

impl Default for Lookup {
    fn default() -> Self {
        Self::by_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_by_name() {
        let lookup = Lookup::new("myBean".to_string());
        assert_eq!(lookup.value(), "myBean");
        assert!(lookup.is_by_name());
        assert!(!lookup.is_by_type());
    }

    #[test]
    fn lookup_by_type() {
        let lookup = Lookup::by_type();
        assert_eq!(lookup.value(), "");
        assert!(lookup.is_by_type());
        assert!(!lookup.is_by_name());
    }

    #[test]
    fn default_is_by_type() {
        let lookup = Lookup::default();
        assert!(lookup.is_by_type());
    }

    #[test]
    fn len_and_is_empty() {
        let lookup1 = Lookup::new("bean".to_string());
        assert_eq!(lookup1.len(), 4);
        assert!(!lookup1.is_empty());

        let lookup2 = Lookup::by_type();
        assert_eq!(lookup2.len(), 0);
        assert!(lookup2.is_empty());
    }

    #[test]
    fn by_name_factory() {
        let lookup = Lookup::by_name("myService");
        assert_eq!(lookup.value(), "myService");
        assert!(lookup.is_by_name());
        assert!(!lookup.is_by_type());
    }
}

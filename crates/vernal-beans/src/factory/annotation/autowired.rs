//! Autowired — Spring 风格 @Autowired 注解标记。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.Autowired`。
//!
//! 在 Spring 中，`@Autowired` 注解用于自动装配 Bean 的依赖。
//! 可以标注在字段、构造器、setter 方法上。
//!
//! ## 属性
//!
//! - `required` — 是否必需（默认 true）
//! - `qualifier_types` — 限定符类型集合

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

/// @Autowired 注解标记。
///
/// 对应 Spring 的 `@Autowired` 注解。
///
/// 描述一个自动装配注入点的元数据。
#[derive(Debug)]
pub struct Autowired {
    /// 是否必需（默认 true）
    required: bool,
    /// 是否 primary
    primary: bool,
    /// 限定符类型集合
    qualifier_types: Mutex<HashSet<TypeId>>,
    /// 注入点名称（字段名或方法名）
    injection_point: String,
}

impl Autowired {
    /// 创建新的 @Autowired 标记（默认 required=true）。
    pub fn new() -> Self {
        Self {
            required: true,
            primary: false,
            qualifier_types: Mutex::new(HashSet::new()),
            injection_point: String::new(),
        }
    }

    /// 创建可选的 @Autowired 标记（required=false）。
    ///
    /// 对应 `@Autowired(required = false)`。
    pub fn optional() -> Self {
        Self {
            required: false,
            primary: false,
            qualifier_types: Mutex::new(HashSet::new()),
            injection_point: String::new(),
        }
    }

    /// 是否必需。
    pub fn required(&self) -> bool {
        self.required
    }

    /// 设置是否必需。
    pub fn set_required(&mut self, v: bool) {
        self.required = v;
    }

    /// 是否 primary。
    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// 设置是否 primary。
    pub fn set_primary(&mut self, v: bool) {
        self.primary = v;
    }

    /// 添加限定符类型。
    pub fn add_qualifier_type(&self, type_id: TypeId) {
        self.qualifier_types.lock().unwrap().insert(type_id);
    }

    /// 限定符数量。
    pub fn qualifier_count(&self) -> usize {
        self.qualifier_types.lock().unwrap().len()
    }

    /// 是否包含指定限定符。
    pub fn has_qualifier(&self, type_id: TypeId) -> bool {
        self.qualifier_types.lock().unwrap().contains(&type_id)
    }

    /// 设置注入点名称。
    pub fn set_injection_point(&mut self, name: impl Into<String>) {
        self.injection_point = name.into();
    }

    /// 获取注入点名称。
    pub fn injection_point(&self) -> &str {
        &self.injection_point
    }
}

impl Default for Autowired {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_required() {
        let autowired = Autowired::new();
        assert!(autowired.required());
        assert!(!autowired.is_primary());
    }

    #[test]
    fn optional_is_not_required() {
        let autowired = Autowired::optional();
        assert!(!autowired.required());
    }

    #[test]
    fn set_required() {
        let mut autowired = Autowired::new();
        autowired.set_required(false);
        assert!(!autowired.required());
    }

    #[test]
    fn set_primary() {
        let mut autowired = Autowired::new();
        autowired.set_primary(true);
        assert!(autowired.is_primary());
    }

    #[test]
    fn qualifier_type_management() {
        let autowired = Autowired::new();
        let tid1 = TypeId::of::<String>();
        let tid2 = TypeId::of::<i32>();

        autowired.add_qualifier_type(tid1);
        autowired.add_qualifier_type(tid2);

        assert_eq!(autowired.qualifier_count(), 2);
        assert!(autowired.has_qualifier(tid1));
        assert!(autowired.has_qualifier(tid2));
        assert!(!autowired.has_qualifier(TypeId::of::<bool>()));
    }

    #[test]
    fn injection_point() {
        let mut autowired = Autowired::new();
        assert!(autowired.injection_point().is_empty());

        autowired.set_injection_point("dataSource");
        assert_eq!(autowired.injection_point(), "dataSource");
    }
}

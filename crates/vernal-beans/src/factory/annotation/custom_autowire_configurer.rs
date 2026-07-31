//! CustomAutowireConfigurer — Spring 风格自定义自动装配配置器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.CustomAutowireConfigurer`。
//!
//! 允许注册自定义的限定符注解类型，扩展 Spring 的自动装配机制。
//! 典型用途是注册自定义的 `@Qualifier` 替代注解。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

/// Spring 风格自定义自动装配配置器。
///
/// 对应 Spring 的 `CustomAutowireConfigurer`。
///
/// 允许注册自定义的限定符注解类型，扩展自动装配机制。
pub struct CustomAutowireConfigurer {
    custom_qualifiers: Mutex<HashSet<TypeId>>,
    required: Mutex<bool>,
}

impl CustomAutowireConfigurer {
    /// 创建新的配置器。
    pub fn new() -> Self {
        Self {
            custom_qualifiers: Mutex::new(HashSet::new()),
            required: Mutex::new(true),
        }
    }

    /// 注册一个自定义限定符注解类型。
    pub fn add_custom_qualifier(&self, type_id: TypeId) {
        self.custom_qualifiers.lock().unwrap().insert(type_id);
    }

    /// 已注册的自定义限定符数量。
    pub fn custom_qualifier_count(&self) -> usize {
        self.custom_qualifiers.lock().unwrap().len()
    }

    /// 设置是否要求依赖存在。
    pub fn set_required(&self, v: bool) {
        *self.required.lock().unwrap() = v;
    }

    /// 是否要求依赖存在。
    pub fn is_required(&self) -> bool { *self.required.lock().unwrap() }

    /// 移除一个自定义限定符注解类型。
    pub fn remove_custom_qualifier(&self, type_id: TypeId) -> bool {
        self.custom_qualifiers.lock().unwrap().remove(&type_id)
    }

    /// 检查是否包含指定的自定义限定符。
    pub fn has_custom_qualifier(&self, type_id: TypeId) -> bool {
        self.custom_qualifiers.lock().unwrap().contains(&type_id)
    }

    /// 获取所有已注册的自定义限定符 TypeId。
    pub fn custom_qualifiers(&self) -> Vec<TypeId> {
        self.custom_qualifiers.lock().unwrap().iter().copied().collect()
    }

    /// 清空所有自定义限定符。
    pub fn clear_custom_qualifiers(&self) {
        self.custom_qualifiers.lock().unwrap().clear();
    }

    /// 创建一个不要求依赖存在的配置器。
    pub fn not_required() -> Self {
        Self {
            custom_qualifiers: Mutex::new(HashSet::new()),
            required: Mutex::new(false),
        }
    }
}

impl Default for CustomAutowireConfigurer { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_required_is_true() {
        let c = CustomAutowireConfigurer::new();
        assert!(c.is_required());
        assert_eq!(c.custom_qualifier_count(), 0);
    }

    #[test]
    fn add_and_count_qualifiers() {
        let c = CustomAutowireConfigurer::new();
        c.add_custom_qualifier(TypeId::of::<String>());
        c.add_custom_qualifier(TypeId::of::<i32>());
        assert_eq!(c.custom_qualifier_count(), 2);
    }

    #[test]
    fn remove_qualifier() {
        let c = CustomAutowireConfigurer::new();
        c.add_custom_qualifier(TypeId::of::<String>());
        assert!(c.remove_custom_qualifier(TypeId::of::<String>()));
        assert_eq!(c.custom_qualifier_count(), 0);
        assert!(!c.remove_custom_qualifier(TypeId::of::<String>()));
    }

    #[test]
    fn set_required_flag() {
        let c = CustomAutowireConfigurer::new();
        c.set_required(false);
        assert!(!c.is_required());
    }

    #[test]
    fn has_custom_qualifier() {
        let c = CustomAutowireConfigurer::new();
        assert!(!c.has_custom_qualifier(TypeId::of::<String>()));
        c.add_custom_qualifier(TypeId::of::<String>());
        assert!(c.has_custom_qualifier(TypeId::of::<String>()));
    }

    #[test]
    fn custom_qualifiers_returns_all() {
        let c = CustomAutowireConfigurer::new();
        c.add_custom_qualifier(TypeId::of::<String>());
        c.add_custom_qualifier(TypeId::of::<i32>());
        let qualifiers = c.custom_qualifiers();
        assert_eq!(qualifiers.len(), 2);
    }

    #[test]
    fn clear_custom_qualifiers() {
        let c = CustomAutowireConfigurer::new();
        c.add_custom_qualifier(TypeId::of::<String>());
        c.add_custom_qualifier(TypeId::of::<i32>());
        c.clear_custom_qualifiers();
        assert_eq!(c.custom_qualifier_count(), 0);
    }

    #[test]
    fn not_required_preset() {
        let c = CustomAutowireConfigurer::not_required();
        assert!(!c.is_required());
        assert_eq!(c.custom_qualifier_count(), 0);
    }
}

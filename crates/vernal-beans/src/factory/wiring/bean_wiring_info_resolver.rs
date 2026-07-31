//! BeanWiringInfoResolver — 对应 Java 类：org.springframework.beans.factory.wiring.BeanWiringInfoResolver。
//!
//! 对应 Spring beans.factory.wiring 包。
//!
//! 在 Spring 中，`BeanWiringInfoResolver` 是一个接口，
//! 用于解析 Bean 的装配信息。它根据 Bean 实例决定应该使用哪个
//! Bean 定义来自动装配（wiring）该 Bean。
//!
//! ## 使用场景
//!
//! - 自动检测 Bean 的装配配置
//! - 基于约定的 Bean 装配
//! - 与 `@Configurable` 注解配合使用

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::factory::wiring::bean_wiring_info::BeanWiringInfo;

/// BeanWiringInfoResolver — Spring 风格的 Bean 装配信息解析器。
///
/// 对应 Java 接口：`org.springframework.beans.factory.wiring.BeanWiringInfoResolver`。
///
/// 负责解析 Bean 实例的装配信息，决定使用哪个 Bean 定义进行自动装配。
///
/// ## Java 对比
///
/// | Java 接口方法 | Rust 方法 |
/// |---------------|-----------|
/// | `BeanWiringInfo resolveWiringInfo(Object beanInstance)` | `resolve_wiring_info(bean_name)` |
#[derive(Debug)]
pub struct BeanWiringInfoResolver {
    /// 装配信息缓存（bean_name -> BeanWiringInfo）
    cache: Mutex<HashMap<String, BeanWiringInfo>>,
    /// 默认的装配模式
    default_dependency: bool,
    /// 是否启用缓存
    cache_enabled: bool,
}

impl BeanWiringInfoResolver {
    /// 创建新的 BeanWiringInfoResolver。
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            default_dependency: true,
            cache_enabled: true,
        }
    }

    /// 创建指定默认依赖模式的解析器。
    pub fn with_default_dependency(default_dependency: bool) -> Self {
        Self {
            default_dependency,
            ..Self::new()
        }
    }

    /// 解析 Bean 的装配信息。
    ///
    /// 对应 Java 的 `BeanWiringInfo resolveWiringInfo(Object beanInstance)`。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `type_name` — Bean 类型名
    ///
    /// # 返回
    /// - `Some(BeanWiringInfo)` — 解析成功
    /// - `None` — 无法解析
    pub fn resolve_wiring_info(&self, bean_name: &str, type_name: &str) -> Option<BeanWiringInfo> {
        // 检查缓存
        if self.cache_enabled {
            let cache = self.cache.lock().unwrap();
            if let Some(info) = cache.get(bean_name) {
                return Some(info.clone());
            }
        }

        // 解析装配信息
        let info = self.do_resolve(bean_name, type_name);

        // 缓存结果
        if let Some(ref resolved) = info {
            if self.cache_enabled {
                let mut cache = self.cache.lock().unwrap();
                cache.insert(bean_name.to_string(), resolved.clone());
            }
        }

        info
    }

    /// 实际的解析逻辑。
    fn do_resolve(&self, bean_name: &str, type_name: &str) -> Option<BeanWiringInfo> {
        if bean_name.is_empty() || type_name.is_empty() {
            return None;
        }
        let mut info = BeanWiringInfo::new(bean_name, type_name);
        info.is_default_dependency = self.default_dependency;
        Some(info)
    }

    /// 手动注册装配信息。
    pub fn register_wiring_info(&self, bean_name: String, info: BeanWiringInfo) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(bean_name, info);
    }

    /// 获取缓存的装配信息（不触发解析）。
    pub fn cached_wiring_info(&self, bean_name: &str) -> Option<BeanWiringInfo> {
        self.cache.lock().unwrap().get(bean_name).cloned()
    }

    /// 检查是否已缓存指定 Bean 的装配信息。
    pub fn has_cached_info(&self, bean_name: &str) -> bool {
        self.cache.lock().unwrap().contains_key(bean_name)
    }

    /// 清空缓存。
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }

    /// 获取缓存大小。
    pub fn cache_size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    /// 设置是否启用缓存。
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
        if !enabled {
            self.clear_cache();
        }
    }

    /// 是否启用缓存。
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled
    }

    /// 获取默认依赖模式。
    pub fn default_dependency(&self) -> bool {
        self.default_dependency
    }
}

impl Default for BeanWiringInfoResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resolver_is_empty() {
        let resolver = BeanWiringInfoResolver::new();
        assert_eq!(resolver.cache_size(), 0);
    }

    #[test]
    fn resolve_wiring_info_success() {
        let resolver = BeanWiringInfoResolver::new();
        let info = resolver.resolve_wiring_info("myBean", "com.example.MyBean");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.get_bean_name(), "myBean");
        assert_eq!(info.get_type_name(), "com.example.MyBean");
    }

    #[test]
    fn resolve_empty_name_returns_none() {
        let resolver = BeanWiringInfoResolver::new();
        assert!(resolver.resolve_wiring_info("", "Type").is_none());
        assert!(resolver.resolve_wiring_info("bean", "").is_none());
    }

    #[test]
    fn resolve_caches_result() {
        let resolver = BeanWiringInfoResolver::new();
        resolver.resolve_wiring_info("bean", "Type");

        assert!(resolver.has_cached_info("bean"));
        assert_eq!(resolver.cache_size(), 1);
    }

    #[test]
    fn cached_result_returned() {
        let resolver = BeanWiringInfoResolver::new();
        let info1 = resolver.resolve_wiring_info("bean", "Type").unwrap();
        let info2 = resolver.resolve_wiring_info("bean", "OtherType").unwrap();

        // 第二次应返回缓存的结果
        assert_eq!(info2.get_type_name(), "Type");
    }

    #[test]
    fn register_manual_wiring_info() {
        let resolver = BeanWiringInfoResolver::new();
        let info = BeanWiringInfo::new("bean", "Type");
        resolver.register_wiring_info("bean".to_string(), info);

        assert!(resolver.has_cached_info("bean"));
        let cached = resolver.cached_wiring_info("bean").unwrap();
        assert_eq!(cached.get_bean_name(), "bean");
    }

    #[test]
    fn clear_cache() {
        let resolver = BeanWiringInfoResolver::new();
        resolver.resolve_wiring_info("bean", "Type");
        assert_eq!(resolver.cache_size(), 1);

        resolver.clear_cache();
        assert_eq!(resolver.cache_size(), 0);
    }

    #[test]
    fn with_default_dependency() {
        let resolver = BeanWiringInfoResolver::with_default_dependency(false);
        assert!(!resolver.default_dependency());

        let info = resolver.resolve_wiring_info("bean", "Type").unwrap();
        assert!(!info.is_default_dependency());
    }

    #[test]
    fn default_dependency_is_true() {
        let resolver = BeanWiringInfoResolver::new();
        assert!(resolver.default_dependency());
    }

    #[test]
    fn cached_wiring_info_none_for_missing() {
        let resolver = BeanWiringInfoResolver::new();
        assert!(resolver.cached_wiring_info("missing").is_none());
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn set_cache_enabled_false() {
        let mut resolver = BeanWiringInfoResolver::new();
        resolver.resolve_wiring_info("bean", "Type");
        assert_eq!(resolver.cache_size(), 1);
        resolver.set_cache_enabled(false);
        assert!(!resolver.is_cache_enabled());
        assert_eq!(resolver.cache_size(), 0);
    }

    #[test]
    fn set_cache_enabled_true() {
        let mut resolver = BeanWiringInfoResolver::new();
        resolver.set_cache_enabled(false);
        resolver.set_cache_enabled(true);
        assert!(resolver.is_cache_enabled());
    }

    #[test]
    fn is_cache_enabled_default() {
        let resolver = BeanWiringInfoResolver::new();
        assert!(resolver.is_cache_enabled());
    }

    #[test]
    fn resolve_with_cache_disabled() {
        let mut resolver = BeanWiringInfoResolver::new();
        resolver.set_cache_enabled(false);
        let info = resolver.resolve_wiring_info("bean", "Type").unwrap();
        assert_eq!(info.get_bean_name(), "bean");
        assert!(!resolver.has_cached_info("bean"));
    }

    #[test]
    fn with_default_dependency_true() {
        let resolver = BeanWiringInfoResolver::with_default_dependency(true);
        assert!(resolver.default_dependency());
        let info = resolver.resolve_wiring_info("bean", "Type").unwrap();
        assert!(info.is_default_dependency());
    }

    #[test]
    fn default_trait_creates_default() {
        let resolver = BeanWiringInfoResolver::default();
        assert_eq!(resolver.cache_size(), 0);
        assert!(resolver.default_dependency());
        assert!(resolver.is_cache_enabled());
    }

    #[test]
    fn register_and_retrieve_wiring_info() {
        let resolver = BeanWiringInfoResolver::new();
        let mut info = BeanWiringInfo::new("myBean", "MyType");
        info.is_default_dependency = false;
        resolver.register_wiring_info("myBean".to_string(), info);
        let cached = resolver.cached_wiring_info("myBean").unwrap();
        assert_eq!(cached.get_bean_name(), "myBean");
        assert!(!cached.is_default_dependency());
    }

    #[test]
    fn resolve_multiple_beans() {
        let resolver = BeanWiringInfoResolver::new();
        resolver.resolve_wiring_info("bean1", "Type1");
        resolver.resolve_wiring_info("bean2", "Type2");
        assert_eq!(resolver.cache_size(), 2);
    }

    #[test]
    fn has_cached_info_false_for_missing() {
        let resolver = BeanWiringInfoResolver::new();
        assert!(!resolver.has_cached_info("missing"));
    }
}

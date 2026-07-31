//! ClassNameBeanWiringInfoResolver — 对应 Java 类：org.springframework.beans.factory.wiring.ClassNameBeanWiringInfoResolver。
//!
//! 对应 Spring beans.factory.wiring 包。
//!
//! 在 Spring 中，`ClassNameBeanWiringInfoResolver` 通过 Bean 的类名来
//! 解析装配信息。它使用 Bean 类的全限定名作为 Bean 名称，
//! 在容器中查找对应的 Bean 定义进行自动装配。
//!
//! ## 工作原理
//!
//! 1. 获取 Bean 实例的类名
//! 2. 使用类名作为 Bean 名称在容器中查找
//! 3. 如果找到，返回对应的 `BeanWiringInfo`
//!
//! ## 使用场景
//!
//! - 与 `@Configurable` 注解配合
//! - 类名即 Bean 名称的约定
//! - AspectJ 织入的自动装配

use std::collections::HashMap;
use std::sync::Mutex;

use crate::factory::wiring::bean_wiring_info::BeanWiringInfo;

/// ClassNameBeanWiringInfoResolver — Spring 风格的类名装配信息解析器。
///
/// 对应 Java 类：`org.springframework.beans.factory.wiring.ClassNameBeanWiringInfoResolver`。
///
/// 通过 Bean 的类名来解析装配信息。
/// 将类名作为 Bean 名称在容器中查找对应的 Bean 定义。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `ClassNameBeanWiringInfoResolver` | `ClassNameBeanWiringInfoResolver` |
/// | `resolveWiringInfo(Object)` | `resolve_by_class_name(type_name)` |
/// | 使用 `bean.getClass().getName()` | 使用 `type_name` 参数 |
#[derive(Debug)]
pub struct ClassNameBeanWiringInfoResolver {
    /// 类名到 Bean 名称的映射缓存
    class_to_bean: Mutex<HashMap<String, String>>,
    /// 是否使用简短类名（不含包名）
    use_short_name: bool,
    /// 类名前缀（可选，用于过滤）
    class_name_prefix: Option<String>,
    /// 是否已初始化
    initialized: bool,
}

impl ClassNameBeanWiringInfoResolver {
    /// 创建新的 ClassNameBeanWiringInfoResolver。
    pub fn new() -> Self {
        Self {
            class_to_bean: Mutex::new(HashMap::new()),
            use_short_name: false,
            class_name_prefix: None,
            initialized: false,
        }
    }

    /// 创建使用简短类名的解析器。
    ///
    /// 例如 `com.example.MyBean` -> `MyBean`。
    pub fn with_short_name() -> Self {
        Self {
            use_short_name: true,
            ..Self::new()
        }
    }

    /// 创建带类名前缀过滤的解析器。
    ///
    /// 只处理以指定前缀开头的类名。
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            class_name_prefix: Some(prefix.into()),
            ..Self::new()
        }
    }

    /// 通过类名解析装配信息。
    ///
    /// 对应 Java 的 `BeanWiringInfo resolveWiringInfo(Object beanInstance)`。
    ///
    /// # 参数
    /// - `class_name` — Bean 的全限定类名
    ///
    /// # 返回
    /// - `Some(BeanWiringInfo)` — 解析成功
    /// - `None` — 无法解析（类名为空或不匹配前缀）
    pub fn resolve_by_class_name(&self, class_name: &str) -> Option<BeanWiringInfo> {
        if class_name.is_empty() {
            return None;
        }

        // 检查前缀过滤
        if let Some(ref prefix) = self.class_name_prefix {
            if !class_name.starts_with(prefix.as_str()) {
                return None;
            }
        }

        // 确定 Bean 名称
        let bean_name = if self.use_short_name {
            Self::short_class_name(class_name)
        } else {
            class_name.to_string()
        };

        Some(BeanWiringInfo::with_type_name(&bean_name))
    }

    /// 通过类名解析装配信息（带缓存）。
    pub fn resolve_with_cache(&self, class_name: &str) -> Option<BeanWiringInfo> {
        // 检查缓存
        {
            let cache = self.class_to_bean.lock().unwrap();
            if let Some(bean_name) = cache.get(class_name) {
                return Some(BeanWiringInfo::with_type_name(bean_name));
            }
        }

        // 解析
        let info = self.resolve_by_class_name(class_name);

        // 缓存结果
        if let Some(ref resolved) = info {
            let mut cache = self.class_to_bean.lock().unwrap();
            cache.insert(class_name.to_string(), resolved.get_type_name().to_string());
        }

        info
    }

    /// 获取简短类名（最后一个 `.` 之后的部分）。
    ///
    /// 例如 `com.example.MyBean` -> `MyBean`。
    pub fn short_class_name(full_name: &str) -> String {
        full_name
            .rfind('.')
            .map(|pos| &full_name[pos + 1..])
            .unwrap_or(full_name)
            .to_string()
    }

    /// 手动注册类名到 Bean 名称的映射。
    pub fn register_mapping(&self, class_name: String, bean_name: String) {
        let mut cache = self.class_to_bean.lock().unwrap();
        cache.insert(class_name, bean_name);
    }

    /// 获取缓存的映射。
    pub fn cached_bean_name(&self, class_name: &str) -> Option<String> {
        self.class_to_bean.lock().unwrap().get(class_name).cloned()
    }

    /// 检查是否已缓存指定类名。
    pub fn has_cached_mapping(&self, class_name: &str) -> bool {
        self.class_to_bean.lock().unwrap().contains_key(class_name)
    }

    /// 获取缓存大小。
    pub fn cache_size(&self) -> usize {
        self.class_to_bean.lock().unwrap().len()
    }

    /// 清空缓存。
    pub fn clear_cache(&self) {
        self.class_to_bean.lock().unwrap().clear();
    }

    /// 是否使用简短类名。
    pub fn is_use_short_name(&self) -> bool {
        self.use_short_name
    }

    /// 获取类名前缀。
    pub fn class_name_prefix(&self) -> Option<&str> {
        self.class_name_prefix.as_deref()
    }

    /// 初始化解析器。
    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 获取所有缓存的类名。
    pub fn cached_class_names(&self) -> Vec<String> {
        self.class_to_bean.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for ClassNameBeanWiringInfoResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resolver() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        assert!(!resolver.is_use_short_name());
        assert!(resolver.class_name_prefix().is_none());
    }

    #[test]
    fn resolve_by_full_class_name() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        let info = resolver.resolve_by_class_name("com.example.MyBean");
        assert!(info.is_some());
        assert_eq!(info.unwrap().get_type_name(), "com.example.MyBean");
    }

    #[test]
    fn resolve_empty_returns_none() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        assert!(resolver.resolve_by_class_name("").is_none());
    }

    #[test]
    fn short_class_name_extraction() {
        assert_eq!(
            ClassNameBeanWiringInfoResolver::short_class_name("com.example.MyBean"),
            "MyBean"
        );
        assert_eq!(
            ClassNameBeanWiringInfoResolver::short_class_name("MyBean"),
            "MyBean"
        );
    }

    #[test]
    fn with_short_name_resolver() {
        let resolver = ClassNameBeanWiringInfoResolver::with_short_name();
        assert!(resolver.is_use_short_name());

        let info = resolver.resolve_by_class_name("com.example.MyBean").unwrap();
        assert_eq!(info.get_type_name(), "MyBean");
    }

    #[test]
    fn with_prefix_filter() {
        let resolver = ClassNameBeanWiringInfoResolver::with_prefix("com.example");
        assert_eq!(resolver.class_name_prefix(), Some("com.example"));

        // 匹配前缀
        assert!(resolver.resolve_by_class_name("com.example.MyBean").is_some());
        // 不匹配前缀
        assert!(resolver.resolve_by_class_name("org.other.Bean").is_none());
    }

    #[test]
    fn resolve_with_cache() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        resolver.resolve_with_cache("com.example.MyBean");
        assert!(resolver.has_cached_mapping("com.example.MyBean"));
        assert_eq!(resolver.cache_size(), 1);
    }

    #[test]
    fn register_manual_mapping() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        resolver.register_mapping(
            "com.example.MyBean".to_string(),
            "myBean".to_string(),
        );

        assert_eq!(
            resolver.cached_bean_name("com.example.MyBean"),
            Some("myBean".to_string())
        );
    }

    #[test]
    fn clear_cache() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        resolver.resolve_with_cache("com.example.Bean");
        assert_eq!(resolver.cache_size(), 1);

        resolver.clear_cache();
        assert_eq!(resolver.cache_size(), 0);
    }

    #[test]
    fn cached_bean_name_none_for_missing() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        assert!(resolver.cached_bean_name("missing").is_none());
    }

    #[test]
    fn initialize_sets_flag() {
        let mut resolver = ClassNameBeanWiringInfoResolver::new();
        assert!(!resolver.is_initialized());
        resolver.initialize();
        assert!(resolver.is_initialized());
    }

    #[test]
    fn cached_class_names() {
        let resolver = ClassNameBeanWiringInfoResolver::new();
        resolver.resolve_with_cache("com.a.Bean1");
        resolver.resolve_with_cache("com.b.Bean2");

        let mut names = resolver.cached_class_names();
        names.sort();
        assert_eq!(names, vec!["com.a.Bean1", "com.b.Bean2"]);
    }
}

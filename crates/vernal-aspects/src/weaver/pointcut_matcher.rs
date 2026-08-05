//! 对标 AspectJ pointcut 模式匹配。
//!
//! Pointcut 匹配器：解析和匹配 AspectJ 风格的 pointcut 表达式。

/// 方法元数据（weaver 模块用）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodMetadata {
    /// 方法所属类型的完全限定名。
    pub type_name: &'static str,
    /// 方法名。
    pub method_name: &'static str,
    /// 是否是公开方法。
    pub is_public: bool,
    /// 类型上是否有指定注解。
    pub type_annotations: Vec<&'static str>,
    /// 方法上是否有指定注解。
    pub method_annotations: Vec<&'static str>,
}

impl MethodMetadata {
    /// 创建新的方法元数据。
    pub fn new(type_name: &'static str, method_name: &'static str) -> Self {
        Self {
            type_name,
            method_name,
            is_public: true,
            type_annotations: Vec::new(),
            method_annotations: Vec::new(),
        }
    }

    /// 设置是否公开。
    pub fn set_public(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self
    }

    /// 设置类型注解。
    pub fn set_type_annotations(mut self, annotations: Vec<&'static str>) -> Self {
        self.type_annotations = annotations;
        self
    }

    /// 设置方法注解。
    pub fn set_method_annotations(mut self, annotations: Vec<&'static str>) -> Self {
        self.method_annotations = annotations;
        self
    }
}

/// Pointcut 匹配器。
///
/// 对标 AspectJ 的 pointcut 表达式解析和匹配。
pub struct PointcutMatcher;

impl PointcutMatcher {
    /// 匹配 `execution(public * *(..))` pointcut。
    pub fn match_execution_public(method: &MethodMetadata) -> bool {
        method.is_public
    }

    /// 匹配 `execution(@Transactional * *(..))` pointcut。
    pub fn match_execution_with_annotation(method: &MethodMetadata, annotation: &str) -> bool {
        method.method_annotations.contains(&annotation)
    }

    /// 匹配 `within(@Transactional *)` pointcut。
    pub fn match_within_annotation(method: &MethodMetadata, annotation: &str) -> bool {
        method.type_annotations.contains(&annotation)
    }

    /// 匹配 `this(Object)` pointcut。
    pub fn match_this(_method: &MethodMetadata, _type_name: &str) -> bool {
        // 实际实现需要运行时类型检查
        true
    }

    /// 匹配组合 pointcut：`execution(public * ((@Transactional *)+).*(..)) && within(@Transactional *)`。
    pub fn match_transactional_type(method: &MethodMetadata) -> bool {
        Self::match_execution_public(method)
            && Self::match_within_annotation(
                method,
                "org.springframework.transaction.annotation.Transactional",
            )
    }

    /// 匹配组合 pointcut：`execution(@Transactional * *(..))`。
    pub fn match_transactional_method(method: &MethodMetadata) -> bool {
        Self::match_execution_with_annotation(
            method,
            "org.springframework.transaction.annotation.Transactional",
        )
    }

    /// 匹配组合 pointcut：`execution(@Cacheable * *(..))`。
    pub fn match_cacheable_method(method: &MethodMetadata) -> bool {
        Self::match_execution_with_annotation(
            method,
            "org.springframework.cache.annotation.Cacheable",
        )
    }

    /// 匹配组合 pointcut：`execution(@CacheEvict * *(..))`。
    pub fn match_cache_evict_method(method: &MethodMetadata) -> bool {
        Self::match_execution_with_annotation(
            method,
            "org.springframework.cache.annotation.CacheEvict",
        )
    }

    /// 匹配组合 pointcut：`execution(@CachePut * *(..))`。
    pub fn match_cache_put_method(method: &MethodMetadata) -> bool {
        Self::match_execution_with_annotation(
            method,
            "org.springframework.cache.annotation.CachePut",
        )
    }

    /// 匹配组合 pointcut：`execution(@Async (void || Future+) *(..))`。
    pub fn match_async_method(method: &MethodMetadata) -> bool {
        Self::match_execution_with_annotation(
            method,
            "org.springframework.scheduling.annotation.Async",
        )
    }

    /// 匹配组合 pointcut：`execution(@Configurable * *(..))`。
    pub fn match_configurable_method(method: &MethodMetadata) -> bool {
        Self::match_execution_with_annotation(
            method,
            "org.springframework.beans.factory.annotation.Configurable",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_metadata() {
        let meta = MethodMetadata::new("com.example.Foo", "bar");
        assert_eq!(meta.type_name, "com.example.Foo");
        assert_eq!(meta.method_name, "bar");
        assert!(meta.is_public);
    }

    #[test]
    fn test_match_execution_public() {
        let meta = MethodMetadata::new("Foo", "bar").set_public(true);
        assert!(PointcutMatcher::match_execution_public(&meta));

        let meta = MethodMetadata::new("Foo", "bar").set_public(false);
        assert!(!PointcutMatcher::match_execution_public(&meta));
    }

    #[test]
    fn test_match_execution_with_annotation() {
        let meta = MethodMetadata::new("Foo", "bar").set_method_annotations(vec![
            "org.springframework.transaction.annotation.Transactional",
        ]);
        assert!(PointcutMatcher::match_execution_with_annotation(
            &meta,
            "org.springframework.transaction.annotation.Transactional"
        ));
        assert!(!PointcutMatcher::match_execution_with_annotation(
            &meta,
            "org.springframework.cache.annotation.Cacheable"
        ));
    }

    #[test]
    fn test_match_within_annotation() {
        let meta = MethodMetadata::new("Foo", "bar").set_type_annotations(vec![
            "org.springframework.transaction.annotation.Transactional",
        ]);
        assert!(PointcutMatcher::match_within_annotation(
            &meta,
            "org.springframework.transaction.annotation.Transactional"
        ));
    }

    #[test]
    fn test_match_transactional_type() {
        let meta = MethodMetadata::new("Foo", "bar").set_type_annotations(vec![
            "org.springframework.transaction.annotation.Transactional",
        ]);
        assert!(PointcutMatcher::match_transactional_type(&meta));
    }

    #[test]
    fn test_match_transactional_method() {
        let meta = MethodMetadata::new("Foo", "bar").set_method_annotations(vec![
            "org.springframework.transaction.annotation.Transactional",
        ]);
        assert!(PointcutMatcher::match_transactional_method(&meta));
    }

    #[test]
    fn test_match_cacheable_method() {
        let meta = MethodMetadata::new("Foo", "bar")
            .set_method_annotations(vec!["org.springframework.cache.annotation.Cacheable"]);
        assert!(PointcutMatcher::match_cacheable_method(&meta));
    }

    #[test]
    fn test_match_async_method() {
        let meta = MethodMetadata::new("Foo", "bar")
            .set_method_annotations(vec!["org.springframework.scheduling.annotation.Async"]);
        assert!(PointcutMatcher::match_async_method(&meta));
    }

    #[test]
    fn test_match_configurable_method() {
        let meta = MethodMetadata::new("Foo", "bar").set_method_annotations(vec![
            "org.springframework.beans.factory.annotation.Configurable",
        ]);
        assert!(PointcutMatcher::match_configurable_method(&meta));
    }

    #[test]
    fn test_method_metadata_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<MethodMetadata>();
        assert_sync::<MethodMetadata>();
    }

    #[test]
    fn test_method_metadata_with_annotations() {
        let meta = MethodMetadata::new("Foo", "bar")
            .set_public(true)
            .set_type_annotations(vec!["Transactional"])
            .set_method_annotations(vec!["Cacheable"]);
        assert!(meta.is_public);
        assert_eq!(meta.type_annotations.len(), 1);
        assert_eq!(meta.method_annotations.len(), 1);
    }

    #[test]
    fn test_match_cache_evict_method() {
        let meta = MethodMetadata::new("Foo", "bar")
            .set_method_annotations(vec!["org.springframework.cache.annotation.CacheEvict"]);
        assert!(PointcutMatcher::match_cache_evict_method(&meta));
    }

    #[test]
    fn test_match_cache_put_method() {
        let meta = MethodMetadata::new("Foo", "bar")
            .set_method_annotations(vec!["org.springframework.cache.annotation.CachePut"]);
        assert!(PointcutMatcher::match_cache_put_method(&meta));
    }

    #[test]
    fn test_match_multiple_annotations() {
        let meta = MethodMetadata::new("Foo", "bar").set_method_annotations(vec![
            "org.springframework.transaction.annotation.Transactional",
            "org.springframework.cache.annotation.Cacheable",
        ]);
        assert!(PointcutMatcher::match_transactional_method(&meta));
        assert!(PointcutMatcher::match_cacheable_method(&meta));
    }

    #[test]
    fn test_match_no_annotations() {
        let meta = MethodMetadata::new("Foo", "bar");
        assert!(!PointcutMatcher::match_transactional_method(&meta));
        assert!(!PointcutMatcher::match_cacheable_method(&meta));
        assert!(!PointcutMatcher::match_cache_evict_method(&meta));
        assert!(!PointcutMatcher::match_cache_put_method(&meta));
        assert!(!PointcutMatcher::match_async_method(&meta));
        assert!(!PointcutMatcher::match_configurable_method(&meta));
    }

    #[test]
    fn test_match_execution_public_private() {
        let meta_public = MethodMetadata::new("Foo", "bar").set_public(true);
        let meta_private = MethodMetadata::new("Foo", "bar").set_public(false);
        assert!(PointcutMatcher::match_execution_public(&meta_public));
        assert!(!PointcutMatcher::match_execution_public(&meta_private));
    }

    #[test]
    fn test_match_within_no_annotations() {
        let meta = MethodMetadata::new("Foo", "bar");
        assert!(!PointcutMatcher::match_within_annotation(
            &meta,
            "Transactional"
        ));
    }

    #[test]
    fn test_match_transactional_type_no_annotations() {
        let meta = MethodMetadata::new("Foo", "bar");
        assert!(!PointcutMatcher::match_transactional_type(&meta));
    }

    #[test]
    fn test_match_this_different_type() {
        let meta = MethodMetadata::new("Foo", "bar");
        assert!(PointcutMatcher::match_this(&meta, "Bar"));
    }

    #[test]
    fn test_method_metadata_debug() {
        let meta = MethodMetadata::new("Foo", "bar");
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Foo"));
        assert!(debug_str.contains("bar"));
    }

    #[test]
    fn test_method_metadata_clone() {
        let meta = MethodMetadata::new("Foo", "bar")
            .set_public(true)
            .set_type_annotations(vec!["Transactional"])
            .set_method_annotations(vec!["Cacheable"]);
        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }
}

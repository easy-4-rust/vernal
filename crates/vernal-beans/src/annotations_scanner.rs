//! AnnotationsScanner — Spring 风格的注解扫描器 trait。
//!
//! 对应 Java 类：`org.springframework.core.type.classreading.AnnotationsScanner`。
//!
//! 从给定的源（类名、资源等）中扫描并采集注解元数据。

use std::sync::Arc;

use crate::annotation_metadata::AnnotationMetadata;

/// Spring 风格的注解扫描器 trait。
///
/// 对应 Spring 的 `AnnotationsScanner`。
///
/// 扫描给定目标上的注解，返回类型擦除的 `AnnotationMetadata`。
/// 实现可以是基于反射、基于编译期宏产物或基于外部描述文件的扫描器。
pub trait AnnotationsScanner: Send + Sync {
    /// 扫描指定类名上的注解。
    ///
    /// 对应 Spring 的扫描类/方法/字段注解的核心能力。
    ///
    /// # 参数
    ///
    /// * `class_name` — 要扫描的类全限定名
    ///
    /// # 返回
    ///
    /// 扫描到的注解元数据；若目标不可扫描或无注解，返回 `None`。
    fn scan_class(
        &self,
        class_name: &str,
    ) -> Result<Option<Arc<dyn AnnotationMetadata>>, Box<dyn std::error::Error + Send + Sync>>;

    /// 扫描一组类名，返回所有采集到的注解元数据。
    ///
    /// 默认实现逐个调用 `scan_class` 并收集非 `None` 结果。
    fn scan(
        &self,
        class_names: &[String],
    ) -> Result<Vec<Arc<dyn AnnotationMetadata>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut out = Vec::new();
        for name in class_names {
            if let Some(meta) = self.scan_class(name)? {
                out.push(meta);
            }
        }
        Ok(out)
    }
}

/// 基于预注册映射的简单扫描器实现。
///
/// 将类名映射到预先构造好的 `AnnotationMetadata`，便于测试和静态装配。
pub struct StaticAnnotationsScanner {
    entries: std::collections::HashMap<String, Arc<dyn AnnotationMetadata>>,
}

impl std::fmt::Debug for StaticAnnotationsScanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticAnnotationsScanner")
            .field("entries", &self.entries.len())
            .finish()
    }
}

impl StaticAnnotationsScanner {
    /// 创建空的静态扫描器。
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    /// 注册一条类名到注解元数据的映射。
    pub fn register(&mut self, class_name: impl Into<String>, meta: Arc<dyn AnnotationMetadata>) {
        self.entries.insert(class_name.into(), meta);
    }
}

impl Default for StaticAnnotationsScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl AnnotationsScanner for StaticAnnotationsScanner {
    fn scan_class(
        &self,
        class_name: &str,
    ) -> Result<Option<Arc<dyn AnnotationMetadata>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.entries.get(class_name).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation_metadata::AnnotationDescriptor;

    struct EmptyMetadata;
    impl AnnotationMetadata for EmptyMetadata {
        fn annotations(&self) -> &[AnnotationDescriptor] {
            &[]
        }
    }

    #[test]
    fn test_static_scanner() {
        let mut scanner = StaticAnnotationsScanner::new();
        scanner.register("com.example.Foo", Arc::new(EmptyMetadata));
        assert!(scanner.scan_class("com.example.Foo").unwrap().is_some());
        assert!(scanner.scan_class("com.example.Missing").unwrap().is_none());

        let batch = scanner
            .scan(&[
                "com.example.Foo".to_owned(),
                "com.example.Missing".to_owned(),
            ])
            .unwrap();
        assert_eq!(batch.len(), 1);
    }
}

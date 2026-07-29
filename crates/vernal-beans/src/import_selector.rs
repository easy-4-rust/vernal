//! ImportSelector — Spring 风格的导入选择器 trait。
//!
//! 对应 Java 类：`org.springframework.context.annotation.ImportSelector`。
//!
//! 根据导入类的注解元数据，动态决定需要导入的类全限定名集合。

use crate::annotation_metadata::AnnotationMetadata;

/// Spring 风格的导入选择器 trait。
///
/// 对应 Spring 的 `ImportSelector`。
///
/// 实现该 trait 的类型在被 `@Import` 引入时，
/// 容器会调用 `select_imports` 并将返回的类名当作新的候选类继续处理。
pub trait ImportSelector: Send + Sync {
    /// 根据导入类的注解元数据选择要导入的类全限定名集合。
    ///
    /// 对应 Spring 的 `selectImports(AnnotationMetadata importingClassMetadata)`。
    ///
    /// # 参数
    ///
    /// * `importing_metadata` — 触发导入的类的注解元数据
    ///
    /// # 返回
    ///
    /// 需要导入的类全限定名列表；空切片表示不导入任何额外类。
    fn select_imports(&self, importing_metadata: &dyn AnnotationMetadata) -> Vec<String>;

    /// 是否排除（过滤）某个候选导入类。
    ///
    /// 对应 Spring 的 `excludeFilter` 周边语义。
    /// 默认返回 `false`（不排除任何类）。
    fn is_excluded(&self, _class_name: &str) -> bool {
        false
    }
}

/// 始终返回固定导入集合的选择器。
///
/// 便于在测试或静态装配中表达「导入这些类」的需求。
pub struct FixedImportSelector {
    imports: Vec<String>,
    excluded: Vec<String>,
}

impl std::fmt::Debug for FixedImportSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixedImportSelector")
            .field("imports", &self.imports)
            .field("excluded", &self.excluded)
            .finish()
    }
}

impl FixedImportSelector {
    /// 创建固定导入选择器。
    pub fn new<I, S>(imports: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            imports: imports.into_iter().map(Into::into).collect(),
            excluded: Vec::new(),
        }
    }

    /// 添加一个排除项。
    pub fn with_excluded<I, S>(mut self, excluded: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.excluded = excluded.into_iter().map(Into::into).collect();
        self
    }
}

impl ImportSelector for FixedImportSelector {
    fn select_imports(&self, _importing_metadata: &dyn AnnotationMetadata) -> Vec<String> {
        self.imports
            .iter()
            .filter(|name| !self.excluded.iter().any(|e| e == *name))
            .cloned()
            .collect()
    }

    fn is_excluded(&self, class_name: &str) -> bool {
        self.excluded.iter().any(|e| e == class_name)
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
    fn test_fixed_selector() {
        let selector = FixedImportSelector::new(["com.example.A", "com.example.B"])
            .with_excluded(["com.example.B"]);
        let meta = EmptyMetadata;
        let imports = selector.select_imports(&meta);
        assert_eq!(imports, vec!["com.example.A".to_owned()]);
        assert!(selector.is_excluded("com.example.B"));
    }
}

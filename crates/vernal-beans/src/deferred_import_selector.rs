//! DeferredImportSelector — 延迟导入选择器。
//!
//! 对应 Java 类：`org.springframework.context.annotation.DeferredImportSelector`。
//!
//! 扩展 `ImportSelector`，将导入决策推迟到所有配置类处理完毕之后执行。

use std::sync::Arc;

use crate::annotation_metadata::AnnotationMetadata;
use crate::import_selector::ImportSelector;

/// 延迟导入的运行入口。
///
/// 对应 Spring 的 `DeferredImportSelector.Group`。
///
/// 收集所有延迟选择器，最终统一执行 `process` 并返回聚合后的导入类名集合。
pub trait DeferredImportSelectorGroup: Send + Sync {
    /// 处理单个延迟选择器，收集其导入结果。
    ///
    /// 对应 Spring 的 `Group#process(AnnotationMetadata, DeferredImportSelector)`。
    fn process(
        &mut self,
        importing_metadata: &dyn AnnotationMetadata,
        selector: &dyn DeferredImportSelector,
    );

    /// 返回该组聚合后的导入类名列表。
    ///
    /// 对应 Spring 的 `Group#selectImports()`。
    fn select_imports(&self) -> Vec<String>;
}

/// 延迟导入选择器。
///
/// 对应 Spring 的 `DeferredImportSelector`。
///
/// 与普通 `ImportSelector` 不同，延迟选择器在所有 `@Configuration` 类被
/// 解析完毕后才执行，适用于需要全局视图的导入逻辑（例如条件化配置）。
pub trait DeferredImportSelector: ImportSelector {
    /// 提供此选择器归属的 `Group` 类型。
    ///
    /// 默认返回 `None`，表示使用默认组。
    fn group(&self) -> Option<Arc<dyn DeferredImportSelectorGroup>> {
        None
    }
}

/// 默认的延迟导入组实现。
///
/// 按调用顺序收集所有选择器的导入结果。
#[derive(Default)]
pub struct DefaultDeferredImportGroup {
    collected: Vec<String>,
}

impl std::fmt::Debug for DefaultDeferredImportGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultDeferredImportGroup")
            .field("collected", &self.collected)
            .finish()
    }
}

impl DefaultDeferredImportGroup {
    /// 创建空的默认组。
    pub fn new() -> Self {
        Self::default()
    }

    /// 已收集的导入数。
    pub fn len(&self) -> usize {
        self.collected.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.collected.is_empty()
    }

    /// 清空已收集结果。
    pub fn clear(&mut self) {
        self.collected.clear();
    }
}

impl DeferredImportSelectorGroup for DefaultDeferredImportGroup {
    fn process(
        &mut self,
        importing_metadata: &dyn AnnotationMetadata,
        selector: &dyn DeferredImportSelector,
    ) {
        let mut imports = selector.select_imports(importing_metadata);
        self.collected.append(&mut imports);
    }

    fn select_imports(&self) -> Vec<String> {
        self.collected.clone()
    }
}

/// 基于固定集合的延迟选择器实现。
pub struct FixedDeferredImportSelector {
    imports: Vec<String>,
}

impl std::fmt::Debug for FixedDeferredImportSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixedDeferredImportSelector")
            .field("imports", &self.imports)
            .finish()
    }
}

impl FixedDeferredImportSelector {
    /// 创建固定延迟选择器。
    pub fn new<I, S>(imports: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            imports: imports.into_iter().map(Into::into).collect(),
        }
    }
}

impl ImportSelector for FixedDeferredImportSelector {
    fn select_imports(&self, _importing_metadata: &dyn AnnotationMetadata) -> Vec<String> {
        self.imports.clone()
    }
}

impl DeferredImportSelector for FixedDeferredImportSelector {}

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
    fn test_default_group() {
        let mut group = DefaultDeferredImportGroup::new();
        let selector = FixedDeferredImportSelector::new(["com.example.A", "com.example.B"]);
        let meta = EmptyMetadata;
        group.process(&meta, &selector);
        assert_eq!(group.select_imports().len(), 2);
        assert_eq!(
            group.select_imports(),
            vec!["com.example.A".to_owned(), "com.example.B".to_owned()]
        );
    }
}

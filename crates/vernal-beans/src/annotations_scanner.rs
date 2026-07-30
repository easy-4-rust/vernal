//! AnnotationsScanner — 注解扫描器。
use crate::annotation_metadata::AnnotationMetadata;

/// 注解扫描器。
#[derive(Clone, Debug, Default)]
pub struct AnnotationsScanner;
impl AnnotationsScanner {
    pub fn new() -> Self { Self }
    pub fn scan(&self, _type_name: &str) -> AnnotationMetadata {
        AnnotationMetadata::new("unknown")
    }
}

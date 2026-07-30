//! MetadataReader — 元数据读取器。
use crate::annotation_metadata::AnnotationMetadata;

/// 元数据读取器 trait。
pub trait MetadataReader: Send + Sync {
    fn get_annotation_metadata(&self) -> AnnotationMetadata;
    fn get_class_metadata(&self) -> AnnotationMetadata;
}

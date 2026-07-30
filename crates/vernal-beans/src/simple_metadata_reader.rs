//! SimpleMetadataReader — 简单元数据读取器。
use crate::annotation_metadata::AnnotationMetadata;
use crate::metadata_reader::MetadataReader;

/// 简单元数据读取器。
#[derive(Clone, Debug)]
pub struct SimpleMetadataReader {
    pub type_name: String,
}
impl SimpleMetadataReader {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into() }
    }
}
impl MetadataReader for SimpleMetadataReader {
    fn get_annotation_metadata(&self) -> AnnotationMetadata {
        AnnotationMetadata::new(&self.type_name)
    }
    fn get_class_metadata(&self) -> AnnotationMetadata {
        AnnotationMetadata::new(&self.type_name)
    }
}

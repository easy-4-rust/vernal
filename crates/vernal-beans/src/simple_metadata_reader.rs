//! SimpleMetadataReader — 元数据读取器的简单实现。
//!
//! 对应 Java 类：Spring `SimpleMetadataReader`（`org.springframework.core.type.classreading.SimpleMetadataReader`）。
//!
//! 将注解元数据与类元数据存储在内存映射中，便于在无反射环境下使用。

use std::collections::HashMap;
use std::sync::Arc;

use crate::annotation_metadata::{AnnotationDescriptor, AnnotationMetadata};
use crate::metadata_reader::{ClassMetadata, MetadataReader};

/// 简单的类元数据实现。
#[derive(Debug, Clone)]
pub struct SimpleClassMetadata {
    class_name: String,
    is_interface: bool,
    is_abstract: bool,
    is_final: bool,
    super_class_name: Option<String>,
    interface_names: Vec<String>,
}

impl SimpleClassMetadata {
    /// 创建新的类元数据。
    pub fn new(class_name: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            is_interface: false,
            is_abstract: false,
            is_final: false,
            super_class_name: None,
            interface_names: Vec::new(),
        }
    }

    /// 设置是否接口。
    pub fn with_interface(mut self, is_interface: bool) -> Self {
        self.is_interface = is_interface;
        self
    }

    /// 设置是否抽象。
    pub fn with_abstract(mut self, is_abstract: bool) -> Self {
        self.is_abstract = is_abstract;
        self
    }

    /// 设置是否最终。
    pub fn with_final(mut self, is_final: bool) -> Self {
        self.is_final = is_final;
        self
    }

    /// 设置父类名。
    pub fn with_super_class(mut self, super_class: impl Into<String>) -> Self {
        self.super_class_name = Some(super_class.into());
        self
    }

    /// 设置实现的接口列表。
    pub fn with_interfaces<I, S>(mut self, interfaces: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.interface_names = interfaces.into_iter().map(Into::into).collect();
        self
    }
}

impl ClassMetadata for SimpleClassMetadata {
    fn class_name(&self) -> &str {
        &self.class_name
    }

    fn is_interface(&self) -> bool {
        self.is_interface
    }

    fn is_abstract(&self) -> bool {
        self.is_abstract
    }

    fn is_final(&self) -> bool {
        self.is_final
    }

    fn super_class_name(&self) -> Option<&str> {
        self.super_class_name.as_deref()
    }

    fn interface_names(&self) -> &[String] {
        &self.interface_names
    }
}

/// 简单的注解元数据实现，内部保存注解描述列表。
#[derive(Debug, Clone, Default)]
pub struct SimpleAnnotationMetadata {
    annotations: Vec<AnnotationDescriptor>,
}

impl SimpleAnnotationMetadata {
    /// 创建空的注解元数据。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从注解列表创建。
    pub fn from_annotations(annotations: Vec<AnnotationDescriptor>) -> Self {
        Self { annotations }
    }

    /// 添加一个注解描述。
    pub fn add(&mut self, descriptor: AnnotationDescriptor) {
        self.annotations.push(descriptor);
    }
}

impl AnnotationMetadata for SimpleAnnotationMetadata {
    fn annotations(&self) -> &[AnnotationDescriptor] {
        &self.annotations
    }
}

/// 元数据读取器的简单实现。
///
/// 对应 Spring 的 `SimpleMetadataReader`。
///
/// 持有单个类的注解元数据、类元数据与资源描述，
/// 通常由 `SimpleMetadataReaderFactory` 按类名预填充后构造。
pub struct SimpleMetadataReader {
    annotation_metadata: Arc<SimpleAnnotationMetadata>,
    class_metadata: Arc<SimpleClassMetadata>,
    resource_description: String,
}

impl std::fmt::Debug for SimpleMetadataReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMetadataReader")
            .field("resource_description", &self.resource_description)
            .field(
                "annotation_count",
                &self.annotation_metadata.annotations().len(),
            )
            .field("class_name", &self.class_metadata.class_name())
            .finish()
    }
}

impl SimpleMetadataReader {
    /// 创建新的 `SimpleMetadataReader`。
    pub fn new(
        annotation_metadata: Arc<SimpleAnnotationMetadata>,
        class_metadata: Arc<SimpleClassMetadata>,
        resource_description: impl Into<String>,
    ) -> Self {
        Self {
            annotation_metadata,
            class_metadata,
            resource_description: resource_description.into(),
        }
    }

    /// 获取注解元数据（强类型）。
    pub fn simple_annotation_metadata(&self) -> &SimpleAnnotationMetadata {
        &self.annotation_metadata
    }

    /// 获取类元数据（强类型）。
    pub fn simple_class_metadata(&self) -> &SimpleClassMetadata {
        &self.class_metadata
    }
}

impl MetadataReader for SimpleMetadataReader {
    fn get_annotation_metadata(&self) -> Arc<dyn AnnotationMetadata> {
        self.annotation_metadata.clone()
    }

    fn get_class_metadata(&self) -> Arc<dyn ClassMetadata> {
        self.class_metadata.clone()
    }

    fn resource_description(&self) -> &str {
        &self.resource_description
    }
}

/// 简单的元数据读取器工厂。
///
/// 对应 Spring 的 `SimpleMetadataReaderFactory`。
///
/// 维护类名到预构造 `SimpleMetadataReader` 的映射。
pub struct SimpleMetadataReaderFactory {
    readers: HashMap<String, Arc<SimpleMetadataReader>>,
}

impl std::fmt::Debug for SimpleMetadataReaderFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMetadataReaderFactory")
            .field("readers", &self.readers.len())
            .finish()
    }
}

impl SimpleMetadataReaderFactory {
    /// 创建空的工厂。
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
        }
    }

    /// 注册一个预构造的读取器。
    pub fn register(&mut self, class_name: impl Into<String>, reader: Arc<SimpleMetadataReader>) {
        self.readers.insert(class_name.into(), reader);
    }

    /// 获取指定类的读取器。
    pub fn get_metadata_reader(&self, class_name: &str) -> Option<Arc<SimpleMetadataReader>> {
        self.readers.get(class_name).cloned()
    }

    /// 已注册的读取器数量。
    pub fn len(&self) -> usize {
        self.readers.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.readers.is_empty()
    }
}

impl Default for SimpleMetadataReaderFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_class_metadata() {
        let meta = SimpleClassMetadata::new("com.example.Foo")
            .with_super_class("com.example.Base")
            .with_interface(true)
            .with_interfaces(["java.io.Serializable"]);
        assert_eq!(meta.class_name(), "com.example.Foo");
        assert!(meta.is_interface());
        assert!(!meta.is_concrete());
        assert!(meta.has_super_class());
        assert_eq!(meta.interface_names().len(), 1);
    }

    #[test]
    fn test_simple_metadata_reader() {
        let mut annotations = SimpleAnnotationMetadata::new();
        annotations.add(AnnotationDescriptor::new_class("Component"));
        let class = SimpleClassMetadata::new("com.example.Foo");

        let reader = SimpleMetadataReader::new(
            Arc::new(annotations),
            Arc::new(class),
            "classpath:com/example/Foo.class",
        );
        assert!(reader.get_annotation_metadata().has_annotation("Component"));
        assert_eq!(reader.get_class_metadata().class_name(), "com.example.Foo");
        assert_eq!(
            reader.resource_description(),
            "classpath:com/example/Foo.class"
        );
    }
}

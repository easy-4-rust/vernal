//! MetadataReader — Spring 风格的元数据读取器 trait。
//!
//! 对应 Java 类：`org.springframework.core.type.classreading.MetadataReader`。
//!
//! 提供对单个类的注解元数据与类元数据的读取访问。

use std::sync::Arc;

use crate::annotation_metadata::AnnotationMetadata;

/// Spring 风格的类元数据 trait。
///
/// 对应 Spring 的 `ClassMetadata`。
///
/// 描述单个类的结构信息：类全限定名、是否接口、是否抽象等。
pub trait ClassMetadata: Send + Sync {
    /// 类全限定名。
    fn class_name(&self) -> &str;

    /// 是否为接口。
    fn is_interface(&self) -> bool {
        false
    }

    /// 是否为抽象类。
    fn is_abstract(&self) -> bool {
        false
    }

    /// 是否为最终类。
    fn is_final(&self) -> bool {
        false
    }

    /// 是否独立可实例化（非抽象、非接口）。
    fn is_concrete(&self) -> bool {
        !self.is_interface() && !self.is_abstract()
    }

    /// 是否有父类信息（非 `Object`）。
    fn has_super_class(&self) -> bool {
        self.super_class_name().is_some()
    }

    /// 父类全限定名。
    fn super_class_name(&self) -> Option<&str> {
        None
    }

    /// 实现的接口全限定名列表。
    fn interface_names(&self) -> &[String] {
        &[]
    }
}

/// Spring 风格的元数据读取器 trait。
///
/// 对应 Spring 的 `MetadataReader`。
///
/// 封装对单个类的元数据读取能力，提供注解元数据与类元数据的统一访问入口。
/// `MetadataReaderFactory` 负责按类名创建 `MetadataReader` 实例。
pub trait MetadataReader: Send + Sync {
    /// 获取该类对应的注解元数据。
    ///
    /// 对应 Spring 的 `getAnnotationMetadata()`。
    fn get_annotation_metadata(&self) -> Arc<dyn AnnotationMetadata>;

    /// 获取该类对应的类元数据。
    ///
    /// 对应 Spring 的 `getClassMetadata()`。
    fn get_class_metadata(&self) -> Arc<dyn ClassMetadata>;

    /// 获取底层资源描述（如类路径位置或文件路径）。
    ///
    /// 对应 Spring 的 `getResource()`。
    fn resource_description(&self) -> &str;
}

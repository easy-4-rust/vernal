//! BeanMetadataElement — Spring 风格的 Bean 元数据元素 trait。
//!
//! 对应 Java 类：`org.springframework.beans.BeanMetadataElement`。
//!
//! 表示具有配置来源的 Bean 元数据元素。

use std::any::Any;

/// Spring 风格的 Bean 元数据元素 trait。
///
/// 对应 Spring 的 `BeanMetadataElement`。
///
/// 实现此 trait 的类型可以关联一个配置来源（source），
/// 用于错误报告和诊断。默认实现返回 `None`。
pub trait BeanMetadataElement {
    /// 获取此元数据元素的配置来源。
    ///
    /// 返回关联的源对象引用，通常用于错误报告和日志记录。
    /// 默认返回 `None`。
    fn get_source(&self) -> Option<&dyn Any> {
        None
    }
}

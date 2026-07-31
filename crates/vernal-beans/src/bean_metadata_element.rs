//! BeanMetadataElement — 对应 Spring `org.springframework.beans.BeanMetadataElement`。
//!
//! 由携带配置源对象的 bean 元数据元素实现的接口。

use std::any::Any;

/// 由携带配置源对象的 bean 元数据元素实现的接口。
///
/// 对应 Java 接口：`org.springframework.beans.BeanMetadataElement`。
///
/// 允许元数据元素携带关于其配置源对象的信息。
pub trait BeanMetadataElement: Any + Send + Sync {
    /// 返回此元数据元素的配置源对象（如果有）。
    ///
    /// 对应 Java 方法：`Object getSource()`
    fn source(&self) -> Option<&(dyn Any + 'static)> {
        None
    }
}

/// 默认实现，不携带源对象。
#[derive(Debug, Clone)]
pub struct SimpleBeanMetadataElement;

impl BeanMetadataElement for SimpleBeanMetadataElement {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_source() {
        let element = SimpleBeanMetadataElement;
        assert!(element.source().is_none());
    }
}

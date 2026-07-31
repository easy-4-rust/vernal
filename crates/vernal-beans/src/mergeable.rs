//! Mergeable — 对应 Spring `org.springframework.beans.Mergeable`。
//!
//! 表示可以合并的 bean 元素的接口。

use std::any::Any;

/// 表示可以合并的 bean 元素的接口。
///
/// 对应 Java 接口：`org.springframework.beans.Mergeable`。
///
/// 由可以合并的 bean 元素实现。合并用于覆盖父 bean 定义中的值。
pub trait Mergeable: Any + Send + Sync {
    /// 返回此元素是否可以与给定的父元素合并。
    ///
    /// 对应 Java 方法：`boolean isMergeEnabled()`
    fn is_merge_enabled(&self) -> bool {
        false
    }

    /// 将给定的父值与此元素合并。
    ///
    /// 对应 Java 方法：`Object merge(Object parentVal)`
    ///
    /// # Arguments
    /// * `parent_val` - 父 bean 定义中的值
    ///
    /// # Returns
    /// 合并后的值
    fn merge(&self, _parent_val: &(dyn Any + 'static)) -> Result<Box<dyn Any + Send + Sync>, String> {
        Err(format!(
            "Merge not supported for type '{}'",
            std::any::type_name::<Self>()
        ))
    }
}

/// 一个简单的 Mergeable 实现，用于测试和默认行为。
#[derive(Debug, Clone, Default)]
pub struct SimpleMergeable {
    merge_enabled: bool,
    value: Option<String>,
}

impl SimpleMergeable {
    /// 创建一个新的 SimpleMergeable。
    pub fn new(merge_enabled: bool) -> Self {
        Self {
            merge_enabled,
            value: None,
        }
    }

    /// 创建一个带有值的 SimpleMergeable。
    pub fn with_value(merge_enabled: bool, value: impl Into<String>) -> Self {
        Self {
            merge_enabled,
            value: Some(value.into()),
        }
    }

    /// 获取值。
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl Mergeable for SimpleMergeable {
    fn is_merge_enabled(&self) -> bool {
        self.merge_enabled
    }

    fn merge(&self, parent_val: &(dyn Any + 'static)) -> Result<Box<dyn Any + Send + Sync>, String> {
        if !self.merge_enabled {
            return Err("Merge is not enabled for this element".to_string());
        }
        // 简单合并策略：如果子值存在则使用子值，否则使用父值
        if let Some(ref val) = self.value {
            Ok(Box::new(val.clone()))
        } else if let Some(parent_str) = parent_val.downcast_ref::<String>() {
            Ok(Box::new(parent_str.clone()))
        } else {
            Err("Cannot merge: parent value is not a String".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mergeable_default() {
        let m = SimpleMergeable::new(false);
        assert!(!m.is_merge_enabled());
        assert!(m.value().is_none());
    }

    #[test]
    fn test_mergeable_with_value() {
        let m = SimpleMergeable::with_value(true, "child");
        assert!(m.is_merge_enabled());
        assert_eq!(m.value(), Some("child"));
    }

    #[test]
    fn test_merge_with_child_value() {
        let m = SimpleMergeable::with_value(true, "child");
        let parent = "parent".to_string();
        let result = m.merge(&parent).unwrap();
        assert_eq!(result.downcast_ref::<String>().unwrap(), "child");
    }

    #[test]
    fn test_merge_with_parent_value() {
        let m = SimpleMergeable::new(true);
        let parent = "parent".to_string();
        let result = m.merge(&parent).unwrap();
        assert_eq!(result.downcast_ref::<String>().unwrap(), "parent");
    }

    #[test]
    fn test_merge_not_enabled() {
        let m = SimpleMergeable::new(false);
        let parent = "parent".to_string();
        assert!(m.merge(&parent).is_err());
    }
}

//! AutowiredPropertyMarker — 对应 Spring `org.springframework.beans.factory.config.AutowiredPropertyMarker`。
//!
//! 自动装配属性标记。

/// 自动装配属性标记。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.AutowiredPropertyMarker`。
///
/// 用于标记需要自动装配的属性。
#[derive(Debug, Clone)]
pub struct AutowiredPropertyMarker {
    property_name: String,
    required: bool,
}

impl AutowiredPropertyMarker {
    pub fn new(property_name: impl Into<String>, required: bool) -> Self {
        Self {
            property_name: property_name.into(),
            required,
        }
    }

    pub fn property_name(&self) -> &str {
        &self.property_name
    }

    pub fn is_required(&self) -> bool {
        self.required
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker() {
        let marker = AutowiredPropertyMarker::new("userService", true);
        assert_eq!(marker.property_name(), "userService");
        assert!(marker.is_required());
    }
}

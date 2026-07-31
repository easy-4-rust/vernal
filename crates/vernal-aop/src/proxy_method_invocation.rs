//! 代理方法调用。
//!
//! 对应 spring-aop `ProxyMethodInvocation`。
//! 扩展 MethodInvocation 以支持代理相关功能。

use crate::Operation;

/// 代理方法调用接口。
///
/// 对应 spring-aop `ProxyMethodInvocation`。
pub trait ProxyMethodInvocation: Send + Sync + 'static {
    /// 获取代理对象类型名。
    fn get_proxy_type(&self) -> &str;

    /// 获取用户属性。
    fn get_user_attribute(&self, key: &str) -> Option<&str>;

    /// 设置用户属性。
    fn set_user_attribute(&mut self, key: String, value: String);

    /// 是否有用户属性。
    fn has_user_attributes(&self) -> bool;
}

/// 简单的代理方法调用实现。
pub struct SimpleProxyMethodInvocation {
    operation: Operation,
    proxy_type: String,
    user_attributes: std::collections::HashMap<String, String>,
}

impl SimpleProxyMethodInvocation {
    /// 创建新的简单代理方法调用。
    pub fn new(operation: Operation, proxy_type: impl Into<String>) -> Self {
        Self {
            operation,
            proxy_type: proxy_type.into(),
            user_attributes: std::collections::HashMap::new(),
        }
    }
}

impl ProxyMethodInvocation for SimpleProxyMethodInvocation {
    fn get_proxy_type(&self) -> &str {
        &self.proxy_type
    }

    fn get_user_attribute(&self, key: &str) -> Option<&str> {
        self.user_attributes.get(key).map(|s| s.as_str())
    }

    fn set_user_attribute(&mut self, key: String, value: String) {
        self.user_attributes.insert(key, value);
    }

    fn has_user_attributes(&self) -> bool {
        !self.user_attributes.is_empty()
    }
}

impl std::fmt::Debug for SimpleProxyMethodInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleProxyMethodInvocation")
            .field("operation", &self.operation)
            .field("proxy_type", &self.proxy_type)
            .field("user_attribute_count", &self.user_attributes.len())
            .finish()
    }
}

/// 引入感知方法匹配器。
///
/// 对应 spring-aop `IntroductionAwareMethodMatcher`。
pub trait IntroductionAwareMethodMatcher: crate::MethodMatcher {
    /// 是否匹配引入的方法。
    fn matches_introduction(&self, operation: &Operation) -> bool;
}

/// AspectJ 优先级信息。
///
/// 对应 spring-aop `AspectJPrecedenceInformation`。
pub trait AspectJPrecedenceInformation: Send + Sync + 'static {
    /// 获取切面名称。
    fn get_aspect_name(&self) -> &str;

    /// 获取声明优先级。
    fn get_declaration_order(&self) -> i32;

    /// 是否在前置通知之前。
    fn is_before_advice(&self) -> bool;

    /// 是否在后置通知之后。
    fn is_after_advice(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_proxy_method_invocation() {
        let op = Operation::new("Service", "method");
        let invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");

        assert_eq!(invocation.get_proxy_type(), "TestProxy");
        assert!(!invocation.has_user_attributes());
    }

    #[test]
    fn proxy_method_invocation_user_attributes() {
        let op = Operation::new("Service", "method");
        let mut invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");

        assert!(invocation.get_user_attribute("key").is_none());

        invocation.set_user_attribute("key".to_string(), "value".to_string());
        assert!(invocation.has_user_attributes());

        let value = invocation.get_user_attribute("key").unwrap();
        assert_eq!(value, "value");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn simple_proxy_method_invocation_debug() {
        let op = Operation::new("Service", "method");
        let invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");
        let debug = format!("{:?}", invocation);
        assert!(debug.contains("TestProxy"));
    }

    #[test]
    fn simple_proxy_method_invocation_get_proxy_type() {
        let op = Operation::new("Service", "method");
        let invocation = SimpleProxyMethodInvocation::new(op, "MyProxy");
        assert_eq!(invocation.get_proxy_type(), "MyProxy");
    }

    #[test]
    fn simple_proxy_method_invocation_user_attribute_missing() {
        let op = Operation::new("Service", "method");
        let invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");
        assert!(invocation.get_user_attribute("missing").is_none());
    }

    #[test]
    fn simple_proxy_method_invocation_user_attribute_present() {
        let op = Operation::new("Service", "method");
        let mut invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");
        invocation.set_user_attribute("key".to_string(), "value".to_string());
        assert_eq!(invocation.get_user_attribute("key"), Some("value"));
    }

    #[test]
    fn simple_proxy_method_invocation_has_user_attributes_empty() {
        let op = Operation::new("Service", "method");
        let invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");
        assert!(!invocation.has_user_attributes());
    }

    #[test]
    fn simple_proxy_method_invocation_has_user_attributes_non_empty() {
        let op = Operation::new("Service", "method");
        let mut invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");
        invocation.set_user_attribute("key".to_string(), "value".to_string());
        assert!(invocation.has_user_attributes());
    }

    #[test]
    fn simple_proxy_method_invocation_multiple_attributes() {
        let op = Operation::new("Service", "method");
        let mut invocation = SimpleProxyMethodInvocation::new(op, "TestProxy");
        invocation.set_user_attribute("key1".to_string(), "value1".to_string());
        invocation.set_user_attribute("key2".to_string(), "value2".to_string());
        assert_eq!(invocation.get_user_attribute("key1"), Some("value1"));
        assert_eq!(invocation.get_user_attribute("key2"), Some("value2"));
        assert!(invocation.has_user_attributes());
    }
}

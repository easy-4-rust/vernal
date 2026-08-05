//! 反射方法调用。
//!
//! 对应 spring-aop `org.springframework.aop.framework.ReflectiveMethodInvocation`。
//! Spring 的 MethodInvocation 实现。

use std::any::Any;
use std::collections::HashMap;

use crate::Operation;
use crate::intercept::joinpoint::Joinpoint;
use crate::proxy_method_invocation::ProxyMethodInvocation;

/// 反射方法调用。
///
/// 对应 spring-aop `ReflectiveMethodInvocation`。
///
/// 使用反射调用目标对象。子类可以覆盖 `invoke_joinpoint()` 方法
/// 来改变此行为。
///
/// # 特性
///
/// - 可以克隆调用以重复调用 `proceed()`
/// - 可以附加自定义属性
/// - 支持拦截器链
pub struct ReflectiveMethodInvocation {
    /// 操作描述。
    operation: Operation,
    /// 代理类型名。
    proxy_type: String,
    /// 用户属性。
    // 对标 Spring AOP 的 API 脚手架：保留用户属性映射，反射调用暂未被内部使用。
    #[allow(dead_code)]
    user_attributes: HashMap<String, String>,
    /// 当前拦截器索引。
    current_index: usize,
    /// 拦截器数量。
    interceptor_count: usize,
}

impl ReflectiveMethodInvocation {
    /// 创建新的反射方法调用。
    pub fn new(
        operation: Operation,
        proxy_type: impl Into<String>,
        interceptor_count: usize,
    ) -> Self {
        Self {
            operation,
            proxy_type: proxy_type.into(),
            user_attributes: HashMap::new(),
            current_index: 0,
            interceptor_count,
        }
    }

    /// 获取当前拦截器索引。
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// 推进到下一个拦截器。
    pub fn proceed_to_next(&mut self) -> bool {
        if self.current_index < self.interceptor_count {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// 调用连接点（目标方法）。
    pub fn invoke_joinpoint(
        &self,
    ) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
        // 默认实现：返回错误（需要子类覆盖）
        Err("invoke_joinpoint not implemented".into())
    }
}

impl Joinpoint for ReflectiveMethodInvocation {
    fn proceed(&self) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
        self.invoke_joinpoint()
    }

    fn get_static_part(&self) -> &Operation {
        &self.operation
    }
}

impl ProxyMethodInvocation for ReflectiveMethodInvocation {
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

impl Clone for ReflectiveMethodInvocation {
    fn clone(&self) -> Self {
        Self {
            operation: self.operation.clone(),
            proxy_type: self.proxy_type.clone(),
            user_attributes: HashMap::new(), // 克隆时不复制用户属性
            current_index: self.current_index,
            interceptor_count: self.interceptor_count,
        }
    }
}

impl std::fmt::Debug for ReflectiveMethodInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectiveMethodInvocation")
            .field("operation", &self.operation)
            .field("proxy_type", &self.proxy_type)
            .field("current_index", &self.current_index)
            .field("interceptor_count", &self.interceptor_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflective_method_invocation_creation() {
        let op = Operation::new("Service", "method");
        let invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 3);

        assert_eq!(invocation.get_proxy_type(), "TestProxy");
        assert_eq!(invocation.current_index(), 0);
        assert!(!invocation.has_user_attributes());
    }

    #[test]
    fn reflective_method_invocation_proceed() {
        let op = Operation::new("Service", "method");
        let mut invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 3);

        assert!(invocation.proceed_to_next());
        assert_eq!(invocation.current_index(), 1);

        assert!(invocation.proceed_to_next());
        assert_eq!(invocation.current_index(), 2);

        assert!(invocation.proceed_to_next());
        assert_eq!(invocation.current_index(), 3);

        assert!(!invocation.proceed_to_next());
        assert_eq!(invocation.current_index(), 3);
    }

    #[test]
    fn reflective_method_invocation_user_attributes() {
        let op = Operation::new("Service", "method");
        let mut invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 0);

        assert!(invocation.get_user_attribute("key").is_none());

        invocation.set_user_attribute("key".to_string(), "value".to_string());
        assert!(invocation.has_user_attributes());
        assert_eq!(invocation.get_user_attribute("key"), Some("value"));
    }

    #[test]
    fn reflective_method_invocation_clone() {
        let op = Operation::new("Service", "method");
        let mut invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 3);

        invocation.proceed_to_next();
        invocation.set_user_attribute("key".to_string(), "value".to_string());

        let cloned = invocation.clone();
        assert_eq!(cloned.current_index(), 1);
        assert!(!cloned.has_user_attributes()); // 克隆时不复制用户属性
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn reflective_method_invocation_get_method() {
        let op = Operation::new("Service", "method");
        let invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 0);
        assert_eq!(invocation.get_static_part().method(), "method");
    }

    #[test]
    fn reflective_method_invocation_get_static_part() {
        let op = Operation::new("Service", "method");
        let invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 0);
        assert_eq!(invocation.get_static_part().component(), "Service");
    }

    #[test]
    fn reflective_method_invocation_user_attribute() {
        let op = Operation::new("Service", "method");
        let mut invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 0);
        invocation.set_user_attribute("key".to_string(), "value".to_string());
        assert!(invocation.has_user_attributes());
        assert_eq!(invocation.get_user_attribute("key"), Some("value"));
    }

    #[test]
    fn reflective_method_invocation_user_attribute_missing() {
        let op = Operation::new("Service", "method");
        let invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 0);
        assert!(!invocation.has_user_attributes());
        assert!(invocation.get_user_attribute("key").is_none());
    }

    #[test]
    fn reflective_method_invocation_debug() {
        let op = Operation::new("Service", "method");
        let invocation = ReflectiveMethodInvocation::new(op, "TestProxy", 0);
        let debug = format!("{:?}", invocation);
        assert!(!debug.is_empty());
    }
}

//! Spring 代理标记。
//!
//! 对应 spring-aop `org.springframework.aop.SpringProxy`。

/// Spring 代理标记接口。
///
/// 对应 spring-aop `SpringProxy`。
///
/// 标记一个对象为 Spring AOP 代理。用于识别代理对象。
pub trait SpringProxy: Send + Sync + 'static {
    /// 获取代理类型名。
    fn get_proxy_type(&self) -> &str;

    /// 是否为 Spring 代理。
    fn is_spring_proxy(&self) -> bool {
        true
    }
}

/// 默认 Spring 代理实现。
pub struct DefaultSpringProxy {
    proxy_type: String,
}

impl DefaultSpringProxy {
    /// 创建新的 Spring 代理。
    pub fn new(proxy_type: impl Into<String>) -> Self {
        Self {
            proxy_type: proxy_type.into(),
        }
    }
}

impl SpringProxy for DefaultSpringProxy {
    fn get_proxy_type(&self) -> &str {
        &self.proxy_type
    }

    fn is_spring_proxy(&self) -> bool {
        true
    }
}

impl std::fmt::Debug for DefaultSpringProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultSpringProxy")
            .field("proxy_type", &self.proxy_type)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_proxy() {
        let proxy = DefaultSpringProxy::new("TestProxy");
        assert_eq!(proxy.get_proxy_type(), "TestProxy");
        assert!(proxy.is_spring_proxy());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn spring_proxy_get_proxy_type() {
        let proxy = DefaultSpringProxy::new("TestProxy");
        assert_eq!(proxy.get_proxy_type(), "TestProxy");
    }

    #[test]
    fn spring_proxy_is_spring_proxy() {
        let proxy = DefaultSpringProxy::new("TestProxy");
        assert!(proxy.is_spring_proxy());
    }

    #[test]
    fn spring_proxy_debug() {
        let proxy = DefaultSpringProxy::new("TestProxy");
        let debug = format!("{:?}", proxy);
        assert!(debug.contains("TestProxy"));
    }

    #[test]
    fn spring_proxy_clone() {
        let proxy = DefaultSpringProxy::new("TestProxy");
        // DefaultSpringProxy doesn't implement Clone, but we can test creation
        let _ = proxy;
    }
}

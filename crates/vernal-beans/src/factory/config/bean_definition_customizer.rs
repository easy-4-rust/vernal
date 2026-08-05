//! BeanDefinitionCustomizer — 对应 Spring `org.springframework.beans.factory.config.BeanDefinitionCustomizer`。
//!
//! Bean 定义自定义器。

/// Bean 定义自定义器。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.BeanDefinitionCustomizer`。
///
/// 用于自定义 Bean 定义的回调接口。
pub trait BeanDefinitionCustomizer: Send + Sync {
    /// 自定义 Bean 定义。
    fn customize(&self, bean_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// BeanDefinitionCustomizer 的闭包实现。
pub struct ClosureBeanDefinitionCustomizer {
    callback:
        Box<dyn Fn(&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl ClosureBeanDefinitionCustomizer {
    /// 创建一个新的实例。
    pub fn new(
        callback: impl Fn(&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl BeanDefinitionCustomizer for ClosureBeanDefinitionCustomizer {
    fn customize(&self, bean_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self.callback)(bean_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customizer() {
        let customizer = ClosureBeanDefinitionCustomizer::new(|_name| Ok(()));
        assert!(customizer.customize("myBean").is_ok());
    }
}

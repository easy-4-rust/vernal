//! BeanRegistrar — 对应 Spring `org.springframework.beans.factory.BeanRegistrar`。
//!
//! Bean 注册器接口。

/// Bean 注册器接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.BeanRegistrar`。
///
/// 用于在运行时动态注册 Bean。
pub trait BeanRegistrar: Send + Sync {
    /// 注册 Bean。
    fn register_bean(
        &self,
        bean_name: &str,
        bean_class_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// BeanRegistrar 的简单实现。
#[derive(Debug)]
pub struct SimpleBeanRegistrar;

impl BeanRegistrar for SimpleBeanRegistrar {
    fn register_bean(
        &self,
        _bean_name: &str,
        _bean_class_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registrar() {
        let registrar = SimpleBeanRegistrar;
        assert!(
            registrar
                .register_bean("myBean", "com.example.MyBean")
                .is_ok()
        );
    }
}

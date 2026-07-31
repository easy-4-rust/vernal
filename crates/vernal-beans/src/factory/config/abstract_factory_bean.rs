//! AbstractFactoryBean — 对应 Spring `org.springframework.beans.factory.config.AbstractFactoryBean`。
//!
//! 抽象工厂 Bean 基类。

use std::any::Any;
use std::sync::Arc;

/// 抽象工厂 Bean 基类。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.AbstractFactoryBean`。
///
/// FactoryBean 的简单实现基类。
pub trait AbstractFactoryBean: Send + Sync {
    /// 创建 Bean 实例。
    fn create_instance(&self) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 返回此工厂创建的 Bean 类型。
    fn object_type_name(&self) -> &'static str;

    /// 是否是单例。
    fn is_singleton(&self) -> bool {
        true
    }
}

/// AbstractFactoryBean 的简单实现。
pub struct SimpleFactoryBean<T: Any + Send + Sync> {
    creator: Box<dyn Fn() -> T + Send + Sync>,
    type_name: &'static str,
    singleton: bool,
}

impl<T: Any + Send + Sync> SimpleFactoryBean<T> {
    pub fn new(creator: impl Fn() -> T + Send + Sync + 'static, type_name: &'static str) -> Self {
        Self { creator: Box::new(creator), type_name, singleton: true }
    }

    pub fn with_singleton(mut self, singleton: bool) -> Self {
        self.singleton = singleton;
        self
    }
}

impl<T: Any + Send + Sync> AbstractFactoryBean for SimpleFactoryBean<T> {
    fn create_instance(&self) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::new((self.creator)()))
    }

    fn object_type_name(&self) -> &'static str {
        self.type_name
    }

    fn is_singleton(&self) -> bool {
        self.singleton
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_bean() {
        let factory = SimpleFactoryBean::new(|| String::from("hello"), "String");
        assert!(factory.is_singleton());
        assert_eq!(factory.object_type_name(), "String");

        let obj = factory.create_instance().unwrap();
        assert!(obj.downcast_ref::<String>().is_some());
    }
}

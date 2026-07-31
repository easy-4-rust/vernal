//! SmartFactoryBean — 对应 Spring `org.springframework.beans.factory.SmartFactoryBean`。
//!
//! 智能工厂 Bean 接口。

use std::any::Any;
use std::sync::Arc;

/// 智能工厂 Bean 接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.SmartFactoryBean`。
///
/// 扩展了 FactoryBean，提供了更多的元数据信息。
pub trait SmartFactoryBean<T: Any + Send + Sync>: Send + Sync {
    /// 返回此工厂创建的 Bean 是否是单例。
    ///
    /// 对应 Java 方法：`boolean isSingleton()`
    fn is_singleton(&self) -> bool {
        true
    }

    /// 返回此工厂创建的 Bean 是否是原型。
    ///
    /// 对应 Java 方法：`boolean isPrototype()`
    fn is_prototype(&self) -> bool {
        false
    }

    /// 创建 Bean 实例。
    ///
    /// 对应 Java 方法：`T getObject()`
    fn get_object(&self) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>>;

    /// 返回此工厂创建的 Bean 类型。
    ///
    /// 对应 Java 方法：`Class<?> getObjectType()`
    fn object_type_name(&self) -> &'static str;
}

/// SmartFactoryBean 的简单实现。
pub struct SimpleSmartFactoryBean<T: Any + Send + Sync> {
    creator: Box<dyn Fn() -> Result<T, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
    singleton: bool,
    type_name: &'static str,
}

impl<T: Any + Send + Sync> SimpleSmartFactoryBean<T> {
    /// 创建一个新的 SimpleSmartFactoryBean。
    pub fn new(
        creator: impl Fn() -> Result<T, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
        singleton: bool,
        type_name: &'static str,
    ) -> Self {
        Self {
            creator: Box::new(creator),
            singleton,
            type_name,
        }
    }
}

impl<T: Any + Send + Sync> SmartFactoryBean<T> for SimpleSmartFactoryBean<T> {
    fn is_singleton(&self) -> bool {
        self.singleton
    }

    fn is_prototype(&self) -> bool {
        !self.singleton
    }

    fn get_object(&self) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>> {
        (self.creator)().map(Arc::new)
    }

    fn object_type_name(&self) -> &'static str {
        self.type_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_smart_factory_bean() {
        let factory = SimpleSmartFactoryBean::new(
            || Ok(String::from("hello")),
            true,
            "String",
        );

        assert!(factory.is_singleton());
        assert!(!factory.is_prototype());
        assert_eq!(factory.object_type_name(), "String");

        let obj = factory.get_object().unwrap();
        assert_eq!(*obj, "hello");
    }
}

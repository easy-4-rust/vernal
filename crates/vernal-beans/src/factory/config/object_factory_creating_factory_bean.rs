//! ObjectFactoryCreatingFactoryBean — 对应 Spring `org.springframework.beans.factory.config.ObjectFactoryCreatingFactoryBean`。
//!
//! ObjectFactory 创建工厂 Bean。


/// ObjectFactory 创建工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.ObjectFactoryCreatingFactoryBean`。
///
/// 用于创建 `ObjectFactory<T>` 实例。
/// ObjectFactory 是一个简单的工厂接口，每次调用 `getObject()` 返回新的 Bean 实例。
///
/// ## 使用场景
///
/// - 原型 Bean 的延迟获取
/// - 工厂模式实现
/// - 按需创建 Bean
#[derive(Debug)]
pub struct ObjectFactoryCreatingFactoryBean {
    /// 目标 Bean 名称。
    target_bean_name: String,
}

impl ObjectFactoryCreatingFactoryBean {
    /// 创建新的 ObjectFactoryCreatingFactoryBean。
    pub fn new(target_bean_name: impl Into<String>) -> Self {
        Self {
            target_bean_name: target_bean_name.into(),
        }
    }

    /// 获取目标 Bean 名称。
    pub fn target_bean_name(&self) -> &str {
        &self.target_bean_name
    }
}

/// ObjectFactory 实现。
#[derive(Debug, Clone)]
pub struct BeanObjectFactory {
    /// Bean 名称。
    bean_name: String,
}

impl BeanObjectFactory {
    /// 创建新的 BeanObjectFactory。
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_factory_creating_factory_bean_new() {
        let factory = ObjectFactoryCreatingFactoryBean::new("myBean");
        assert_eq!(factory.target_bean_name(), "myBean");
    }

    #[test]
    fn test_bean_object_factory_new() {
        let factory = BeanObjectFactory::new("myBean");
        assert_eq!(factory.bean_name(), "myBean");
    }

    #[test]
    fn test_bean_object_factory_clone() {
        let factory1 = BeanObjectFactory::new("myBean");
        let factory2 = factory1.clone();
        assert_eq!(factory1.bean_name(), factory2.bean_name());
    }
}

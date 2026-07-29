//! FactoryBeanDelegate — Spring 风格的 FactoryBean 对象提取委托。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.FactoryBeanRegistrySupport` 的 getObject 逻辑。
//!
//! 帮助从 `FactoryBean` 实例中提取产品对象。处理 FactoryBean 的类型
//! 查询、对象创建和缓存逻辑。

use std::any::Any;
use std::sync::Arc;

use crate::factory_bean::FactoryBean;

/// Spring 风格的 FactoryBean 对象提取委托。
///
/// 对应 Spring 的 `FactoryBeanRegistrySupport` 中与 `getObject` 相关的逻辑。
///
/// 负责从 `FactoryBean` 实例中提取产品对象。支持：
/// - 从 FactoryBean 获取产品实例
/// - 查询 FactoryBean 的产品类型
/// - 判断 FactoryBean 是否为 singleton
///
/// ## 使用场景
///
/// 当容器检测到某个 Bean 是 FactoryBean 类型时，使用此委托：
/// - 获取实际的 Bean 实例（而非 FactoryBean 本身）
/// - 处理 `&` 前缀（获取 FactoryBean 本身）
///
/// ## 注意
///
/// 由于 `FactoryBean` trait 定义了关联常量（`OBJECT_TYPE_ATTRIBUTE`），
/// 它不是 dyn-compatible 的，因此本委托使用泛型方式操作 FactoryBean 实例。
#[derive(Debug)]
pub struct FactoryBeanDelegate;

impl FactoryBeanDelegate {
    /// 从 FactoryBean 获取产品对象。
    ///
    /// 对应 Spring 的 `FactoryBean.getObject()`。
    ///
    /// # 参数
    ///
    /// * `factory_bean` — FactoryBean 实例的引用
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — FactoryBean 创建的产品实例
    /// - `Err` — 对象创建失败
    pub fn get_object_from_factory_bean(
        factory_bean: &impl FactoryBean,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        factory_bean.get_object()
    }

    /// 查询 FactoryBean 的产品类型。
    ///
    /// 对应 Spring 的 `FactoryBean.getObjectType()`。
    ///
    /// # 参数
    ///
    /// * `factory_bean` — FactoryBean 实例的引用
    ///
    /// # 返回
    ///
    /// 产品类型的 `TypeId`，如果未知则返回 `None`。
    pub fn get_object_type(factory_bean: &impl FactoryBean) -> Option<std::any::TypeId> {
        factory_bean.get_object_type()
    }

    /// 判断 FactoryBean 是否为 singleton 产品。
    ///
    /// 对应 Spring 的 `FactoryBean.isSingleton()`。
    ///
    /// # 参数
    ///
    /// * `factory_bean` — FactoryBean 实例的引用
    ///
    /// # 返回
    ///
    /// `true` 表示工厂每次都返回同一个实例（singleton），
    /// `false` 表示每次调用 `get_object()` 都创建新实例（prototype）。
    pub fn is_singleton(factory_bean: &impl FactoryBean) -> bool {
        factory_bean.is_singleton()
    }

    /// 检查是否应该从 FactoryBean 提取产品对象。
    ///
    /// 对应 Spring 中检查 Bean 名称是否以 `"&"` 开头的逻辑。
    ///
    /// 如果 `bean_name` 以 `&` 开头，应按原样返回 FactoryBean 本身。
    /// 否则，应调用 `get_object()` 返回产品对象。
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称（可能包含 `&` 前缀）
    ///
    /// # 返回
    ///
    /// - `true` — 应提取产品对象（调用 `get_object()`）
    /// - `false` — 应返回 FactoryBean 本身
    pub fn should_extract_object(bean_name: &str) -> bool {
        !bean_name.starts_with('&')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFactory;
    impl FactoryBean for TestFactory {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Arc::new(42i32))
        }

        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<i32>())
        }

        fn is_singleton(&self) -> bool {
            true
        }
    }

    struct PrototypeFactory;
    impl FactoryBean for PrototypeFactory {
        fn get_object(
            &self,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Arc::new(100i32))
        }

        fn get_object_type(&self) -> Option<std::any::TypeId> {
            Some(std::any::TypeId::of::<i32>())
        }

        fn is_singleton(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_get_object_from_factory_bean() {
        let factory = TestFactory;
        let obj = FactoryBeanDelegate::get_object_from_factory_bean(&factory).unwrap();
        let value: &i32 = obj.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_get_object_type() {
        let factory = TestFactory;
        let type_id = FactoryBeanDelegate::get_object_type(&factory);
        assert_eq!(type_id, Some(std::any::TypeId::of::<i32>()));
    }

    #[test]
    fn test_is_singleton() {
        let singleton = TestFactory;
        assert!(FactoryBeanDelegate::is_singleton(&singleton));

        let prototype = PrototypeFactory;
        assert!(!FactoryBeanDelegate::is_singleton(&prototype));
    }

    #[test]
    fn test_should_extract_object() {
        assert!(FactoryBeanDelegate::should_extract_object("myService"));
        assert!(!FactoryBeanDelegate::should_extract_object("&myFactory"));
    }
}

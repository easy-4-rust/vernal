//! BeanDefinitionRegistry — Spring 风格的 Bean 定义注册表接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionRegistry`。
//!
//! 提供注册、查询、删除 Bean 定义的能力。

use crate::bean_definition::BeanDefinition;

/// Spring 风格的 Bean 定义注册表接口。
///
/// 对应 Spring 的 `BeanDefinitionRegistry`。
///
/// 提供注册、查询、删除 Bean 定义的能力。
/// `RegistryBuilder` 是此接口的主要实现（构建期），`Container` 是运行期实现。
///
/// ## 方法
///
/// - `register_bean_definition` — 注册 Bean 定义
/// - `remove_bean_definition` — 删除 Bean 定义
/// - `get_bean_definition` — 获取 Bean 定义
/// - `contains_bean_definition` — 检查是否包含
/// - `bean_definition_count` — 总数
/// - `bean_definition_names` — 所有名称
pub trait BeanDefinitionRegistry: Send + Sync + 'static {
    /// 注册 Bean 定义。
    ///
    /// 对应 Spring 的 `void registerBeanDefinition(String beanName, BeanDefinition beanDefinition)`。
    ///
    /// # 错误
    ///
    /// - Bean 名称已存在且不允许覆盖时返回 `Err`
    /// - Bean 定义无效时返回 `Err`
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 删除 Bean 定义。
    ///
    /// 对应 Spring 的 `BeanDefinition removeBeanDefinition(String beanName)`。
    ///
    /// # 返回
    ///
    /// - `Ok(definition)` — 被删除的 Bean 定义
    /// - `Err` — Bean 名称不存在
    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取 Bean 定义。
    ///
    /// 对应 Spring 的 `BeanDefinition getBeanDefinition(String beanName)`。
    fn get_bean_definition(
        &self,
        bean_name: &str,
    ) -> Option<&dyn BeanDefinition>;

    /// 检查是否包含指定 Bean 定义。
    ///
    /// 对应 Spring 的 `boolean containsBeanDefinition(String beanName)`。
    fn contains_bean_definition(&self, bean_name: &str) -> bool;

    /// Bean 定义总数。
    ///
    /// 对应 Spring 的 `int getBeanDefinitionCount()`。
    fn bean_definition_count(&self) -> usize;

    /// 获取所有 Bean 定义名称。
    ///
    /// 对应 Spring 的 `String[] getBeanDefinitionNames()`。
    fn bean_definition_names(&self) -> Vec<String>;
}

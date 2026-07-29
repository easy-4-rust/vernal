//! BeanDefinitionReader — Spring 风格的 Bean 定义读取器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionReader`。
//!
//! 从某种源（XML、注解、Groovy 等）读取 Bean 定义并注册到注册表。

use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_definition_resource::BeanDefinitionResource;
use crate::bean_name_generator::BeanNameGenerator;

/// Spring 风格的 Bean 定义读取器 trait。
///
/// 对应 Spring 的 `BeanDefinitionReader`。
///
/// 定义从外部源加载 Bean 定义的契约。具体实现负责解析 XML、
/// 注解、Groovy 脚本等格式并注册到 BeanDefinitionRegistry。
pub trait BeanDefinitionReader {
    /// 获取此读取器使用的注册表。
    fn registry(&self) -> &dyn BeanDefinitionRegistry;

    /// 从指定资源加载 Bean 定义。
    fn load_bean_definitions(
        &self,
        resource: &dyn BeanDefinitionResource,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取 Bean 名称生成器。
    fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator>;

    /// 设置 Bean 名称生成器。
    fn set_bean_name_generator(&mut self, generator: Box<dyn BeanNameGenerator>);
}

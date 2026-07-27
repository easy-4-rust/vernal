//! BeanDefinitionReaderUtils — Spring 风格的 Bean 定义工具。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionReaderUtils`。
//!
//! 提供 Bean 定义的通用工具方法。

use crate::bean_definition_registry::BeanDefinitionRegistry;

/// 生成唯一的 Bean 名称。
///
/// 对应 Spring 的 `BeanDefinitionReaderUtils.generateBeanName(BeanDefinition definition, BeanDefinitionRegistry registry)`。
///
/// 如果 Bean 类名不为空，使用类名作为基础名称；
/// 如果冲突则追加数字后缀。
pub fn generate_bean_name(
    class_name: Option<&str>,
    registry: &dyn BeanDefinitionRegistry,
) -> String {
    let base_name = class_name.unwrap_or("anonymous");
    let mut candidate = base_name.to_string();
    let mut counter = 0;
    while registry.contains_bean_definition(&candidate) {
        counter += 1;
        candidate = format!("{}#{}", base_name, counter);
    }
    candidate
}

/// 注册 Bean 定义并返回生成的名称。
///
/// 对应 Spring 的 `BeanDefinitionReaderUtils.registerBeanDefinition(BeanDefinitionHolder definitionHolder, BeanDefinitionRegistry registry)`。
///
/// 自动为未命名的 Bean 生成唯一名称。
pub fn register_bean_definition(
    definition: Box<dyn crate::bean_definition::BeanDefinition>,
    registry: &mut dyn BeanDefinitionRegistry,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let bean_name = definition.bean_name().to_string();
    let name_to_register = if registry.contains_bean_definition(&bean_name) {
        generate_bean_name(Some(definition.bean_class_name()), registry)
    } else {
        bean_name
    };
    registry.register_bean_definition(name_to_register.clone(), definition)?;
    Ok(name_to_register)
}

//! BeanDefinitionHolder — Spring 风格的 Bean 定义持有者。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanDefinitionHolder`。
//!
//! 包装 BeanDefinition + bean name + 别名列表。
//! 在 Bean 定义注册时携带名称和别名信息。

use crate::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use std::sync::Arc;

/// Spring 风格的 Bean 定义持有者。
///
/// 对应 Spring 的 `BeanDefinitionHolder`。
///
/// 包装 BeanDefinition + bean name + 别名列表。
#[derive(Debug)]
pub struct BeanDefinitionHolder {
    /// Bean 定义。
    bean_definition: Arc<dyn BeanDefinition>,
    /// Bean 名称。
    bean_name: String,
    /// 别名列表。
    aliases: Vec<String>,
}

impl BeanDefinitionHolder {
    /// 创建新的 BeanDefinitionHolder。
    pub fn new(bean_definition: Arc<dyn BeanDefinition>, bean_name: impl Into<String>) -> Self {
        Self {
            bean_definition,
            bean_name: bean_name.into(),
            aliases: Vec::new(),
        }
    }

    /// 创建带别名的 BeanDefinitionHolder。
    pub fn with_aliases(
        bean_definition: Arc<dyn BeanDefinition>,
        bean_name: impl Into<String>,
        aliases: Vec<String>,
    ) -> Self {
        Self {
            bean_definition,
            bean_name: bean_name.into(),
            aliases,
        }
    }

    /// 获取 Bean 定义。
    pub fn bean_definition(&self) -> &dyn BeanDefinition {
        &*self.bean_definition
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取别名列表。
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// 添加别名。
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }

    /// 获取短名称（去掉包路径）。
    pub fn short_name(&self) -> &str {
        self.bean_name
            .rsplit("::")
            .next()
            .unwrap_or(&self.bean_name)
    }
}

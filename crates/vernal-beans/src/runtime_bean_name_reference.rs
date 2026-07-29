//! RuntimeBeanNameReference — Spring 风格的运行时 Bean 名称引用。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.RuntimeBeanNameReference`。
//!
//! 表示对另一个 Bean 的名称的引用（容器解析后返回名称字符串本身，而非实例）。

use std::any::Any;

use crate::bean_reference::BeanReference;

/// Spring 风格的运行时 Bean 名称引用。
///
/// 对应 Spring 的 `RuntimeBeanNameReference`。
///
/// 与 `RuntimeBeanReference` 不同，此引用只传递 Bean 的名称字符串，
/// 而非解析后的 Bean 实例。适用于需要 Bean 名称而非实例的场景
/// （如 FactoryBean 的模式）。
#[derive(Debug)]
pub struct RuntimeBeanNameReference {
    /// 目标 Bean 的名称。
    bean_name: String,
    /// 引用的来源（可选）。
    source: Option<Box<dyn Any + Send + Sync>>,
}

impl RuntimeBeanNameReference {
    /// 创建一个新的 RuntimeBeanNameReference。
    ///
    /// # 参数
    ///
    /// * `bean_name` — 目标 Bean 的名称
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            source: None,
        }
    }

    /// 创建一个带有来源信息的 RuntimeBeanNameReference。
    ///
    /// # 参数
    ///
    /// * `bean_name` — 目标 Bean 的名称
    /// * `source` — 引用的来源对象
    pub fn with_source(bean_name: impl Into<String>, source: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            bean_name: bean_name.into(),
            source: Some(source),
        }
    }

    /// 获取目标 Bean 的名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取引用的来源。
    pub fn source(&self) -> Option<&dyn Any> {
        self.source.as_ref().map(|b| &**b as &dyn Any)
    }

    /// 设置引用的来源。
    pub fn set_source(&mut self, source: Box<dyn Any + Send + Sync>) {
        self.source = Some(source);
    }
}

impl BeanReference for RuntimeBeanNameReference {
    fn get_bean_name(&self) -> &str {
        &self.bean_name
    }

    fn get_source(&self) -> Option<&dyn Any> {
        self.source.as_ref().map(|b| &**b as &dyn Any)
    }
}

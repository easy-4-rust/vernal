//! RuntimeBeanReference — Spring 风格的运行时 Bean 引用。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.RuntimeBeanReference`。
//!
//! 表示在运行时解析的对另一个 Bean 的引用。

use std::any::Any;

use crate::bean_reference::BeanReference;

/// Spring 风格的运行时 Bean 引用。
///
/// 对应 Spring 的 `RuntimeBeanReference`。
///
/// 用于在 Bean 定义中声明一个属性值或构造参数值是对另一个 Bean 的引用。
/// 容器在实例化该 Bean 时，会通过 `BeanDefinitionValueResolver` 解析此引用，
/// 获取对应的 Bean 实例。
///
/// 与 `RuntimeBeanNameReference` 的区别：
/// - `RuntimeBeanReference` — 直接引用 Bean，容器会解析为 Bean 实例
/// - `RuntimeBeanNameReference` — 只引用 Bean 名称，容器返回 Bean 名称字符串
#[derive(Debug)]
pub struct RuntimeBeanReference {
    /// 目标 Bean 的名称。
    bean_name: String,
    /// 引用的来源（可选）。
    source: Option<Box<dyn Any + Send + Sync>>,
}

impl RuntimeBeanReference {
    /// 创建一个新的 RuntimeBeanReference。
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

    /// 创建一个带有来源信息的 RuntimeBeanReference。
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
    pub fn get_bean_name(&self) -> &str {
        &self.bean_name
    }
}

impl PartialEq for RuntimeBeanReference {
    fn eq(&self, other: &Self) -> bool {
        self.bean_name == other.bean_name
    }
}

impl BeanReference for RuntimeBeanReference {
    fn get_bean_name(&self) -> &str {
        &self.bean_name
    }

    fn get_source(&self) -> Option<&dyn Any> {
        self.source.as_ref().map(|b| &**b as &dyn Any)
    }
}

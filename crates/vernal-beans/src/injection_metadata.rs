//! InjectionMetadata — Spring 风格的注入元数据。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.InjectionMetadata`。
//!
//! 存储一个 Bean 的所有注入点（字段、方法），并在 Bean 初始化时统一执行注入。

use std::any::Any;
use std::sync::Arc;

use crate::injected_element::InjectedElement;
use crate::injection_point::InjectionPoint;

/// 注入元数据。
///
/// 对应 Spring 的 `InjectionMetadata`。
///
/// 一个 `InjectionMetadata` 实例对应一个目标 Bean 类，包含该类的全部注入元素
/// （字段注入、方法注入）。在 Bean 实例化后，容器调用 `inject()` 完成依赖装配。
#[derive(Debug, Clone)]
pub struct InjectionMetadata {
    /// 目标 Bean 类名。
    target_class: String,
    /// 所有注入点。
    injection_points: Vec<InjectionPoint>,
    /// 所有注入元素（字段/方法）。
    injection_elements: Vec<InjectedElement>,
}

impl InjectionMetadata {
    /// 创建空的注入元数据。
    pub fn new(target_class: impl Into<String>) -> Self {
        Self {
            target_class: target_class.into(),
            injection_points: Vec::new(),
            injection_elements: Vec::new(),
        }
    }

    /// 获取目标类名。
    pub fn target_class(&self) -> &str {
        &self.target_class
    }

    /// 添加一个注入点。
    pub fn add_injection_point(&mut self, point: InjectionPoint) {
        self.injection_points.push(point);
    }

    /// 添加一个注入元素。
    pub fn add_injected_element(&mut self, element: InjectedElement) {
        self.injection_elements.push(element);
    }

    /// 获取所有注入点。
    pub fn injection_points(&self) -> &[InjectionPoint] {
        &self.injection_points
    }

    /// 获取所有注入元素。
    pub fn injected_elements(&self) -> &[InjectedElement] {
        &self.injection_elements
    }

    /// 注入点数量。
    pub fn len(&self) -> usize {
        self.injection_points.len() + self.injection_elements.len()
    }

    /// 是否为空（无任何注入点）。
    pub fn is_empty(&self) -> bool {
        self.injection_points.is_empty() && self.injection_elements.is_empty()
    }

    /// 执行注入。
    ///
    /// 对应 Spring 的 `InjectionMetadata.inject(Object target, String beanName, PropertyValues pvs)`。
    ///
    /// 遍历所有注入元素，调用其 `inject()` 方法。任一元素注入失败则中止并返回错误。
    ///
    /// # 参数
    ///
    /// - `target` — 目标 Bean 实例
    /// - `resolver` — 依赖解析闭包，根据注入点返回已解析的依赖值
    pub fn inject(
        &mut self,
        target: &mut dyn Any,
        value: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for element in &mut self.injection_elements {
            element.inject(target, value.clone())?;
        }
        Ok(())
    }

    /// 清除所有注入元素的已注入标记。
    ///
    /// 对应 Spring 的 `InjectionMetadata.checkConfigMembers(RootBeanDefinition beanDefinition)`
    /// 之后，原型 Bean 多次创建时需要重置注入状态。
    pub fn clear(&mut self) {
        for element in &mut self.injection_elements {
            element.clear_injected();
        }
    }
}

impl Default for InjectionMetadata {
    fn default() -> Self {
        Self::new(String::new())
    }
}

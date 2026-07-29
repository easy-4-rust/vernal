//! AutowiredMethodElement — Spring 风格的方法注入元素。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AutowiredAnnotationBeanPostProcessor.AutowiredMethodElement`。
//!
//! 描述一个被 `@Autowired` 标注的方法（通常是 setter）注入点。

use std::any::{Any, TypeId};
use std::sync::Arc;

use crate::injected_element::InjectedElement;

/// 方法注入元素。
///
/// 对应 Spring 的 `AutowiredMethodElement`。
///
/// 封装一个 `@Autowired` 方法的元数据：方法名、参数类型列表、是否必须。
/// 容器在装配时按参数类型依次解析依赖，再调用 `invoke()` 执行方法。
#[derive(Debug, Clone)]
pub struct AutowiredMethodElement {
    /// 基础注入元素。
    base: InjectedElement,
    /// 方法名。
    method_name: String,
    /// 参数类型 id 列表。
    parameter_type_ids: Vec<TypeId>,
    /// 是否必须。
    required: bool,
}

impl AutowiredMethodElement {
    /// 创建方法注入元素。
    pub fn new(
        method_name: impl Into<String>,
        parameter_type_ids: Vec<TypeId>,
        required: bool,
    ) -> Self {
        let method_name = method_name.into();
        Self {
            base: InjectedElement::new(method_name.clone()).with_required(required),
            method_name,
            parameter_type_ids,
            required,
        }
    }

    /// 获取方法名。
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// 获取参数类型 id 列表。
    pub fn parameter_type_ids(&self) -> &[TypeId] {
        &self.parameter_type_ids
    }

    /// 参数数量。
    pub fn parameter_count(&self) -> usize {
        self.parameter_type_ids.len()
    }

    /// 是否必须。
    pub fn required(&self) -> bool {
        self.required
    }

    /// 获取基类引用。
    pub fn base(&self) -> &InjectedElement {
        &self.base
    }

    /// 获取基类可变引用。
    pub fn base_mut(&mut self) -> &mut InjectedElement {
        &mut self.base
    }

    /// 调用注入方法。
    ///
    /// 对应 Spring 的 `AutowiredMethodElement.inject(...)`：逐参数解析依赖，
    /// 收集为参数列表后调用目标方法。
    ///
    /// `resolver` 闭包接收参数类型 id，返回已解析的 Bean 实例。
    /// 任一必须参数无法解析时返回错误。
    pub fn invoke(
        &self,
        resolver: impl FnMut(TypeId) -> Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Vec<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut resolver = resolver;
        let mut args = Vec::with_capacity(self.parameter_type_ids.len());
        for type_id in &self.parameter_type_ids {
            match resolver(*type_id) {
                Some(value) => args.push(value),
                None => {
                    if self.required {
                        return Err(format!(
                            "No qualifying bean for parameter of method '{}'",
                            self.method_name
                        )
                        .into());
                    }
                }
            }
        }
        Ok(args)
    }
}

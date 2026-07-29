//! AutowiredFieldElement — Spring 风格的字段注入元素。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AutowiredAnnotationBeanPostProcessor.AutowiredFieldElement`。
//!
//! 描述一个被 `@Autowired` 标注的字段注入点。

use std::any::{Any, TypeId};
use std::sync::Arc;

use crate::injected_element::InjectedElement;

/// 字段注入元素。
///
/// 对应 Spring 的 `AutowiredFieldElement`。
///
/// 封装一个 `@Autowired` 字段的元数据：字段名、目标类型、是否必须。
/// 容器在装配时调用 `resolve()` 解析依赖，再调用 `inject()`（基类）写入字段。
#[derive(Debug, Clone)]
pub struct AutowiredFieldElement {
    /// 基础注入元素（复用 member_name / is_required）。
    base: InjectedElement,
    /// 字段名。
    field_name: String,
    /// 字段类型 id。
    field_type_id: TypeId,
    /// 是否必须（对应 `@Autowired(required = true)`）。
    required: bool,
}

impl AutowiredFieldElement {
    /// 创建字段注入元素。
    pub fn new<T: 'static>(field_name: impl Into<String>, required: bool) -> Self {
        let field_name = field_name.into();
        Self {
            base: InjectedElement::new(field_name.clone()).with_required(required),
            field_name,
            field_type_id: TypeId::of::<T>(),
            required,
        }
    }

    /// 以显式 `TypeId` 创建字段注入元素（用于动态构造场景）。
    pub fn with_type_id(
        field_name: impl Into<String>,
        field_type_id: TypeId,
        required: bool,
    ) -> Self {
        let field_name = field_name.into();
        Self {
            base: InjectedElement::new(field_name.clone()).with_required(required),
            field_name,
            field_type_id,
            required,
        }
    }

    /// 获取字段名。
    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    /// 获取字段类型 id。
    pub fn field_type_id(&self) -> TypeId {
        self.field_type_id
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

    /// 解析字段依赖。
    ///
    /// 对应 Spring 的 `DefaultListableBeanFactory.resolveDependency(...)`。
    ///
    /// `resolver` 闭包接收字段类型 id，返回已解析的 Bean 实例（类型擦除为 `Any`）。
    /// 当 `required = true` 且解析结果为 `None` 时返回错误；`required = false` 时返回 `Ok(None)`。
    pub fn resolve(
        &self,
        resolver: impl FnOnce(TypeId) -> Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        match resolver(self.field_type_id) {
            Some(value) => Ok(Some(value)),
            None => {
                if self.required {
                    Err(format!(
                        "No qualifying bean of type for required field '{}'",
                        self.field_name
                    )
                    .into())
                } else {
                    Ok(None)
                }
            }
        }
    }
}

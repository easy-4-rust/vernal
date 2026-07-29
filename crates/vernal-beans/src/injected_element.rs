//! InjectedElement — Spring 风格的注入元素基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.InjectionMetadata.InjectedElement`。
//!
//! 所有注入元素（字段注入、方法注入等）的基类，提供成员名称、是否必须等通用元数据。

use std::any::Any;
use std::sync::Arc;

/// 注入元素基类。
///
/// 对应 Spring 的 `InjectionMetadata.InjectedElement`。
///
/// 描述一个注入点（字段或方法）的基础信息。子类（如 `AutowiredFieldElement`、
/// `AutowiredMethodElement`）通过 `inject()` 方法提供具体注入逻辑。
#[derive(Debug, Clone)]
pub struct InjectedElement {
    /// 成员名称（字段名或方法名）。
    member_name: String,
    /// 是否必须（注入失败时是否抛错）。
    is_required: bool,
    /// 是否已标记为已注入（避免重复注入）。
    injected: bool,
}

impl InjectedElement {
    /// 创建新的注入元素。
    ///
    /// 默认 `is_required = false`，`injected = false`。
    pub fn new(member_name: impl Into<String>) -> Self {
        Self {
            member_name: member_name.into(),
            is_required: false,
            injected: false,
        }
    }

    /// 创建带必须标记的注入元素。
    pub fn with_required(mut self, required: bool) -> Self {
        self.is_required = required;
        self
    }

    /// 获取成员名称。
    pub fn member_name(&self) -> &str {
        &self.member_name
    }

    /// 是否必须。
    pub fn is_required(&self) -> bool {
        self.is_required
    }

    /// 设置是否必须。
    pub fn set_required(&mut self, required: bool) {
        self.is_required = required;
    }

    /// 是否已注入。
    pub fn is_injected(&self) -> bool {
        self.injected
    }

    /// 标记为已注入。
    pub fn mark_injected(&mut self) {
        self.injected = true;
    }

    /// 执行注入。
    ///
    /// 对应 Spring 的 `InjectedElement.inject(Object target, String requestingBeanName, PropertyValues pvs)`。
    ///
    /// 基类实现仅做标记和校验；真正的注入逻辑由子类覆盖。
    ///
    /// # 参数
    ///
    /// - `target` — 目标 Bean 实例（类型擦除为 `Any`）
    /// - `value` — 待注入的值（已解析的依赖）
    ///
    /// # 返回
    ///
    /// 注入成功返回 `Ok(())`，失败返回错误。重复注入时直接返回 `Ok(())`。
    pub fn inject(
        &mut self,
        _target: &mut dyn Any,
        _value: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.injected {
            return Ok(());
        }
        self.injected = true;
        Ok(())
    }

    /// 清除已注入标记（用于原型 Bean 的多次实例化）。
    pub fn clear_injected(&mut self) {
        self.injected = false;
    }
}

impl Default for InjectedElement {
    fn default() -> Self {
        Self::new(String::new())
    }
}

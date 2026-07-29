//! LookupOverride — Spring 风格的 lookup 方法注入覆盖。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.LookupOverride`。
//!
//! 描述一个被 `@Lookup` 标注的方法：每次调用该方法时，容器返回指定 Bean 的新实例。

use crate::method_override::{MethodOverride, MethodOverrideBase};

/// Lookup 方法注入覆盖。
///
/// 对应 Spring 的 `LookupOverride`（继承自 `MethodOverride`）。
///
/// 用于方法注入：被声明的方法不执行其原始逻辑，而是每次调用都从容器中
/// 获取指定名称（或类型）的 Bean 并返回。常用于在 singleton 中获取 prototype Bean。
#[derive(Debug, Clone)]
pub struct LookupOverride {
    /// 基础字段。
    base: MethodOverrideBase,
    /// 要返回的 Bean 名称。
    bean_name: String,
    /// 可选的目标类型名（当按类型匹配时使用）。
    type_name: Option<String>,
}

impl LookupOverride {
    /// 创建 lookup 覆盖。
    ///
    /// 对应 Spring 的 `LookupOverride(String method, String beanName)`。
    pub fn new(method_name: impl Into<String>, bean_name: impl Into<String>) -> Self {
        Self {
            base: MethodOverrideBase::new(method_name),
            bean_name: bean_name.into(),
            type_name: None,
        }
    }

    /// 设置可选的目标类型名。
    pub fn with_type_name(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    /// 获取方法名。
    pub fn get_method_name(&self) -> &str {
        self.base.method_name()
    }

    /// 获取 Bean 名称。
    ///
    /// 对应 Spring 的 `getBeanName()`。
    pub fn get_bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取目标类型名。
    pub fn get_type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// 获取来源描述。
    pub fn source_name(&self) -> Option<&str> {
        self.base.source_name()
    }
}

impl MethodOverride for LookupOverride {
    fn get_method_name(&self) -> &str {
        self.base.method_name()
    }

    fn is_overloaded(&self) -> bool {
        self.base.overloaded()
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.base.method_name().is_empty() {
            return Err("LookupOverride method name must not be empty".into());
        }
        if self.bean_name.is_empty() && self.type_name.is_none() {
            return Err("LookupOverride must specify either a bean name or a target type".into());
        }
        Ok(())
    }
}

//! ReplaceOverride — Spring 风格的方法替换覆盖。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ReplaceOverride`。
//!
//! 描述一个被 `@Replace`/`replaced-method` 声明的方法：调用时改由指定的
//! `MethodReplacer` 实现重新实现其逻辑。

use crate::method_override::{MethodOverride, MethodOverrideBase};

/// 方法替换覆盖。
///
/// 对应 Spring 的 `ReplaceOverride`（继承自 `MethodOverride`）。
///
/// 被覆盖的方法调用时不会执行原始逻辑，而是委托给一个实现了
/// `MethodReplacer` 的 Bean（按名称引用）执行重新实现。
#[derive(Debug, Clone)]
pub struct ReplaceOverride {
    /// 基础字段。
    base: MethodOverrideBase,
    /// 实现替换逻辑的 Bean 名称（该 Bean 必须实现 `MethodReplacer`）。
    method_replacer: String,
    /// 可选的类型签名描述（用于区分重载）。
    type_identifiers: Vec<String>,
}

impl ReplaceOverride {
    /// 创建方法替换覆盖。
    ///
    /// 对应 Spring 的 `ReplaceOverride(String method, String methodReplacer)`。
    pub fn new(method_name: impl Into<String>, method_replacer: impl Into<String>) -> Self {
        Self {
            base: MethodOverrideBase::new(method_name),
            method_replacer: method_replacer.into(),
            type_identifiers: Vec::new(),
        }
    }

    /// 添加一个类型标识符（用于匹配重载方法）。
    ///
    /// 对应 Spring 的 `addTypeIdentifier(String identifier)`。
    pub fn add_type_identifier(&mut self, identifier: impl Into<String>) {
        self.type_identifiers.push(identifier.into());
    }

    /// 设置重载标记。
    pub fn with_overloaded(mut self, overloaded: bool) -> Self {
        self.base = self.base.with_overloaded(overloaded);
        self
    }

    /// 获取方法名。
    pub fn get_method_name(&self) -> &str {
        self.base.method_name()
    }

    /// 获取方法替换器 Bean 名称。
    ///
    /// 对应 Spring 的 `getMethodReplacer()`。
    pub fn get_method_replacer(&self) -> &str {
        &self.method_replacer
    }

    /// 获取类型标识符列表。
    pub fn type_identifiers(&self) -> &[String] {
        &self.type_identifiers
    }

    /// 获取来源描述。
    pub fn source_name(&self) -> Option<&str> {
        self.base.source_name()
    }
}

impl MethodOverride for ReplaceOverride {
    fn get_method_name(&self) -> &str {
        self.base.method_name()
    }

    fn is_overloaded(&self) -> bool {
        self.base.overloaded()
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.base.method_name().is_empty() {
            return Err("ReplaceOverride method name must not be empty".into());
        }
        if self.method_replacer.is_empty() {
            return Err("ReplaceOverride method replacer name must not be empty".into());
        }
        Ok(())
    }
}

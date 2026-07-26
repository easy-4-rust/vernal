//! 标准类型定位器。
//!
//! 对标 Spring 的 `StandardTypeLocator`：通过 ClassLoader 按名称查找类型。

use std::any::TypeId;
use crate::evaluation_exception::EvaluationException;
use crate::type_locator::TypeLocator;

/// 标准类型定位器。
///
/// 对标 Spring 的 `org.springframework.expression.spel.support.StandardTypeLocator`。
pub struct StandardTypeLocator {
    imports: Vec<String>,
}

impl StandardTypeLocator {
    /// 创建标准类型定位器。
    #[must_use]
    pub fn new() -> Self {
        Self { imports: vec!["java.lang".to_string()] }
    }

    /// 注册包导入前缀。
    pub fn register_import(&mut self, prefix: impl Into<String>) {
        self.imports.push(prefix.into());
    }
}

impl Default for StandardTypeLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeLocator for StandardTypeLocator {
    fn find_type(&self, type_name: &str) -> Result<TypeId, EvaluationException> {
        // 简化实现：返回未知类型
        // 完整实现需要通过 std::any 模块或 class 路径查找
        let _ = type_name;
        let _ = &self.imports;
        Err(EvaluationException::new(
            type_name.to_string(),
            None,
            "Rust 中无法通过类名字符串查找类型",
        ))
    }
}

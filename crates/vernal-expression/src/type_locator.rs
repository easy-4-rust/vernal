//! 类型定位器 trait。
//!
//! 对标 Spring 的 `TypeLocator`。

use super::evaluation_exception::EvaluationException;

/// 类型定位器 trait。
///
/// 按名称定位类型（短名或全限定名）。
/// 对标 Spring 的 `org.springframework.expression.TypeLocator`。
pub trait TypeLocator: Send + Sync {
    /// 查找类型。
    fn find_type(&self, type_name: &str) -> Result<std::any::TypeId, EvaluationException>;
}

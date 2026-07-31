//! 字面量基类。
//!
//! 对标 Spring 的 `Literal` 抽象类。

use super::spel_node::SpelNode;
use crate::typed_value::TypedValue;

/// 字面量节点 trait。
///
/// 所有字面量节点的公共接口。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.Literal`。
pub trait LiteralNode: SpelNode {
    /// 获取字面量值。
    fn literal_value(&self) -> &TypedValue;
}

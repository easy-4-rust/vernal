//! 值引用接口。
//!
//! 对标 Spring 的 `ValueRef`。

use crate::typed_value::TypedValue;

/// 值引用 trait。
///
/// 引用一个值，支持 get/set。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.ValueRef`。
pub trait ValueRef: Send + Sync {
    /// 获取值。
    fn get_value(&self) -> TypedValue;

    /// 设置值。
    fn set_value(&self, value: TypedValue);

    /// 是否可写。
    fn is_writable(&self) -> bool;
}

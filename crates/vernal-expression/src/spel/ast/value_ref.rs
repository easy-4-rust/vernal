//! 值引用（对标 Spring `ValueRef`）。
//!
//! 在 SpEL 中，`a = b`、`a++`、`a.b.c = d` 等写入路径需要通过
//! `ValueRef` 代理：先求值确定目标，再 set，避免在赋值表达式里
//! 重新走属性/索引解析链。

use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 值引用 trait（对标 Spring `org.springframework.expression.spel.ast.ValueRef`）。
pub trait ValueRef: Send + Sync {
    /// 获取当前值。
    #[must_use]
    fn get_value(&self) -> TypedValue;

    /// 设置新值。
    ///
    /// 返回错误封装语义，对标 Spring `ValueRef.setValue(Object)` 抛 `EvaluationException`。
    fn set_value(&mut self, value: TypedValue) -> Result<(), EvaluationException>;

    /// 当前引用是否可写（可赋值）。
    #[must_use]
    fn is_writable(&self) -> bool;
}

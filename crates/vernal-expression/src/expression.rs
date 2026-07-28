//! 表达式 trait（对标 Spring `Expression` 接口）。
//!
//! 封装已解析的表达式字符串，提供求值 API。
//! 对应 Java 接口：`org.springframework.expression.Expression`。
//!
//! # 实现方式
//!
//! - `SpelExpression`：SpEL 表达式（最常用）
//! - `LiteralExpression`：字面量表达式
//! - `CompositeStringExpression`：模板组合表达式（如 `Hello #{name}!`）

use super::evaluation_context::EvaluationContext;
use super::evaluation_exception::EvaluationException;
use super::typed_value::{TypeDescriptor, TypedValue};

/// 表达式 trait（对标 Spring `Expression` 接口）。
///
/// 封装已解析的表达式字符串，提供求值、赋值和类型查询功能。
/// 表达式对象可被重复求值，通常在一次解析后多次调用。
///
/// # 与 Spring 的关系
///
/// Spring `Expression` 接口有 26 个方法（含大量 Java 重载）。
/// Rust 通过 trait 方法签名差异将语义等价的方法合并为 6 个核心方法。
/// Java 中的 `getValue(Object rootObject)` 重载在 Rust 中通过 `get_value_with_root` 实现。
///
/// # 线程安全性
///
/// 实现者必须满足 `Send + Sync`，以支持多线程并发求值（与 Spring `SpelExpression` 线程安全语义一致）。
pub trait Expression: Send + Sync {
    /// 获取原始表达式字符串（未修改）。
    ///
    /// 对标 Java `Expression.getExpressionString()`。
    fn expression_string(&self) -> &str;

    /// 在默认上下文中求值并返回结果。
    ///
    /// 对标 Java `Expression.getValue()`。
    /// 如果没有默认上下文，实现者应返回 `EvaluationException`。
    fn get_value(&self) -> Result<TypedValue, EvaluationException>;

    /// 在指定上下文中求值并返回结果。
    ///
    /// 对标 Java `Expression.getValue(EvaluationContext context)`。
    ///
    /// # 参数
    ///
    /// - `context` — 表达式求值上下文，提供根对象、变量、访问器等
    fn get_value_with_context(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException>;

    /// 在指定上下文中针对指定根对象求值。
    ///
    /// 对标 Java `Expression.getValue(EvaluationContext context, Object rootObject)`。
    ///
    /// # 参数
    ///
    /// - `context` — 表达式求值上下文
    /// - `root` — 用于覆盖上下文默认根对象的根对象
    ///
    /// # 说明
    ///
    /// 传入的 `root` 将覆盖上下文中配置的默认根对象。
    fn get_value_with_root(
        &self,
        context: &dyn EvaluationContext,
        root: &TypedValue,
    ) -> Result<TypedValue, EvaluationException>;

    /// 获取求值结果的类型描述符。
    ///
    /// 对标 Java `Expression.getValueTypeDescriptor()`。
    /// 默认实现通过求值获取实际结果类型，实现者可覆盖以提供更高效的方式。
    fn get_value_type(&self) -> Result<TypeDescriptor, EvaluationException> {
        let value = self.get_value()?;
        Ok(value.type_descriptor().clone())
    }

    /// 表达式是否可写（是否可调用 `set_value`）。
    ///
    /// 对标 Java `Expression.isWritable(EvaluationContext)`。
    /// 默认返回 `false`，表示此表达式不支持赋值。
    fn is_writable(&self) -> bool {
        false
    }

    /// 在指定上下文中设置表达式的值。
    ///
    /// 对标 Java `Expression.setValue(EvaluationContext, Object)`。
    /// 默认实现返回错误，表示此表达式不支持赋值。
    ///
    /// # 参数
    ///
    /// - `context` — 表达式求值上下文
    /// - `root` — 目标根对象
    /// - `value` — 要设置的新值
    fn set_value(
        &self,
        _context: &dyn EvaluationContext,
        _root: &TypedValue,
        _value: &TypedValue,
    ) -> Result<(), EvaluationException> {
        Err(EvaluationException::new(
            self.expression_string(),
            None,
            "此表达式不支持赋值",
        ))
    }
}

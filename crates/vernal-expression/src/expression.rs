//! 表达式 trait。
//!
//! 对标 Spring 的 `Expression` 接口：封装已解析的表达式，提供求值、赋值和类型查询。

use super::evaluation_context::EvaluationContext;
use super::evaluation_exception::EvaluationException;
use super::typed_value::{TypeDescriptor, TypedValue};

/// 表达式 trait。
///
/// 封装已解析的表达式字符串，提供求值 API。
/// 对标 Spring 的 `org.springframework.expression.Expression`。
///
/// # 实现方式
///
/// - `SpelExpression`：SpEL 表达式
/// - `LiteralExpression`：字面量表达式
/// - `CompositeStringExpression`：模板组合表达式
pub trait Expression: Send + Sync {
    /// 获取原始表达式字符串。
    fn expression_string(&self) -> &str;

    /// 在默认上下文中求值。
    fn get_value(&self) -> Result<TypedValue, EvaluationException>;

    /// 在指定上下文中求值。
    fn get_value_with_context(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException>;

    /// 在指定上下文和根对象下求值。
    fn get_value_with_root(
        &self,
        context: &dyn EvaluationContext,
        root: &TypedValue,
    ) -> Result<TypedValue, EvaluationException>;

    /// 获取求值结果的类型描述符。
    fn get_value_type(&self) -> Result<TypeDescriptor, EvaluationException> {
        let value = self.get_value()?;
        Ok(value.type_descriptor().clone())
    }

    /// 表达式是否可写（可赋值）。
    fn is_writable(&self) -> bool {
        false
    }

    /// 设置值（默认不支持）。
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

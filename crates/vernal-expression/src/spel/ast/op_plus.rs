//! 加法运算符。
//!
//! 对标 Spring 的 `OpPlus`：`+`（加法、字符串连接、一元正号）

use super::operator::BinaryOperator;
use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 加法运算符节点。
///
/// 支持：
/// - 数值加法：`1 + 2` → `3`
/// - 字符串连接：`'hello' + ' world'` → `'hello world'`
/// - 一元正号：`+42` → `42`

pub struct OpPlus {
    left: Box<dyn SpelNode>,
    right: Box<dyn SpelNode>,
}

impl OpPlus {
    /// 创建加法运算符节点。
    #[must_use]
    pub fn new(left: Box<dyn SpelNode>, right: Box<dyn SpelNode>) -> Self {
        Self { left, right }
    }
}

impl SpelNode for OpPlus {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let left = self.left.get_value(context)?;
        let right = self.right.get_value(context)?;
        self.operate(&left, &right)
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} + {})",
            self.left.to_string_ast(),
            self.right.to_string_ast()
        )
    }
}

impl BinaryOperator for OpPlus {
    fn left(&self) -> &dyn SpelNode {
        &*self.left
    }
    fn right(&self) -> &dyn SpelNode {
        &*self.right
    }
    fn operator_name(&self) -> &str {
        "+"
    }

    fn operate(
        &self,
        left: &TypedValue,
        right: &TypedValue,
    ) -> Result<TypedValue, EvaluationException> {
        match (left.value(), right.value()) {
            (ExpressionValue::Int(l), ExpressionValue::Int(r)) => Ok(TypedValue::new(
                ExpressionValue::Int(l + r),
                TypeDescriptor::INT,
            )),
            (ExpressionValue::Float(l), ExpressionValue::Float(r)) => Ok(TypedValue::new(
                ExpressionValue::Float(l + r),
                TypeDescriptor::FLOAT,
            )),
            (ExpressionValue::Int(l), ExpressionValue::Float(r)) => Ok(TypedValue::new(
                ExpressionValue::Float(*l as f64 + r),
                TypeDescriptor::FLOAT,
            )),
            (ExpressionValue::Float(l), ExpressionValue::Int(r)) => Ok(TypedValue::new(
                ExpressionValue::Float(l + *r as f64),
                TypeDescriptor::FLOAT,
            )),
            (ExpressionValue::String(l), ExpressionValue::String(r)) => Ok(TypedValue::new(
                ExpressionValue::String(format!("{l}{r}")),
                TypeDescriptor::STRING,
            )),
            // Spring: String + 任意类型 = 字符串连接
            (ExpressionValue::String(l), other) => {
                let other_str = Self::value_to_string(other);
                Ok(TypedValue::new(
                    ExpressionValue::String(format!("{l}{other_str}")),
                    TypeDescriptor::STRING,
                ))
            }
            (other, ExpressionValue::String(r)) => {
                let other_str = Self::value_to_string(other);
                Ok(TypedValue::new(
                    ExpressionValue::String(format!("{other_str}{r}")),
                    TypeDescriptor::STRING,
                ))
            }
            _ => Err(EvaluationException::new(
                "",
                None,
                "加法运算不支持的操作数类型",
            )),
        }
    }
}

impl OpPlus {
    /// 将 ExpressionValue 转换为字符串表示（用于 String + X 场景）。
    fn value_to_string(val: &ExpressionValue) -> String {
        match val {
            ExpressionValue::Int(i) => i.to_string(),
            ExpressionValue::Long(l) => l.to_string(),
            ExpressionValue::Float(f) => f.to_string(),
            ExpressionValue::Double(d) => d.to_string(),
            ExpressionValue::BigInt(b) => b.to_string(),
            ExpressionValue::Decimal(d) => d.to_string(),
            ExpressionValue::Char(c) => c.to_string(),
            ExpressionValue::Boolean(b) => b.to_string(),
            ExpressionValue::String(s) => s.clone(),
            ExpressionValue::DateTime(dt) => dt.to_string(),
            ExpressionValue::Duration(d) => d.to_string(),
            ExpressionValue::List(l) => {
                let items: Vec<String> = l.iter().map(|v| Self::value_to_string(v.value())).collect();
                format!("[{}]", items.join(", "))
            }
            ExpressionValue::Map(m) => {
                let entries: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}: {}", Self::value_to_string(k.value()), Self::value_to_string(v.value())))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
            ExpressionValue::Object(_) => "<object>".to_string(),
            ExpressionValue::Null => "null".to_string(),
        }
    }
}

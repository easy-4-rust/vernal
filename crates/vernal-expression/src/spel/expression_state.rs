//! 表达式求值状态。
//!
//! 对标 Spring 的 `ExpressionState`。

use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::operation::Operation;
use crate::typed_value::TypedValue;
use crate::typed_value::{ExpressionValue, TypeDescriptor};
use std::collections::HashMap;

/// 表达式求值状态。
///
/// 维护每次表达式求值的局部变量、作用域根对象和活动上下文对象。
/// 对标 Spring 的 `org.springframework.expression.spel.ExpressionState`。
pub struct ExpressionState<'a> {
    /// 求值上下文
    context: &'a dyn EvaluationContext,
    /// 局部变量
    variables: HashMap<String, TypedValue>,
}

impl<'a> ExpressionState<'a> {
    /// 创建表达式求值状态。
    #[must_use]
    pub fn new(context: &'a dyn EvaluationContext) -> Self {
        Self {
            context,
            variables: HashMap::new(),
        }
    }

    /// 获取根上下文对象。
    #[must_use]
    pub fn root_context_object(&self) -> &TypedValue {
        self.context.root_object()
    }

    /// 设置变量。
    pub fn set_variable(&mut self, name: &str, value: TypedValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// 查找变量。
    #[must_use]
    pub fn lookup_variable(&self, name: &str) -> Option<&TypedValue> {
        self.variables
            .get(name)
            .or_else(|| self.context.lookup_variable(name))
    }

    /// 获取求值上下文。
    #[must_use]
    pub fn evaluation_context(&self) -> &dyn EvaluationContext {
        self.context
    }

    /// 执行操作。
    pub fn operate(
        &self,
        op: Operation,
        left: &TypedValue,
        right: &TypedValue,
    ) -> Result<TypedValue, EvaluationException> {
        match (op, left.value(), right.value()) {
            (Operation::Add, ExpressionValue::Int(l), ExpressionValue::Int(r)) => Ok(
                TypedValue::new(ExpressionValue::Int(l + r), TypeDescriptor::INT),
            ),
            (Operation::Subtract, ExpressionValue::Int(l), ExpressionValue::Int(r)) => Ok(
                TypedValue::new(ExpressionValue::Int(l - r), TypeDescriptor::INT),
            ),
            (Operation::Multiply, ExpressionValue::Int(l), ExpressionValue::Int(r)) => Ok(
                TypedValue::new(ExpressionValue::Int(l * r), TypeDescriptor::INT),
            ),
            (Operation::Divide, ExpressionValue::Int(l), ExpressionValue::Int(r)) => {
                if *r == 0 {
                    return Err(EvaluationException::new("", None, "除零错误"));
                }
                Ok(TypedValue::new(
                    ExpressionValue::Int(l / r),
                    TypeDescriptor::INT,
                ))
            }
            _ => Err(EvaluationException::new("", None, "不支持的操作")),
        }
    }
}

//! 正则匹配运算符。
//!
//! 对标 Spring 的 `OperatorMatches`：`value matches regex`

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 正则匹配运算符节点。
///
/// 使用 regex 检查字符串是否匹配模式。
pub struct OperatorMatches {
    value: Box<dyn SpelNode>,
    pattern: Box<dyn SpelNode>,
}

impl OperatorMatches {
    #[must_use]
    pub fn new(value: Box<dyn SpelNode>, pattern: Box<dyn SpelNode>) -> Self {
        Self { value, pattern }
    }
}

impl SpelNode for OperatorMatches {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let value = self.value.get_value(context)?;
        let pattern = self.pattern.get_value(context)?;
        let result = match (value.value(), pattern.value()) {
            (ExpressionValue::String(s), ExpressionValue::String(p)) => {
                #[cfg(feature = "regex")]
                {
                    regex::Regex::new(p).map(|re| re.is_match(s)).map_err(|e| {
                        EvaluationException::new("", None, format!("正则表达式错误: {}", e))
                    })?
                }
                #[cfg(not(feature = "regex"))]
                {
                    let _ = (s, p);
                    return Err(EvaluationException::new("", None, "regex feature 未启用"));
                }
            }
            _ => {
                return Err(EvaluationException::new(
                    "",
                    None,
                    "matches 运算需要字符串类型",
                ));
            }
        };
        Ok(TypedValue::new(
            ExpressionValue::Boolean(result),
            TypeDescriptor::BOOLEAN,
        ))
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} matches {})",
            self.value.to_string_ast(),
            self.pattern.to_string_ast()
        )
    }
}

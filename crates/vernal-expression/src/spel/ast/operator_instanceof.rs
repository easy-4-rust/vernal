//! instanceof 运算符（对标 Spring `OperatorInstanceof`）。
//!
//! 对标 Java `org.springframework.expression.spel.ast.OperatorInstanceof`。
//! 检查值是否为指定类型的实例。
//!
//! # Spring 行为
//!
//! - 右操作数必须是类型引用（`T(...)`）或类型名字符串
//! - 值为 `null` 时返回 `false`
//! - 检查值的类型是否与目标类型匹配（考虑类型层次）

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// instanceof 运算符节点。
///
/// 检查值是否为指定类型的实例。
pub struct OperatorInstanceof {
    /// 左操作数（待检查的值）。
    value: Box<dyn SpelNode>,
    /// 右操作数（类型名字符串）。
    type_name: String,
}

impl OperatorInstanceof {
    /// 创建 instanceof 节点。
    ///
    /// # 参数
    ///
    /// - `value` — 待检查的值表达式
    /// - `type_name` — 类型名（如 `"String"`、`"java.lang.Integer"`）
    #[must_use]
    pub fn new(value: Box<dyn SpelNode>, type_name: String) -> Self {
        Self { value, type_name }
    }

    /// 获取类型名。
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// 将 ExpressionValue 与类型名匹配。
    ///
    /// 对标 Spring `OperatorInstanceof.getValueInternal` 中的类型检查逻辑。
    /// 支持 `T(String)` 格式（从 TypeReference 节点传递）。
    fn matches_type(value: &ExpressionValue, type_name: &str) -> bool {
        // 去掉 T(...) 包装
        let raw = if type_name.starts_with("T(") && type_name.ends_with(')') {
            &type_name[2..type_name.len() - 1]
        } else {
            type_name
        };

        match value {
            ExpressionValue::Null => false,
            ExpressionValue::Boolean(_) => {
                matches!(raw, "boolean" | "Boolean" | "java.lang.Boolean")
            }
            ExpressionValue::Int(_) => {
                matches!(
                    raw,
                    "int"
                        | "Integer"
                        | "java.lang.Integer"
                        | "long"
                        | "Long"
                        | "java.lang.Long"
                        | "Number"
                        | "java.lang.Number"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Long(_) => {
                matches!(
                    raw,
                    "long"
                        | "Long"
                        | "java.lang.Long"
                        | "Number"
                        | "java.lang.Number"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Float(_) => {
                matches!(
                    raw,
                    "float"
                        | "Float"
                        | "java.lang.Float"
                        | "double"
                        | "Double"
                        | "java.lang.Double"
                        | "Number"
                        | "java.lang.Number"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Double(_) => {
                matches!(
                    raw,
                    "double"
                        | "Double"
                        | "java.lang.Double"
                        | "Number"
                        | "java.lang.Number"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::BigInt(_) => {
                matches!(
                    raw,
                    "BigInteger"
                        | "java.math.BigInteger"
                        | "Number"
                        | "java.lang.Number"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Decimal(_) => {
                matches!(
                    raw,
                    "BigDecimal"
                        | "java.math.BigDecimal"
                        | "Number"
                        | "java.lang.Number"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Char(_) => {
                matches!(
                    raw,
                    "char" | "Character" | "java.lang.Character" | "Object" | "java.lang.Object"
                )
            }
            ExpressionValue::String(_) => {
                matches!(
                    raw,
                    "String"
                        | "java.lang.String"
                        | "CharSequence"
                        | "java.lang.CharSequence"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::DateTime(_) => {
                matches!(
                    raw,
                    "Date"
                        | "java.util.Date"
                        | "Instant"
                        | "java.time.Instant"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Duration(_) => {
                matches!(
                    raw,
                    "Duration" | "java.time.Duration" | "Object" | "java.lang.Object"
                )
            }
            ExpressionValue::List(_) => {
                matches!(
                    raw,
                    "List"
                        | "java.util.List"
                        | "Collection"
                        | "java.util.Collection"
                        | "Iterable"
                        | "java.lang.Iterable"
                        | "Object"
                        | "java.lang.Object"
                )
            }
            ExpressionValue::Map(_) => {
                matches!(raw, "Map" | "java.util.Map" | "Object" | "java.lang.Object")
            }
            ExpressionValue::Object(_) => {
                // 对于 Object 类型，检查 TypeId（需要 TypeLocator）
                // 当前实现：返回 true（Phase F: 完整 TypeId 匹配）
                true
            }
        }
    }
}

impl SpelNode for OperatorInstanceof {
    fn get_value(
        &self,
        context: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        let value = self.value.get_value(context)?;

        // null 值始终返回 false
        if value.is_null() {
            return Ok(TypedValue::new(
                ExpressionValue::Boolean(false),
                TypeDescriptor::BOOLEAN,
            ));
        }

        // 尝试通过 TypeLocator 查找类型（对标 Spring T(...) 语法）
        if let Some(locator) = context.type_locator() {
            if let Ok(_type_id) = locator.find_type(&self.type_name) {
                // Phase F: 通过 TypeId 检查运行时类型
                // 当前简化：基于 ExpressionValue 变体匹配
                let result = Self::matches_type(value.value(), &self.type_name);
                return Ok(TypedValue::new(
                    ExpressionValue::Boolean(result),
                    TypeDescriptor::BOOLEAN,
                ));
            }
        }

        // TypeLocator 未配置或类型未找到：基于 ExpressionValue 变体匹配
        let result = Self::matches_type(value.value(), &self.type_name);
        Ok(TypedValue::new(
            ExpressionValue::Boolean(result),
            TypeDescriptor::BOOLEAN,
        ))
    }

    fn to_string_ast(&self) -> String {
        format!(
            "({} instanceof {})",
            self.value.to_string_ast(),
            self.type_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_value::TypedValue;

    fn eval_instanceof(_expr_str: &str, type_name: &str) -> bool {
        let node = OperatorInstanceof::new(
            Box::new(crate::spel::ast::int_literal::IntLiteral::new(
                5,
                "5".to_string(),
            )),
            type_name.to_string(),
        );
        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
            TypedValue::null(),
        );
        match node.get_value(&ctx) {
            Ok(tv) => matches!(tv.value(), ExpressionValue::Boolean(true)),
            Err(_) => false,
        }
    }

    #[test]
    fn int_instanceof_int() {
        assert!(eval_instanceof("5", "int"));
    }

    #[test]
    fn int_instanceof_number() {
        assert!(eval_instanceof("5", "Number"));
    }

    #[test]
    fn int_instanceof_string_is_false() {
        assert!(!eval_instanceof("5", "String"));
    }

    #[test]
    fn null_instanceof_any_is_false() {
        let node = OperatorInstanceof::new(
            Box::new(crate::spel::ast::null_literal::NullLiteral::new()),
            "String".to_string(),
        );
        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
            TypedValue::null(),
        );
        let result = node.get_value(&ctx).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Boolean(false));
    }

    #[test]
    fn t_string_instanceof() {
        // OperatorInstanceof receives type_name from parser
        let node = OperatorInstanceof::new(
            Box::new(crate::spel::ast::string_literal::StringLiteral::new(
                "hello".to_string(),
            )),
            "T(String)".to_string(),
        );
        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
            TypedValue::null(),
        );
        let result = node.get_value(&ctx).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Boolean(true));
    }
}

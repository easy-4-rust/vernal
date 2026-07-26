//! 表达式条件（对标 Spring 的 @ConditionalOnExpression）。
//!
//! 通过 `vernal-expression` 的表达式求值实现条件装配。
//! 表达式可以引用 `ApplicationEnvironment` 中的属性。
//!
//! # 设计来源
//!
//! 对标 Spring 的 `@ConditionalOnExpression("'${app.mode}' == 'production'")`。
//! 使用 vernal-expression 的简化 SpEL 子集进行求值。

use vernal_core::BoxError;
use vernal_expression::ExpressionParser;

use crate::{ApplicationEnvironment, component_condition::ComponentCondition};

/// 表达式条件。
///
/// 通过表达式求值决定组件是否进入 IoC 依赖图。
/// 表达式可以引用环境属性（如 `${app.mode}`）。
///
/// # 使用方式
///
/// ```rust,ignore
/// use vernal_context::ExpressionCondition;
///
/// // 当 app.mode == "production" 时启用组件
/// let condition = ExpressionCondition::new("'${app.mode}' == 'production'");
///
/// // 当 app.debug == true 时启用组件
/// let condition = ExpressionCondition::new("${app.debug} == true");
/// ```
pub struct ExpressionCondition {
    /// 表达式字符串
    expression: String,
}

impl ExpressionCondition {
    /// 创建表达式条件。
    ///
    /// # 参数
    /// - `expression`：表达式字符串，支持 `${key}` 引用环境属性
    #[must_use]
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    /// 获取表达式字符串。
    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// 解析表达式中的环境属性引用。
    ///
    /// 将 `${key}` 替换为环境中的实际值。
    fn resolve_expression(&self, environment: &ApplicationEnvironment) -> Result<String, BoxError> {
        let mut resolved = self.expression.clone();

        // 查找所有 ${key} 模式并替换
        while let Some(start) = resolved.find("${") {
            if let Some(end) = resolved[start..].find('}') {
                let key = &resolved[start + 2..start + end];
                let value: String = environment
                    .get(key)
                    .map_err(|e| -> BoxError { Box::new(e) })?
                    .unwrap_or_default();
                resolved = format!(
                    "{}{}{}",
                    &resolved[..start],
                    value,
                    &resolved[start + end + 1..]
                );
            } else {
                break;
            }
        }

        Ok(resolved)
    }
}

impl ComponentCondition for ExpressionCondition {
    fn name(&self) -> &'static str {
        "expression"
    }

    fn matches(&self, environment: &ApplicationEnvironment) -> Result<bool, BoxError> {
        // 解析表达式中的环境属性引用
        let resolved = self.resolve_expression(environment)?;

        // 简单的表达式求值：支持 ==、!=、true、false
        // 完整 SpEL 求值由 vernal-expression 提供
        let result = evaluate_simple_expression(&resolved)?;
        Ok(result)
    }
}

/// 简单表达式求值。
///
/// 支持的表达式：
/// - `true` / `false` 字面量
/// - `"value1" == "value2"` 字符串相等
/// - `"value1" != "value2"` 字符串不等
/// - `number == number` 数字相等
/// - `number != number` 数字不等
fn evaluate_simple_expression(expr: &str) -> Result<bool, BoxError> {
    let trimmed = expr.trim();

    // 布尔字面量
    if trimmed == "true" {
        return Ok(true);
    }
    if trimmed == "false" {
        return Ok(false);
    }

    // 字符串相等比较："value1" == "value2"
    if let Some(pos) = trimmed.find(" == ") {
        let left = trimmed[..pos].trim();
        let right = trimmed[pos + 4..].trim();
        return Ok(compare_values(left, right));
    }

    // 字符串不等比较："value1" != "value2"
    if let Some(pos) = trimmed.find(" != ") {
        let left = trimmed[..pos].trim();
        let right = trimmed[pos + 4..].trim();
        return Ok(!compare_values(left, right));
    }

    // 无法解析的表达式，尝试作为布尔值
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("无法解析表达式: {}", trimmed),
    )))
}

/// 比较两个值是否相等。
///
/// 支持字符串（带引号）和数字比较。
fn compare_values(left: &str, right: &str) -> bool {
    // 去除引号
    let left_clean = left.trim_matches('"');
    let right_clean = right.trim_matches('"');

    // 尝试数字比较
    if let (Ok(l), Ok(r)) = (left_clean.parse::<f64>(), right_clean.parse::<f64>()) {
        return (l - r).abs() < f64::EPSILON;
    }

    // 字符串比较
    left_clean == right_clean
}

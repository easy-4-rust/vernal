//! 表达式条件（对标 Spring `@ConditionalOnExpression`）。
//!
//! 通过 `vernal-expression` 的 SpEL 求值器实现条件装配。
//!
//! # 设计来源
//!
//! 对标 Spring 的 `@ConditionalOnExpression("'${app.mode}' == 'production'")`。
//! 使用 vernal-expression 的递归下降解析器执行真实 SpEL 表达式求值。

use vernal_core::BoxError;
use vernal_expression::{
    ExpressionParser, EvaluationContext, ExpressionValue, TypeDescriptor, TypedValue,
};

use crate::{ApplicationEnvironment, component_condition::ComponentCondition};

/// 表达式条件。
///
/// 通过 SpEL 表达式求值决定组件是否进入 IoC 依赖图。
/// 表达式可以引用环境属性（如 `${app.mode}`）。
///
/// # 使用方式
///
/// ```rust,ignore
/// use vernal_context::ExpressionCondition;
///
/// // 当 app.mode == "production" 时启用组件
/// let condition = ExpressionCondition::new("'${app.mode}' == 'production'");
/// ```
pub struct ExpressionCondition {
    /// 表达式字符串
    expression: String,
}

impl ExpressionCondition {
    /// 创建表达式条件。
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
    /// 将 `${key}` 替换为 `#env_key` 变量引用，避免表达式注入；
    /// 引号包裹的占位符（`'${key}'`）连同引号整体替换（对标 Spring
    /// 占位符先于 SpEL 求值的语义）。返回（转换后的表达式，变量绑定）。
    fn resolve_expression(
        &self,
        environment: &ApplicationEnvironment,
    ) -> Result<(String, Vec<(String, TypedValue)>), BoxError> {
        let mut resolved = self.expression.clone();
        let mut env_vars: Vec<(String, TypedValue)> = Vec::new();

        // 查找所有 ${key} 模式，替换为 #env_key 变量引用
        while let Some(start) = resolved.find("${") {
            if let Some(end) = resolved[start..].find('}') {
                let key = &resolved[start + 2..start + end];
                let value: String = environment
                    .get(key)
                    .map_err(|e| -> BoxError { Box::new(e) })?
                    .unwrap_or_default();

                // 将 ${key} 替换为 #env_key（变量引用，不是值拼接）
                let var_name = format!("env_{}", key.replace('.', "_"));
                let replacement = format!("#{}", var_name);
                // 引号包裹的占位符连引号一起替换，避免生成字面量字符串
                let (span_start, span_end) = if resolved[..start].ends_with('\'')
                    && resolved[start + end + 1..].starts_with('\'')
                {
                    (start - 1, start + end + 2)
                } else {
                    (start, start + end + 1)
                };
                resolved = format!(
                    "{}{}{}",
                    &resolved[..span_start],
                    replacement,
                    &resolved[span_end..]
                );
                env_vars.push((var_name, typed_value(&value)));
            } else {
                break;
            }
        }

        Ok((resolved, env_vars))
    }
}

/// 把环境值绑定为类型化值：整数 → `Int`、浮点 → `Float`、其余 → `String`
/// （对标 Spring 占位符替换后字面量的类型推断）。
fn typed_value(raw: &str) -> TypedValue {
    if let Ok(int) = raw.parse::<i64>() {
        TypedValue::new(ExpressionValue::Int(int), TypeDescriptor::INT)
    } else if let Ok(float) = raw.parse::<f64>() {
        TypedValue::new(ExpressionValue::Float(float), TypeDescriptor::FLOAT)
    } else {
        TypedValue::new(
            ExpressionValue::String(raw.to_string()),
            TypeDescriptor::STRING,
        )
    }
}

impl ComponentCondition for ExpressionCondition {
    fn name(&self) -> &'static str {
        "expression"
    }

    fn matches(&self, environment: &ApplicationEnvironment) -> Result<bool, BoxError> {
        // 解析表达式中的环境属性引用为变量绑定（避免表达式注入）
        let (resolved, env_vars) = self.resolve_expression(environment)?;

        // 用 vernal-expression 的 SpEL 解析器求值
        let parser = vernal_expression::spel::spel_expression_parser::SpelExpressionParser::new();
        let mut ctx = vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
            vernal_expression::TypedValue::null(),
        );

        // 将环境变量绑定到上下文（值不会被注入到表达式源码中）
        for (var_name, value) in env_vars {
            ctx.set_variable(&var_name, value);
        }

        let expr = parser
            .parse_expression(&resolved)
            .map_err(|e| -> BoxError {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("SpEL 解析失败: {e}"),
                ))
            })?;

        let result = expr
            .get_value_with_context(&ctx)
            .map_err(|e| -> BoxError {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("SpEL 求值失败: {e}"),
                ))
            })?;

        match result.value() {
            vernal_expression::ExpressionValue::Boolean(b) => Ok(*b),
            vernal_expression::ExpressionValue::Null => Ok(false),
            vernal_expression::ExpressionValue::Int(i) => Ok(*i != 0),
            vernal_expression::ExpressionValue::Long(l) => Ok(*l != 0),
            vernal_expression::ExpressionValue::Double(d) => Ok(*d != 0.0),
            vernal_expression::ExpressionValue::Float(f) => Ok(*f != 0.0),
            vernal_expression::ExpressionValue::String(s) => Ok(!s.is_empty()),
            _ => Ok(true),
        }
    }
}

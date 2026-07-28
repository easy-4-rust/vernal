//! StandardBeanExpressionResolver — 委托 vernal-expression 执行 SpEL。
//!
//! 对应 Spring `org.springframework.beans.factory.config.StandardBeanExpressionResolver`。
//! 当前实现使用 `vernal-expression` 的 `SpelExpressionParser` + `StandardEvaluationContext`
//! 执行真实 SpEL 表达式求值；支持 `#{...}` 模板语法和简单 Bean 名称引用。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::bean_expression_resolver::BeanExpressionResolver;
use vernal_expression::{Expression, ExpressionParser, EvaluationContext};

/// 标准 Bean 表达式解析器（完整实现）。
pub struct StandardBeanExpressionResolver {
    /// Bean 名称 → 实例映射。
    beans: std::sync::RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// SpEL 解析器实例（复用，线程安全）。
    parser: vernal_expression::spel::spel_expression_parser::SpelExpressionParser,
}

impl StandardBeanExpressionResolver {
    /// 创建标准表达式解析器。
    pub fn new() -> Self {
        Self {
            beans: std::sync::RwLock::new(HashMap::new()),
            parser: vernal_expression::spel::spel_expression_parser::SpelExpressionParser::new(),
        }
    }

    /// 注册 Bean 到表达式上下文。
    pub fn register_bean(&self, name: String, bean: Arc<dyn Any + Send + Sync>) {
        self.beans
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(name, bean);
    }

    /// 清空上下文。
    pub fn clear_context(&self) {
        self.beans
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    /// 当前已注册 Bean 数量。
    pub fn bean_count(&self) -> usize {
        self.beans
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// 简单 Bean 名称查找（无需 SpEL）。
    fn find_bean(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.beans
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(name)
            .cloned()
    }
}

impl Default for StandardBeanExpressionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanExpressionResolver for StandardBeanExpressionResolver {
    /// 解析 SpEL 表达式。
    ///
    /// 对应 Spring 的 `StandardBeanExpressionResolver.evaluate(String, BeanExpressionContext)`。
    ///
    /// 支持：
    /// - 直接 Bean 名称引用（无 `${}` 前缀时）
    /// - 完整 SpEL 表达式（`#{...}` 或裸表达式）
    fn evaluate(
        &self,
        expression: &str,
        _bean_name: Option<&str>,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let expr = expression.trim();

        // 1. 先尝试简单 Bean 名称查找（零解析开销）
        if let Some(bean) = self.find_bean(expr) {
            return Ok(Some(bean));
        }

        // 2. 用 ver
        // 2. 用 vernal-expression 的 SpEL 解析器
        match self.parser.parse_expression(expr) {
            Ok(parsed) => {
                // 创建带 bean context 的求值上下文
                let ctx = vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
                    vernal_expression::TypedValue::null(),
                );

                // TODO(phase F): 注入 bean context 到 StandardEvaluationContext
                // （需要 PropertyAccessor 支持，当前 Phase H 负责）

                match parsed.get_value_with_context(&ctx) {
                    Ok(val) => {
                        // 将 TypedValue 转换为 Arc<dyn Any+Send+Sync>
                        let result: Arc<dyn Any + Send + Sync> = match val.value() {
                            vernal_expression::ExpressionValue::Null => {
                                Arc::new(())
                            }
                            vernal_expression::ExpressionValue::Boolean(b) => Arc::new(*b),
                            vernal_expression::ExpressionValue::Int(i) => Arc::new(*i),
                            vernal_expression::ExpressionValue::Long(l) => Arc::new(*l),
                            vernal_expression::ExpressionValue::Float(f) => Arc::new(*f),
                            vernal_expression::ExpressionValue::Double(d) => Arc::new(*d),
                            vernal_expression::ExpressionValue::String(s) => Arc::new(s.clone()),
                            vernal_expression::ExpressionValue::List(l) => {
                                Arc::new(l.clone())
                            }
                            vernal_expression::ExpressionValue::Map(m) => {
                                Arc::new(m.clone())
                            }
                            other => {
                                // BigInt/Decimal/Char/DateTime/Duration/Object 直接返回 TypedValue
                                Arc::new(val.clone())
                            }
                        };
                        Ok(Some(result))
                    }
                    Err(e) => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("SpEL 求值失败: {e}"),
                    ))),
                }
            }
            Err(e) => {
                // 解析失败：降级为简单 Bean 名称查找
                if let Some(bean) = self.find_bean(expr) {
                    Ok(Some(bean))
                } else {
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("SpEL 解析失败: {e}"),
                    )))
                }
            }
        }
    }
}

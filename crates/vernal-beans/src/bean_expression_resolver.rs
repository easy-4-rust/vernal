//! BeanExpressionResolver — Spring 风格的 Bean 表达式解析器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanExpressionResolver`。
//!
//! 解析 Bean 定义中的 SpEL 表达式（`#{...}`）和属性占位符（`${...}`）。

use std::sync::Arc;

/// Spring 风格的 Bean 表达式解析器接口。
///
/// 对应 Spring 的 `BeanExpressionResolver`。
///
/// 解析 Bean 定义中的表达式：
/// - SpEL 表达式：`#{...}`
/// - 属性占位符：`${...}`
///
/// ## 与 vernal-expression 的关系
///
/// `StandardBeanExpressionResolver` 会委托 `vernal-expression` crate
/// 执行 SpEL 表达式求值。
pub trait BeanExpressionResolver: Send + Sync + 'static {
    /// 解析表达式。
    ///
    /// 对应 Spring 的 `Object evaluate(String expression, BeanExpressionContext ctx)`。
    ///
    /// # 参数
    ///
    /// - `expression` — 表达式字符串（如 `"#{systemProperties['user.name']}"`）
    /// - `bean_name` — 当前 Bean 名称（用于 `#root` 或 `#this`）
    ///
    /// # 返回
    ///
    /// - `Ok(value)` — 解析后的值
    /// - `Err` — 解析失败
    fn evaluate(
        &self,
        expression: &str,
        bean_name: Option<&str>,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;
}

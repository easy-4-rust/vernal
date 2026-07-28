//! `#[value("...")]` — Spring @Value 注解的 Rust 等价物。
//!
//! 在 Spring 中，`@Value("#{beanName.property}")` 用于在 Bean 属性注入时
//! 解析 SpEL 表达式。Rust 没有运行时注解，因此我们用类型化结构体 `ValueBinding`
//! 代替，用户在定义 BeanDefinition 时通过 builder 显式声明绑定关系。
//!
//! # 对标 Spring
//!
//! - `@Value("#{systemProperties['user.name']}")`
//! - `@Value("${app.port:8080}")`（占位符）
//! - `@Value("1 + 2")`（直接 SpEL）
//!
//! 集成路径：`ConfigurableListableBeanFactory` 在 `populateBean` 阶段调用
//! `ValueExpressionResolver.resolve_all(bean)` → 委托 `BeanExpressionResolver`。

use std::sync::Arc;

/// 值绑定（Spring `@Value` 的 Rust 等价物）。
///
/// # Spring 映射
///
/// ```java
/// @Value("#{user.email}")
/// private String email;
///
/// @Value("${server.port:8080}")
/// private int port;
///
/// @Value("1 + 2")
/// private int sum;
/// ```
#[derive(Debug, Clone)]
pub struct ValueBinding {
    /// 绑定的表达式字符串。
    ///
    /// 支持三种形式：
    /// - SpEL：`#{expr}` 或裸 `expr`（如 `1 + 2`、`foo.bar`）
    /// - 占位符：`${key}` 或 `${key:default}`
    /// - Bean 名称：直接引用 bean 名称（等价于 `#{beanName}`）
    pub expression: String,
    /// 目标字段名称（日志/诊断用）。
    pub field_name: String,
    /// 是否为必需（无默认值时抛错）。
    pub required: bool,
}

impl ValueBinding {
    /// 创建绑定。
    #[must_use]
    pub fn new(expression: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            field_name: field_name.into(),
            required: true,
        }
    }

    /// 设置为可选（占位符支持默认值时）。
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// 判断是否为占位符形式（`${...}`）。
    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        self.expression.starts_with("${") && self.expression.ends_with('}')
    }

    /// 判断是否为 SpEL 形式（`#{...}` 或裸表达式）。
    #[must_use]
    pub fn is_spel(&self) -> bool {
        self.expression.starts_with("#{") && self.expression.ends_with('}')
            || !self.is_placeholder()
    }

    /// 提取占位符 key（去除 `${}` 包裹）。
    #[must_use]
    pub fn placeholder_key(&self) -> Option<&str> {
        if self.is_placeholder() {
            Some(&self.expression[2..self.expression.len() - 1])
        } else {
            None
        }
    }

    /// 提取 SpEL 表达式（去除 `#{}` 包裹）。
    #[must_use]
    pub fn spel_expression(&self) -> &str {
        if self.expression.starts_with("#{") && self.expression.ends_with('}') {
            &self.expression[2..self.expression.len() - 1]
        } else {
            &self.expression
        }
    }
}

/// 值表达式解析器（Spring `ValueExpressionResolver` 的简化版）。
///
/// 协调 `BeanExpressionResolver` 和属性源（`Environment`）执行表达式求值。
pub struct ValueExpressionResolver {
    /// Bean 表达式解析器引用。
    bean_expression_resolver: Arc<dyn value_expression_resolver_trait::BeanExpressionResolverBridge>,
}

/// Bean 表达式解析器桥接 trait（避免循环依赖）。
pub mod value_expression_resolver_trait {
    use std::any::Any;
    use std::sync::Arc;

    /// 桥接 trait：表达式求值 → `Arc<dyn Any>`。
    pub trait BeanExpressionResolverBridge: Send + Sync + 'static {
        fn evaluate(
            &self,
            expression: &str,
            bean_name: Option<&str>,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;
    }
}

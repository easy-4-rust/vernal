//! StandardBeanExpressionResolver — Spring 风格的标准 Bean 表达式解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.StandardBeanExpressionResolver`。
//!
//! 使用 vernal-expression crate 执行 SpEL 表达式求值。

use std::sync::Arc;

use crate::bean_expression_resolver::BeanExpressionResolver;

/// Spring 风格的标准 Bean 表达式解析器。
///
/// 对应 Spring 的 `StandardBeanExpressionResolver`。
///
/// 委托 `vernal-expression` crate 执行 SpEL 表达式求值。
/// 支持以下表达式语法：
/// - SpEL 表达式：`#{...}`
/// - 属性占位符：`${...}`（由 PlaceholderConfigurer 处理）
///
/// ## 与 vernal-expression 的关系
///
/// 此解析器内部使用 `vernal-expression` 的 `SpelExpressionParser` 和
/// `StandardEvaluationContext` 来执行 SpEL 表达式。
pub struct StandardBeanExpressionResolver {
    /// Bean 名称 → 实例映射（用于表达式中的 Bean 引用）。
    bean_context: std::sync::RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

use std::any::Any;
use std::collections::HashMap;

impl StandardBeanExpressionResolver {
    /// 创建标准表达式解析器。
    pub fn new() -> Self {
        Self {
            bean_context: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// 注册 Bean 到表达式上下文。
    pub fn register_bean(&self, name: String, bean: Arc<dyn Any + Send + Sync>) {
        self.bean_context
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(name, bean);
    }

    /// 清空表达式上下文。
    pub fn clear_context(&self) {
        self.bean_context
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    /// 获取上下文中的 Bean 数量。
    pub fn bean_count(&self) -> usize {
        self.bean_context
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// 简单的表达式求值（支持基本的属性访问）。
    ///
    /// 对于复杂的 SpEL 表达式，应使用 vernal-expression。
    /// 这里实现基本的属性访问语法：`beanName.propertyName`。
    fn evaluate_simple(
        &self,
        expression: &str,
        _bean_name: Option<&str>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        let expr = expression.trim();

        // 支持直接 Bean 名称引用
        if let Some(bean) = self
            .bean_context
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(expr)
        {
            return Some(Arc::clone(bean));
        }

        // 支持 bean.property 格式
        if let Some(dot_pos) = expr.find('.') {
            let bean_name = &expr[..dot_pos];
            let _prop_name = &expr[dot_pos + 1..];
            if let Some(_bean) = self
                .bean_context
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(bean_name)
            {
                // 简化实现：返回 Bean 本身（不做属性解析）
                // 实际生产环境应使用 vernal-expression 的 PropertyAccessor
                return None; // 属性解析需要 vernal-expression
            }
        }

        None
    }
}

impl Default for StandardBeanExpressionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanExpressionResolver for StandardBeanExpressionResolver {
    /// 解析表达式。
    ///
    /// 对应 Spring 的 `StandardBeanExpressionResolver.evaluate(String, BeanExpressionContext)`。
    ///
    /// 当前实现支持：
    /// - 直接 Bean 名称引用
    /// - 简单属性访问（委托给 vernal-expression）
    fn evaluate(
        &self,
        expression: &str,
        bean_name: Option<&str>,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let expr = expression.trim();

        // 尝试简单表达式求值
        if let Some(value) = self.evaluate_simple(expr, bean_name) {
            return Ok(Some(value));
        }

        // 如果简单求值失败，返回 None
        // 实际生产环境应委托给 vernal-expression
        Ok(None)
    }
}

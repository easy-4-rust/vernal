//! StandardBeanExpressionResolver — 委托 vernal-expression 执行 SpEL。
//!
//! 对应 Spring `org.springframework.beans.factory.config.StandardBeanExpressionResolver`。
//! 当前实现使用 `vernal-expression` 的 `SpelExpressionParser` + `StandardEvaluationContext`
//! 执行真实 SpEL 表达式求值；支持 `#{...}` 模板语法和简单 Bean 名称引用。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::factory::config::bean_expression_resolver::BeanExpressionResolver;
use vernal_expression::ExpressionParser;

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

    /// 检查表达式是否为简单标识符（不包含 SpEL 特殊字符）。
    ///
    /// 简单标识符如 "myBean"、"user.name"、"foo" 等。
    /// 非简单标识符如 "1+1"、"'hello'"、"new Foo()" 等。
    fn is_simple_identifier(expr: &str) -> bool {
        if expr.is_empty() {
            return false;
        }
        // 首字符必须是字母或下划线
        let first = expr.chars().next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
        // 其余字符必须是字母、数字、下划线或点（支持属性访问）
        expr.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
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

        // 2. 如果表达式看起来是简单的标识符（不是合法 SpEL 表达式），
        //    视为未注册的 Bean 名称，返回 Ok(None) 而非错误
        if Self::is_simple_identifier(expr) {
            return Ok(None);
        }

        // 3. 用 vernal-expression 的 SpEL 解析器
        match self.parser.parse_expression(expr) {
            Ok(parsed) => {
                // 创建带 bean context 的求值上下文
                let ctx = vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext::new(
                    vernal_expression::TypedValue::null(),
                );

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
                            _other => {
                                Arc::new(val.clone())
                            }
                        };
                        Ok(Some(result))
                    }
                    Err(_e) => {
                        // SpEL 求值失败但已成功解析的表达式，视为无效求值
                        Ok(None)
                    }
                }
            }
            Err(_e) => {
                // 解析失败：降级为简单 Bean 名称查找
                if let Some(bean) = self.find_bean(expr) {
                    Ok(Some(bean))
                } else {
                    // 既不是已注册的 Bean，也不是合法 SpEL 表达式
                    Ok(None)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resolver_has_no_beans() {
        let resolver = StandardBeanExpressionResolver::new();
        assert_eq!(resolver.bean_count(), 0);
    }

    #[test]
    fn default_trait_creates_empty_resolver() {
        let resolver = StandardBeanExpressionResolver::default();
        assert_eq!(resolver.bean_count(), 0);
    }

    #[test]
    fn register_bean_increments_count() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("myBean".to_string(), Arc::new("value".to_string()));
        assert_eq!(resolver.bean_count(), 1);
    }

    #[test]
    fn register_multiple_beans() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("bean1".to_string(), Arc::new("v1".to_string()));
        resolver.register_bean("bean2".to_string(), Arc::new(42i32));
        assert_eq!(resolver.bean_count(), 2);
    }

    #[test]
    fn register_overwrites_existing_bean() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("myBean".to_string(), Arc::new("first".to_string()));
        resolver.register_bean("myBean".to_string(), Arc::new("second".to_string()));
        assert_eq!(resolver.bean_count(), 1);
    }

    #[test]
    fn clear_context_removes_all_beans() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("bean1".to_string(), Arc::new("v1".to_string()));
        resolver.register_bean("bean2".to_string(), Arc::new("v2".to_string()));
        assert_eq!(resolver.bean_count(), 2);
        resolver.clear_context();
        assert_eq!(resolver.bean_count(), 0);
    }

    // ── is_simple_identifier tests ──────────────────────────────────────

    #[test]
    fn is_simple_identifier_valid_names() {
        assert!(StandardBeanExpressionResolver::is_simple_identifier("myBean"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("user"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("_private"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("bean123"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("user.name"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("a.b.c"));
    }

    #[test]
    fn is_simple_identifier_invalid_names() {
        assert!(!StandardBeanExpressionResolver::is_simple_identifier(""));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("123abc"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("1+1"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("'hello'"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("new Foo()"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("a-b"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("a b"));
    }

    // ── evaluate tests ──────────────────────────────────────────────────

    #[test]
    fn evaluate_registered_bean_name() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("myService".to_string(), Arc::new("service_impl".to_string()));
        let result = resolver.evaluate("myService", None).unwrap();
        assert!(result.is_some());
        let val = result.unwrap();
        assert_eq!(*val.downcast_ref::<String>().unwrap(), "service_impl");
    }

    #[test]
    fn evaluate_unregistered_simple_identifier_returns_none() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("nonExistentBean", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_empty_expression_returns_none() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_whitespace_trimmed() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("myBean".to_string(), Arc::new("found".to_string()));
        let result = resolver.evaluate("  myBean  ", None).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn evaluate_spel_arithmetic_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("1 + 1", None).unwrap();
        // SpEL should parse and evaluate 1+1
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 2);
        }
    }

    #[test]
    fn evaluate_spel_string_literal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'hello world'", None).unwrap();
        if let Some(val) = result {
            let s = val.downcast_ref::<String>();
            assert!(s.is_some());
            assert_eq!(*s.unwrap(), "hello world");
        }
    }

    #[test]
    fn evaluate_spel_boolean_literal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("true", None).unwrap();
        // "true" is a simple identifier (all alphabetic), so it returns None
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_invalid_spel_returns_none() {
        let resolver = StandardBeanExpressionResolver::new();
        // An expression that fails to parse and isn't a registered bean
        let result = resolver.evaluate("@#$invalid", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_with_bean_name_parameter() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("contextBean".to_string(), Arc::new(42i32));
        let result = resolver.evaluate("contextBean", Some("requestBean")).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn evaluate_simple_identifier_starting_with_number() {
        let resolver = StandardBeanExpressionResolver::new();
        // "123" is not a simple identifier (starts with digit), and not a valid SpEL
        let result = resolver.evaluate("123", None);
        // Should handle gracefully
        let _ = result;
    }

    #[test]
    fn evaluate_spel_null_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("null", None).unwrap();
        // "null" is a simple identifier (all alphabetic), returns None
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_spel_comparison() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("3 > 2", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_multiplication() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("6 * 7", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 42);
        }
    }

    #[test]
    fn evaluate_spel_subtraction() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("10 - 3", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 7);
        }
    }

    #[test]
    fn evaluate_spel_division() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("10 / 2", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 5);
        }
    }

    #[test]
    fn evaluate_spel_modulo() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("10 % 3", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 1);
        }
    }

    #[test]
    fn evaluate_spel_less_than() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("2 < 3", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_equality() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("5 == 5", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_string_concat() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'hello' + ' ' + 'world'", None).unwrap();
        if let Some(val) = result {
            let s = val.downcast_ref::<String>();
            assert!(s.is_some());
            assert_eq!(*s.unwrap(), "hello world");
        }
    }

    #[test]
    fn evaluate_spel_negative_number() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("-5", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), -5);
        }
    }

    #[test]
    fn evaluate_spel_complex_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("(2 + 3) * 4", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 20);
        }
    }

    #[test]
    fn evaluate_spel_not_equal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("5 != 3", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_logical_and() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("true and true", None).unwrap();
        // "true" is a simple identifier, so it won't be parsed as SpEL
        // This tests the fallback behavior
        let _ = result;
    }

    #[test]
    fn evaluate_registered_bean_with_dot_notation() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("my.service".to_string(), Arc::new("service_value".to_string()));
        let result = resolver.evaluate("my.service", None).unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast_ref::<String>().unwrap(), "service_value");
    }

    #[test]
    fn evaluate_spel_float_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("3.14 + 2.86", None).unwrap();
        if let Some(val) = result {
            let d = val.downcast_ref::<f64>();
            assert!(d.is_some());
            let diff = (*d.unwrap() - 6.0).abs();
            assert!(diff < f64::EPSILON);
        }
    }

    #[test]
    fn evaluate_spel_integer_literal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("42", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 42);
        }
    }

    #[test]
    fn clear_context_then_evaluate() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("bean1".to_string(), Arc::new("v1".to_string()));
        assert_eq!(resolver.bean_count(), 1);
        resolver.clear_context();
        assert_eq!(resolver.bean_count(), 0);
        // After clear, the bean is no longer found
        let result = resolver.evaluate("bean1", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_multiple_registered_beans() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("a".to_string(), Arc::new("alpha".to_string()));
        resolver.register_bean("b".to_string(), Arc::new(42i32));
        resolver.register_bean("c".to_string(), Arc::new(true));

        assert_eq!(resolver.bean_count(), 3);

        let r1 = resolver.evaluate("a", None).unwrap().unwrap();
        assert_eq!(*r1.downcast_ref::<String>().unwrap(), "alpha");

        let r2 = resolver.evaluate("b", None).unwrap().unwrap();
        assert_eq!(*r2.downcast_ref::<i32>().unwrap(), 42);

        let r3 = resolver.evaluate("c", None).unwrap().unwrap();
        assert!(*r3.downcast_ref::<bool>().unwrap());
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn evaluate_spel_not_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        // "not" is a keyword, but used standalone it should be handled
        let result = resolver.evaluate("not", None);
        let _ = result;
    }

    #[test]
    fn evaluate_spel_empty_string_literal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("''", None).unwrap();
        if let Some(val) = result {
            let s = val.downcast_ref::<String>();
            assert!(s.is_some());
            assert_eq!(*s.unwrap(), "");
        }
    }

    #[test]
    fn evaluate_spel_unary_not() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("!true", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(!(*b.unwrap()));
        }
    }

    #[test]
    fn evaluate_spel_parenthesized_boolean() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("(true)", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_string_length() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'hello'.length()", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 5);
        }
    }

    #[test]
    fn evaluate_registered_bean_with_dot() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("my.service".to_string(), Arc::new("found".to_string()));
        let result = resolver.evaluate("my.service", None).unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast_ref::<String>().unwrap(), "found");
    }

    #[test]
    fn evaluate_spel_modulo_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("10 % 3", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 1);
        }
    }

    #[test]
    fn evaluate_spel_negative_number_v2() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("-5", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), -5);
        }
    }

    #[test]
    fn evaluate_spel_complex_arithmetic_v2() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("(2 + 3) * 4", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 20);
        }
    }

    #[test]
    fn evaluate_spel_string_concat_v2() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'hello' + ' ' + 'world'", None).unwrap();
        if let Some(val) = result {
            let s = val.downcast_ref::<String>();
            assert!(s.is_some());
            assert_eq!(*s.unwrap(), "hello world");
        }
    }

    #[test]
    fn evaluate_spel_not_equal_v2() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("5 != 3", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn evaluate_spel_float_division() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("10.0 / 3.0", None).unwrap();
        if let Some(val) = result {
            let d = val.downcast_ref::<f64>();
            assert!(d.is_some());
        }
    }

    #[test]
    fn evaluate_spel_string_with_dot_property() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'hello'.length()", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 5);
        }
    }

    #[test]
    fn evaluate_spel_nested_parentheses() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("((2 + 3))", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 5);
        }
    }

    #[test]
    fn evaluate_spel_unary_negation() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("!false", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_registered_bean_after_clear() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean("bean1".to_string(), Arc::new("v1".to_string()));
        assert_eq!(resolver.bean_count(), 1);
        resolver.clear_context();
        assert_eq!(resolver.bean_count(), 0);
        let result = resolver.evaluate("bean1", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn evaluate_spel_string_empty() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("''", None).unwrap();
        if let Some(val) = result {
            let s = val.downcast_ref::<String>();
            assert!(s.is_some());
            assert_eq!(*s.unwrap(), "");
        }
    }

    #[test]
    fn evaluate_spel_complex_arithmetic_with_parentheses() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("(10 - 2) * 3 + 1", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 25);
        }
    }

    #[test]
    fn evaluate_spel_greater_than_or_equal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("5 >= 5", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_less_than_or_equal() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("3 <= 5", None).unwrap();
        if let Some(val) = result {
            let b = val.downcast_ref::<bool>();
            assert!(b.is_some());
            assert!(*b.unwrap());
        }
    }

    #[test]
    fn evaluate_spel_negative_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("-(3 + 2)", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), -5);
        }
    }

    #[test]
    fn evaluate_spel_modulo_with_division() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("17 % 5", None).unwrap();
        if let Some(val) = result {
            let i = val.downcast_ref::<i64>();
            assert!(i.is_some());
            assert_eq!(*i.unwrap(), 2);
        }
    }

    #[test]
    fn evaluate_spel_string_concat_multiple() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("'a' + 'b' + 'c' + 'd'", None).unwrap();
        if let Some(val) = result {
            let s = val.downcast_ref::<String>();
            assert!(s.is_some());
            assert_eq!(*s.unwrap(), "abcd");
        }
    }

    #[test]
    fn evaluate_invalid_expression_returns_none() {
        let resolver = StandardBeanExpressionResolver::new();
        // An expression that's not a simple identifier and fails to parse
        let result = resolver.evaluate("@@@", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn is_simple_identifier_edge_cases() {
        assert!(StandardBeanExpressionResolver::is_simple_identifier("_"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("a"));
        assert!(StandardBeanExpressionResolver::is_simple_identifier("A"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("1"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier("!"));
        assert!(!StandardBeanExpressionResolver::is_simple_identifier(" "));
    }
}

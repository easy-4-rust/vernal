//! 正则匹配运算符节点。
//!
//! 对标 Spring `org.springframework.expression.spel.ast.OperatorMatches`：
//! `'foo' matches '[a-z]+'` 返回 boolean。
//!
//! # 正则缓存
//!
//! Spring 用 `ConcurrentLruCache<String, Pattern>` 缓存编译后的 Pattern。
//! vernal-expression 使用 `moka::sync::Cache`（与 Spring 等价的同步 LRU）：
//! - key: regex pattern 字符串
//! - value: 已编译的 `regex::Regex`
//!
//! 容量 256（与 Spring 默认对齐），大小由 cache 自身 TTL/LFU 策略控制。

use std::sync::OnceLock;

use moka::sync::Cache;

use super::spel_node::SpelNode;
use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::spel::spel_message::SpelMessage;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// 全局正则缓存容量（与 Spring OperatorMatches.MAX_PATTERN_CACHE_SIZE 对齐）。
const REGEX_CACHE_SIZE: u64 = 256;

/// 进程级正则缓存（lazy 初始化）。
fn regex_cache() -> &'static Cache<String, Result<regex::Regex, String>> {
    static CACHE: OnceLock<Cache<String, Result<regex::Regex, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Cache::new(REGEX_CACHE_SIZE))
}

/// 编译并缓存正则表达式（线程安全）。
///
/// 对标 Spring `Pattern.compile(String)` + `ConcurrentLruCache`。
fn compile_cached(pattern: &str) -> Result<regex::Regex, String> {
    if let Some(cached) = regex_cache().get(pattern) {
        return cached;
    }
    let compiled = regex::Regex::new(pattern).map_err(|e| e.to_string());
    regex_cache().insert(pattern.to_string(), compiled.clone());
    compiled
}

/// 正则匹配运算符节点。
pub struct OperatorMatches {
    value: Box<dyn SpelNode>,
    pattern: Box<dyn SpelNode>,
}

impl OperatorMatches {
    /// 创建正则匹配节点。
    ///
    /// # 参数
    ///
    /// - `value` — 待匹配的字符串表达式
    /// - `pattern` — 正则表达式字符串
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

        let (s, p) = match (value.value(), pattern.value()) {
            (ExpressionValue::String(s), ExpressionValue::String(p)) => (s.as_str(), p.as_str()),
            _ => {
                return Err(EvaluationException::new(
                    "",
                    Some(self.start_position() as i32),
                    SpelMessage::InvalidFirstOperandForMatchesOperator
                        .format_message(&[&format!("{:?}", value.value())]),
                ));
            }
        };

        // 编译/查找缓存的正则
        let re = compile_cached(p).map_err(|e| {
            EvaluationException::new(
                p,
                Some(self.start_position() as i32),
                SpelMessage::InvalidPattern.format_message(&[&e]),
            )
        })?;

        Ok(TypedValue::new(
            ExpressionValue::Boolean(re.is_match(s)),
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

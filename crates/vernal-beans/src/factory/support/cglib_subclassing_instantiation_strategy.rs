//! CglibSubclassingInstantiationStrategy — Spring 风格的 CGLIB 子类实例化策略。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.CglibSubclassingInstantiationStrategy`。
//!
//! 在 Spring 中，当 Bean 定义包含方法覆盖（`<lookup-method>` 或 `<replaced-method>`）时，
//! `SimpleInstantiationStrategy` 无法直接使用反射创建实例，
//! 需要通过 CGLIB 生成子类来拦截方法调用。
//!
//! ## 设计说明
//!
//! 在 vernal 中，由于 Rust 没有 CGLIB 或运行时字节码生成，
//! 此策略通过工厂闭包 + 方法替换器来实现等效功能。
//! 方法覆盖通过闭包委托模式实现，而非运行时代理。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::factory::support::instantiation_strategy::InstantiationStrategy;
use crate::factory::support::simple_instantiation_strategy::SimpleInstantiationStrategy;

/// 方法替换闭包类型。
///
/// 对应 Spring 的 `MethodReplacer` 接口。
/// 接收原始方法调用的参数，返回替换后的结果。
pub type MethodReplacerFn = Arc<dyn Fn(&[Arc<dyn Any + Send + Sync>]) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// CGLIB 子类实例化策略。
///
/// 对应 Spring 的 `CglibSubclassingInstantiationStrategy`。
///
/// 在 vernal 中通过工厂闭包 + 方法替换映射表实现。
/// 当存在方法覆盖时，使用委托闭包替代原始方法。
#[derive(Clone, Default)]
pub struct CglibSubclassingInstantiationStrategy {
    /// 已注册的方法替换器映射：bean_name -> (method_name -> replacer_fn)
    method_replacements: HashMap<String, HashMap<String, MethodReplacerFn>>,
}

impl CglibSubclassingInstantiationStrategy {
    /// 创建 CGLIB 子类实例化策略。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册方法替换器。
    ///
    /// 对应 Spring 的 `MethodOverride` 注册机制。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `method_name` — 要替换的方法名
    /// - `replacer` — 替换闭包
    pub fn register_method_replacement(
        &mut self,
        bean_name: &str,
        method_name: &str,
        replacer: MethodReplacerFn,
    ) {
        self.method_replacements
            .entry(bean_name.to_string())
            .or_default()
            .insert(method_name.to_string(), replacer);
    }

    /// 检查指定 Bean 是否有方法替换。
    pub fn has_method_replacement(&self, bean_name: &str, method_name: &str) -> bool {
        self.method_replacements
            .get(bean_name)
            .map(|m| m.contains_key(method_name))
            .unwrap_or(false)
    }

    /// 获取已注册的方法替换器数量。
    pub fn replacement_count(&self) -> usize {
        self.method_replacements.values().map(|m| m.len()).sum()
    }

    /// 执行方法替换。
    ///
    /// 如果 Bean 有注册的方法替换器，则使用替换器；
    /// 否则回退到 `SimpleInstantiationStrategy`。
    pub fn invoke_with_replacement(
        &self,
        bean_name: &str,
        method_name: &str,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(method_map) = self.method_replacements.get(bean_name) {
            if let Some(replacer) = method_map.get(method_name) {
                return Ok(replacer(args));
            }
        }
        Err(format!(
            "CglibSubclassingInstantiationStrategy: no method replacement for '{}::{}'",
            bean_name, method_name
        ).into())
    }
}

impl InstantiationStrategy for CglibSubclassingInstantiationStrategy {
    fn instantiate(
        &self,
        bean_class: &str,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 当没有工厂闭包时，使用默认的 SimpleInstantiationStrategy 行为
        SimpleInstantiationStrategy::new().instantiate(bean_class, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_strategy_has_no_replacements() {
        let strategy = CglibSubclassingInstantiationStrategy::new();
        assert_eq!(strategy.replacement_count(), 0);
        assert!(!strategy.has_method_replacement("myBean", "someMethod"));
    }

    #[test]
    fn register_and_invoke_method_replacement() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer: MethodReplacerFn = Arc::new(|args| {
            if let Some(first) = args.first() {
                if let Some(val) = first.downcast_ref::<i32>() {
                    return Arc::new(val * 2);
                }
            }
            Arc::new(0_i32)
        });

        strategy.register_method_replacement("myBean", "compute", replacer);
        assert!(strategy.has_method_replacement("myBean", "compute"));
        assert_eq!(strategy.replacement_count(), 1);

        let result = strategy.invoke_with_replacement("myBean", "compute", &[Arc::new(21_i32)]).unwrap();
        assert_eq!(result.downcast_ref::<i32>(), Some(&42));
    }

    #[test]
    fn invoke_without_replacement_returns_error() {
        let strategy = CglibSubclassingInstantiationStrategy::new();
        let result = strategy.invoke_with_replacement("unknown", "method", &[]);
        assert!(result.is_err());
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn default_trait_creates_empty_strategy() {
        let strategy = CglibSubclassingInstantiationStrategy::default();
        assert_eq!(strategy.replacement_count(), 0);
    }

    #[test]
    fn register_multiple_replacements() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer1: MethodReplacerFn = Arc::new(|_| Arc::new(1_i32));
        let replacer2: MethodReplacerFn = Arc::new(|_| Arc::new(2_i32));
        strategy.register_method_replacement("bean1", "method1", replacer1);
        strategy.register_method_replacement("bean1", "method2", replacer2);
        assert_eq!(strategy.replacement_count(), 2);
    }

    #[test]
    fn register_replacement_for_different_beans() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer: MethodReplacerFn = Arc::new(|_| Arc::new(42_i32));
        strategy.register_method_replacement("bean1", "method", replacer.clone());
        strategy.register_method_replacement("bean2", "method", replacer);
        assert_eq!(strategy.replacement_count(), 2);
        assert!(strategy.has_method_replacement("bean1", "method"));
        assert!(strategy.has_method_replacement("bean2", "method"));
    }

    #[test]
    fn invoke_with_empty_args() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer: MethodReplacerFn = Arc::new(|_| Arc::new("result".to_string()));
        strategy.register_method_replacement("bean", "method", replacer);
        let result = strategy.invoke_with_replacement("bean", "method", &[]).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "result");
    }

    #[test]
    fn has_method_replacement_false_for_wrong_method() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer: MethodReplacerFn = Arc::new(|_| Arc::new(0_i32));
        strategy.register_method_replacement("bean", "method1", replacer);
        assert!(!strategy.has_method_replacement("bean", "method2"));
    }

    #[test]
    fn invoke_wrong_method_returns_error() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer: MethodReplacerFn = Arc::new(|_| Arc::new(0_i32));
        strategy.register_method_replacement("bean", "method1", replacer);
        let result = strategy.invoke_with_replacement("bean", "method2", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn clone_strategy() {
        let mut strategy = CglibSubclassingInstantiationStrategy::new();
        let replacer: MethodReplacerFn = Arc::new(|_| Arc::new(42_i32));
        strategy.register_method_replacement("bean", "method", replacer);
        let cloned = strategy.clone();
        assert_eq!(cloned.replacement_count(), 1);
        assert!(cloned.has_method_replacement("bean", "method"));
    }

    #[test]
    fn instantiate_delegates_to_simple_strategy() {
        let strategy = CglibSubclassingInstantiationStrategy::new();
        // This will fail because "nonexistent::Type" isn't a real type,
        // but it exercises the instantiate method
        let result = strategy.instantiate("nonexistent::Type", &[]);
        assert!(result.is_err());
    }
}

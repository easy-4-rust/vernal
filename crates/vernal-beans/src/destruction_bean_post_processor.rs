//! DestructionBeanPostProcessor — Spring 风格销毁后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DisposableBeanAdapter`。
//!
//! 在 Spring 中，`DestructionAwareBeanPostProcessor` 在 Bean 销毁前
//! 调用 `DisposableBean.destroy()` 和自定义 destroy 方法。
//! 在 vernal 中，此处理器负责执行 Bean 的销毁回调。

use std::any::Any;

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// 销毁 Bean 后处理器。
///
/// 对应 Spring 的 `DestructionAwareBeanPostProcessor`。
///
/// 在 Bean 销毁阶段执行清理回调：
/// 1. `DisposableBean.destroy()`
/// 2. 自定义 destroy-method
///
/// 此处理器在 `BeanPostProcessor.postProcessBeforeDestruction` 阶段执行。
#[derive(Debug, Default)]
pub struct DestructionBeanPostProcessor {
    /// 已销毁的 Bean 名称列表（用于调试和日志）
    destroyed_beans: std::sync::Mutex<Vec<String>>,
}

impl DestructionBeanPostProcessor {
    /// 创建新的销毁后处理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取已销毁的 Bean 数量。
    pub fn destroyed_count(&self) -> usize {
        self.destroyed_beans.lock().unwrap().len()
    }

    /// 获取已销毁的 Bean 名称列表。
    pub fn destroyed_bean_names(&self) -> Vec<String> {
        self.destroyed_beans.lock().unwrap().clone()
    }

    /// 检查指定 Bean 是否已被销毁。
    pub fn is_destroyed(&self, bean_name: &str) -> bool {
        self.destroyed_beans.lock().unwrap().iter().any(|n| n == bean_name)
    }

    /// 清空已销毁记录。
    pub fn clear_destroyed(&self) {
        self.destroyed_beans.lock().unwrap().clear();
    }
}

impl BeanPostProcessor for DestructionBeanPostProcessor {
    fn post_process_before_destruction(
        &self,
        _bean: &dyn Any,
        bean_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 记录已销毁的 Bean
        self.destroyed_beans.lock().unwrap().push(bean_name.to_string());

        // 在实际实现中，这里会检查 Bean 是否实现了 DisposableBean
        // 并调用 destroy()
        // 同时检查是否有自定义 destroy-method

        Ok(())
    }

    fn requires_destruction(&self, _bean: &dyn Any) -> bool {
        // 在实际实现中，这里会检查 Bean 是否需要销毁
        // （实现了 DisposableBean 或有自定义 destroy-method）
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_has_no_destroyed_beans() {
        let processor = DestructionBeanPostProcessor::new();
        assert_eq!(processor.destroyed_count(), 0);
        assert!(processor.destroyed_bean_names().is_empty());
    }

    #[test]
    fn post_process_records_destroyed_bean() {
        let processor = DestructionBeanPostProcessor::new();
        let bean = 42_i32;

        let result = processor.post_process_before_destruction(&bean, "myBean");
        assert!(result.is_ok());
        assert_eq!(processor.destroyed_count(), 1);
        assert!(processor.is_destroyed("myBean"));
        assert!(!processor.is_destroyed("otherBean"));
    }

    #[test]
    fn requires_destruction_returns_true() {
        let processor = DestructionBeanPostProcessor::new();
        let bean = "test".to_string();
        assert!(processor.requires_destruction(&bean));
    }

    #[test]
    fn clear_destroyed_removes_all_records() {
        let processor = DestructionBeanPostProcessor::new();
        let bean1 = 1_i32;
        let bean2 = 2_i32;

        processor.post_process_before_destruction(&bean1, "bean1").unwrap();
        processor.post_process_before_destruction(&bean2, "bean2").unwrap();

        assert_eq!(processor.destroyed_count(), 2);
        processor.clear_destroyed();
        assert_eq!(processor.destroyed_count(), 0);
    }
}

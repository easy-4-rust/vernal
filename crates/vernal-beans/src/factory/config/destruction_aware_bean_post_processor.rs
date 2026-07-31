//! DestructionAwareBeanPostProcessor — 对应 Spring `org.springframework.beans.factory.config.DestructionAwareBeanPostProcessor`。
//!
//! 感知销毁的 BeanPostProcessor。

use std::any::Any;
use std::sync::Arc;

/// 感知销毁的 BeanPostProcessor。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.DestructionAwareBeanPostProcessor`。
///
/// 在 Bean 销毁前执行回调。
pub trait DestructionAwareBeanPostProcessor: Send + Sync {
    /// 在 Bean 销毁前执行。
    fn post_process_before_destruction(&self, bean: Arc<dyn Any + Send + Sync>, bean_name: &str) -> Result<(), Box<dyn std::error::Error + Send +Sync>>;

    /// 是否需要处理销毁。
    fn requires_destruction(&self, _bean: Arc<dyn Any + Send + Sync>) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProcessor;

    impl DestructionAwareBeanPostProcessor for TestProcessor {
        fn post_process_before_destruction(&self, _bean: Arc<dyn Any + Send + Sync>, _bean_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    #[test]
    fn test_processor() {
        let processor = TestProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new(String::from("test"));
        assert!(processor.post_process_before_destruction(bean, "test").is_ok());
    }
}

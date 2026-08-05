//! BeanRegistrationAotProcessor — Spring 风格的 Bean 注册 AOT 处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.aot.BeanRegistrationAotProcessor`。
//!
//! 在 AOT 阶段处理 Bean 定义，生成优化代码。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

/// AOT 贡献（trait）。
pub trait AotContribution: Send + Sync {
    /// 执行class_name操作。
    fn class_name(&self) -> &str;
    /// 执行method_name操作。
    fn method_name(&self) -> &str;
}

/// Spring 风格的 `BeanRegistrationAotProcessor`。
///
/// 对应 Spring 的 `BeanRegistrationAotProcessor`。
///
/// 在 AOT 编译阶段处理 Bean 定义，
/// 生成优化的 Bean 注册代码。
pub struct BeanRegistrationAotProcessor {
    processed: Mutex<HashMap<TypeId, String>>,
}

impl BeanRegistrationAotProcessor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            processed: Mutex::new(HashMap::new()),
        }
    }

    /// 处理输入并返回结果。
    pub fn process(&self, type_id: TypeId, class_name: String) -> Result<(), AotProcessingError> {
        let mut processed = self.processed.lock().unwrap();
        processed.insert(type_id, class_name);
        Ok(())
    }

    /// 获取processed数量。
    pub fn processed_count(&self) -> usize {
        self.processed.lock().unwrap().len()
    }

    /// 判断是否包含指定条目。
    pub fn contains(&self, type_id: TypeId) -> bool {
        self.processed.lock().unwrap().contains_key(&type_id)
    }

    /// 获取processed。
    pub fn get_processed(&self, type_id: TypeId) -> Option<String> {
        self.processed.lock().unwrap().get(&type_id).cloned()
    }

    /// 移除。
    pub fn clear(&self) {
        self.processed.lock().unwrap().clear();
    }
}

impl Default for BeanRegistrationAotProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// AOT 处理错误。
#[derive(Debug, Clone)]
pub struct AotProcessingError {
    message: String,
}

impl AotProcessingError {
    /// 创建一个新的实例。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
    /// 获取消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for AotProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AotProcessingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_has_no_processed() {
        let processor = BeanRegistrationAotProcessor::new();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn default_trait_creates_empty_processor() {
        let processor = BeanRegistrationAotProcessor::default();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn process_adds_entry() {
        let processor = BeanRegistrationAotProcessor::new();
        processor
            .process(TypeId::of::<String>(), "String".to_string())
            .unwrap();
        assert_eq!(processor.processed_count(), 1);
        assert!(processor.contains(TypeId::of::<String>()));
    }

    #[test]
    fn process_overwrites_existing() {
        let processor = BeanRegistrationAotProcessor::new();
        processor
            .process(TypeId::of::<String>(), "first".to_string())
            .unwrap();
        processor
            .process(TypeId::of::<String>(), "second".to_string())
            .unwrap();
        assert_eq!(processor.processed_count(), 1);
        assert_eq!(
            processor.get_processed(TypeId::of::<String>()).unwrap(),
            "second"
        );
    }

    #[test]
    fn get_processed_returns_none_for_missing() {
        let processor = BeanRegistrationAotProcessor::new();
        assert!(processor.get_processed(TypeId::of::<i32>()).is_none());
    }

    #[test]
    fn contains_returns_false_for_missing() {
        let processor = BeanRegistrationAotProcessor::new();
        assert!(!processor.contains(TypeId::of::<i32>()));
    }

    #[test]
    fn clear_removes_all() {
        let processor = BeanRegistrationAotProcessor::new();
        processor
            .process(TypeId::of::<String>(), "String".to_string())
            .unwrap();
        processor
            .process(TypeId::of::<i32>(), "i32".to_string())
            .unwrap();
        assert_eq!(processor.processed_count(), 2);
        processor.clear();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn process_multiple_types() {
        let processor = BeanRegistrationAotProcessor::new();
        processor
            .process(TypeId::of::<String>(), "String".to_string())
            .unwrap();
        processor
            .process(TypeId::of::<i32>(), "i32".to_string())
            .unwrap();
        processor
            .process(TypeId::of::<bool>(), "bool".to_string())
            .unwrap();
        assert_eq!(processor.processed_count(), 3);
        assert!(processor.contains(TypeId::of::<String>()));
        assert!(processor.contains(TypeId::of::<i32>()));
        assert!(processor.contains(TypeId::of::<bool>()));
    }

    #[test]
    fn aot_processing_error_display() {
        let err = AotProcessingError::new("test error");
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn aot_processing_error_message() {
        let err = AotProcessingError::new("test message");
        assert_eq!(err.message(), "test message");
    }

    #[test]
    fn aot_processing_error_is_std_error() {
        let err = AotProcessingError::new("error");
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn aot_processing_error_from_string() {
        let err = AotProcessingError::new(String::from("owned string"));
        assert_eq!(err.message(), "owned string");
    }

    #[test]
    fn aot_processing_error_clone() {
        let err = AotProcessingError::new("clone me");
        let err2 = err.clone();
        assert_eq!(err.message(), err2.message());
    }

    #[test]
    fn aot_processing_error_debug() {
        let err = AotProcessingError::new("debug");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("AotProcessingError"));
    }
}

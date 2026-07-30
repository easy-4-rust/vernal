//! BeanRegistrationAotProcessor — Spring 风格的 Bean 注册 AOT 处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.aot.BeanRegistrationAotProcessor`。
//!
//! 在 AOT 阶段处理 Bean 定义，生成优化代码。

use std::any::TypeId;
use std::sync::Mutex;
use std::collections::HashMap;

/// AOT 贡献（trait）。
pub trait AotContribution: Send + Sync {
    fn class_name(&self) -> &str;
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
    pub fn new() -> Self {
        Self {
            processed: Mutex::new(HashMap::new()),
        }
    }

    pub fn process(&self, type_id: TypeId, class_name: String) -> Result<(), AotProcessingError> {
        let mut processed = self.processed.lock().unwrap();
        processed.insert(type_id, class_name);
        Ok(())
    }

    pub fn processed_count(&self) -> usize {
        self.processed.lock().unwrap().len()
    }

    pub fn contains(&self, type_id: TypeId) -> bool {
        self.processed.lock().unwrap().contains_key(&type_id)
    }

    pub fn get_processed(&self, type_id: TypeId) -> Option<String> {
        self.processed.lock().unwrap().get(&type_id).cloned()
    }

    pub fn clear(&self) {
        self.processed.lock().unwrap().clear();
    }
}

impl Default for BeanRegistrationAotProcessor {
    fn default() -> Self { Self::new() }
}

/// AOT 处理错误。
#[derive(Debug, Clone)]
pub struct AotProcessingError {
    message: String,
}

impl AotProcessingError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl std::fmt::Display for AotProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AotProcessingError {}

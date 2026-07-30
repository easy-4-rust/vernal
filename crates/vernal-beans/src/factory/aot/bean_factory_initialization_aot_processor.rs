//! BeanFactoryInitializationAotProcessor — Spring 风格的工厂初始化 AOT 处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.aot.BeanFactoryInitializationAotProcessor`。
//!
//! 在 Bean 工厂初始化之前执行 AOT 处理。

use std::sync::Mutex;
use std::collections::HashSet;

/// Spring 风格的 `BeanFactoryInitializationAotProcessor`。
///
/// 对应 Spring 的 `BeanFactoryInitializationAotProcessor`。
///
/// 在 Bean 工厂初始化阶段执行 AOT 处理逻辑。
pub struct BeanFactoryInitializationAotProcessor {
    initialized: Mutex<bool>,
    processed_steps: Mutex<HashSet<String>>,
}

impl BeanFactoryInitializationAotProcessor {
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
            processed_steps: Mutex::new(HashSet::new()),
        }
    }

    pub fn process_step(&self, step_name: &str) {
        self.processed_steps.lock().unwrap().insert(step_name.to_string());
    }

    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    pub fn mark_initialized(&self) {
        *self.initialized.lock().unwrap() = true;
    }

    pub fn step_count(&self) -> usize {
        self.processed_steps.lock().unwrap().len()
    }

    pub fn has_step(&self, name: &str) -> bool {
        self.processed_steps.lock().unwrap().contains(name)
    }

    pub fn clear_steps(&self) {
        self.processed_steps.lock().unwrap().clear();
    }
}

impl Default for BeanFactoryInitializationAotProcessor {
    fn default() -> Self { Self::new() }
}

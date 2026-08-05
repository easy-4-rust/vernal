//! BeanFactoryInitializationAotProcessor — Spring 风格的工厂初始化 AOT 处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.aot.BeanFactoryInitializationAotProcessor`。
//!
//! 在 Bean 工厂初始化之前执行 AOT 处理。

use std::collections::HashSet;
use std::sync::Mutex;

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
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            initialized: Mutex::new(false),
            processed_steps: Mutex::new(HashSet::new()),
        }
    }

    /// 执行process_step操作。
    pub fn process_step(&self, step_name: &str) {
        self.processed_steps
            .lock()
            .unwrap()
            .insert(step_name.to_string());
    }

    /// 判断是否initialized。
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    /// 执行mark_initialized操作。
    pub fn mark_initialized(&self) {
        *self.initialized.lock().unwrap() = true;
    }

    /// 获取step数量。
    pub fn step_count(&self) -> usize {
        self.processed_steps.lock().unwrap().len()
    }

    /// 判断是否step。
    pub fn has_step(&self, name: &str) -> bool {
        self.processed_steps.lock().unwrap().contains(name)
    }

    /// 移除steps。
    pub fn clear_steps(&self) {
        self.processed_steps.lock().unwrap().clear();
    }
}

impl Default for BeanFactoryInitializationAotProcessor {
    fn default() -> Self {
        Self::new()
    }
}

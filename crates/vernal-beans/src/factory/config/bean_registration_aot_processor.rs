//! BeanRegistrationAotProcessor — 对应 Spring `org.springframework.beans.factory.config.BeanRegistrationAotProcessor`。
//!
//! Bean 注册 AOT 处理器接口。
//!
//! 在 Spring 6+ 中，AOT（Ahead-of-Time）处理是 GraalVM 原生镜像编译的关键环节。
//! `BeanRegistrationAotProcessor` 是一个回调接口，允许在 AOT 阶段
//! 对 Bean 注册进行自定义处理，生成优化的注册代码。
//!
//! # Spring 对标
//!
//! 对应 Java 接口：`org.springframework.beans.factory.config.BeanRegistrationAotProcessor`。
//!
//! # 设计说明
//!
//! 在 Rust 中，此接口通过 trait 实现，允许用户提供自定义的 AOT 处理逻辑。
//! 处理器可以在编译阶段分析 Bean 定义并生成优化代码。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

/// Bean 注册 AOT 处理器 trait。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.BeanRegistrationAotProcessor`。
///
/// 在 AOT 阶段处理 Bean 注册，生成优化的注册代码。
pub trait BeanRegistrationAotProcessor: Send + Sync {
    /// 处理 Bean 注册。
    ///
    /// # Arguments
    ///
    /// * `bean_type` - Bean 的类型 ID
    /// * `bean_name` - Bean 的名称
    ///
    /// # Returns
    ///
    /// 处理结果，成功返回 `Ok(())`，失败返回错误描述。
    fn process_bean_registration(
        &self,
        bean_type: TypeId,
        bean_name: &str,
    ) -> Result<(), String>;

    /// 判断是否支持处理指定类型的 Bean。
    ///
    /// # Arguments
    ///
    /// * `bean_type` - Bean 的类型 ID
    ///
    /// # Returns
    ///
    /// 如果支持处理返回 `true`，否则返回 `false`。
    fn supports(&self, bean_type: TypeId) -> bool;
}

/// 默认的 Bean 注册 AOT 处理器实现。
///
/// 提供基本的 Bean 注册 AOT 处理功能。
pub struct DefaultBeanRegistrationAotProcessor {
    processed: Mutex<HashMap<String, TypeId>>,
}

impl DefaultBeanRegistrationAotProcessor {
    /// 创建一个新的 DefaultBeanRegistrationAotProcessor。
    pub fn new() -> Self {
        Self {
            processed: Mutex::new(HashMap::new()),
        }
    }

    /// 获取已处理的 Bean 数量。
    pub fn processed_count(&self) -> usize {
        self.processed.lock().unwrap().len()
    }

    /// 判断指定 Bean 名称是否已被处理。
    pub fn is_processed(&self, bean_name: &str) -> bool {
        self.processed.lock().unwrap().contains_key(bean_name)
    }

    /// 清空已处理记录。
    pub fn clear(&self) {
        self.processed.lock().unwrap().clear();
    }

    /// 获取所有已处理的 Bean 名称。
    pub fn processed_bean_names(&self) -> Vec<String> {
        self.processed.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for DefaultBeanRegistrationAotProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanRegistrationAotProcessor for DefaultBeanRegistrationAotProcessor {
    fn process_bean_registration(
        &self,
        bean_type: TypeId,
        bean_name: &str,
    ) -> Result<(), String> {
        let mut processed = self.processed.lock().unwrap();
        processed.insert(bean_name.to_string(), bean_type);
        Ok(())
    }

    fn supports(&self, _bean_type: TypeId) -> bool {
        // 默认支持所有类型
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_processor_new() {
        let processor = DefaultBeanRegistrationAotProcessor::new();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn test_process_bean_registration() {
        let processor = DefaultBeanRegistrationAotProcessor::new();
        let result =
            processor.process_bean_registration(TypeId::of::<String>(), "myString");
        assert!(result.is_ok());
        assert_eq!(processor.processed_count(), 1);
        assert!(processor.is_processed("myString"));
    }

    #[test]
    fn test_supports() {
        let processor = DefaultBeanRegistrationAotProcessor::new();
        assert!(processor.supports(TypeId::of::<i32>()));
        assert!(processor.supports(TypeId::of::<String>()));
    }

    #[test]
    fn test_clear() {
        let processor = DefaultBeanRegistrationAotProcessor::new();
        processor
            .process_bean_registration(TypeId::of::<i32>(), "bean1")
            .unwrap();
        processor
            .process_bean_registration(TypeId::of::<String>(), "bean2")
            .unwrap();
        assert_eq!(processor.processed_count(), 2);

        processor.clear();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn test_processed_bean_names() {
        let processor = DefaultBeanRegistrationAotProcessor::new();
        processor
            .process_bean_registration(TypeId::of::<i32>(), "alpha")
            .unwrap();
        processor
            .process_bean_registration(TypeId::of::<String>(), "beta")
            .unwrap();

        let mut names = processor.processed_bean_names();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }
}

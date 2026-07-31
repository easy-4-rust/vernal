//! BeanDefinitionReaderUtils — Bean 定义读取器工具。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionReaderUtils`。
//!
//! 在 Spring 中，此工具类提供了 Bean 定义读取相关的静态工具方法，
//! 如生成唯一 Bean 名称、验证 Bean 定义等。
//!
//! ## 设计说明
//!
//! Spring 的 `BeanDefinitionReaderUtils` 是纯静态工具类，
//! 在 vernal 中改为实例方法，因为需要维护计数器状态。

use std::collections::HashMap;
use std::sync::Mutex;

/// Bean 定义读取器工具。
///
/// 对应 Spring 的 `BeanDefinitionReaderUtils`。
///
/// 提供 Bean 定义读取过程中的工具方法，包括：
/// - 唯一 Bean 名称生成
/// - Bean 名称验证
/// - 计数器管理
pub struct BeanDefinitionReaderUtils {
    /// 名称计数器（prefix -> count）
    name_counters: Mutex<HashMap<String, u32>>,
    /// 已生成的名称记录
    generated_names: Mutex<Vec<String>>,
}

impl BeanDefinitionReaderUtils {
    /// 创建新的工具实例。
    pub fn new() -> Self {
        Self {
            name_counters: Mutex::new(HashMap::new()),
            generated_names: Mutex::new(Vec::new()),
        }
    }

    /// 生成唯一的 Bean 名称。
    ///
    /// 对应 Spring 的 `generateBeanName(BeanDefinition, BeanDefinitionRegistry, boolean)`。
    ///
    /// 使用格式 `prefix#count` 生成唯一名称。
    ///
    /// # 参数
    /// - `prefix` — 名称前缀（通常是类名）
    ///
    /// # 返回
    /// 唯一的 Bean 名称
    pub fn generate_bean_name(&self, prefix: &str) -> String {
        let mut counters = self.name_counters.lock().unwrap();
        let count = counters.entry(prefix.to_string()).or_insert(0);
        *count += 1;
        let name = format!("{}#{}", prefix, count);
        self.generated_names.lock().unwrap().push(name.clone());
        name
    }

    /// 重置指定前缀的计数器。
    pub fn reset_counter(&self, prefix: &str) {
        self.name_counters.lock().unwrap().remove(prefix);
    }

    /// 重置所有计数器。
    pub fn reset_all_counters(&self) {
        self.name_counters.lock().unwrap().clear();
    }

    /// 获取指定前缀的当前计数。
    pub fn counter_value(&self, prefix: &str) -> u32 {
        self.name_counters.lock().unwrap()
            .get(prefix)
            .copied()
            .unwrap_or(0)
    }

    /// 获取已生成的名称总数。
    pub fn generated_name_count(&self) -> usize {
        self.generated_names.lock().unwrap().len()
    }

    /// 获取所有已生成的名称。
    pub fn generated_names(&self) -> Vec<String> {
        self.generated_names.lock().unwrap().clone()
    }

    /// 验证 Bean 名称是否合法。
    ///
    /// 对应 Spring 的名称验证逻辑：不能为空、不能包含特殊字符。
    pub fn validate_bean_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Bean name must not be empty".to_string());
        }
        if name.contains(' ') {
            return Err("Bean name must not contain spaces".to_string());
        }
        Ok(())
    }
}

impl Default for BeanDefinitionReaderUtils {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_unique_bean_names() {
        let utils = BeanDefinitionReaderUtils::new();
        let name1 = utils.generate_bean_name("MyService");
        let name2 = utils.generate_bean_name("MyService");
        let name3 = utils.generate_bean_name("MyService");

        assert_eq!(name1, "MyService#1");
        assert_eq!(name2, "MyService#2");
        assert_eq!(name3, "MyService#3");
        assert_eq!(utils.generated_name_count(), 3);
    }

    #[test]
    fn different_prefixes_have_independent_counters() {
        let utils = BeanDefinitionReaderUtils::new();
        utils.generate_bean_name("Service");
        utils.generate_bean_name("Controller");
        utils.generate_bean_name("Service");

        assert_eq!(utils.counter_value("Service"), 2);
        assert_eq!(utils.counter_value("Controller"), 1);
    }

    #[test]
    fn reset_counter() {
        let utils = BeanDefinitionReaderUtils::new();
        utils.generate_bean_name("test");
        utils.generate_bean_name("test");
        assert_eq!(utils.counter_value("test"), 2);

        utils.reset_counter("test");
        assert_eq!(utils.counter_value("test"), 0);
    }

    #[test]
    fn validate_bean_name() {
        assert!(BeanDefinitionReaderUtils::validate_bean_name("myBean").is_ok());
        assert!(BeanDefinitionReaderUtils::validate_bean_name("").is_err());
        assert!(BeanDefinitionReaderUtils::validate_bean_name("my bean").is_err());
    }

    #[test]
    fn generated_names_recorded() {
        let utils = BeanDefinitionReaderUtils::new();
        utils.generate_bean_name("A");
        utils.generate_bean_name("B");
        let names = utils.generated_names();
        assert_eq!(names, vec!["A#1", "B#1"]);
    }
}

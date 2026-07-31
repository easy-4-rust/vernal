//! BeanFactoryAnnotationUtils — Bean 工厂注解工具。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.BeanFactoryAnnotationUtils`。
//!
//! 在 Spring 中，此类提供了与 Bean 工厂注解相关的工具方法，
//! 如查找带有特定限定符的 Bean、验证注解配置等。
//!
//! ## 设计说明
//!
//! Spring 的 `BeanFactoryAnnotationUtils` 是纯静态工具类，
//! 在 vernal 中改为实例方法以维护状态。

use std::collections::HashSet;
use std::sync::Mutex;

/// Bean 工厂注解工具。
///
/// 对应 Spring 的 `BeanFactoryAnnotationUtils`。
///
/// 提供注解处理过程中的工具方法。
pub struct BeanFactoryAnnotationUtils {
    /// 已处理的注解集合
    processed_annotations: Mutex<HashSet<String>>,
    /// 已验证的 Bean 名称
    validated_beans: Mutex<HashSet<String>>,
}

impl BeanFactoryAnnotationUtils {
    /// 创建新的工具实例。
    pub fn new() -> Self {
        Self {
            processed_annotations: Mutex::new(HashSet::new()),
            validated_beans: Mutex::new(HashSet::new()),
        }
    }

    /// 标记注解已处理。
    ///
    /// 用于防止重复处理同一注解。
    pub fn mark_processed(&self, name: String) {
        self.processed_annotations.lock().unwrap().insert(name);
    }

    /// 检查注解是否已处理。
    pub fn is_processed(&self, name: &str) -> bool {
        self.processed_annotations.lock().unwrap().contains(name)
    }

    /// 已处理的注解数量。
    pub fn processed_count(&self) -> usize {
        self.processed_annotations.lock().unwrap().len()
    }

    /// 标记 Bean 已验证。
    pub fn mark_validated(&self, bean_name: String) {
        self.validated_beans.lock().unwrap().insert(bean_name);
    }

    /// 检查 Bean 是否已验证。
    pub fn is_validated(&self, bean_name: &str) -> bool {
        self.validated_beans.lock().unwrap().contains(bean_name)
    }

    /// 已验证的 Bean 数量。
    pub fn validated_count(&self) -> usize {
        self.validated_beans.lock().unwrap().len()
    }

    /// 清空所有状态。
    pub fn clear(&self) {
        self.processed_annotations.lock().unwrap().clear();
        self.validated_beans.lock().unwrap().clear();
    }
}

impl Default for BeanFactoryAnnotationUtils {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_utils_is_empty() {
        let utils = BeanFactoryAnnotationUtils::new();
        assert_eq!(utils.processed_count(), 0);
        assert_eq!(utils.validated_count(), 0);
    }

    #[test]
    fn mark_and_check_processed() {
        let utils = BeanFactoryAnnotationUtils::new();
        utils.mark_processed("@Autowired".to_string());
        utils.mark_processed("@Qualifier".to_string());

        assert!(utils.is_processed("@Autowired"));
        assert!(utils.is_processed("@Qualifier"));
        assert!(!utils.is_processed("@Inject"));
        assert_eq!(utils.processed_count(), 2);
    }

    #[test]
    fn mark_and_check_validated() {
        let utils = BeanFactoryAnnotationUtils::new();
        utils.mark_validated("myService".to_string());

        assert!(utils.is_validated("myService"));
        assert!(!utils.is_validated("otherService"));
        assert_eq!(utils.validated_count(), 1);
    }

    #[test]
    fn clear_removes_all() {
        let utils = BeanFactoryAnnotationUtils::new();
        utils.mark_processed("test".to_string());
        utils.mark_validated("bean".to_string());

        utils.clear();
        assert_eq!(utils.processed_count(), 0);
        assert_eq!(utils.validated_count(), 0);
    }
}

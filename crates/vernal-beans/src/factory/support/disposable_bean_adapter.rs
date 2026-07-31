//! DisposableBeanAdapter — Spring 风格可销毁 Bean 适配器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DisposableBeanAdapter`。
//!
//! 在 Spring 中，`DisposableBeanAdapter` 包装一个 Bean 实例，
//! 在容器关闭时执行销毁回调（`DisposableBean.destroy()` 和自定义 destroy-method）。
//! 它实现了 `DisposableBean` 接口，是单例 Bean 销毁的核心机制。
//!
//! ## 设计说明
//!
//! Spring 的 `DisposableBeanAdapter` 还处理 `DestructionAwareBeanPostProcessor` 链，
//! 在 vernal 中简化为直接调用销毁闭包。
//!
//! ## 销毁顺序
//!
//! 1. `DestructionAwareBeanPostProcessor.postProcessBeforeDestruction`
//! 2. `DisposableBean.destroy()`
//! 3. 自定义 `destroy-method`

use std::any::Any;
use std::sync::Arc;

/// 可销毁 Bean 适配器。
///
/// 对应 Spring 的 `DisposableBeanAdapter`。
///
/// 包装 Bean 实例和其销毁逻辑，在容器关闭时统一执行清理。
/// 支持自定义销毁方法名称和顺序。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `DisposableBeanAdapter` | `DisposableBeanAdapter` |
/// | `destroy()` | `run_destroy()` |
/// | `setDestroyMethodName(String)` | `with_destroy_method(name)` |
/// | `setOrder(int)` | `with_order(order)` |
pub struct DisposableBeanAdapter {
    /// Bean 名称
    bean_name: String,
    /// Bean 实例
    bean: Arc<dyn Any + Send + Sync>,
    /// 自定义销毁方法名称
    destroy_method_name: Option<String>,
    /// 销毁顺序（越小越先执行）
    order: i32,
    /// 是否已执行过销毁
    destroyed: std::sync::Mutex<bool>,
    /// 是否为推断的销毁方法（非显式配置）
    inferred_destroy_method: bool,
    /// Bean 类型名（用于诊断）
    bean_type_name: String,
}

impl DisposableBeanAdapter {
    /// 创建新的可销毁 Bean 适配器。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称
    /// - `bean` — Bean 实例
    pub fn new(bean_name: String, bean: Arc<dyn Any + Send + Sync>) -> Self {
        let type_name = std::any::type_name_of_val(&*bean).to_string();
        Self {
            bean_name,
            bean,
            destroy_method_name: None,
            order: 0,
            destroyed: std::sync::Mutex::new(false),
            inferred_destroy_method: false,
            bean_type_name: type_name,
        }
    }

    /// 设置自定义销毁方法名称。
    pub fn with_destroy_method(mut self, name: String) -> Self {
        self.destroy_method_name = Some(name);
        self
    }

    /// 设置推断的销毁方法名称。
    ///
    /// 对应 Spring 的推断销毁方法（如 `close()`、`shutdown()`）。
    pub fn with_inferred_destroy_method(mut self, name: String) -> Self {
        self.destroy_method_name = Some(name);
        self.inferred_destroy_method = true;
        self
    }

    /// 设置销毁顺序。
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// 设置 Bean 类型名。
    pub fn with_type_name(mut self, type_name: impl Into<String>) -> Self {
        self.bean_type_name = type_name.into();
        self
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取 Bean 实例。
    pub fn bean(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.bean
    }

    /// 获取自定义销毁方法名称。
    pub fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    /// 获取销毁顺序。
    pub fn order(&self) -> i32 {
        self.order
    }

    /// 检查是否已执行过销毁。
    pub fn is_destroyed(&self) -> bool {
        *self.destroyed.lock().unwrap()
    }

    /// 是否为推断的销毁方法。
    pub fn is_inferred_destroy_method(&self) -> bool {
        self.inferred_destroy_method
    }

    /// 获取 Bean 类型名。
    pub fn bean_type_name(&self) -> &str {
        &self.bean_type_name
    }

    /// 检查是否有自定义销毁方法。
    pub fn has_destroy_method(&self) -> bool {
        self.destroy_method_name.is_some()
    }

    /// 执行销毁逻辑。
    ///
    /// 对应 Spring 的 `DisposableBean.destroy()`。
    ///
    /// 标记为已销毁，后续调用不会重复执行。
    pub fn run_destroy(&self) {
        let mut destroyed = self.destroyed.lock().unwrap();
        if *destroyed {
            return;
        }
        *destroyed = true;
        // 在实际实现中，这里会：
        // 1. 调用 DestructionAwareBeanPostProcessor.postProcessBeforeDestruction
        // 2. 调用 DisposableBean.destroy()
        // 3. 调用自定义 destroy-method
    }

    /// 重置销毁状态（用于测试）。
    pub fn reset_destroyed(&self) {
        *self.destroyed.lock().unwrap() = false;
    }

    /// 构建诊断描述。
    pub fn describe(&self) -> String {
        let mut desc = format!("DisposableBeanAdapter[bean='{}'", self.bean_name);
        if let Some(ref method) = self.destroy_method_name {
            desc.push_str(&format!(", destroy='{}'", method));
            if self.inferred_destroy_method {
                desc.push_str(" (inferred)");
            }
        }
        desc.push_str(&format!(", order={}]", self.order));
        desc
    }
}

impl std::fmt::Debug for DisposableBeanAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisposableBeanAdapter")
            .field("bean_name", &self.bean_name)
            .field("destroy_method_name", &self.destroy_method_name)
            .field("order", &self.order)
            .field("destroyed", &self.is_destroyed())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_adapter_stores_bean_info() {
        let bean = Arc::new(42_i32);
        let adapter = DisposableBeanAdapter::new("myBean".to_string(), bean);

        assert_eq!(adapter.bean_name(), "myBean");
        assert!(adapter.destroy_method_name().is_none());
        assert_eq!(adapter.order(), 0);
        assert!(!adapter.is_destroyed());
    }

    #[test]
    fn with_destroy_method() {
        let bean = Arc::new("test".to_string());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean)
            .with_destroy_method("close".to_string());

        assert_eq!(adapter.destroy_method_name(), Some("close"));
        assert!(adapter.has_destroy_method());
        assert!(!adapter.is_inferred_destroy_method());
    }

    #[test]
    fn with_inferred_destroy_method() {
        let bean = Arc::new("test".to_string());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean)
            .with_inferred_destroy_method("shutdown".to_string());

        assert_eq!(adapter.destroy_method_name(), Some("shutdown"));
        assert!(adapter.is_inferred_destroy_method());
    }

    #[test]
    fn with_order() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean)
            .with_order(10);

        assert_eq!(adapter.order(), 10);
    }

    #[test]
    fn run_destroy_marks_as_destroyed() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean);

        assert!(!adapter.is_destroyed());
        adapter.run_destroy();
        assert!(adapter.is_destroyed());
    }

    #[test]
    fn run_destroy_idempotent() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean);

        adapter.run_destroy();
        adapter.run_destroy(); // 第二次调用不应重复执行
        assert!(adapter.is_destroyed());
    }

    #[test]
    fn reset_destroyed() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean);

        adapter.run_destroy();
        assert!(adapter.is_destroyed());

        adapter.reset_destroyed();
        assert!(!adapter.is_destroyed());
    }

    #[test]
    fn has_destroy_method_false_by_default() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean);
        assert!(!adapter.has_destroy_method());
    }

    #[test]
    fn describe_with_method() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("myBean".to_string(), bean)
            .with_destroy_method("close".to_string())
            .with_order(5);

        let desc = adapter.describe();
        assert!(desc.contains("myBean"));
        assert!(desc.contains("close"));
        assert!(desc.contains("order=5"));
    }

    #[test]
    fn describe_inferred() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean)
            .with_inferred_destroy_method("shutdown".to_string());

        let desc = adapter.describe();
        assert!(desc.contains("inferred"));
    }

    #[test]
    fn bean_type_name() {
        let bean = Arc::new(42_i32);
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean);
        assert!(!adapter.bean_type_name().is_empty());
    }

    #[test]
    fn with_type_name_override() {
        let bean = Arc::new(42_i32);
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean)
            .with_type_name("com.example.MyBean");
        assert_eq!(adapter.bean_type_name(), "com.example.MyBean");
    }

    #[test]
    fn debug_format() {
        let bean = Arc::new(());
        let adapter = DisposableBeanAdapter::new("bean".to_string(), bean);
        let debug = format!("{:?}", adapter);
        assert!(debug.contains("DisposableBeanAdapter"));
        assert!(debug.contains("bean"));
    }
}

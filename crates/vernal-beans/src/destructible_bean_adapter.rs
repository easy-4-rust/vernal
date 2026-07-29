//! DisposableBeanAdapter — Spring 风格的可销毁 Bean 适配器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DisposableBeanAdapter`。
//!
//! 包装 `DisposableBean` 实例，提供统一的销毁接口。
//! 用于容器管理 Bean 的生命周期，确保在容器关闭时正确销毁所有可销毁 Bean。

use std::fmt;
use std::sync::Arc;

use crate::disposable_bean::DisposableBean;

/// Spring 风格的可销毁 Bean 适配器。
///
/// 对应 Spring 的 `DisposableBeanAdapter`。
///
/// 包装 `DisposableBean` 实例，为容器提供统一的 Bean 销毁能力。
/// 当容器关闭时，遍历所有注册的适配器并调用 `destroy()` 方法。
///
/// ## 使用场景
///
/// - 容器关闭时清理资源
/// - 管理实现了 `DisposableBean` 的 Bean
/// - 配合 `destroy-method` 使用
pub struct DisposableBeanAdapter {
    /// 内部持有的可销毁 Bean。
    bean: Arc<dyn DisposableBean>,
    /// Bean 在容器中的名称。
    bean_name: String,
    /// 可选的销毁方法名（来自 Bean 定义的 destroy-method）。
    destroy_method_name: Option<String>,
}

impl fmt::Debug for DisposableBeanAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisposableBeanAdapter")
            .field("bean_name", &self.bean_name)
            .field("destroy_method_name", &self.destroy_method_name)
            .finish()
    }
}

impl DisposableBeanAdapter {
    /// 创建新的 DisposableBeanAdapter。
    ///
    /// # 参数
    ///
    /// * `bean` — 实现了 `DisposableBean` 的 Bean 实例
    /// * `bean_name` — Bean 在容器中的名称
    pub fn new(bean: Arc<dyn DisposableBean>, bean_name: impl Into<String>) -> Self {
        Self {
            bean,
            bean_name: bean_name.into(),
            destroy_method_name: None,
        }
    }

    /// 创建带销毁方法名的 DisposableBeanAdapter。
    ///
    /// # 参数
    ///
    /// * `bean` — 实现了 `DisposableBean` 的 Bean 实例
    /// * `bean_name` — Bean 在容器中的名称
    /// * `destroy_method_name` — 自定义销毁方法名
    pub fn with_destroy_method(
        bean: Arc<dyn DisposableBean>,
        bean_name: impl Into<String>,
        destroy_method_name: impl Into<String>,
    ) -> Self {
        Self {
            bean,
            bean_name: bean_name.into(),
            destroy_method_name: Some(destroy_method_name.into()),
        }
    }

    /// 获取内部持有的 DisposableBean 引用。
    pub fn bean(&self) -> &Arc<dyn DisposableBean> {
        &self.bean
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取自定义销毁方法名。
    pub fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    /// 销毁 Bean。
    ///
    /// 对应 Spring 的 `DisposableBeanAdapter.destroy()`。
    ///
    /// 调用内部 DisposableBean 的 `destroy()` 方法。如果 Bean 定义了
    /// 自定义 destroy-method，会在调用 `destroy()` 后尝试调用该方法。
    ///
    /// # 返回
    ///
    /// - `Ok(())` — 销毁成功
    /// - `Err` — 销毁过程中发生错误
    pub fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 调用 DisposableBean::destroy()
        self.bean.destroy()?;

        // 如果有自定义 destroy-method 且与 destroy() 不同，此处可调用
        // （当前实现简化处理，仅调用 destroy()）
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aware::Aware;

    #[derive(Debug)]
    struct TestDisposableBean {
        destroyed: std::sync::Mutex<bool>,
    }

    impl TestDisposableBean {
        fn new() -> Self {
            Self {
                destroyed: std::sync::Mutex::new(false),
            }
        }

        fn is_destroyed(&self) -> bool {
            *self.destroyed.lock().unwrap()
        }
    }

    impl Aware for TestDisposableBean {}

    impl DisposableBean for TestDisposableBean {
        fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut destroyed = self.destroyed.lock().unwrap();
            *destroyed = true;
            Ok(())
        }
    }

    #[test]
    fn test_new_adapter() {
        let bean = Arc::new(TestDisposableBean::new());
        let adapter = DisposableBeanAdapter::new(bean.clone(), "testBean");
        assert_eq!(adapter.bean_name(), "testBean");
        assert!(adapter.destroy_method_name().is_none());
    }

    #[test]
    fn test_with_destroy_method() {
        let bean = Arc::new(TestDisposableBean::new());
        let adapter =
            DisposableBeanAdapter::with_destroy_method(bean.clone(), "testBean", "cleanup");
        assert_eq!(adapter.destroy_method_name(), Some("cleanup"));
    }

    #[test]
    fn test_destroy() {
        let bean = Arc::new(TestDisposableBean::new());
        let adapter = DisposableBeanAdapter::new(bean.clone(), "testBean");
        assert!(!bean.is_destroyed());
        adapter.destroy().unwrap();
        assert!(bean.is_destroyed());
    }
}

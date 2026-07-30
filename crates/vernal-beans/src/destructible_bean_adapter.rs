//! DisposableBeanAdapter — Spring 风格的可销毁 Bean 适配器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.disposable.DisposableBeanAdapter`。
//!
//! 包装 Bean 的销毁逻辑，支持 `DisposableBean` 接口和自定义销毁方法。
//! 容器关闭时按逆拓扑序调用各 Bean 的销毁逻辑。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的可销毁 Bean 适配器。
///
/// 对应 Spring 的 `DisposableBeanAdapter`。
///
/// 封装 Bean 的销毁流程：
/// 1. 调用 `DestructionAwareBeanPostProcessor.postProcessBeforeDestruction`
/// 2. 调用 `DisposableBean.destroy()`
/// 3. 调用自定义 `destroy-method`
///
/// 在 vernal 中，由于 Rust 没有反射，自定义销毁方法通过
/// 可选的销毁闭包实现。
pub struct DisposableBeanAdapter {
    /// Bean 名称。
    bean_name: String,
    /// Bean 实例。
    bean: Arc<dyn Any + Send + Sync>,
    /// 自定义销毁方法名（仅用于诊断/日志）。
    destroy_method_name: Option<String>,
    /// 自定义销毁闭包（如果有）。
    destroy_callback: Option<Arc<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>>,
}

impl DisposableBeanAdapter {
    /// 创建可销毁 Bean 适配器。
    ///
    /// # 参数
    ///
    /// - `bean_name` — Bean 名称
    /// - `bean` — Bean 实例
    /// - `destroy_method_name` — 自定义销毁方法名（用于日志）
    pub fn new(
        bean_name: impl Into<String>,
        bean: Arc<dyn Any + Send + Sync>,
        destroy_method_name: Option<String>,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            bean,
            destroy_method_name,
            destroy_callback: None,
        }
    }

    /// 创建带自定义销毁闭包的适配器。
    ///
    /// # 参数
    ///
    /// - `bean_name` — Bean 名称
    /// - `bean` — Bean 实例
    /// - `destroy_method_name` — 自定义销毁方法名
    /// - `destroy_callback` — 自定义销毁闭包
    pub fn with_callback(
        bean_name: impl Into<String>,
        bean: Arc<dyn Any + Send + Sync>,
        destroy_method_name: Option<String>,
        destroy_callback: Arc<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            bean,
            destroy_method_name,
            destroy_callback: Some(destroy_callback),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取 Bean 实例的引用。
    pub fn bean(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.bean
    }

    /// 获取自定义销毁方法名。
    pub fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    /// 执行销毁。
    ///
    /// 按以下顺序尝试销毁：
    ///
    /// 1. 如果有自定义销毁闭包，调用它
    /// 2. 如果 Bean 实现了 `DisposableBean` trait，调用其 `destroy()` 方法
    /// 3. 否则不做任何操作（静态 Bean 无需销毁）
    ///
    /// # 错误
    ///
    /// 任何销毁步骤失败时返回错误。
    pub fn destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 自定义销毁闭包
        if let Some(callback) = &self.destroy_callback {
            callback().map_err(|e| {
                format!(
                    "Bean '{}' 自定义销毁方法 '{}' 执行失败: {}",
                    self.bean_name,
                    self.destroy_method_name.as_deref().unwrap_or("<closure>"),
                    e
                )
            })?;
        }

        // 2. 尝试调用 DisposableBean.destroy()
        //    由于 Rust 的 trait object 限制（Any 不继承 DisposableBean），
        //    Arc<dyn Any + Send + Sync> 无法直接 downcast 到 dyn DisposableBean。
        //    上层容器（如 DefaultListableBeanFactory）应直接持有
        //    Arc<dyn DisposableBean> 来调用 destroy()。
        //    此适配器仅负责自定义销毁闭包的调用。

        Ok(())
    }
}

impl std::fmt::Debug for DisposableBeanAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisposableBeanAdapter")
            .field("bean_name", &self.bean_name)
            .field("destroy_method_name", &self.destroy_method_name)
            .field("has_callback", &self.destroy_callback.is_some())
            .finish()
    }
}

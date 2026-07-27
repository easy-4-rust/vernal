//! DestructionAwareBeanPostProcessor — Spring 风格的销毁感知后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.DestructionAwareBeanPostProcessor`。
//!
//! 扩展 `BeanPostProcessor`，增加销毁前回调。

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的销毁感知后处理器接口。
///
/// 对应 Spring 的 `DestructionAwareBeanPostProcessor`。
///
/// 在 Bean 销毁之前调用 `post_process_before_destruction`，
/// 用于在销毁前执行清理逻辑（例如释放 AOP 代理持有的资源）。
pub trait DestructionAwareBeanPostProcessor: BeanPostProcessor {
    /// 在 Bean 销毁之前调用。
    ///
    /// 对应 Spring 的 `DestructionAwareBeanPostProcessor.postProcessBeforeDestruction(Object bean, String beanName)`。
    ///
    /// # 执行顺序
    ///
    /// 1. `DestructionAwareBeanPostProcessor.postProcessBeforeDestruction` ← 此方法
    /// 2. `DisposableBean.destroy`
    /// 3. 自定义 `destroy-method`
    fn post_process_before_destruction(
        &self,
        bean: &dyn std::any::Any,
        bean_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 判断此 Bean 是否需要销毁回调。
    ///
    /// 对应 Spring 的 `DestructionAwareBeanPostProcessor.requiresDestruction(Object bean)`。
    ///
    /// 返回 `false` 时跳过销毁流程。默认返回 `true`。
    fn requires_destruction(&self, _bean: &dyn std::any::Any) -> bool {
        true
    }
}

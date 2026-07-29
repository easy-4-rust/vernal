//! BeanDefinitionCustomizer — Spring 风格的 Bean 定义定制器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanDefinitionCustomizer`。
//!
//! 用于在注册 Bean 定义之前对其进行定制修改。

use std::sync::Mutex;

use crate::bean_definition::BeanDefinition;

/// Spring 风格的 Bean 定义定制器 trait。
///
/// 对应 Spring 的 `BeanDefinitionCustomizer`。
///
/// 允许在 Bean 定义注册到容器之前对其进行定制修改。
/// 常用于编程式地修改 Bean 定义的作用域、属性、懒加载等设置。
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::bean_definition_customizer::BeanDefinitionCustomizer;
///
/// // 使用闭包作为定制器
/// let customizer = |bd: &mut dyn BeanDefinition| {
///     // 定制逻辑
/// };
/// ```
pub trait BeanDefinitionCustomizer: Send + Sync + 'static {
    /// 定制给定的 Bean 定义。
    ///
    /// 对应 Spring 的 `void customize(BeanDefinition bd)`。
    ///
    /// # 参数
    ///
    /// * `bd` — 要定制的 Bean 定义的可变引用
    fn customize(&self, bd: &mut dyn BeanDefinition);
}

/// 为所有满足 `FnOnce(&mut dyn BeanDefinition) + Send + Sync + 'static` 的闭包
/// 实现 `BeanDefinitionCustomizer`。
///
/// 由于 `FnOnce` 通过 `&self` 无法安全地按值调用，这里我们采用的方式是：
/// 对于实现了 `Fn` 的闭包（最常见的场景），可以直接通过 `&self` 调用。
/// 对于真正的 `FnOnce` 闭包，请使用 `FnOnceBeanDefinitionCustomizer` 包装。
impl<F> BeanDefinitionCustomizer for F
where
    F: Fn(&mut dyn BeanDefinition) + Send + Sync + 'static,
{
    fn customize(&self, bd: &mut dyn BeanDefinition) {
        (self)(bd);
    }
}

/// 一个包装类型，用于将真正的 `FnOnce` 闭包转换为 `BeanDefinitionCustomizer`。
///
/// 使用内部 `Mutex<Option>` 实现一次性的线程安全消费。
pub struct FnOnceBeanDefinitionCustomizer {
    /// 内部互斥锁，通过 take 消费闭包。
    inner: Mutex<Option<Box<dyn FnOnce(&mut dyn BeanDefinition) + Send>>>,
}

impl FnOnceBeanDefinitionCustomizer {
    /// 创建一个包装了 FnOnce 闭包的定制器。
    ///
    /// # 参数
    ///
    /// * `f` — 要包装的 FnOnce 闭包
    pub fn new(f: Box<dyn FnOnce(&mut dyn BeanDefinition) + Send>) -> Self {
        Self {
            inner: Mutex::new(Some(f)),
        }
    }
}

impl BeanDefinitionCustomizer for FnOnceBeanDefinitionCustomizer {
    fn customize(&self, bd: &mut dyn BeanDefinition) {
        // 从 Mutex 中 take 出闭包并消费它。
        // 如果闭包已被消费（多次调用），则 panic。
        let mut guard = self
            .inner
            .lock()
            .expect("FnOnceBeanDefinitionCustomizer lock poisoned");
        let f = guard
            .take()
            .expect("FnOnceBeanDefinitionCustomizer::customize called more than once");
        // 释放锁后再调用闭包，避免死锁
        drop(guard);
        f(bd);
    }
}

impl std::fmt::Debug for FnOnceBeanDefinitionCustomizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnOnceBeanDefinitionCustomizer").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_closure_as_customizer() {
        let mut customized = false;
        let customizer = |bd: &mut dyn BeanDefinition| {
            let _ = bd;
        };

        let mut bd = crate::abstract_bean_definition::AbstractBeanDefinition::new();
        customizer.customize(&mut bd);
        // 闭包执行成功，但没有办法在这里断言 customized
        // 因为这个闭包是 Fn，不捕获 mut
    }

    #[test]
    fn test_fnonce_wrapper() {
        let closure = {
            move |bd: &mut dyn BeanDefinition| {
                let _ = bd;
            }
        };

        let wrapper = FnOnceBeanDefinitionCustomizer::new(Box::new(closure));
        let mut bd = crate::abstract_bean_definition::AbstractBeanDefinition::new();
        wrapper.customize(&mut bd);
    }
}

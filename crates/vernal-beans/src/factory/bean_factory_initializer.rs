//! BeanFactoryInitializer — 对应 Spring `org.springframework.beans.factory.BeanFactoryInitializer`。
//!
//! Bean 工厂初始化器接口。

use std::any::Any;

/// Bean 工厂初始化器接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.BeanFactoryInitializer`。
///
/// 用于在 Bean 工厂创建后进行初始化。
pub trait BeanFactoryInitializer<T: Any + Send + Sync>: Send + Sync {
    /// 初始化 Bean 工厂。
    fn initialize(&self, factory: &mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// BeanFactoryInitializer 的闭包实现。
pub struct ClosureBeanFactoryInitializer<T: Any + Send + Sync> {
    callback:
        Box<dyn Fn(&mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl<T: Any + Send + Sync> ClosureBeanFactoryInitializer<T> {
    /// 创建一个新的实例。
    pub fn new(
        callback: impl Fn(&mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl<T: Any + Send + Sync> BeanFactoryInitializer<T> for ClosureBeanFactoryInitializer<T> {
    fn initialize(&self, factory: &mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self.callback)(factory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_initializer() {
        let initializer = ClosureBeanFactoryInitializer::new(|_factory| Ok(()));
        let mut factory = String::new();
        assert!(initializer.initialize(&mut factory).is_ok());
    }
}

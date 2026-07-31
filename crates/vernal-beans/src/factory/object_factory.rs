//! ObjectFactory — 对应 Spring `org.springframework.beans.factory.ObjectFactory`。
//!
//! 对象工厂接口。

use std::any::Any;
use std::sync::Arc;

/// 对象工厂接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.ObjectFactory`。
///
/// 用于延迟创建对象的工厂接口。
pub trait ObjectFactory<T: Any + Send + Sync>: Send + Sync {
    /// 创建对象实例。
    ///
    /// 对应 Java 方法：`T getObject()`
    fn get_object(&self) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>>;
}

/// ObjectFactory 的闭包实现。
pub struct ClosureObjectFactory<T: Any + Send + Sync> {
    creator: Box<dyn Fn() -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl<T: Any + Send + Sync> ClosureObjectFactory<T> {
    pub fn new(creator: impl Fn() -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static) -> Self {
        Self { creator: Box::new(creator) }
    }
}

impl<T: Any + Send + Sync> ObjectFactory<T> for ClosureObjectFactory<T> {
    fn get_object(&self) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>> {
        (self.creator)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closure_factory() {
        let factory = ClosureObjectFactory::new(|| Ok(Arc::new(String::from("hello"))));
        let obj = factory.get_object().unwrap();
        assert_eq!(*obj, "hello");
    }
}

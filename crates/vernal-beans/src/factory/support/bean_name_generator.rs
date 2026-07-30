//! BeanNameGenerator — Spring 风格 Bean 名称生成器。

use std::any::TypeId;

pub trait BeanNameGenerator: Send + Sync {
    fn generate_bean_name(&self, type_id: TypeId) -> String;
}

pub struct DefaultBeanNameGenerator {
    counter: std::sync::atomic::AtomicU32,
}

impl DefaultBeanNameGenerator {
    pub fn new() -> Self { Self { counter: std::sync::atomic::AtomicU32::new(0) } }
}

impl BeanNameGenerator for DefaultBeanNameGenerator {
    fn generate_bean_name(&self, type_id: TypeId) -> String {
        let n = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("bean_{}_{:?}", n, type_id)
    }
}

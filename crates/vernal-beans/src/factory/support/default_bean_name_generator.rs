//! DefaultBeanNameGenerator — Spring 风格默认 Bean 名称生成器。

use std::any::TypeId;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::factory::support::bean_name_generator::BeanNameGenerator;

pub struct DefaultBeanNameGenerator {
    counter: AtomicU32,
}

impl DefaultBeanNameGenerator {
    pub fn new() -> Self { Self { counter: AtomicU32::new(0) } }
}

impl BeanNameGenerator for DefaultBeanNameGenerator {
    fn generate_bean_name(&self, _type_id: TypeId) -> String {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("bean_{}", n)
    }
}

impl Default for DefaultBeanNameGenerator {
    fn default() -> Self { Self::new() }
}

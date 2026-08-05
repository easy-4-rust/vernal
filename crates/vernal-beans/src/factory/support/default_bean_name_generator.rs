//! DefaultBeanNameGenerator — Spring 风格默认 Bean 名称生成器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultBeanNameGenerator`。
//!
//! 在 Spring 中，`DefaultBeanNameGenerator` 使用 Bean 的类名作为默认名称。
//! 如果同名 Bean 已存在，则追加 `#N` 后缀以保证唯一性。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::factory::support::bean_name_generator::BeanNameGenerator;

/// 默认 Bean 名称生成器。
///
/// 对应 Spring 的 `DefaultBeanNameGenerator`。
///
/// 使用 `"bean_" + 递增序号` 的格式生成唯一 Bean 名称。
/// 保证每次生成的名称唯一。
pub struct DefaultBeanNameGenerator {
    counter: AtomicU32,
    generated_names: Mutex<HashSet<String>>,
}

impl DefaultBeanNameGenerator {
    /// 创建新的默认 Bean 名称生成器。
    pub fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            generated_names: Mutex::new(HashSet::new()),
        }
    }

    /// 获取已生成的名称数量。
    pub fn generated_count(&self) -> usize {
        self.generated_names.lock().unwrap().len()
    }

    /// 检查指定名称是否已生成。
    pub fn has_generated(&self, name: &str) -> bool {
        self.generated_names.lock().unwrap().contains(name)
    }
}

impl BeanNameGenerator for DefaultBeanNameGenerator {
    fn generate_bean_name(&self, _type_id: TypeId) -> String {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        let name = format!("bean_{}", n);
        self.generated_names.lock().unwrap().insert(name.clone());
        name
    }
}

impl Default for DefaultBeanNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_sequential_names() {
        let generator = DefaultBeanNameGenerator::new();
        assert_eq!(
            generator.generate_bean_name(TypeId::of::<String>()),
            "bean_0"
        );
        assert_eq!(
            generator.generate_bean_name(TypeId::of::<String>()),
            "bean_1"
        );
        assert_eq!(generator.generate_bean_name(TypeId::of::<i32>()), "bean_2");
    }

    #[test]
    fn tracks_generated_names() {
        let generator = DefaultBeanNameGenerator::new();
        generator.generate_bean_name(TypeId::of::<String>());
        generator.generate_bean_name(TypeId::of::<String>());

        assert_eq!(generator.generated_count(), 2);
        assert!(generator.has_generated("bean_0"));
        assert!(generator.has_generated("bean_1"));
        assert!(!generator.has_generated("bean_99"));
    }
}

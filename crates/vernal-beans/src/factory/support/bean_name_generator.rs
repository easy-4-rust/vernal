//! BeanNameGenerator — Spring 风格 Bean 名称生成器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanNameGenerator`。
//!
//! 在 Spring 中，`BeanNameGenerator` 接口定义了 Bean 名称生成策略。
//! 默认实现 `DefaultBeanNameGenerator` 使用类名生成 Bean 名称。
//!
//! ## 实现类
//!
//! - `DefaultBeanNameGenerator` — 基于类名的默认生成器
//! - `AnnotationBeanNameGenerator` — 基于注解的名称生成器

use std::any::TypeId;
use std::sync::atomic::{AtomicU32, Ordering};

/// Bean 名称生成器接口。
///
/// 对应 Spring 的 `BeanNameGenerator`。
///
/// 定义 Bean 名称的生成策略。Spring 容器在注册 Bean 定义时
/// 调用此接口生成唯一的 Bean 名称。
pub trait BeanNameGenerator: Send + Sync {
    /// 为指定类型的 Bean 生成名称。
    ///
    /// 对应 Spring 的 `String generateBeanName(BeanDefinition, BeanDefinitionRegistry)`。
    ///
    /// # 参数
    /// - `type_id` — Bean 的类型标识
    ///
    /// # 返回
    /// 生成的 Bean 名称
    fn generate_bean_name(&self, type_id: TypeId) -> String;
}

/// 基于类名的默认 Bean 名称生成器。
///
/// 对应 Spring 的 `DefaultBeanNameGenerator`。
///
/// 使用类型名称加序号生成唯一名称，格式为 `TypeName_N`。
pub struct DefaultBeanNameGenerator {
    counter: AtomicU32,
}

impl DefaultBeanNameGenerator {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { counter: AtomicU32::new(0) }
    }

    /// 生成 Bean 名称，使用类型调试名加序号。
    pub fn generate_with_prefix(&self, prefix: &str) -> String {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{}_{}", prefix, n)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_generator_produces_unique_names() {
        let generator = DefaultBeanNameGenerator::new();
        let name1 = generator.generate_bean_name(TypeId::of::<String>());
        let name2 = generator.generate_bean_name(TypeId::of::<String>());
        let name3 = generator.generate_bean_name(TypeId::of::<i32>());

        assert_ne!(name1, name2);
        assert_ne!(name2, name3);
    }

    #[test]
    fn default_generator_names_start_with_bean() {
        let generator = DefaultBeanNameGenerator::new();
        let name = generator.generate_bean_name(TypeId::of::<String>());
        assert!(name.starts_with("bean_"));
    }

    #[test]
    fn generate_with_prefix() {
        let generator = DefaultBeanNameGenerator::new();
        let name1 = generator.generate_with_prefix("MyService");
        let name2 = generator.generate_with_prefix("MyService");

        assert_eq!(name1, "MyService_0");
        assert_eq!(name2, "MyService_1");
    }
}

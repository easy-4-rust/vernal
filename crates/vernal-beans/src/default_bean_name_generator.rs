//! DefaultBeanNameGenerator — Spring 风格的默认 Bean 名称生成器。
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultBeanNameGenerator`。
use crate::bean_definition::BeanDefinition;

/// Spring 风格的默认 Bean 名称生成器。
#[derive(Clone, Debug, Default)]
pub struct DefaultBeanNameGenerator;

impl DefaultBeanNameGenerator {
    pub fn generate_bean_name(&self, definition: &dyn BeanDefinition) -> String {
        let class = definition.bean_class_name();
        if class.is_empty() {
            format!("bean_{}", rand_num())
        } else {
            class.split("::").last().unwrap_or(class).to_string()
        }
    }
}

fn rand_num() -> u64 { use std::time::{SystemTime, UNIX_EPOCH}; SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64 }

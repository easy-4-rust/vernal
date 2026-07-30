//! AOT 模块: bean_factory_initialization_aot_processor。
use std::any::Any;
use std::sync::Arc;

/// BeanFactoryInitializationAotProcessor — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanFactoryInitializationAotProcessor {
    // TODO: 添加字段
}

impl BeanFactoryInitializationAotProcessor {
    pub fn new() -> Self { Self::default() }
}

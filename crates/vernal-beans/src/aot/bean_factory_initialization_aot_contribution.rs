//! AOT 模块: bean_factory_initialization_aot_contribution。
use std::any::Any;
use std::sync::Arc;

/// BeanFactoryInitializationAotContribution — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanFactoryInitializationAotContribution {
    // TODO: 添加字段
}

impl BeanFactoryInitializationAotContribution {
    pub fn new() -> Self { Self::default() }
}

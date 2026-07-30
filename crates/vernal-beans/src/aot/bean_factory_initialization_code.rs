//! AOT 模块: bean_factory_initialization_code。
use std::any::Any;
use std::sync::Arc;

/// BeanFactoryInitializationCode — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanFactoryInitializationCode {
    // TODO: 添加字段
}

impl BeanFactoryInitializationCode {
    pub fn new() -> Self { Self::default() }
}

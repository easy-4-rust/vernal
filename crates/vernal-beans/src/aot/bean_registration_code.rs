//! AOT 模块: bean_registration_code。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationCode — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationCode {
    // TODO: 添加字段
}

impl BeanRegistrationCode {
    pub fn new() -> Self { Self::default() }
}

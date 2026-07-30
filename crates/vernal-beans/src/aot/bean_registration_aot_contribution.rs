//! AOT 模块: bean_registration_aot_contribution。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationAotContribution — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationAotContribution {
    // TODO: 添加字段
}

impl BeanRegistrationAotContribution {
    pub fn new() -> Self { Self::default() }
}

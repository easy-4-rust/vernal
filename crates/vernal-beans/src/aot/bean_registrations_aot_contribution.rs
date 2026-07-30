//! AOT 模块: bean_registrations_aot_contribution。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationsAotContribution — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationsAotContribution {
    // TODO: 添加字段
}

impl BeanRegistrationsAotContribution {
    pub fn new() -> Self { Self::default() }
}

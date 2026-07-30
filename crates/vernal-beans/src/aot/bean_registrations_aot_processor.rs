//! AOT 模块: bean_registrations_aot_processor。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationsAotProcessor — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationsAotProcessor {
    // TODO: 添加字段
}

impl BeanRegistrationsAotProcessor {
    pub fn new() -> Self { Self::default() }
}

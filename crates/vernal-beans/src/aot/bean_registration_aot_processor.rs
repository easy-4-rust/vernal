//! AOT 模块: bean_registration_aot_processor。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationAotProcessor — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationAotProcessor {
    // TODO: 添加字段
}

impl BeanRegistrationAotProcessor {
    pub fn new() -> Self { Self::default() }
}

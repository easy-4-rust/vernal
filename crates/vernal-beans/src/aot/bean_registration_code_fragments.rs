//! AOT 模块: bean_registration_code_fragments。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationCodeFragments — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationCodeFragments {
    // TODO: 添加字段
}

impl BeanRegistrationCodeFragments {
    pub fn new() -> Self { Self::default() }
}

//! AOT 模块: default_bean_registration_code_fragments。
use std::any::Any;
use std::sync::Arc;

/// DefaultBeanRegistrationCodeFragments — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct DefaultBeanRegistrationCodeFragments {
    // TODO: 添加字段
}

impl DefaultBeanRegistrationCodeFragments {
    pub fn new() -> Self { Self::default() }
}

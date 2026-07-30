//! AOT 模块: bean_registration_code_generator。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationCodeGenerator — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationCodeGenerator {
    // TODO: 添加字段
}

impl BeanRegistrationCodeGenerator {
    pub fn new() -> Self { Self::default() }
}

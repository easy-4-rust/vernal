//! AOT 模块: bean_registration_code_fragments_decorator。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationCodeFragmentsDecorator — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationCodeFragmentsDecorator {
    // TODO: 添加字段
}

impl BeanRegistrationCodeFragmentsDecorator {
    pub fn new() -> Self { Self::default() }
}

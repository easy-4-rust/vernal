//! AOT 模块: bean_registration_exclude_filter。
use std::any::Any;
use std::sync::Arc;

/// BeanRegistrationExcludeFilter — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct BeanRegistrationExcludeFilter {
    // TODO: 添加字段
}

impl BeanRegistrationExcludeFilter {
    pub fn new() -> Self { Self::default() }
}

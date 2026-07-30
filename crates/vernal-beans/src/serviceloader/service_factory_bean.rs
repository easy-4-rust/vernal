//! ServiceLoader 模块: service_factory_bean。
use std::any::Any;
use std::sync::Arc;

/// ServiceFactoryBean — 对应 Spring ServiceLoader 组件。
#[derive(Debug, Clone, Default)]
pub struct ServiceFactoryBean {
    // TODO: 添加字段
}

impl ServiceFactoryBean {
    pub fn new() -> Self { Self::default() }
}

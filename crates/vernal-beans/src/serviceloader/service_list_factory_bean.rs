//! ServiceLoader 模块: service_list_factory_bean。
use std::any::Any;
use std::sync::Arc;

/// ServiceListFactoryBean — 对应 Spring ServiceLoader 组件。
#[derive(Debug, Clone, Default)]
pub struct ServiceListFactoryBean {
    // TODO: 添加字段
}

impl ServiceListFactoryBean {
    pub fn new() -> Self { Self::default() }
}

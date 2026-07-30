//! service_factory_bean — 对应 Java 类：org.springframework.beans.factory.serviceloader.ServiceFactoryBean。
//!
//! 对应 Spring beans.factory.serviceloader 包。

use std::any::Any;
use std::sync::Arc;

/// ServiceFactoryBean — Spring factory.serviceloader 组件。
#[derive(Debug, Clone, Default)]
pub struct ServiceFactoryBean {
    // TODO: 添加字段
}

impl ServiceFactoryBean {
    pub fn new() -> Self { Self::default() }
}

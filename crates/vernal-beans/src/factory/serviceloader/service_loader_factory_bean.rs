//! service_loader_factory_bean — 对应 Java 类：org.springframework.beans.factory.serviceloader.ServiceLoaderFactoryBean。
//!
//! 对应 Spring beans.factory.serviceloader 包。

use std::any::Any;
use std::sync::Arc;

/// ServiceLoaderFactoryBean — Spring factory.serviceloader 组件。
#[derive(Debug, Clone, Default)]
pub struct ServiceLoaderFactoryBean {
    // TODO: 添加字段
}

impl ServiceLoaderFactoryBean {
    pub fn new() -> Self { Self::default() }
}

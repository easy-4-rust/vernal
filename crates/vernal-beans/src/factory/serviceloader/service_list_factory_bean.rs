//! service_list_factory_bean — 对应 Java 类：org.springframework.beans.factory.serviceloader.ServiceListFactoryBean。
//!
//! 对应 Spring beans.factory.serviceloader 包。

use std::any::Any;
use std::sync::Arc;

/// ServiceListFactoryBean — Spring factory.serviceloader 组件。
#[derive(Debug, Clone, Default)]
pub struct ServiceListFactoryBean {
    // TODO: 添加字段
}

impl ServiceListFactoryBean {
    pub fn new() -> Self { Self::default() }
}

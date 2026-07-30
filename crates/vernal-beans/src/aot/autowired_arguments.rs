//! AOT 模块: autowired_arguments。
use std::any::Any;
use std::sync::Arc;

/// AutowiredArguments — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct AutowiredArguments {
    // TODO: 添加字段
}

impl AutowiredArguments {
    pub fn new() -> Self { Self::default() }
}

//! AOT 模块: autowired_method_arguments_resolver。
use std::any::Any;
use std::sync::Arc;

/// AutowiredMethodArgumentsResolver — 对应 Spring AOT 组件。
#[derive(Debug, Clone, Default)]
pub struct AutowiredMethodArgumentsResolver {
    // TODO: 添加字段
}

impl AutowiredMethodArgumentsResolver {
    pub fn new() -> Self { Self::default() }
}

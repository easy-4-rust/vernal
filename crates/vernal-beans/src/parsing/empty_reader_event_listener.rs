//! 解析模块: empty_reader_event_listener。
use std::any::Any;
use std::sync::Arc;

/// EmptyReaderEventListener — 对应 Spring 解析组件。
#[derive(Debug, Clone, Default)]
pub struct EmptyReaderEventListener {
    // TODO: 添加字段
}

impl EmptyReaderEventListener {
    pub fn new() -> Self { Self::default() }
}

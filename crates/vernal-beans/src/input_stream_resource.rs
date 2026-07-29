//! InputStreamResource — 以输入流/字节缓冲为后端的资源。
//!
//! 对应 Java 类：`org.springframework.core.io.InputStreamResource`。
//!
//! 包装一段已就绪的字节内容，对应 Spring 中"以给定 InputStream 构造、
//! 不可重复读取"的资源语义。本实现使用 [`Vec<u8>`] 承载内容，
//! 因此实际可重复读取，但 [`Resource::is_open`] 仍返回 `true`
//! 以反映其语义定位（不应作为长期资源使用）。

use std::fmt;
use std::io::{Cursor, Read};
use std::sync::Arc;

use crate::resource::Resource;

/// 以字节缓冲为后端的资源。
///
/// 对应 Spring 的 `InputStreamResource`。
///
/// 持有一份共享字节内容与描述。读取时返回内容的副本光标。
#[derive(Clone)]
pub struct InputStreamResource {
    /// 字节内容（共享不可变）。
    content: Arc<Vec<u8>>,
    /// 描述。
    description: String,
}

impl InputStreamResource {
    /// 创建一个新的 `InputStreamResource`。
    pub fn new(content: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            content: Arc::new(content),
            description: description.into(),
        }
    }

    /// 从字符串内容创建。
    pub fn from_string(content: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(content.into().into_bytes(), description)
    }

    /// 返回内容长度的只读视图。
    pub fn content_len(&self) -> usize {
        self.content.len()
    }
}

impl fmt::Debug for InputStreamResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputStreamResource")
            .field("description", &self.description)
            .field("content_length", &self.content.len())
            .finish()
    }
}

impl Resource for InputStreamResource {
    fn exists(&self) -> bool {
        true
    }

    fn is_readable(&self) -> bool {
        true
    }

    fn is_open(&self) -> bool {
        true
    }

    fn url(&self) -> Option<String> {
        None
    }

    fn file_path(&self) -> Option<String> {
        None
    }

    fn filename(&self) -> Option<String> {
        None
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new(Cursor::new((*self.content).clone())))
    }

    fn content_length(&self) -> Option<u64> {
        Some(self.content.len() as u64)
    }

    fn read_to_bytes(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok((*self.content).clone())
    }
}

//! AbstractResource — Spring 风格的资源抽象基类（Rust 翻译）。
//!
//! 对应 Java 类：`org.springframework.core.io.AbstractResource`。
//!
//! 在 Spring 中，`AbstractResource` 是 `Resource` 接口的基类，
//! 为大多数方法提供合理默认实现。由于 Rust 没有继承，本模块以一个
//! 具体结构 [`AbstractResource`] 复现其"携带描述与字节内容"的用法，
//! 同时各类具体资源可以直接实现 [`Resource`] trait 并复用 trait 默认实现。

use std::fmt;
use std::io::{Cursor, Read};

use crate::resource::Resource;

/// Spring 风格的资源抽象基类实现。
///
/// 对应 Spring 的 `AbstractResource`。
///
/// 持有一段字节内容与一个描述字符串，作为一个可复用的"内存资源"基类。
/// 可用作测试、回退实现，或被具体资源结构内嵌以共享默认行为。
#[derive(Clone)]
pub struct AbstractResource {
    /// 字节内容。
    content: Vec<u8>,
    /// 人类可读描述。
    description: String,
    /// 可选文件名。
    filename: Option<String>,
}

impl AbstractResource {
    /// 创建一个新的 `AbstractResource`。
    ///
    /// `content` 为字节内容，`description` 为人类可读描述。
    pub fn new(content: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            content,
            description: description.into(),
            filename: None,
        }
    }

    /// 从字符串内容创建。
    pub fn from_string(content: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(content.into().into_bytes(), description)
    }

    /// 设置文件名。
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// 返回内部字节内容的引用。
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// 返回描述。
    pub fn description_str(&self) -> &str {
        &self.description
    }
}

impl fmt::Debug for AbstractResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbstractResource")
            .field("description", &self.description)
            .field("filename", &self.filename)
            .field("content_length", &self.content.len())
            .finish()
    }
}

impl Resource for AbstractResource {
    fn exists(&self) -> bool {
        true
    }

    fn is_readable(&self) -> bool {
        true
    }

    fn is_open(&self) -> bool {
        false
    }

    fn url(&self) -> Option<String> {
        None
    }

    fn file_path(&self) -> Option<String> {
        None
    }

    fn filename(&self) -> Option<String> {
        self.filename.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new(Cursor::new(self.content.clone())))
    }

    fn content_length(&self) -> Option<u64> {
        Some(self.content.len() as u64)
    }

    fn read_to_bytes(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.content.clone())
    }
}

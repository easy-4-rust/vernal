//! Resource — Spring 风格的资源抽象 trait。
//!
//! 对应 Java 类：`org.springframework.core.io.Resource`。
//!
//! 表示一个底层资源（文件、类路径条目、URL 等）的统一抽象。
//! 各种具体实现（[`FileSystemResource`]、[`ClassPathResource`]、
//! [`UrlResource`] 等）均实现此 trait。
//!
//! [`FileSystemResource`]: crate::filesystem_resource::FileSystemResource
//! [`ClassPathResource`]: crate::classpath_resource::ClassPathResource
//! [`UrlResource`]: crate::url_resource::UrlResource

use std::io::Read;

/// Spring 风格的资源抽象 trait。
///
/// 对应 Spring 的 `org.springframework.core.io.Resource`。
///
/// 抽象文件、类路径条目、URL 等资源，提供统一的元信息查询与字节读取入口。
/// 实现方负责提供具体的 `exists`、`is_readable` 与 `input_stream` 行为。
pub trait Resource: Send + Sync + std::fmt::Debug {
    /// 资源是否实际存在。
    ///
    /// 对应 Spring 的 `exists()`。
    fn exists(&self) -> bool;

    /// 资源是否可读。
    ///
    /// 对应 Spring 的 `isReadable()`。
    fn is_readable(&self) -> bool;

    /// 资源是否以打开的输入流为后端（即不可重复读取）。
    ///
    /// 对应 Spring 的 `isOpen()`。默认返回 `false`。
    fn is_open(&self) -> bool {
        false
    }

    /// 资源底层是否为文件。
    ///
    /// 对应 Spring 的 `isFile()`。默认返回 `false`。
    fn is_file(&self) -> bool {
        false
    }

    /// 返回资源的 URL 字符串（如 `file:/...`、`http://...`）。
    ///
    /// 对应 Spring 的 `getURL()`。若资源不对应 URL 则返回 `None`。
    fn url(&self) -> Option<String>;

    /// 返回资源的 URI 字符串。
    ///
    /// 对应 Spring 的 `getURI()`。默认与 [`Resource::url`] 相同。
    fn uri(&self) -> Option<String> {
        self.url()
    }

    /// 返回资源的文件路径（如果底层是文件系统）。
    ///
    /// 对应 Spring 的 `getFile()`。默认返回 `None`。
    fn file_path(&self) -> Option<String>;

    /// 返回资源的文件名（最后一个路径片段）。
    ///
    /// 对应 Spring 的 `getFilename()`。
    fn filename(&self) -> Option<String>;

    /// 返回用于错误消息的人类可读描述。
    ///
    /// 对应 Spring 的 `getDescription()`。
    fn description(&self) -> String;

    /// 返回一个可读取资源内容的输入流。
    ///
    /// 对应 Spring 的 `getInputStream()`。
    ///
    /// # 错误
    ///
    /// 资源不可读或底层 I/O 失败时返回 `Err`。
    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>>;

    /// 将资源内容一次性读取为字节向量。
    ///
    /// 默认实现基于 [`Resource::input_stream`]。
    ///
    /// # 错误
    ///
    /// 底层 I/O 失败时返回 `Err`。
    fn read_to_bytes(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = self.input_stream()?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// 将资源内容一次性读取为字符串。
    ///
    /// 默认实现基于 [`Resource::read_to_bytes`]。
    ///
    /// # 错误
    ///
    /// 底层 I/O 失败或字节非有效 UTF-8 时返回 `Err`。
    fn read_to_string(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = self.read_to_bytes()?;
        Ok(String::from_utf8(bytes)?)
    }

    /// 返回内容长度（字节数），未知时返回 `None`。
    ///
    /// 对应 Spring 的 `contentLength()`。
    fn content_length(&self) -> Option<u64> {
        None
    }
}

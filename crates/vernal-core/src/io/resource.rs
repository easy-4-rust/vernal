//! 资源抽象 trait。
//!
//! 对标 Spring `org.springframework.core.io.Resource`。

use std::io;

/// 资源抽象 trait。
///
/// 对应 Java: org.springframework.core.io.Resource
pub trait Resource: Send + Sync {
    /// 检查资源是否存在。
    fn exists(&self) -> bool;
    /// 资源是否可读。
    fn is_readable(&self) -> bool;
    /// 资源文件名（如适用）。
    fn filename(&self) -> Option<&str>;
    /// 资源描述（用于诊断日志）。
    fn description(&self) -> String;
    /// 读取为字节。
    fn read_bytes(&self) -> io::Result<Vec<u8>>;
    /// 读取为 UTF-8 字符串（默认实现）。
    fn read_string(&self) -> io::Result<String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// 资源错误。
///
/// 对应 Spring 资源加载的 `IOException` 聚合
#[derive(Debug)]
pub enum ResourceError {
    /// 资源未找到
    NotFound(String),
    /// 资源不可读
    NotReadable(String),
    /// IO 错误
    Io(io::Error),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "资源未找到: {msg}"),
            Self::NotReadable(msg) => write!(f, "资源不可读: {msg}"),
            Self::Io(e) => write!(f, "资源 IO 错误: {e}"),
        }
    }
}

impl std::error::Error for ResourceError {}

impl From<io::Error> for ResourceError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

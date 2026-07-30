//! MultipartResolver — 多部分解析器。
use crate::multipart::MultipartFile;

/// 多部分解析器 trait。
pub trait MultipartResolver: Send + Sync {
    fn is_multipart(&self, content_type: &str) -> bool;
    fn resolve(&self, data: &[u8]) -> Result<Vec<MultipartFile>, Box<dyn std::error::Error + Send + Sync>>;
}

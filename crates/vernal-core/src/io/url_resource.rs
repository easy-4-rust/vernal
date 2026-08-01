//! URL 资源。
//!
//! 对标 Spring `org.springframework.core.io.UrlResource`。
//! 仅在 feature `convert-url` 下编译（依赖 `url` crate 做 URL 解析）。

use std::io;

use super::Resource;

/// URL 资源。
///
/// 对应 Java: org.springframework.core.io.UrlResource
///
/// Spring 语义：包装 `java.net.URL`，支持 `http(s)://`、`file://` 等协议。
/// 目前支持 `file://` 协议的文件读取；网络协议返回"不支持"错误。
pub struct UrlResource {
    url: url::Url,
}

impl UrlResource {
    /// 从 URL 字符串创建资源。
    ///
    /// # 错误
    ///
    /// URL 格式非法时返回 [`std::io::Error`]。
    pub fn new(url_str: &str) -> io::Result<Self> {
        let url = url::Url::parse(url_str).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("invalid URL: {e}"))
        })?;
        Ok(Self { url })
    }
}

impl Resource for UrlResource {
    fn exists(&self) -> bool {
        match self.url.scheme() {
            "file" => self.url.to_file_path().map(|p| p.exists()).unwrap_or(false),
            _ => false,
        }
    }

    fn is_readable(&self) -> bool {
        match self.url.scheme() {
            "file" => self.url.to_file_path().map(|p| p.is_file()).unwrap_or(false),
            _ => false,
        }
    }

    fn filename(&self) -> Option<&str> {
        self.url.path_segments().and_then(|mut seg| seg.next_back())
    }

    fn description(&self) -> String {
        format!("UrlResource [{}]", self.url)
    }

    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        match self.url.scheme() {
            "file" => {
                let path = self.url.to_file_path().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "cannot convert URL to path")
                })?;
                std::fs::read(path)
            }
            scheme => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("unsupported URL scheme: {scheme}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_file_url() {
        // A 类（合同对齐）：对标 Spring `new UrlResource("file:...")`
        let resource = UrlResource::new("file:///tmp/nonexistent-xyz").unwrap();
        assert!(!resource.exists());
        assert!(resource.description().contains("file:///tmp/nonexistent-xyz"));
    }

    #[test]
    fn reads_file_content() {
        // A 类（合同对齐）：file 协议读取
        let dir = std::env::temp_dir();
        let path = dir.join("vernal-url-resource-test.txt");
        std::fs::write(&path, b"url-content").unwrap();
        let url = format!("file://{}", path.display());
        let resource = UrlResource::new(&url).unwrap();
        assert!(resource.exists());
        assert_eq!(resource.read_bytes().unwrap(), b"url-content");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unsupported_scheme_returns_error() {
        // C 类（错误路径）：网络协议暂不支持
        let resource = UrlResource::new("http://example.com/x").unwrap();
        assert!(!resource.exists());
        assert!(resource.read_bytes().is_err());
    }

    #[test]
    fn invalid_url_returns_error() {
        // C 类（错误路径）
        assert!(UrlResource::new("not a url").is_err());
    }
}

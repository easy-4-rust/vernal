//! UrlResource — URL 资源。
//!
//! 对应 Java 类：`org.springframework.core.io.UrlResource`。
//!
//! 以 URL 字符串为后端的 [`Resource`] 实现。本实现内建支持
//! `file:` 协议（通过 `std::fs`）；其他协议（`http:`、`https:` 等）
//! 由于当前 crate 未引入 HTTP 客户端依赖，其 `input_stream` 会返回
//! 一个明确的"不支持"错误，但元信息查询仍可用。

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::resource::Resource;

/// URL 资源。
///
/// 对应 Spring 的 `UrlResource`。
///
/// 持有一个 URL 字符串。`file:` URL 直接映射到本地文件；其它协议
/// 的内容读取目前不支持，但描述、URL、文件名等元信息可用。
#[derive(Clone)]
pub struct UrlResource {
    /// 原始 URL。
    url: String,
    /// 清理后的协议（小写 scheme）。
    scheme: String,
    /// 对于 file: 协议，解析后的本地路径。
    file_path: Option<PathBuf>,
}

impl UrlResource {
    /// 从 URL 字符串创建。
    ///
    /// `file:` 协议会被解析为本地文件路径。
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        let (scheme, file_path) = parse_url(&url);
        Self {
            url,
            scheme,
            file_path,
        }
    }

    /// 返回原始 URL。
    pub fn url_str(&self) -> &str {
        &self.url
    }

    /// 返回协议（小写 scheme）。
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// 是否是 `file:` URL。
    pub fn is_file_url(&self) -> bool {
        self.scheme == "file"
    }
}

/// 解析 URL，返回 (scheme, 可选的本地文件路径)。
fn parse_url(url: &str) -> (String, Option<PathBuf>) {
    let (scheme, rest) = match url.split_once(':') {
        Some((s, r)) => (s.to_ascii_lowercase(), r),
        None => (String::new(), url),
    };
    let file_path = if scheme == "file" {
        // 去掉最多两个前导斜杠，得到本地路径。
        let stripped = rest.trim_start_matches('/');
        Some(PathBuf::from(format!("/{stripped}")))
    } else {
        None
    };
    (scheme, file_path)
}

impl fmt::Debug for UrlResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UrlResource")
            .field("url", &self.url)
            .field("scheme", &self.scheme)
            .finish()
    }
}

impl Resource for UrlResource {
    fn exists(&self) -> bool {
        if let Some(ref p) = self.file_path {
            return p.exists();
        }
        // 非 file: URL 无法静态判定，保守返回 true（与 Spring 一致：URL 通常认为存在）。
        true
    }

    fn is_readable(&self) -> bool {
        if let Some(ref p) = self.file_path {
            return p.is_file();
        }
        false
    }

    fn is_file(&self) -> bool {
        self.is_file_url()
    }

    fn url(&self) -> Option<String> {
        Some(self.url.clone())
    }

    fn file_path(&self) -> Option<String> {
        self.file_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
    }

    fn filename(&self) -> Option<String> {
        if let Some(ref p) = self.file_path {
            return p.file_name().map(|n| n.to_string_lossy().into_owned());
        }
        // 从 URL 尾部提取最后一段作为文件名。
        self.url
            .rsplit(['/', '\\'])
            .next()
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
    }

    fn description(&self) -> String {
        format!("URL [{}]", self.url)
    }

    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref p) = self.file_path {
            let file = File::open(p)?;
            return Ok(Box::new(file));
        }
        Err(format!(
            "UrlResource: reading non-file URL '{}' is not supported by vernal-beans \
             (no HTTP client dependency available)",
            self.url
        )
        .into())
    }

    fn content_length(&self) -> Option<u64> {
        self.file_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
    }
}

//! Web 服务器抽象模块（feature = "web"）。
//!
//! 对标 Spring `jetty-io`（Web 服务器）。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `org.eclipse.jetty.io` | `hyper` crate |
//! | `org.eclipse.jetty.server.Server` | `hyper::server::Server` |

/// HTTP 请求方法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET 请求
    Get,
    /// POST 请求
    Post,
    /// PUT 请求
    Put,
    /// DELETE 请求
    Delete,
    /// PATCH 请求
    Patch,
    /// HEAD 请求
    Head,
    /// OPTIONS 请求
    Options,
}

impl HttpMethod {
    /// 获取 HTTP 方法的字符串表示。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    #[test]
    fn http_method_equality() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }
}

/// Hyper Web 服务器工具（feature = "web"）。
///
/// 对标 Spring `org.eclipse.jetty.io`。
///
/// 使用 `hyper` crate 作为后端，提供 HTTP 服务器功能。
#[cfg(feature = "web")]
pub mod hyper_server {
    use super::HttpMethod;

    /// Hyper Web 服务器工具。
    ///
    /// 对标 Spring `Server`。
    #[derive(Debug, Clone)]
    pub struct HyperServer {
        host: String,
        port: u16,
    }

    impl HyperServer {
        /// 创建新的 Hyper 服务器。
        pub fn new(host: impl Into<String>, port: u16) -> Self {
            Self {
                host: host.into(),
                port,
            }
        }

        /// 获取服务器地址。
        pub fn address(&self) -> String {
            format!("{}:{}", self.host, self.port)
        }

        /// 获取主机名。
        pub fn host(&self) -> &str {
            &self.host
        }

        /// 获取端口号。
        pub fn port(&self) -> u16 {
            self.port
        }
    }

    impl Default for HyperServer {
        fn default() -> Self {
            Self::new("127.0.0.1", 8080)
        }
    }
}

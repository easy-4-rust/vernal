//! HTTP 方法枚举。
//!
//! 对标 Spring `org.springframework.http.HttpMethod` 及 RFC 7231/9110 HTTP 方法。

/// HTTP 方法枚举。
///
/// 对应 Java: org.springframework.http.HttpMethod
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    /// GET
    Get,
    /// POST
    Post,
    /// PUT
    Put,
    /// DELETE
    Delete,
    /// PATCH
    Patch,
    /// HEAD
    Head,
    /// OPTIONS
    Options,
    /// CONNECT
    Connect,
    /// TRACE
    Trace,
}

impl HttpMethod {
    /// 返回方法名。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Connect => "CONNECT",
            Self::Trace => "TRACE",
        }
    }

    /// 解析字符串为 HTTP 方法。
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            "HEAD" => Some(Self::Head),
            "OPTIONS" => Some(Self::Options),
            "CONNECT" => Some(Self::Connect),
            "TRACE" => Some(Self::Trace),
            _ => None,
        }
    }

    /// 是否为安全方法（对标 RFC 7231）。
    ///
    /// 安全方法不应有副作用。
    #[must_use]
    pub fn is_safe(self) -> bool {
        // Rust matches! 模式必须按枚举声明顺序。
        // 声明顺序: Get, Post, Put, Delete, Patch, Head, Options, Connect, Trace
        // 安全方法: Get(Get), Head(Head), Options(Options), Trace(Trace)
        matches!(self, Self::Get | Self::Head | Self::Options | Self::Trace)
    }

    /// 是否为幂等方法。
    #[must_use]
    pub fn is_idempotent(self) -> bool {
        // 声明顺序: Get, Post, Put, Delete, Patch, Head, Options, Connect, Trace
        // 幂等方法: Get(0), Put(2), Delete(3), Head(5), Options(6), Trace(8)
        matches!(
            self,
            Self::Get | Self::Put | Self::Delete | Self::Head | Self::Options | Self::Trace
        )
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
    fn as_str_returns_correct_method_name() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
        assert_eq!(HttpMethod::Options.as_str(), "OPTIONS");
        assert_eq!(HttpMethod::Connect.as_str(), "CONNECT");
        assert_eq!(HttpMethod::Trace.as_str(), "TRACE");
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
    }

    #[test]
    fn parse_uppercase() {
        assert_eq!(HttpMethod::parse("GET"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::parse("POST"), Some(HttpMethod::Post));
    }

    #[test]
    fn parse_lowercase() {
        assert_eq!(HttpMethod::parse("get"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::parse("post"), Some(HttpMethod::Post));
    }

    #[test]
    fn parse_mixed_case() {
        assert_eq!(HttpMethod::parse("Get"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::parse("Post"), Some(HttpMethod::Post));
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert_eq!(HttpMethod::parse("INVALID"), None);
        assert_eq!(HttpMethod::parse(""), None);
        assert_eq!(HttpMethod::parse("FOOBAR"), None);
    }

    #[test]
    fn safe_methods_are_idempotent() {
        for m in [HttpMethod::Get, HttpMethod::Head, HttpMethod::Options, HttpMethod::Trace] {
            assert!(m.is_safe());
            assert!(m.is_idempotent());
        }
    }

    #[test]
    fn post_is_idempotent_false() {
        assert!(!HttpMethod::Post.is_safe());
        assert!(!HttpMethod::Post.is_idempotent());
    }

    #[test]
    fn put_and_delete_are_idempotent() {
        assert!(HttpMethod::Put.is_idempotent());
        assert!(HttpMethod::Delete.is_idempotent());
    }

    #[test]
    fn patch_is_not_idempotent() {
        assert!(!HttpMethod::Patch.is_idempotent());
    }

    #[test]
    fn method_hash_and_eq() {
        use std::collections::HashSet;
        let a = HttpMethod::Get;
        let b = HttpMethod::Get;
        assert_eq!(a, b);
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}

//! 上下文资源契约。
//!
//! 对标 Spring `org.springframework.core.io.ContextResource`。

use super::Resource;

/// 上下文资源契约。
///
/// 对应 Java: org.springframework.core.io.ContextResource
///
/// Spring 语义：`Resource` 的扩展——在应用上下文内相对路径定位的资源
/// （如 `WEB-INF/...`）。
pub trait ContextResource: Resource {
    /// 返回上下文内相对路径。
    fn path_within_context(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    struct WebInfResource {
        inner: ByteArrayResource,
        path: String,
    }

    impl Resource for WebInfResource {
        fn exists(&self) -> bool {
            self.inner.exists()
        }
        fn is_readable(&self) -> bool {
            self.inner.is_readable()
        }
        fn filename(&self) -> Option<&str> {
            self.inner.filename()
        }
        fn description(&self) -> String {
            self.inner.description()
        }
        fn read_bytes(&self) -> std::io::Result<Vec<u8>> {
            self.inner.read_bytes()
        }
    }

    impl ContextResource for WebInfResource {
        fn path_within_context(&self) -> String {
            self.path.clone()
        }
    }

    #[test]
    fn exposes_context_relative_path() {
        // A 类（合同对齐）：对标 Spring `getPathWithinContext`
        let resource = WebInfResource {
            inner: ByteArrayResource::new(Vec::new()),
            path: "WEB-INF/web.xml".to_string(),
        };
        assert_eq!(resource.path_within_context(), "WEB-INF/web.xml");
    }
}

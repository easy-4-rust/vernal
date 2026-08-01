//! 协议解析器契约。
//!
//! 对标 Spring `org.springframework.core.io.ProtocolResolver`。

use std::io;

use super::Resource;

/// 协议解析器契约。
///
/// 对应 Java: org.springframework.core.io.ProtocolResolver
///
/// Spring 语义：`ResourceLoader` 的扩展点——`resolve(location)` 处理自定义
/// 协议（返回 `null` 表示不处理）。
pub trait ProtocolResolver: Send + Sync {
    /// 解析自定义协议位置。
    ///
    /// 返回 `None` 表示本解析器不处理该位置（对标 Spring 返回 `null`）。
    fn resolve(&self, location: &str) -> Option<Box<dyn Resource>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopResolver;

    impl ProtocolResolver for NoopResolver {
        fn resolve(&self, location: &str) -> Option<Box<dyn Resource>> {
            if location.starts_with("custom:") {
                Some(Box::new(crate::io::ByteArrayResource::new(
                    b"custom".to_vec(),
                )))
            } else {
                None
            }
        }
    }

    #[test]
    fn resolver_handles_own_protocol() {
        // A 类（合同对齐）：对标 Spring 自定义协议解析
        let resolver = NoopResolver;
        let resource = resolver.resolve("custom:item").expect("应解析");
        assert_eq!(resource.read_bytes().unwrap(), b"custom");
    }

    #[test]
    fn resolver_declines_unknown_protocol() {
        // B 类（边界行为）：对标 Spring 返回 null 不处理
        let resolver = NoopResolver;
        assert!(resolver.resolve("classpath:x").is_none());
    }
}

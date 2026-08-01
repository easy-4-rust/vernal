//! 抽象资源契约。
//!
//! 对标 Spring `org.springframework.core.io.AbstractResource`。

use std::io;

use super::Resource;

/// 抽象资源契约。
///
/// 对应 Java: org.springframework.core.io.AbstractResource
///
/// Spring 语义：`AbstractResource` 为 `Resource` 提供默认实现（`exists` 通过
/// 读取探测、`description` 默认格式等）；Rust 中以带默认方法的 trait 表达。
pub trait AbstractResource: Resource {
    /// 默认存在性探测：可读取即存在。
    fn default_exists(&self) -> bool {
        self.is_readable() && self.read_bytes().is_ok()
    }

    /// 默认可读性：`read_bytes` 成功即认为可读。
    fn default_is_readable(&self) -> bool {
        self.read_bytes().is_ok()
    }

    /// 创建用于诊断的默认描述。
    fn default_description(&self, type_name: &str) -> String {
        format!("{type_name} [{}]", self.filename().unwrap_or("?"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResource;

    impl Resource for TestResource {
        fn exists(&self) -> bool {
            true
        }
        fn is_readable(&self) -> bool {
            true
        }
        fn filename(&self) -> Option<&str> {
            Some("test.txt")
        }
        fn description(&self) -> String {
            "test".to_string()
        }
        fn read_bytes(&self) -> io::Result<Vec<u8>> {
            Ok(b"data".to_vec())
        }
    }

    impl AbstractResource for TestResource {}

    #[test]
    fn default_exists_uses_readability() {
        // A 类（合同对齐）：对标 Spring 默认 exists 语义
        let resource = TestResource;
        assert!(resource.default_exists());
        assert!(resource.default_is_readable());
    }

    #[test]
    fn default_description_format() {
        // B 类（边界行为）：对标 Spring `ClassPathResource [filename]` 格式
        let resource = TestResource;
        assert_eq!(
            resource.default_description("TestResource"),
            "TestResource [test.txt]"
        );
    }
}

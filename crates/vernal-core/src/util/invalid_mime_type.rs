//! 无效 MIME 类型异常。
//!
//! 对标 Spring `org.springframework.util.InvalidMimeTypeException`。
//!
//! vernal-core 用 [`InvalidMimeType`] 结构体表达等价语义。
//! 此文件需要 feature = "mime"(因为它扩展 mime_type::InvalidMimeType)。

#![cfg(feature = "mime")]

pub use super::mime_type::InvalidMimeType;

impl InvalidMimeType {
    /// 获取原始输入(对标 Spring `getMimeType()`)。
    #[must_use]
    pub fn mime_type(&self) -> &str {
        &self.input
    }

    /// 获取错误原因(对标 Spring `getMessage()` 由 Display 提供)。
    #[must_use]
    pub fn reason_message(&self) -> &str {
        &self.reason
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_correct_fields() {
        let err = InvalidMimeType {
            input: "bad-input".to_string(),
            reason: "missing /".to_string(),
        };
        assert_eq!(err.mime_type(), "bad-input");
        assert_eq!(err.reason_message(), "missing /");
    }
}

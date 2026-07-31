//! 操作声明元数据校验错误对象。

use std::{error::Error, fmt};

/// 描述标签或限定符不满足稳定声明约束的错误。
///
/// 标签和限定符是开发期低基数元数据，不能包含空白、控制字符或空字符串，避免
/// 同一个逻辑声明因不可见字符产生不同切点结果。错误保留被拒绝的声明值，便于在
/// 应用启动阶段直接定位配置或宏输入问题。
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OperationMetadataError {
    /// 标签为空或包含空白、控制字符。
    InvalidTag {
        /// 被拒绝的标签。
        tag: String,
    },
    /// 限定符为空或包含空白、控制字符。
    InvalidQualifier {
        /// 被拒绝的限定符。
        qualifier: String,
    },
}

impl fmt::Display for OperationMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTag { tag } => {
                write!(formatter, "invalid operation tag: {tag:?}")
            }
            Self::InvalidQualifier { qualifier } => {
                write!(formatter, "invalid operation qualifier: {qualifier:?}")
            }
        }
    }
}

impl Error for OperationMetadataError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_tag_display() {
        let err = OperationMetadataError::InvalidTag {
            tag: "bad tag".to_string(),
        };
        assert!(format!("{}", err).contains("invalid operation tag"));
    }

    #[test]
    fn invalid_qualifier_display() {
        let err = OperationMetadataError::InvalidQualifier {
            qualifier: "bad qualifier".to_string(),
        };
        assert!(format!("{}", err).contains("invalid operation qualifier"));
    }

    #[test]
    fn error_trait() {
        let err = OperationMetadataError::InvalidTag {
            tag: "test".to_string(),
        };
        let _: &dyn Error = &err;
    }
}

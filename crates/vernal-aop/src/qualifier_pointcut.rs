//! 操作限定符切点对象。

use std::sync::Arc;

use crate::{Operation, OperationMetadata, OperationMetadataError, Pointcut};

/// 匹配具有指定精确限定符的操作声明。
///
/// 限定符适合同一组件方法存在多个静态语义变体的情况，例如读写策略或租户模式；
/// 请求期动态租户、用户和参数不能放入该字段，应继续使用 [`crate::InvocationContext`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifierPointcut {
    qualifier: Arc<str>,
}

impl QualifierPointcut {
    /// 创建精确限定符切点。
    ///
    /// # Errors
    ///
    /// 限定符为空或包含空白、控制字符时返回 [`OperationMetadataError`]。
    pub fn new(qualifier: impl Into<Arc<str>>) -> Result<Self, OperationMetadataError> {
        let qualifier = qualifier.into();
        OperationMetadata::empty().with_qualifier(Arc::clone(&qualifier))?;
        Ok(Self { qualifier })
    }

    /// 返回目标限定符。
    #[must_use]
    pub fn qualifier(&self) -> &str {
        &self.qualifier
    }
}

impl Pointcut for QualifierPointcut {
    /// 判断操作声明是否具有目标限定符。
    fn matches(&self, operation: &Operation) -> bool {
        operation.metadata().qualifier() == Some(self.qualifier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifier_pointcut_new_valid() {
        let pc = QualifierPointcut::new("primary");
        assert!(pc.is_ok());
    }

    #[test]
    fn qualifier_pointcut_new_invalid() {
        let pc = QualifierPointcut::new("");
        assert!(pc.is_err());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn qualifier_pointcut_qualifier() {
        let pc = QualifierPointcut::new("primary").unwrap();
        assert_eq!(pc.qualifier(), "primary");
    }

    #[test]
    fn qualifier_pointcut_matches() {
        let pc = QualifierPointcut::new("primary").unwrap();
        let op = Operation::new("Service", "method").with_metadata(
            OperationMetadata::empty()
                .with_qualifier("primary")
                .unwrap(),
        );
        assert!(pc.matches(&op));
    }

    #[test]
    fn qualifier_pointcut_no_match() {
        let pc = QualifierPointcut::new("primary").unwrap();
        let op = Operation::new("Service", "method");
        assert!(!pc.matches(&op));
    }

    #[test]
    fn qualifier_pointcut_clone() {
        let pc = QualifierPointcut::new("primary").unwrap();
        let cloned = pc.clone();
        assert_eq!(cloned.qualifier(), "primary");
    }

    #[test]
    fn qualifier_pointcut_debug() {
        let pc = QualifierPointcut::new("primary").unwrap();
        let debug = format!("{:?}", pc);
        assert!(debug.contains("primary"));
    }

    #[test]
    fn qualifier_pointcut_partial_eq() {
        let pc1 = QualifierPointcut::new("primary").unwrap();
        let pc2 = QualifierPointcut::new("primary").unwrap();
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn qualifier_pointcut_not_equal() {
        let pc1 = QualifierPointcut::new("primary").unwrap();
        let pc2 = QualifierPointcut::new("secondary").unwrap();
        assert_ne!(pc1, pc2);
    }
}

//! 操作标签切点对象。

use std::sync::Arc;

use crate::{Operation, OperationMetadata, OperationMetadataError, Pointcut};

/// 匹配包含指定精确标签的操作声明。
///
/// 标签在计划编译阶段从 [`OperationMetadata`] 读取；计划封存后不再执行标签查找。
/// 该对象可与组件、方法及限定符切点组合，形成类似注解切点的静态选择能力。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagPointcut {
    tag: Arc<str>,
}

impl TagPointcut {
    /// 创建精确标签切点。
    ///
    /// # Errors
    ///
    /// 标签为空或包含空白、控制字符时返回 [`OperationMetadataError`]。
    pub fn new(tag: impl Into<Arc<str>>) -> Result<Self, OperationMetadataError> {
        let tag = tag.into();
        OperationMetadata::empty().with_tag(Arc::clone(&tag))?;
        Ok(Self { tag })
    }

    /// 返回目标标签。
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }
}

impl Pointcut for TagPointcut {
    /// 判断操作声明是否包含目标标签。
    fn matches(&self, operation: &Operation) -> bool {
        operation.metadata().has_tag(self.tag())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_pointcut_new_valid() {
        let pc = TagPointcut::new("secured");
        assert!(pc.is_ok());
    }

    #[test]
    fn tag_pointcut_new_invalid() {
        let pc = TagPointcut::new("");
        assert!(pc.is_err());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn tag_pointcut_tag() {
        let pc = TagPointcut::new("secured").unwrap();
        assert_eq!(pc.tag(), "secured");
    }

    #[test]
    fn tag_pointcut_matches() {
        let pc = TagPointcut::new("secured").unwrap();
        let op = Operation::new("Service", "method")
            .with_metadata(OperationMetadata::empty().with_tag("secured").unwrap());
        assert!(pc.matches(&op));
    }

    #[test]
    fn tag_pointcut_no_match() {
        let pc = TagPointcut::new("secured").unwrap();
        let op = Operation::new("Service", "method");
        assert!(!pc.matches(&op));
    }

    #[test]
    fn tag_pointcut_clone() {
        let pc = TagPointcut::new("secured").unwrap();
        let cloned = pc.clone();
        assert_eq!(cloned.tag(), "secured");
    }

    #[test]
    fn tag_pointcut_debug() {
        let pc = TagPointcut::new("secured").unwrap();
        let debug = format!("{:?}", pc);
        assert!(debug.contains("secured"));
    }

    #[test]
    fn tag_pointcut_partial_eq() {
        let pc1 = TagPointcut::new("secured").unwrap();
        let pc2 = TagPointcut::new("secured").unwrap();
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn tag_pointcut_not_equal() {
        let pc1 = TagPointcut::new("secured").unwrap();
        let pc2 = TagPointcut::new("public").unwrap();
        assert_ne!(pc1, pc2);
    }
}

//! 操作声明元数据对象。

use std::sync::Arc;

use crate::OperationMetadataError;

/// 一个操作在应用计划编译阶段使用的不可变声明元数据。
///
/// 标签表达可叠加语义，例如 `transactional`、`secured` 或 `idempotent`；
/// 限定符表达同类操作中的单一命名变体。标签始终按字典序去重，保证重复模块注册
/// 和诊断输出不依赖输入顺序。该对象不承载请求参数、用户标识或其他高基数数据。
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct OperationMetadata {
    tags: Arc<[Arc<str>]>,
    qualifier: Option<Arc<str>>,
}

impl OperationMetadata {
    /// 创建没有标签和限定符的声明元数据。
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// 添加一个标签并返回新的不可变元数据。
    ///
    /// 重复标签会被确定性去重。
    ///
    /// # Errors
    ///
    /// 标签为空或包含空白、控制字符时返回 [`OperationMetadataError`]。
    pub fn with_tag(self, tag: impl Into<Arc<str>>) -> Result<Self, OperationMetadataError> {
        let tag = tag.into();
        Self::validate_tag(&tag)?;

        // 标签数量通常很小，复制后排序比引入运行期可变集合更简单；最终结果冻结为
        // Arc 切片，可被全部调用计划廉价共享。
        let mut tags = self.tags.iter().cloned().collect::<Vec<_>>();
        tags.push(tag);
        tags.sort_unstable_by(|left, right| left.as_ref().cmp(right.as_ref()));
        tags.dedup_by(|left, right| left.as_ref() == right.as_ref());

        Ok(Self {
            tags: tags.into(),
            qualifier: self.qualifier,
        })
    }

    /// 设置或替换单一限定符并返回新的不可变元数据。
    ///
    /// # Errors
    ///
    /// 限定符为空或包含空白、控制字符时返回 [`OperationMetadataError`]。
    pub fn with_qualifier(
        self,
        qualifier: impl Into<Arc<str>>,
    ) -> Result<Self, OperationMetadataError> {
        let qualifier = qualifier.into();
        Self::validate_qualifier(&qualifier)?;
        Ok(Self {
            tags: self.tags,
            qualifier: Some(qualifier),
        })
    }

    /// 清除限定符并保留全部标签。
    #[must_use]
    pub fn without_qualifier(self) -> Self {
        Self {
            tags: self.tags,
            qualifier: None,
        }
    }

    /// 返回按字典序排列的标签切片。
    #[must_use]
    pub fn tags(&self) -> &[Arc<str>] {
        &self.tags
    }

    /// 判断是否包含指定精确标签。
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags
            .binary_search_by(|candidate| candidate.as_ref().cmp(tag))
            .is_ok()
    }

    /// 返回可选限定符。
    #[must_use]
    pub fn qualifier(&self) -> Option<&str> {
        self.qualifier.as_deref()
    }

    /// 返回当前元数据是否没有标签和限定符。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty() && self.qualifier.is_none()
    }

    /// 校验标签是可稳定比较的低基数名称。
    fn validate_tag(tag: &str) -> Result<(), OperationMetadataError> {
        if tag.is_empty()
            || tag
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(OperationMetadataError::InvalidTag {
                tag: tag.to_owned(),
            });
        }
        Ok(())
    }

    /// 校验限定符是可稳定比较的低基数名称。
    fn validate_qualifier(qualifier: &str) -> Result<(), OperationMetadataError> {
        if qualifier.is_empty()
            || qualifier
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(OperationMetadataError::InvalidQualifier {
                qualifier: qualifier.to_owned(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod operation_metadata_tests {
    use super::*;

    #[test]
    fn operation_metadata_empty() {
        let metadata = OperationMetadata::empty();
        assert!(metadata.is_empty());
        assert!(metadata.tags().is_empty());
        assert!(metadata.qualifier().is_none());
    }

    #[test]
    fn operation_metadata_with_tag() {
        let metadata = OperationMetadata::empty().with_tag("secured").unwrap();
        assert!(!metadata.is_empty());
        assert!(metadata.has_tag("secured"));
    }

    #[test]
    fn operation_metadata_with_qualifier() {
        let metadata = OperationMetadata::empty()
            .with_qualifier("primary")
            .unwrap();
        assert!(!metadata.is_empty());
        assert_eq!(metadata.qualifier(), Some("primary"));
    }

    #[test]
    fn operation_metadata_with_tag_invalid() {
        let result = OperationMetadata::empty().with_tag("");
        assert!(result.is_err());
    }

    #[test]
    fn operation_metadata_with_qualifier_invalid() {
        let result = OperationMetadata::empty().with_qualifier("");
        assert!(result.is_err());
    }

    #[test]
    fn operation_metadata_without_qualifier() {
        let metadata = OperationMetadata::empty()
            .with_qualifier("primary")
            .unwrap()
            .without_qualifier();
        assert!(metadata.qualifier().is_none());
    }

    #[test]
    fn operation_metadata_has_tag() {
        let metadata = OperationMetadata::empty().with_tag("secured").unwrap();
        assert!(metadata.has_tag("secured"));
        assert!(!metadata.has_tag("public"));
    }

    #[test]
    fn operation_metadata_tags_sorted() {
        let metadata = OperationMetadata::empty()
            .with_tag("b")
            .unwrap()
            .with_tag("a")
            .unwrap();
        let tags = metadata.tags();
        assert_eq!(tags[0].as_ref(), "a");
        assert_eq!(tags[1].as_ref(), "b");
    }

    #[test]
    fn operation_metadata_tags_dedup() {
        let metadata = OperationMetadata::empty()
            .with_tag("secured")
            .unwrap()
            .with_tag("secured")
            .unwrap();
        assert_eq!(metadata.tags().len(), 1);
    }

    #[test]
    fn operation_metadata_clone() {
        let metadata = OperationMetadata::empty().with_tag("secured").unwrap();
        let cloned = metadata.clone();
        assert_eq!(metadata, cloned);
    }

    #[test]
    fn operation_metadata_debug() {
        let metadata = OperationMetadata::empty();
        let debug = format!("{:?}", metadata);
        assert!(!debug.is_empty());
    }

    #[test]
    fn operation_metadata_default() {
        let metadata = OperationMetadata::default();
        assert!(metadata.is_empty());
    }
}

//! 被拦截操作描述对象。

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{OperationMetadata, OperationMetadataError};

/// 描述一次可被切面匹配的稳定操作身份及其声明元数据。
///
/// `component` 通常是组件类型或逻辑服务名，`method` 是方法或端点名称。两段式
/// 表达既适合普通组件方法，也能映射 HTTP 路由、消息消费和任务执行。
///
/// 相等性与 Hash 只使用 `component + method`，不包含标签和限定符。这样 Web/RPC
/// Adapter 可以用运行期解析出的低基数身份查找应用启动时预编译的计划，不必复制
/// 声明元数据。计划建造器会单独校验重复身份的元数据一致性，避免静默覆盖。
#[derive(Clone, Debug)]
pub struct Operation {
    component: Arc<str>,
    method: Arc<str>,
    metadata: Arc<OperationMetadata>,
}

impl Operation {
    /// 创建操作描述。
    #[must_use]
    pub fn new(component: impl Into<Arc<str>>, method: impl Into<Arc<str>>) -> Self {
        Self {
            component: component.into(),
            method: method.into(),
            metadata: Arc::new(OperationMetadata::empty()),
        }
    }

    /// 使用已经校验完成的声明元数据替换当前元数据。
    #[must_use]
    pub fn with_metadata(mut self, metadata: OperationMetadata) -> Self {
        self.metadata = Arc::new(metadata);
        self
    }

    /// 为操作添加声明标签。
    ///
    /// # Errors
    ///
    /// 标签为空或包含空白、控制字符时返回 [`OperationMetadataError`]。
    pub fn with_tag(mut self, tag: impl Into<Arc<str>>) -> Result<Self, OperationMetadataError> {
        self.metadata = Arc::new(Arc::unwrap_or_clone(self.metadata).with_tag(tag)?);
        Ok(self)
    }

    /// 为操作设置声明限定符。
    ///
    /// # Errors
    ///
    /// 限定符为空或包含空白、控制字符时返回 [`OperationMetadataError`]。
    pub fn with_qualifier(
        mut self,
        qualifier: impl Into<Arc<str>>,
    ) -> Result<Self, OperationMetadataError> {
        self.metadata = Arc::new(Arc::unwrap_or_clone(self.metadata).with_qualifier(qualifier)?);
        Ok(self)
    }

    /// 返回组件逻辑名称。
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// 返回方法或端点名称。
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// 返回不可变声明元数据。
    #[must_use]
    pub fn metadata(&self) -> &OperationMetadata {
        self.metadata.as_ref()
    }

    /// 判断稳定身份与声明元数据是否都完全一致。
    ///
    /// [`PartialEq`] 只比较运行期查找身份；目录构建阶段使用本方法区分可安全合并
    /// 的重复声明与必须拒绝的元数据冲突。
    #[must_use]
    pub fn same_declaration(&self, other: &Self) -> bool {
        self == other && self.metadata == other.metadata
    }
}

impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component && self.method == other.method
    }
}

impl Eq for Operation {}

impl Hash for Operation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.component.hash(state);
        self.method.hash(state);
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.component, self.method)
    }
}

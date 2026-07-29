//! DescriptiveResource — 仅携带名称与描述的简单资源。
//!
//! 对应 Java 类：`org.springframework.core.io.DescriptiveResource`。
//!
//! 当只需要一个"占位"资源、用以承载描述信息（例如错误消息中的占位符），
//! 而无需真正可读的内容时使用。它的 [`crate::resource::Resource::exists`]
//! 返回 `false`，[`crate::resource::Resource::input_stream`] 返回错误。

use std::fmt;
use std::io::Read;

use crate::resource::Resource;

/// 仅携带描述信息的简单资源。
///
/// 对应 Spring 的 `DescriptiveResource`。
///
/// 不对应任何真实底层资源：`exists()` 为 `false`，读取会失败。
/// 常用于在 API 中需要一个 `Resource` 占位对象时。
#[derive(Clone)]
pub struct DescriptiveResource {
    /// 描述信息。
    description: String,
}

impl DescriptiveResource {
    /// 创建一个新的 `DescriptiveResource`。
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }

    /// 返回描述信息。
    pub fn description_str(&self) -> &str {
        &self.description
    }
}

impl fmt::Debug for DescriptiveResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptiveResource")
            .field("description", &self.description)
            .finish()
    }
}

impl Resource for DescriptiveResource {
    fn exists(&self) -> bool {
        false
    }

    fn is_readable(&self) -> bool {
        false
    }

    fn url(&self) -> Option<String> {
        None
    }

    fn file_path(&self) -> Option<String> {
        None
    }

    fn filename(&self) -> Option<String> {
        None
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!(
            "DescriptiveResource '{}' has no readable content",
            self.description
        )
        .into())
    }
}

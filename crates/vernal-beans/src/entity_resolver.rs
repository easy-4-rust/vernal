//! EntityResolver — XML 实体解析器 trait。
//!
//! 对应 Java 类：`org.xml.sax.EntityResolver`。
//!
//! 在 XML 解析遇到外部实体（DTD、外部 Schema）时被调用，允许应用程序
//! 把对外部网络资源的引用重定向到本地资源。

use std::io::Read;
use std::sync::Arc;

/// 表示解析得到的实体内容。
#[derive(Clone)]
pub struct ResolvedEntity {
    /// 公共 ID（可能为空）。
    pub public_id: String,
    /// 系统 ID（可能为空）。
    pub system_id: String,
    /// 实体内容的字节流。
    pub input: Arc<dyn Read + Send>,
}

impl std::fmt::Debug for ResolvedEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedEntity")
            .field("public_id", &self.public_id)
            .field("system_id", &self.system_id)
            .finish_non_exhaustive()
    }
}

impl ResolvedEntity {
    /// 创建一个新的已解析实体。
    pub fn new(
        public_id: impl Into<String>,
        system_id: impl Into<String>,
        input: Arc<dyn Read + Send>,
    ) -> Self {
        Self {
            public_id: public_id.into(),
            system_id: system_id.into(),
            input,
        }
    }
}

/// Spring 风格的 XML 实体解析器 trait。
///
/// 对应 SAX 的 `EntityResolver`。
///
/// 在解析 XML 时遇到外部实体，调用方以此询问是否能本地解析。
pub trait EntityResolver: Send + Sync {
    /// 尝试解析给定公共/系统 ID 对应的实体。
    ///
    /// 返回 `Some` 表示已本地解析；`None` 表示交由默认机制处理。
    fn resolve_entity(
        &self,
        public_id: &str,
        system_id: &str,
    ) -> Result<Option<ResolvedEntity>, Box<dyn std::error::Error + Send + Sync>>;
}

impl std::fmt::Debug for dyn EntityResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityResolver").finish_non_exhaustive()
    }
}

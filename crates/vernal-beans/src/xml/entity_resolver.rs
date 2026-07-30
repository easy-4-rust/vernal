//! EntityResolver — XML 实体解析接口。
//!
//! 对应 Java 类：`org.springframework.beans.xml.EntityResolver`。
//!
//! 负责解析 XML 文档中的外部实体引用（DTD、XSD）。

/// 解析后的实体。
#[derive(Debug, Clone)]
pub struct ResolvedEntity {
    /// 实体的公共 ID。
    pub public_id: Option<String>,
    /// 实体的系统 ID。
    pub system_id: String,
    /// 实体的内容（字节流）。
    pub content: Vec<u8>,
}

/// XML 实体解析接口。
///
/// 对应 Spring 的 `EntityResolver`。
///
/// 用于解析 XML 文档中的外部实体引用。
/// 在 vernal-beans 中，主要用于解析 DTD 和 XSD schema。
pub trait EntityResolver: Send + Sync {
    /// 解析外部实体。
    ///
    /// # 参数
    /// - `public_id` — 实体的公共标识符
    /// - `system_id` — 实体的系统标识符
    ///
    /// # 返回
    /// 解析后的实体，或错误。
    fn resolve_entity(
        &self,
        public_id: Option<&str>,
        system_id: &str,
    ) -> Result<ResolvedEntity, Box<dyn std::error::Error + Send + Sync>>;
}

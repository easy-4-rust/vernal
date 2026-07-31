//! DocumentLoader — XML 文档加载接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DocumentLoader`。
//!
//! 负责将 XML 输入源加载为文档对象。


/// XML 文档对象（简化表示）。
#[derive(Debug, Clone)]
pub struct Document {
    /// 文档的根元素。
    pub root_element: Option<XmlElement>,
    /// 文档的系统 ID（来源标识）。
    pub system_id: Option<String>,
}

/// XML 元素（简化表示）。
#[derive(Debug, Clone)]
pub struct XmlElement {
    /// 元素的本地名称。
    pub local_name: String,
    /// 元素的命名空间 URI。
    pub namespace_uri: Option<String>,
    /// 元素的属性。
    pub attributes: Vec<(String, String)>,
    /// 子元素。
    pub children: Vec<XmlElement>,
    /// 文本内容。
    pub text: Option<String>,
}

/// XML 文档加载接口。
///
/// 对应 Spring 的 `DocumentLoader`。
///
/// 负责将 `InputSource`（或等价物）加载为 `Document` 对象。
pub trait DocumentLoader: Send + Sync {
    /// 加载 XML 文档。
    ///
    /// # 参数
    /// - `input` — XML 输入（字符串或字节流）
    /// - `entity_resolver` — 实体解析器（用于 DTD/XSD）
    /// - `error_handler` — 错误处理器
    /// - `validation_mode` — 验证模式（NONE, AUTO, DTD, XSD）
    /// - `namespace_aware` — 是否支持命名空间
    fn load_document(
        &self,
        input: &str,
        entity_resolver: Option<&dyn EntityResolver>,
        validation_mode: ValidationMode,
        namespace_aware: bool,
    ) -> Result<Document, Box<dyn std::error::Error + Send + Sync>>;
}

/// XML 验证模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// 不验证。
    None,
    /// 自动检测。
    Auto,
    /// DTD 验证。
    Dtd,
    /// XSD 验证。
    Xsd,
}

use crate::factory::xml::entity_resolver::EntityResolver;

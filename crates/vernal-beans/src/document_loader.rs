//! DocumentLoader — XML 文档加载 trait 与轻量 DOM 模型。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DocumentLoader`。
//!
//! 由于 vernal-beans 目前未引入外部 XML 解析依赖，此处定义一套极简的
//! DOM 抽象（[`Document`] / [`Element`] / [`Node`]）与加载 trait
//! [`DocumentLoader`]，供 XML Bean 定义读取流程使用。

use std::collections::HashMap;
use std::fmt;
use std::io::Read;

/// XML 文档的轻量表示。
///
/// 对应 Java 的 `org.w3c.dom.Document`。
#[derive(Debug, Clone)]
pub struct Document {
    /// 文档根元素。
    pub root: Option<Element>,
    /// 文档的 XML 声明版本（如 `"1.0"`）。
    pub version: Option<String>,
    /// 文档字符编码。
    pub encoding: Option<String>,
}

impl Document {
    /// 创建空文档。
    pub fn new() -> Self {
        Self {
            root: None,
            version: None,
            encoding: None,
        }
    }

    /// 返回文档根元素引用。
    pub fn document_element(&self) -> Option<&Element> {
        self.root.as_ref()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// XML 元素。
///
/// 对应 Java 的 `org.w3c.dom.Element`。
#[derive(Debug, Clone)]
pub struct Element {
    /// 命名空间 URI（可能为空字符串表示无命名空间）。
    pub namespace_uri: String,
    /// 限定名（带前缀，如 `beans:bean`）。
    pub qualified_name: String,
    /// 本地名（不含前缀，如 `bean`）。
    pub local_name: String,
    /// 属性映射（限定名 → 值）。
    pub attributes: HashMap<String, String>,
    /// 子节点列表。
    pub children: Vec<Node>,
    /// 元素的文本内容。
    pub text_content: String,
}

impl Element {
    /// 创建新元素。
    pub fn new(namespace_uri: impl Into<String>, local_name: impl Into<String>) -> Self {
        let local_name = local_name.into();
        Self {
            namespace_uri: namespace_uri.into(),
            qualified_name: local_name.clone(),
            local_name,
            attributes: HashMap::new(),
            children: Vec::new(),
            text_content: String::new(),
        }
    }

    /// 设置限定名。
    pub fn set_qualified_name(&mut self, qname: impl Into<String>) {
        self.qualified_name = qname.into();
    }

    /// 获取属性值。
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }

    /// 设置属性值。
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(name.into(), value.into());
    }

    /// 返回子元素迭代器。
    pub fn child_elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            _ => None,
        })
    }
}

/// XML 节点：元素或文本。
///
/// 对应 Java 的 `org.w3c.dom.Node`。
#[derive(Debug, Clone)]
pub enum Node {
    /// 元素节点。
    Element(Element),
    /// 文本节点。
    Text(String),
}

impl Node {
    /// 返回是否为元素节点。
    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    /// 返回是否为文本节点。
    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }
}

/// XML 文档加载 trait。
///
/// 对应 Spring 的 `DocumentLoader`。
///
/// 负责从输入流加载为 [`Document`]。
pub trait DocumentLoader: Send + Sync {
    /// 从输入流加载 XML 文档。
    ///
    /// 对应 Spring 的 `Document loadDocument(...)`。
    ///
    /// # 错误
    ///
    /// 解析失败时返回 `Err`。
    fn load_document(
        &self,
        input: &mut dyn Read,
    ) -> Result<Document, Box<dyn std::error::Error + Send + Sync>>;
}

impl fmt::Debug for dyn DocumentLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentLoader").finish_non_exhaustive()
    }
}

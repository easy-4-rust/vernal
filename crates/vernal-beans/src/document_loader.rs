//! DocumentLoader — XML 文档加载器 trait。
use std::fmt;

/// 简单 DOM 元素。
#[derive(Clone, Debug)]
pub struct Element {
    pub name: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub children: Vec<Element>,
    pub text: Option<String>,
}

/// 简单 XML 文档。
#[derive(Clone, Debug)]
pub struct Document {
    pub root: Option<Element>,
    pub encoding: Option<String>,
}

/// 文档加载器 trait。
pub trait DocumentLoader: fmt::Debug + Send + Sync {
    fn load_document(&self, content: &str) -> Result<Document, Box<dyn std::error::Error + Send + Sync>>;
}

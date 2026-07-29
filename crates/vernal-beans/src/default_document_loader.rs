//! DefaultDocumentLoader — 默认 XML 文档加载器实现。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DefaultDocumentLoader`。
//!
//! 内建一个极简但可用的 XML 文本解析器（递归下降），把输入流解析为
//! [`Document`]。支持元素、属性、嵌套、文本节点与注释；不支持 DTD、
//! 内部实体展开与 CDATA 段（遇到 CDATA 将原样保留为文本）。

use std::io::Read;

use crate::document_loader::{Document, DocumentLoader, Element, Node};

/// 默认 XML 文档加载器。
///
/// 对应 Spring 的 `DefaultDocumentLoader`。
#[derive(Debug, Clone, Default)]
pub struct DefaultDocumentLoader;

impl DefaultDocumentLoader {
    /// 创建新实例。
    pub fn new() -> Self {
        Self
    }
}

impl DocumentLoader for DefaultDocumentLoader {
    fn load_document(
        &self,
        input: &mut dyn Read,
    ) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        let mut parser = XmlParser::new(&buf);
        parser.parse()
    }
}

/// 极简 XML 解析器。
struct XmlParser<'a> {
    /// 输入字节。
    src: &'a [u8],
    /// 当前位置。
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        let mut doc = Document::new();
        self.skip_bom();
        self.skip_ws();
        // 可选 XML 声明。
        if self.starts_with(b"<?xml") {
            let (version, encoding) = self.parse_xml_decl()?;
            doc.version = version;
            doc.encoding = encoding;
        }
        loop {
            self.skip_ws_comments_and_pis();
            if self.pos >= self.src.len() {
                break;
            }
            if self.starts_with(b"<") {
                let element = self.parse_element()?;
                doc.root = Some(element);
                break;
            }
            // 非预期内容，跳过。
            self.pos += 1;
        }
        // 尾部忽略剩余空白/注释。
        Ok(doc)
    }

    fn skip_bom(&mut self) {
        if self.src.len() >= 3 && &self.src[0..3] == b"\xEF\xBB\xBF" {
            self.pos = 3;
        }
    }

    fn parse_xml_decl(
        &mut self,
    ) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
        self.expect("<?xml")?;
        let mut version = None;
        let mut encoding = None;
        loop {
            self.skip_ws();
            if self.starts_with(b"?>") {
                self.pos += 2;
                break;
            }
            if self.pos >= self.src.len() {
                break;
            }
            let name = self.parse_name();
            self.skip_ws();
            self.expect("=")?;
            self.skip_ws();
            let value = self.parse_quoted_value()?;
            match name.as_str() {
                "version" => version = Some(value),
                "encoding" => encoding = Some(value),
                _ => {}
            }
        }
        Ok((version, encoding))
    }

    fn parse_element(&mut self) -> Result<Element, Box<dyn std::error::Error + Send + Sync>> {
        self.expect("<")?;
        // 解析标签名（支持前缀:local 形式）。
        let qname = self.parse_name();
        let local_name = qname.split(':').next_back().unwrap_or(&qname).to_string();
        let namespace_uri = String::new();
        let mut element = Element::new(namespace_uri, &local_name);
        element.set_qualified_name(qname);

        // 解析属性。
        loop {
            self.skip_ws();
            if self.starts_with(b"/>") {
                self.pos += 2;
                return Ok(element);
            }
            if self.starts_with(b">") {
                self.pos += 1;
                break;
            }
            if self.pos >= self.src.len() {
                return Err("unexpected EOF in element header".into());
            }
            let attr_name = self.parse_name();
            self.skip_ws();
            self.expect("=")?;
            self.skip_ws();
            let attr_value = self.parse_quoted_value()?;
            element.set_attribute(attr_name, attr_value);
        }

        // 解析子节点直到匹配的结束标签。
        loop {
            self.skip_ws_comments_and_pis();
            if self.starts_with(b"</") {
                self.pos += 2;
                let end_name = self.parse_name();
                self.skip_ws();
                self.expect(">")?;
                let _ = end_name; // 不严格校验结束标签名。
                return Ok(element);
            }
            if self.starts_with(b"<") {
                let child = self.parse_element()?;
                element.children.push(Node::Element(child));
            } else {
                let text = self.parse_text();
                if !text.trim().is_empty() {
                    element.children.push(Node::Text(text.clone()));
                    element.text_content.push_str(&text);
                }
            }
            if self.pos >= self.src.len() {
                return Err("unexpected EOF before closing element".into());
            }
        }
    }

    fn parse_text(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.src.len() && self.src[self.pos] != b'<' {
            self.pos += 1;
        }
        let raw = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("");
        unescape(raw)
    }

    fn parse_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c.is_ascii_alphanumeric() || c == b':' || c == b'_' || c == b'-' || c == b'.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.src[start..self.pos])
            .unwrap_or("")
            .to_string()
    }

    fn parse_quoted_value(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if self.pos >= self.src.len() {
            return Err("expected quote".into());
        }
        let quote = self.src[self.pos];
        if quote != b'"' && quote != b'\'' {
            return Err(format!("expected quote, got {:?}", quote as char).into());
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.src.len() && self.src[self.pos] != quote {
            self.pos += 1;
        }
        let raw = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("");
        if self.pos < self.src.len() {
            self.pos += 1; // 跳过结束引号
        }
        Ok(unescape(raw))
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_ws_comments_and_pis(&mut self) {
        loop {
            self.skip_ws();
            if self.starts_with(b"<!--") {
                self.pos += 4;
                while self.pos + 2 < self.src.len() && &self.src[self.pos..self.pos + 3] != b"-->" {
                    self.pos += 1;
                }
                if self.pos + 2 < self.src.len() {
                    self.pos += 3;
                }
                continue;
            }
            if self.starts_with(b"<?") {
                while self.pos + 1 < self.src.len() && &self.src[self.pos..self.pos + 2] != b"?>" {
                    self.pos += 1;
                }
                if self.pos + 1 < self.src.len() {
                    self.pos += 2;
                }
                continue;
            }
            if self.starts_with(b"<!") {
                // DOCTYPE 等：跳到下一个 >
                while self.pos < self.src.len() && self.src[self.pos] != b'>' {
                    self.pos += 1;
                }
                if self.pos < self.src.len() {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.pos + prefix.len() <= self.src.len()
            && &self.src[self.pos..self.pos + prefix.len()] == prefix
    }

    fn expect(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.starts_with(s.as_bytes()) {
            self.pos += s.len();
            Ok(())
        } else {
            Err(format!("expected '{}' at position {}", s, self.pos).into())
        }
    }
}

/// 反转义常见 XML 实体。
fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_simple_element() {
        let mut loader = DefaultDocumentLoader::new();
        let xml = b"<?xml version=\"1.0\"?><root attr=\"v\"><child>text</child></root>";
        let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
        assert_eq!(doc.version.as_deref(), Some("1.0"));
        let root = doc.document_element().unwrap();
        assert_eq!(root.local_name, "root");
        assert_eq!(root.get_attribute("attr"), Some("v"));
        let mut kids = root.child_elements();
        let child = kids.next().unwrap();
        assert_eq!(child.local_name, "child");
        assert_eq!(child.text_content, "text");
    }
}

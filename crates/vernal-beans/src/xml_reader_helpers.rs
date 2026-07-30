//! XML 读取器辅助函数。
use std::collections::HashMap;

/// 解析 XML 属性。
pub fn parse_attributes(xml: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    for part in xml.split_whitespace() {
        if let Some((k, v)) = part.split_once('=') {
            attrs.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }
    attrs
}

/// 检查是否是默认命名空间。
pub fn is_default_namespace(namespace: &str) -> bool {
    namespace.is_empty() || namespace == "http://www.w3.org/2000/xmlns/"
}

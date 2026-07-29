//! xml_reader_helpers — XML Bean 定义读取辅助函数。
//!
//! 对应 Spring `org.springframework.beans.factory.xml.BeanDefinitionParserDelegate`
//! 中常用的元素属性读取、布尔/引用判定等辅助逻辑。
//!
//! 这些函数作用于 [`crate::document_loader::Element`]，便于读取器实现复用。

use crate::document_loader::Element;

/// 默认 beans 命名空间 URI。
pub const BEANS_NAMESPACE_URI: &str = "http://www.springframework.org/schema/beans";

/// `id` 属性名。
pub const ID_ATTRIBUTE: &str = "id";

/// `name` 属性名。
pub const NAME_ATTRIBUTE: &str = "name";

/// `class` 属性名。
pub const CLASS_ATTRIBUTE: &str = "class";

/// `parent` 属性名。
pub const PARENT_ATTRIBUTE: &str = "parent";

/// `scope` 属性名。
pub const SCOPE_ATTRIBUTE: &str = "scope";

/// `abstract` 属性名。
pub const ABSTRACT_ATTRIBUTE: &str = "abstract";

/// `lazy-init` 属性名。
pub const LAZY_INIT_ATTRIBUTE: &str = "lazy-init";

/// `autowire` 属性名。
pub const AUTOWIRE_ATTRIBUTE: &str = "autowire";

/// `depends-on` 属性名。
pub const DEPENDS_ON_ATTRIBUTE: &str = "depends-on";

/// `init-method` 属性名。
pub const INIT_METHOD_ATTRIBUTE: &str = "init-method";

/// `destroy-method` 属性名。
pub const DESTROY_METHOD_ATTRIBUTE: &str = "destroy-method";

/// `factory-bean` 属性名。
pub const FACTORY_BEAN_ATTRIBUTE: &str = "factory-bean";

/// `factory-method` 属性名。
pub const FACTORY_METHOD_ATTRIBUTE: &str = "factory-method";

/// `primary` 属性名。
pub const PRIMARY_ATTRIBUTE: &str = "primary";

/// `true` 对应的字面量集合。
const TRUE_LITERALS: &[&str] = &["true", "1", "yes", "on"];

/// `false` 对应的字面量集合。
const FALSE_LITERALS: &[&str] = &["false", "0", "no", "off"];

/// 读取元素的 `id` 属性。
pub fn element_id(element: &Element) -> Option<String> {
    element.get_attribute(ID_ATTRIBUTE).map(str::to_string)
}

/// 读取元素的 `name` 属性（可能含逗号/分号/空白分隔的多个别名）。
pub fn element_names(element: &Element) -> Vec<String> {
    element
        .get_attribute(NAME_ATTRIBUTE)
        .map(|s| {
            s.split([',', ';', ' ', '\t', '\n'])
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// 读取元素的 `class` 属性。
pub fn element_class(element: &Element) -> Option<String> {
    element.get_attribute(CLASS_ATTRIBUTE).map(str::to_string)
}

/// 读取元素的 `scope` 属性。
pub fn element_scope(element: &Element) -> Option<String> {
    element.get_attribute(SCOPE_ATTRIBUTE).map(str::to_string)
}

/// 读取元素的 `parent` 属性。
pub fn element_parent(element: &Element) -> Option<String> {
    element.get_attribute(PARENT_ATTRIBUTE).map(str::to_string)
}

/// 读取布尔属性的值，未设置时返回 `default_value`。
pub fn boolean_attribute(element: &Element, name: &str, default_value: bool) -> bool {
    match element.get_attribute(name) {
        Some(raw) => parse_boolean(raw).unwrap_or(default_value),
        None => default_value,
    }
}

/// 解析布尔字面量，无法识别时返回 `None`。
pub fn parse_boolean(raw: &str) -> Option<bool> {
    let lower = raw.trim().to_ascii_lowercase();
    if TRUE_LITERALS.contains(&lower.as_str()) {
        Some(true)
    } else if FALSE_LITERALS.contains(&lower.as_str()) {
        Some(false)
    } else {
        None
    }
}

/// 判断元素是否属于默认 beans 命名空间。
pub fn is_default_namespace(element: &Element) -> bool {
    element.namespace_uri.is_empty() || element.namespace_uri == BEANS_NAMESPACE_URI
}

/// 读取逗号/空格分隔的 `depends-on` 列表。
pub fn depends_on_list(element: &Element) -> Vec<String> {
    element
        .get_attribute(DEPENDS_ON_ATTRIBUTE)
        .map(|s| {
            s.split([',', ' '])
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// 将元素属性中除若干已知键外的所有属性收集为（名称, 值）对。
///
/// 用于读取 `<property>` / `<constructor-arg>` 等元素上的值属性。
pub fn value_attributes_excluding(element: &Element, excluded: &[&str]) -> Vec<(String, String)> {
    element
        .attributes
        .iter()
        .filter(|(k, _)| !excluded.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// 返回元素的本地名。
pub fn local_name(element: &Element) -> &str {
    &element.local_name
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_loader::Element;

    #[test]
    fn parses_boolean_variants() {
        assert_eq!(parse_boolean("true"), Some(true));
        assert_eq!(parse_boolean("FALSE"), Some(false));
        assert_eq!(parse_boolean("yes"), Some(true));
        assert_eq!(parse_boolean("nope"), None);
    }

    #[test]
    fn reads_names_split() {
        let mut el = Element::new("", "bean");
        el.set_attribute(NAME_ATTRIBUTE, "a, b;c d");
        assert_eq!(
            element_names(&el),
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string()
            ]
        );
    }
}

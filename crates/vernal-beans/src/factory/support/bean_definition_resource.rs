//! BeanDefinitionResource — Spring 风格 Bean 定义资源。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionResource`。
//!
//! 在 Spring 中，`BeanDefinitionResource` 实现了 `Resource` 接口，
//! 表示一个 Bean 定义的来源资源（例如 XML 片段或注解元数据）。
//! 在 vernal 中，它用于描述 Bean 定义的来源和内容。

/// Bean 定义资源。
///
/// 对应 Spring 的 `BeanDefinitionResource`。
///
/// 表示一个 Bean 定义的来源资源，包含描述和内容字符串。
/// 用于在 Bean 定义解析和注册流程中携带来源信息。
#[derive(Debug, Clone)]
pub struct BeanDefinitionResource {
    /// 资源描述（如文件名、注解来源等）
    description: String,
    /// 资源内容（如 XML 片段、注解元数据等）
    content: String,
}

impl BeanDefinitionResource {
    /// 创建新的 BeanDefinitionResource。
    ///
    /// # 参数
    /// - `description` — 资源描述
    /// - `content` — 资源内容
    pub fn new(description: String, content: String) -> Self {
        Self {
            description,
            content,
        }
    }

    /// 获取资源描述。
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取资源内容。
    pub fn content(&self) -> &str {
        &self.content
    }

    /// 获取资源内容长度。
    pub fn content_len(&self) -> usize {
        self.content.len()
    }

    /// 资源内容是否为空。
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// 判断资源是否包含指定文本。
    pub fn contains(&self, text: &str) -> bool {
        self.content.contains(text)
    }

    /// 更新资源内容。
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    /// 更新资源描述。
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// 判断资源内容是否以指定前缀开头。
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.content.starts_with(prefix)
    }

    /// 判断资源内容是否以指定后缀结尾。
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.content.ends_with(suffix)
    }

    /// 获取资源内容的行数。
    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            0
        } else {
            self.content.lines().count()
        }
    }

    /// 创建一个 XML 格式的 Bean 定义资源。
    pub fn xml(description: &str, bean_id: &str, bean_class: &str) -> Self {
        Self {
            description: description.to_string(),
            content: format!(r#"<bean id="{}" class="{}"/>"#, bean_id, bean_class),
        }
    }

    /// 创建一个注解格式的 Bean 定义资源。
    pub fn annotation(description: &str, annotation: &str) -> Self {
        Self {
            description: description.to_string(),
            content: annotation.to_string(),
        }
    }
}

impl std::fmt::Display for BeanDefinitionResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BeanDefinitionResource [{}]", self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resource_has_correct_fields() {
        let r =
            BeanDefinitionResource::new("test.xml".to_string(), "<bean id=\"foo\"/>".to_string());
        assert_eq!(r.description(), "test.xml");
        assert_eq!(r.content(), "<bean id=\"foo\"/>");
        assert_eq!(r.content_len(), 16);
        assert!(!r.is_empty());
    }

    #[test]
    fn empty_content_is_empty() {
        let r = BeanDefinitionResource::new("empty".to_string(), String::new());
        assert!(r.is_empty());
        assert_eq!(r.content_len(), 0);
    }

    #[test]
    fn contains_finds_substring() {
        let r = BeanDefinitionResource::new(
            "cfg".to_string(),
            "spring.datasource.url=jdbc:h2:mem".to_string(),
        );
        assert!(r.contains("datasource"));
        assert!(!r.contains("redis"));
    }

    #[test]
    fn display_format() {
        let r = BeanDefinitionResource::new("my-beans.xml".to_string(), String::new());
        assert_eq!(format!("{}", r), "BeanDefinitionResource [my-beans.xml]");
    }

    #[test]
    fn set_content_updates_value() {
        let mut r = BeanDefinitionResource::new("test".to_string(), "old".to_string());
        r.set_content("new content".to_string());
        assert_eq!(r.content(), "new content");
    }

    #[test]
    fn starts_with_and_ends_with() {
        let r = BeanDefinitionResource::new("test".to_string(), "<bean id=\"foo\"/>".to_string());
        assert!(r.starts_with("<bean"));
        assert!(r.ends_with("/>"));
        assert!(!r.starts_with("bean"));
    }

    #[test]
    fn line_count() {
        let r1 = BeanDefinitionResource::new("empty".to_string(), String::new());
        assert_eq!(r1.line_count(), 0);

        let r2 =
            BeanDefinitionResource::new("multi".to_string(), "line1\nline2\nline3".to_string());
        assert_eq!(r2.line_count(), 3);
    }

    #[test]
    fn xml_factory_creates_valid_resource() {
        let r = BeanDefinitionResource::xml("test.xml", "myBean", "com.example.MyClass");
        assert!(r.contains("myBean"));
        assert!(r.contains("com.example.MyClass"));
        assert!(r.starts_with("<bean"));
    }
}

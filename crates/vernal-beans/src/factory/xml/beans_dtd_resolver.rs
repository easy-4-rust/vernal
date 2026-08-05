//! BeansDtdResolver — Spring 风格的 Beans DTD 解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeansDtdResolver`。
//!
//! 在 Spring 中，`BeansDtdResolver` 用于解析 Spring beans DTD 的实体引用。
//! 当 XML 文档引用 `spring-beans.dtd` 时，此解析器从 classpath 中加载 DTD 文件。
//!
//! ## 设计说明
//!
//! 在 vernal 中，DTD 解析器返回内置的 DTD 定义字符串。

use crate::factory::xml::entity_resolver::{EntityResolver, ResolvedEntity};

/// Beans DTD 解析器。
///
/// 对应 Spring 的 `BeansDtdResolver`。
///
/// 解析 Spring beans DTD 的实体引用。
/// 内置了标准的 spring-beans.dtd 内容。
#[derive(Debug, Default)]
pub struct BeansDtdResolver {
    /// 已解析的实体计数。
    resolved_count: std::sync::atomic::AtomicU32,
}

impl BeansDtdResolver {
    /// 创建 Beans DTD 解析器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取已解析的实体数量。
    pub fn resolved_count(&self) -> u32 {
        self.resolved_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取内置 DTD 内容。
    fn builtin_dtd_content(&self) -> Vec<u8> {
        // 简化的 spring-beans.dtd 内容
        r#"<!ELEMENT beans (description?, import*, alias*, bean*)>
<!ATTLIST beans default-lazy-init CDATA #IMPLIED>
<!ATTLIST beans default-autowire CDATA #IMPLIED>
<!ELEMENT bean (description?, constructor-arg*, property*, lookup-method*, replaced-method*)>
<!ATTLIST bean id CDATA #IMPLIED>
<!ATTLIST bean class CDATA #REQUIRED>
<!ATTLIST bean scope CDATA #IMPLIED>"#
            .as_bytes()
            .to_vec()
    }
}

impl EntityResolver for BeansDtdResolver {
    fn resolve_entity(
        &self,
        public_id: Option<&str>,
        system_id: &str,
    ) -> Result<ResolvedEntity, Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否是 Spring beans DTD
        if system_id.contains("spring-beans") || system_id.contains("springframework") {
            self.resolved_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(ResolvedEntity {
                public_id: public_id.map(String::from),
                system_id: system_id.to_string(),
                content: self.builtin_dtd_content(),
            });
        }

        Err(format!(
            "BeansDtdResolver: unrecognized DTD system_id '{}'",
            system_id
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_spring_beans_dtd() {
        let resolver = BeansDtdResolver::new();
        let result = resolver.resolve_entity(
            Some("-//SPRING//DTD BEAN//EN"),
            "http://www.springframework.org/dtd/spring-beans.dtd",
        );
        assert!(result.is_ok());
        let entity = result.unwrap();
        assert!(String::from_utf8_lossy(&entity.content).contains("<!ELEMENT beans"));
    }

    #[test]
    fn rejects_unknown_dtd() {
        let resolver = BeansDtdResolver::new();
        let result = resolver.resolve_entity(None, "http://example.com/unknown.dtd");
        assert!(result.is_err());
    }

    #[test]
    fn tracks_resolved_count() {
        let resolver = BeansDtdResolver::new();
        assert_eq!(resolver.resolved_count(), 0);

        let _ = resolver.resolve_entity(None, "spring-beans.dtd");
        assert_eq!(resolver.resolved_count(), 1);

        let _ =
            resolver.resolve_entity(None, "http://www.springframework.org/dtd/spring-beans.dtd");
        assert_eq!(resolver.resolved_count(), 2);
    }
}

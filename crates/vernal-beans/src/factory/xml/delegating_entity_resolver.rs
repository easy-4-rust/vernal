//! DelegatingEntityResolver — Spring 风格的委托实体解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DelegatingEntityResolver`。
//!
//! 在 Spring 中，`DelegatingEntityResolver` 根据系统 ID 的后缀
//! 委托给不同的实体解析器：
//! - `.dtd` 后缀 → DTD 解析器
//! - `.xsd` 后缀 → Schema 解析器
//! - 其他 → 抛出异常
//!
//! ## 设计说明
//!
//! 在 vernal 中，此解析器维护 DTD 和 XSD 解析器的标识，
//! 根据后缀选择合适的解析策略。

use crate::factory::xml::entity_resolver::{EntityResolver, ResolvedEntity};

/// 委托实体解析器。
///
/// 对应 Spring 的 `DelegatingEntityResolver`。
///
/// 根据系统 ID 后缀委托给合适的实体解析器。
#[derive(Debug, Default)]
pub struct DelegatingEntityResolver {
    /// DTD 解析器标识。
    dtd_resolver_name: String,
    /// XSD 解析器标识。
    xsd_resolver_name: String,
}

impl DelegatingEntityResolver {
    /// 创建委托实体解析器。
    pub fn new() -> Self {
        Self {
            dtd_resolver_name: "BeansDtdResolver".to_string(),
            xsd_resolver_name: "PluggableSchemaResolver".to_string(),
        }
    }

    /// 设置 DTD 解析器名称。
    pub fn with_dtd_resolver(mut self, name: impl Into<String>) -> Self {
        self.dtd_resolver_name = name.into();
        self
    }

    /// 设置 XSD 解析器名称。
    pub fn with_xsd_resolver(mut self, name: impl Into<String>) -> Self {
        self.xsd_resolver_name = name.into();
        self
    }
}

impl EntityResolver for DelegatingEntityResolver {
    fn resolve_entity(
        &self,
        public_id: Option<&str>,
        system_id: &str,
    ) -> Result<ResolvedEntity, Box<dyn std::error::Error + Send + Sync>> {
        if system_id.ends_with(".dtd") || system_id.contains("dtd") {
            // 委托给 DTD 解析器
            Ok(ResolvedEntity {
                public_id: public_id.map(String::from),
                system_id: system_id.to_string(),
                content: format!("<!-- DTD resolved by {} -->", self.dtd_resolver_name)
                    .into_bytes(),
            })
        } else if system_id.ends_with(".xsd") || system_id.contains("xsd") {
            // 委托给 XSD 解析器
            Ok(ResolvedEntity {
                public_id: public_id.map(String::from),
                system_id: system_id.to_string(),
                content: format!("<!-- XSD resolved by {} -->", self.xsd_resolver_name)
                    .into_bytes(),
            })
        } else {
            Err(format!(
                "DelegatingEntityResolver: cannot resolve entity with system_id '{}'",
                system_id
            )
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegates_dtd_resolution() {
        let resolver = DelegatingEntityResolver::new();
        let result = resolver.resolve_entity(None, "spring-beans.dtd").unwrap();
        let content = String::from_utf8_lossy(&result.content);
        assert!(content.contains("DTD resolved"));
    }

    #[test]
    fn delegates_xsd_resolution() {
        let resolver = DelegatingEntityResolver::new();
        let result = resolver.resolve_entity(None, "spring-beans.xsd").unwrap();
        let content = String::from_utf8_lossy(&result.content);
        assert!(content.contains("XSD resolved"));
    }

    #[test]
    fn rejects_unknown_system_id() {
        let resolver = DelegatingEntityResolver::new();
        let result = resolver.resolve_entity(None, "unknown.xml");
        assert!(result.is_err());
    }

    #[test]
    fn custom_resolver_names() {
        let resolver = DelegatingEntityResolver::new()
            .with_dtd_resolver("CustomDTD")
            .with_xsd_resolver("CustomXSD");
        let dtd = resolver.resolve_entity(None, "test.dtd").unwrap();
        let xsd = resolver.resolve_entity(None, "test.xsd").unwrap();
        assert!(String::from_utf8_lossy(&dtd.content).contains("CustomDTD"));
        assert!(String::from_utf8_lossy(&xsd.content).contains("CustomXSD"));
    }
}

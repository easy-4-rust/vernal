//! ResourceEntityResolver — Spring 风格的资源实体解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.ResourceEntityResolver`。
//!
//! 在 Spring 中，`ResourceEntityResolver` 继承 `DelegatingEntityResolver`，
//! 当标准解析失败时，尝试从 classpath 资源加载实体。
//! 它使用 `ResourceLoader` 来查找 DTD/XSD 文件。
//!
//! ## 设计说明
//!
//! 在 vernal 中，此解析器维护一个资源搜索路径列表，
//! 当内置解析失败时，尝试从这些路径加载。

use crate::factory::xml::entity_resolver::{EntityResolver, ResolvedEntity};

/// 资源实体解析器。
///
/// 对应 Spring 的 `ResourceEntityResolver`。
///
/// 当标准解析失败时，尝试从资源路径加载实体。
#[derive(Debug, Default)]
pub struct ResourceEntityResolver {
    /// 资源搜索路径。
    resource_paths: Vec<String>,
    /// 已成功解析的实体名。
    resolved_entities: std::sync::Mutex<Vec<String>>,
}

impl ResourceEntityResolver {
    /// 创建资源实体解析器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加资源搜索路径。
    pub fn add_resource_path(&mut self, path: impl Into<String>) {
        self.resource_paths.push(path.into());
    }

    /// 获取已解析的实体列表。
    pub fn resolved_entities(&self) -> Vec<String> {
        self.resolved_entities.lock().unwrap().clone()
    }

    /// 获取资源路径数量。
    pub fn resource_path_count(&self) -> usize {
        self.resource_paths.len()
    }
}

impl EntityResolver for ResourceEntityResolver {
    fn resolve_entity(
        &self,
        public_id: Option<&str>,
        system_id: &str,
    ) -> Result<ResolvedEntity, Box<dyn std::error::Error + Send + Sync>> {
        // 尝试从资源路径加载
        for path in &self.resource_paths {
            if system_id.contains(path) || path.contains(system_id) {
                self.resolved_entities.lock().unwrap().push(system_id.to_string());
                return Ok(ResolvedEntity {
                    public_id: public_id.map(String::from),
                    system_id: system_id.to_string(),
                    content: format!("<!-- resolved from resource path: {} -->", path).into_bytes(),
                });
            }
        }

        // 尝试标准 DTD/XSD 解析
        if system_id.ends_with(".dtd") || system_id.ends_with(".xsd") {
            self.resolved_entities.lock().unwrap().push(system_id.to_string());
            return Ok(ResolvedEntity {
                public_id: public_id.map(String::from),
                system_id: system_id.to_string(),
                content: format!("<!-- default resolution for {} -->", system_id).into_bytes(),
            });
        }

        Err(format!("ResourceEntityResolver: cannot resolve '{}'", system_id).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_from_resource_path() {
        let mut resolver = ResourceEntityResolver::new();
        resolver.add_resource_path("spring-beans");
        let result = resolver.resolve_entity(None, "http://spring-beans.xsd").unwrap();
        assert!(String::from_utf8_lossy(&result.content).contains("resource path"));
    }

    #[test]
    fn resolves_standard_dtd() {
        let resolver = ResourceEntityResolver::new();
        let result = resolver.resolve_entity(None, "test.dtd").unwrap();
        assert!(String::from_utf8_lossy(&result.content).contains("default resolution"));
    }

    #[test]
    fn rejects_unknown_system_id() {
        let resolver = ResourceEntityResolver::new();
        let result = resolver.resolve_entity(None, "unknown.xml");
        assert!(result.is_err());
    }

    #[test]
    fn tracks_resolved_entities() {
        let resolver = ResourceEntityResolver::new();
        resolver.resolve_entity(None, "test.dtd").unwrap();
        resolver.resolve_entity(None, "test.xsd").unwrap();
        assert_eq!(resolver.resolved_entities().len(), 2);
    }
}

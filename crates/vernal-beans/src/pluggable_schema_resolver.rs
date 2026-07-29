//! PluggableSchemaResolver — 可插拔的 XML Schema 实体解析器。
//!
//! 对应 Java 类：
//! `org.springframework.beans.factory.xml.PluggableSchemaResolver`。
//!
//! 把对 Spring XML Schema（`http://www.springframework.org/schema/.../*.xsd`）
//! 的系统引用重定向到本地资源。映射表通常从
//! `META-INF/spring.schemas` 风格的条目加载（这里以注册 API 提供）。

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::sync::Arc;

use crate::entity_resolver::{EntityResolver, ResolvedEntity};

/// 可插拔的 XML Schema 实体解析器。
///
/// 对应 Spring 的 `PluggableSchemaResolver`。
///
/// 维护一个 system ID（URL）→ 本地类路径/文件路径 的映射表。
/// 解析时先查映射，找不到则回退到以文件名在当前目录查找。
pub struct PluggableSchemaResolver {
    /// 系统 ID → 本地路径映射。
    schema_mappings: HashMap<String, String>,
}

impl PluggableSchemaResolver {
    /// 创建空的解析器。
    pub fn new() -> Self {
        Self {
            schema_mappings: HashMap::new(),
        }
    }

    /// 创建并从"每行一条 `url=localpath`"的文本中加载映射。
    ///
    /// 行格式形如：
    /// ```text
    /// http://www.springframework.org/schema/beans/spring-beans.xsd=spring-beans.xsd
    /// ```
    /// `#` 开头或空行被忽略。
    pub fn from_text(text: &str) -> Self {
        let mut resolver = Self::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((url, local)) = line.split_once('=') {
                resolver
                    .schema_mappings
                    .insert(url.trim().to_string(), local.trim().to_string());
            }
        }
        resolver
    }

    /// 注册一条系统 ID → 本地路径映射。
    pub fn register(&mut self, system_id: impl Into<String>, local_path: impl Into<String>) {
        self.schema_mappings
            .insert(system_id.into(), local_path.into());
    }

    /// 返回映射表引用。
    pub fn schema_mappings(&self) -> &HashMap<String, String> {
        &self.schema_mappings
    }
}

impl Default for PluggableSchemaResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PluggableSchemaResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluggableSchemaResolver")
            .field("mapping_count", &self.schema_mappings.len())
            .finish()
    }
}

impl EntityResolver for PluggableSchemaResolver {
    fn resolve_entity(
        &self,
        public_id: &str,
        system_id: &str,
    ) -> Result<Option<ResolvedEntity>, Box<dyn std::error::Error + Send + Sync>> {
        // 仅处理 .xsd 引用。
        if !system_id.ends_with(".xsd") {
            return Ok(None);
        }
        if let Some(local_path) = self.schema_mappings.get(system_id) {
            if let Ok(file) = File::open(local_path) {
                return Ok(Some(ResolvedEntity::new(
                    public_id,
                    system_id,
                    Arc::new(file),
                )));
            }
        }
        // 回退：按文件名在当前目录尝试打开。
        let filename = system_id.rsplit('/').next().unwrap_or(system_id);
        if let Ok(file) = File::open(filename) {
            return Ok(Some(ResolvedEntity::new(
                public_id,
                system_id,
                Arc::new(file),
            )));
        }
        Ok(None)
    }
}

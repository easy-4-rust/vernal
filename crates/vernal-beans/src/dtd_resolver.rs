//! DtdResolver — Spring DTD 实体解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeansDtdResolver`。
//!
//! 把对 Spring beans DTD（`spring-beans.dtd`、`spring-beans-*.dtd`）
//! 的 `http://www.springframework.org/dtd/...` 系统引用重定向到本地
//! 资源（避免网络访问）。本实现维护一个 system ID → 本地路径的映射表。

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::sync::Arc;

use crate::entity_resolver::{EntityResolver, ResolvedEntity};

/// Spring beans DTD 解析器。
///
/// 对应 Spring 的 `BeansDtdResolver`。
///
/// 识别形如 `http(s)://www.springframework.org/dtd/spring-beans*.dtd`
/// 的系统 ID，并尝试从已注册的本地映射或同目录文件解析。
pub struct DtdResolver {
    /// 系统 ID → 本地文件路径映射。
    mappings: HashMap<String, String>,
}

impl DtdResolver {
    /// 创建新的解析器，预置 Spring beans DTD 的常见映射。
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        let dtds = [
            "spring-beans.dtd",
            "spring-beans-2.0.dtd",
            "spring-beans-3.0.dtd",
            "spring-beans-4.0.dtd",
        ];
        for dtd in dtds {
            mappings.insert(
                format!("http://www.springframework.org/dtd/{dtd}"),
                format!("org/springframework/beans/factory/xml/{dtd}"),
            );
            mappings.insert(
                format!("https://www.springframework.org/dtd/{dtd}"),
                format!("org/springframework/beans/factory/xml/{dtd}"),
            );
        }
        Self { mappings }
    }

    /// 追加一条自定义系统 ID → 本地路径映射。
    pub fn register(&mut self, system_id: impl Into<String>, local_path: impl Into<String>) {
        self.mappings.insert(system_id.into(), local_path.into());
    }

    /// 返回映射表引用。
    pub fn mappings(&self) -> &HashMap<String, String> {
        &self.mappings
    }
}

impl Default for DtdResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DtdResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DtdResolver")
            .field("mapping_count", &self.mappings.len())
            .finish()
    }
}

impl EntityResolver for DtdResolver {
    fn resolve_entity(
        &self,
        public_id: &str,
        system_id: &str,
    ) -> Result<Option<ResolvedEntity>, Box<dyn std::error::Error + Send + Sync>> {
        // 仅处理 Spring beans DTD 引用。
        let is_beans_dtd = system_id.contains("/spring-beans") && system_id.ends_with(".dtd");
        if !is_beans_dtd {
            return Ok(None);
        }
        // 优先查映射。
        if let Some(local_path) = self.mappings.get(system_id) {
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

/// 历史别名：Spring 中名为 `BeansDtdResolver`。
pub type BeansDtdResolver = DtdResolver;

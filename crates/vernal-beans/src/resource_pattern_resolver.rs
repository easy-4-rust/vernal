//! ResourcePatternResolver — 资源模式解析器 trait。
//!
//! 对应 Java 类：`org.springframework.core.io.support.ResourcePatternResolver`。
//!
//! 扩展 [`ResourceLoader`]，支持按带通配符的模式（如
//! `classpath*:com/acme/**/*.xml`）解析出多个 [`Resource`]。

use std::sync::Arc;

use crate::default_resource_loader::ResourceLoader;
use crate::resource::Resource;

/// `classpath*:` 前缀常量，表示在所有类路径根中匹配。
pub const CLASSPATH_ALL_URL_PREFIX: &str = "classpath*:";

/// Spring 风格的资源模式解析器 trait。
///
/// 对应 Spring 的 `ResourcePatternResolver`。
///
/// 在 [`ResourceLoader`] 基础上增加按模式解析多个资源的能力。
pub trait ResourcePatternResolver: ResourceLoader {
    /// 按模式字符串解析出匹配的资源列表。
    ///
    /// 对应 Spring 的 `Resource[] getResources(String locationPattern)`。
    ///
    /// # 错误
    ///
    /// 模式非法或底层 I/O 失败时返回 `Err`。
    fn get_resources(
        &self,
        location_pattern: &str,
    ) -> Result<Vec<Arc<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>>;
}

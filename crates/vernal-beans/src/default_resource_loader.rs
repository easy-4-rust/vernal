//! DefaultResourceLoader — 默认资源加载器与 `ResourceLoader` trait。
//!
//! 对应 Java 类：`org.springframework.core.io.ResourceLoader`
//! 与 `org.springframework.core.io.DefaultResourceLoader`。
//!
//! 定义资源加载的统一入口 [`ResourceLoader`]，并提供默认实现
//! [`DefaultResourceLoader`]，它按 `classpath:`、`file:` 及其它 URL
//! 前缀解析 [`Resource`]，并支持注册自定义 [`ProtocolResolver`]。

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::classpath_resource::ClassPathResource;
use crate::filesystem_resource::FileSystemResource;
use crate::protocol_resolver::ProtocolResolver;
use crate::resource::Resource;
use crate::url_resource::UrlResource;

/// Spring 风格的资源加载器 trait。
///
/// 对应 Spring 的 `ResourceLoader`。
///
/// 负责根据资源位置字符串返回一个 [`Resource`] 实例。
pub trait ResourceLoader: Send + Sync {
    /// 根据位置字符串获取资源。
    ///
    /// 对应 Spring 的 `Resource getResource(String location)`。
    fn get_resource(&self, location: &str) -> Arc<dyn Resource>;

    /// 返回类加载器描述名称（近似 Spring 的 `getClassLoader()`）。
    fn class_loader_name(&self) -> Option<&str> {
        None
    }
}

/// `classpath:` 协议前缀常量。
pub const CLASSPATH_URL_PREFIX: &str = "classpath:";

/// `file:` 协议前缀常量。
pub const FILE_URL_PREFIX: &str = "file:";

/// `url:` 协议前缀常量（Spring 扩展，显式标记 URL）。
pub const URL_PROTOCOL_PREFIX: &str = "url:";

/// 默认资源加载器实现。
///
/// 对应 Spring 的 `DefaultResourceLoader`。
///
/// 解析顺序：
/// 1. 已注册的 [`ProtocolResolver`]（按注册顺序）；
/// 2. `classpath:` 前缀 → [`ClassPathResource`]；
/// 3. `file:` 前缀 → [`FileSystemResource`]；
/// 4. 其它显式 URL scheme（`http:`、`https:` 等）→ [`UrlResource`]；
/// 5. 兜底 → 相对当前工作目录的 [`FileSystemResource`]。
pub struct DefaultResourceLoader {
    /// 类路径搜索根目录。
    classpath_roots: Vec<PathBuf>,
    /// 类加载器描述名称。
    class_loader_name: Option<String>,
    /// 已注册的协议解析器。
    protocol_resolvers: Vec<Arc<dyn ProtocolResolver>>,
}

impl DefaultResourceLoader {
    /// 创建空的默认资源加载器。
    pub fn new() -> Self {
        Self {
            classpath_roots: Vec::new(),
            class_loader_name: None,
            protocol_resolvers: Vec::new(),
        }
    }

    /// 添加一个类路径搜索根目录。
    pub fn add_classpath_root(&mut self, root: impl Into<PathBuf>) -> &mut Self {
        self.classpath_roots.push(root.into());
        self
    }

    /// 设置类加载器描述名称。
    pub fn set_class_loader_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.class_loader_name = Some(name.into());
        self
    }

    /// 注册一个协议解析器。
    ///
    /// 对应 Spring 的 `addProtocolResolver`。
    pub fn add_protocol_resolver(&mut self, resolver: Arc<dyn ProtocolResolver>) -> &mut Self {
        self.protocol_resolvers.push(resolver);
        self
    }

    /// 返回类路径搜索根目录。
    pub fn classpath_roots(&self) -> &[PathBuf] {
        &self.classpath_roots
    }

    /// 返回已注册的协议解析器。
    pub fn protocol_resolvers(&self) -> &[Arc<dyn ProtocolResolver>] {
        &self.protocol_resolvers
    }
}

impl Default for DefaultResourceLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DefaultResourceLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultResourceLoader")
            .field("classpath_roots", &self.classpath_roots)
            .field("class_loader_name", &self.class_loader_name)
            .field("protocol_resolver_count", &self.protocol_resolvers.len())
            .finish()
    }
}

impl ResourceLoader for DefaultResourceLoader {
    fn get_resource(&self, location: &str) -> Arc<dyn Resource> {
        // 1. 询问已注册的协议解析器。
        for resolver in &self.protocol_resolvers {
            if let Some(resource) = resolver.resolve(location) {
                return resource;
            }
        }

        // 2. classpath: 前缀。
        if let Some(rest) = location.strip_prefix(CLASSPATH_URL_PREFIX) {
            let mut cp = ClassPathResource::new(PathBuf::from(rest))
                .with_roots(self.classpath_roots.clone());
            if let Some(ref name) = self.class_loader_name {
                cp = cp.with_class_loader(name.clone());
            }
            return Arc::new(cp);
        }

        // 3. file: 前缀。
        if let Some(rest) = location.strip_prefix(FILE_URL_PREFIX) {
            return Arc::new(FileSystemResource::from_str(rest));
        }

        // 4. url: 显式前缀。
        if let Some(rest) = location.strip_prefix(URL_PROTOCOL_PREFIX) {
            return Arc::new(UrlResource::new(rest));
        }

        // 5. 其它已知 URL scheme。
        if has_known_url_scheme(location) {
            return Arc::new(UrlResource::new(location));
        }

        // 6. 兜底：当作文件系统相对路径。
        Arc::new(FileSystemResource::from_str(location))
    }

    fn class_loader_name(&self) -> Option<&str> {
        self.class_loader_name.as_deref()
    }
}

/// 判断字符串是否以已知的 URL scheme 开头。
///
/// scheme 形如 `alpha://` 或 `alpha:`。
fn has_known_url_scheme(location: &str) -> bool {
    let Some((scheme, _)) = location.split_once(':') else {
        return false;
    };
    if scheme.is_empty() {
        return false;
    }
    scheme
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
}

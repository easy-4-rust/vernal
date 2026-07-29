//! ClassPathResource — 类路径资源。
//!
//! 对应 Java 类：`org.springframework.core.io.ClassPathResource`。
//!
//! Rust 没有 JVM 意义上的 classpath，本实现将其语义映射为：以一组
//! "搜索根目录"为基础、按相对路径查找的资源。若未配置根目录，则回退
//! 为相对当前工作目录解析（语义近似 JVM 默认 classpath 行为）。

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::resource::Resource;

/// 类路径资源。
///
/// 对应 Spring 的 `ClassPathResource`。
///
/// 持有一个相对于类路径根的路径，并提供一组可配置的搜索根目录
/// （通过 [`ClassPathResource::with_roots`]）。默认仅相对当前工作目录解析。
#[derive(Clone)]
pub struct ClassPathResource {
    /// 类路径相对路径。
    path: PathBuf,
    /// 可选的类加载器/根目录（描述用）。
    class_loader_name: Option<String>,
    /// 搜索根目录列表。
    roots: Vec<PathBuf>,
}

impl ClassPathResource {
    /// 从相对路径创建。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            class_loader_name: None,
            roots: Vec::new(),
        }
    }

    /// 设置类加载器描述名称。
    pub fn with_class_loader(mut self, name: impl Into<String>) -> Self {
        self.class_loader_name = Some(name.into());
        self
    }

    /// 追加搜索根目录。
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.roots.push(root.into());
        self
    }

    /// 设置搜索根目录列表。
    pub fn with_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.roots = roots;
        self
    }

    /// 返回类路径相对路径引用。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 返回搜索根目录列表引用。
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// 解析得到实际存在的绝对路径。
    ///
    /// 依次在已配置的根目录下查找，找不到则回退到路径本身。
    fn resolve_existing(&self) -> Option<PathBuf> {
        for root in &self.roots {
            let candidate = root.join(&self.path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if self.path.exists() {
            return Some(self.path.clone());
        }
        None
    }
}

impl fmt::Debug for ClassPathResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClassPathResource")
            .field("path", &self.path.display().to_string())
            .field(
                "class_loader",
                &self.class_loader_name.clone().unwrap_or_default(),
            )
            .field("roots", &self.roots.len())
            .finish()
    }
}

impl Resource for ClassPathResource {
    fn exists(&self) -> bool {
        self.resolve_existing().is_some()
    }

    fn is_readable(&self) -> bool {
        self.resolve_existing()
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    fn url(&self) -> Option<String> {
        Some(format!("classpath:{}", self.path.display()))
    }

    fn file_path(&self) -> Option<String> {
        self.resolve_existing()
            .map(|p| p.to_string_lossy().into_owned())
    }

    fn filename(&self) -> Option<String> {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
    }

    fn description(&self) -> String {
        let loader = self
            .class_loader_name
            .as_deref()
            .map(|n| format!("[{n}]"))
            .unwrap_or_default();
        format!("class path resource {}{}", loader, self.path.display())
    }

    fn input_stream(
        &self,
    ) -> Result<Box<dyn Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        let resolved = self.resolve_existing().unwrap_or_else(|| self.path.clone());
        let file = File::open(&resolved)?;
        Ok(Box::new(file))
    }

    fn content_length(&self) -> Option<u64> {
        self.resolve_existing()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
    }
}

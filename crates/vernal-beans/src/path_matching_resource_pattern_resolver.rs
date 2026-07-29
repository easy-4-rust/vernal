//! PathMatchingResourcePatternResolver — 基于路径模式匹配的资源解析器。
//!
//! 对应 Java 类：
//! `org.springframework.core.io.support.PathMatchingResourcePatternResolver`。
//!
//! 在 [`DefaultResourceLoader`] 基础上实现 [`ResourcePatternResolver`]，
//! 支持 `classpath:`、`classpath*:` 与文件系统模式（含 `**`、`*`、`?`）。

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::classpath_resource::ClassPathResource;
use crate::default_resource_loader::{DefaultResourceLoader, ResourceLoader};
use crate::resource::Resource;
use crate::resource_pattern_resolver::{CLASSPATH_ALL_URL_PREFIX, ResourcePatternResolver};

/// 基于路径模式匹配的资源模式解析器。
///
/// 对应 Spring 的 `PathMatchingResourcePatternResolver`。
///
/// 包装一个 [`DefaultResourceLoader`]，对单资源（无通配符）请求直接代理；
/// 对带通配符的模式则在文件系统中递归扫描并用 glob 匹配。
pub struct PathMatchingResourcePatternResolver {
    /// 内部资源加载器。
    loader: DefaultResourceLoader,
}

impl PathMatchingResourcePatternResolver {
    /// 用给定加载器创建。
    pub fn new(loader: DefaultResourceLoader) -> Self {
        Self { loader }
    }

    /// 创建默认实例。
    pub fn with_defaults() -> Self {
        Self::new(DefaultResourceLoader::new())
    }

    /// 返回内部加载器引用。
    pub fn loader(&self) -> &DefaultResourceLoader {
        &self.loader
    }

    /// 返回内部加载器的可变引用。
    pub fn loader_mut(&mut self) -> &mut DefaultResourceLoader {
        &mut self.loader
    }

    /// 在文件系统根下递归收集匹配 `pattern`（glob）的所有资源。
    fn find_filesystem_matches(
        &self,
        root: &Path,
        pattern: &str,
    ) -> Result<Vec<Arc<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        if !root.exists() {
            return Ok(results);
        }
        walk_and_collect(root, root, pattern, &mut results)?;
        // 去重并保持稳定顺序。
        results.sort_by(|a, b| a.description().cmp(&b.description()));
        results.dedup_by(|a, b| a.description() == b.description());
        Ok(results)
    }

    /// 在类路径根下递归收集匹配模式的所有资源。
    fn find_classpath_matches(
        &self,
        pattern: &str,
    ) -> Result<Vec<Arc<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let roots = if self.loader.classpath_roots().is_empty() {
            vec![PathBuf::from(".")]
        } else {
            self.loader.classpath_roots().to_vec()
        };
        for root in &roots {
            if root.exists() {
                walk_and_collect_classpath(root, pattern, &self.loader, &mut results)?;
            }
        }
        results.sort_by(|a, b| a.description().cmp(&b.description()));
        results.dedup_by(|a, b| a.description() == b.description());
        Ok(results)
    }
}

impl fmt::Debug for PathMatchingResourcePatternResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PathMatchingResourcePatternResolver")
            .field("loader", &self.loader)
            .finish()
    }
}

impl ResourceLoader for PathMatchingResourcePatternResolver {
    fn get_resource(&self, location: &str) -> Arc<dyn Resource> {
        // 无通配符 → 直接代理单资源。
        if !has_wildcard(location) {
            return self.loader.get_resource(location);
        }
        // 有通配符但调用方只要单个：返回第一个匹配项，或描述性占位资源。
        match self.get_resources(location) {
            Ok(resources) if !resources.is_empty() => Arc::clone(&resources[0]),
            _ => self.loader.get_resource(location),
        }
    }

    fn class_loader_name(&self) -> Option<&str> {
        self.loader.class_loader_name()
    }
}

impl ResourcePatternResolver for PathMatchingResourcePatternResolver {
    fn get_resources(
        &self,
        location_pattern: &str,
    ) -> Result<Vec<Arc<dyn Resource>>, Box<dyn std::error::Error + Send + Sync>> {
        // classpath*:
        if let Some(rest) = location_pattern.strip_prefix(CLASSPATH_ALL_URL_PREFIX) {
            return self.find_classpath_matches(rest);
        }
        // classpath: 单根匹配
        if let Some(rest) = location_pattern.strip_prefix("classpath:") {
            if !has_wildcard(rest) {
                return Ok(vec![self.loader.get_resource(location_pattern)]);
            }
            let mut roots = self.loader.classpath_roots().to_vec();
            if roots.is_empty() {
                roots.push(PathBuf::from("."));
            }
            let mut results = Vec::new();
            for root in &roots {
                walk_and_collect_classpath(root, rest, &self.loader, &mut results)?;
            }
            return Ok(results);
        }

        // 文件系统模式（带 file: 前缀或裸路径）。
        let stripped = location_pattern
            .strip_prefix("file:")
            .unwrap_or(location_pattern);
        if !has_wildcard(stripped) {
            return Ok(vec![self.loader.get_resource(location_pattern)]);
        }
        // 将模式拆为"确定根目录"与"通配部分"。
        let (root_dir, pattern_part) = split_root_and_pattern(stripped);
        self.find_filesystem_matches(root_dir.as_path(), &pattern_part)
    }
}

/// 判断模式是否含通配符。
fn has_wildcard(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

/// 把模式拆成不含通配符的最长根目录与剩余通配部分。
fn split_root_and_pattern(pattern: &str) -> (PathBuf, String) {
    let path = Path::new(pattern);
    let mut root_components: Vec<String> = Vec::new();
    let mut rest_components: Vec<String> = Vec::new();
    let mut hit_wildcard = false;
    for comp in path.components() {
        let s = comp.as_os_str().to_string_lossy().into_owned();
        if !hit_wildcard && !has_wildcard(&s) {
            root_components.push(s);
        } else {
            hit_wildcard = true;
            rest_components.push(s);
        }
    }
    let root = if root_components.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(root_components.join(std::path::MAIN_SEPARATOR_STR))
    };
    let pattern_part = rest_components.join(std::path::MAIN_SEPARATOR_STR);
    (root, pattern_part)
}

/// 递归遍历目录，收集匹配 glob 模式的文件资源。
fn walk_and_collect(
    base: &Path,
    current: &Path,
    pattern: &str,
    results: &mut Vec<Arc<dyn Resource>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_and_collect(base, &path, pattern, results)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(base).unwrap_or(&path);
            let rel_str = relative.to_string_lossy();
            if glob_match(pattern, &rel_str) {
                results.push(Arc::new(
                    crate::filesystem_resource::FileSystemResource::new(path),
                ));
            }
        }
    }
    Ok(())
}

/// 递归遍历类路径根，收集匹配模式的 ClassPathResource。
fn walk_and_collect_classpath(
    root: &Path,
    pattern: &str,
    loader: &DefaultResourceLoader,
    results: &mut Vec<Arc<dyn Resource>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_and_collect_classpath(&path, pattern, loader, results)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let rel_str = relative.to_string_lossy();
            if glob_match(pattern, &rel_str) {
                let mut cp = ClassPathResource::new(relative.to_path_buf())
                    .with_roots(loader.classpath_roots().to_vec());
                if let Some(name) = loader.class_loader_name() {
                    cp = cp.with_class_loader(name);
                }
                results.push(Arc::new(cp));
            }
        }
    }
    Ok(())
}

/// 简化版 glob 匹配，支持 `*`（单层任意，不含分隔符）、
/// `**`（跨层任意）、`?`（单字符）。
///
/// 先把模式与输入按路径分隔符切成片段，再分段匹配：`**` 可匹配零或多个片段。
fn glob_match(pattern: &str, input: &str) -> bool {
    let pat_segments: Vec<&str> = pattern.split(['/', '\\']).collect();
    let in_segments: Vec<&str> = input.split(['/', '\\']).collect();
    segments_match(&pat_segments, &in_segments)
}

fn segments_match(pat: &[&str], input: &[&str]) -> bool {
    match (pat.split_first(), input.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((p, prest)), input_rest) => {
            if *p == "**" {
                // ** 匹配零个或多个输入片段：
                // - 零个：用 prest 匹配当前（完整）输入；
                // - 一个或多个：** 吞掉第一个输入片段后继续。
                if segments_match(prest, input) {
                    return true;
                }
                match input_rest {
                    Some((_, irest)) => segments_match(pat, irest),
                    None => false,
                }
            } else {
                match input_rest {
                    Some((i, irest)) => segment_value_match(p, i) && segments_match(prest, irest),
                    None => false,
                }
            }
        }
    }
}

/// 单片段（不含路径分隔符）的 glob 值匹配：支持 `*` 与 `?`。
fn segment_value_match(pat: &str, input: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    let i: Vec<char> = input.chars().collect();
    value_match(&p, &i)
}

fn value_match(p: &[char], i: &[char]) -> bool {
    match (p.split_first(), i.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(('*', prest)), Some((_, irest))) => {
            // * 匹配零或多个字符。
            value_match(prest, i) || value_match(p, irest)
        }
        (Some(('*', _)), None) => value_match(&p[1..], &[]),
        (Some(('?', prest)), Some((_, irest))) => value_match(prest, irest),
        (Some(('?', _)), None) => false,
        (Some((pc, prest)), Some((ic, irest))) if *pc == *ic => value_match(prest, irest),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_double_star() {
        assert!(glob_match("com/**/*.xml", "com/a/b/c.xml"));
        assert!(glob_match("**/*.xml", "x.xml"));
        assert!(!glob_match("com/**/*.xml", "com/a/b/c.txt"));
    }

    #[test]
    fn glob_matches_single_star() {
        assert!(glob_match("*.xml", "a.xml"));
        assert!(!glob_match("*.xml", "dir/a.xml"));
        assert!(glob_match("a/*", "a/b"));
    }

    #[test]
    fn glob_matches_question_mark() {
        assert!(glob_match("a?.txt", "ab.txt"));
        assert!(!glob_match("a?.txt", "abc.txt"));
    }
}

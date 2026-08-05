//! 路径匹配资源模式解析器。
//!
//! 对标 Spring `org.springframework.core.io.support.PathMatchingResourcePatternResolver`。

use std::io;
use std::path::{Path, PathBuf};

use crate::io::Resource;
use crate::io::default_resource_loader::DefaultResourceLoader;
use crate::io::resource_loader::ResourceLoader;
use crate::util::ant_path_matcher::AntPathMatcher;
use crate::util::path_matcher::PathMatcher;

use super::resource_pattern_utils::ResourcePatternUtils;

/// 路径匹配资源模式解析器。
///
/// 对应 Java: org.springframework.core.io.support.PathMatchingResourcePatternResolver
///
/// Spring 语义：按 Ant 风格模式解析资源——
/// 无模式字符时委托资源加载器返回单资源；`file:` 模式扫描文件系统目录；
/// `classpath:` / `classpath*:` 模式对候选类路径路径做模式匹配（vernal 中
/// 候选集由 [`Self::set_candidates`] 显式提供，对标 `ClassLoader` 资源枚举）。
pub struct PathMatchingResourcePatternResolver {
    resource_loader: DefaultResourceLoader,
    path_matcher: AntPathMatcher,
    candidates: Vec<String>,
}

impl PathMatchingResourcePatternResolver {
    /// 创建解析器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource_loader: DefaultResourceLoader::new(),
            path_matcher: AntPathMatcher::new(),
            candidates: Vec::new(),
        }
    }

    /// 设置类路径候选路径（对标 `ClassLoader` 枚举出的资源路径）。
    pub fn set_candidates(&mut self, candidates: Vec<String>) {
        self.candidates = candidates;
    }

    /// 追加类路径候选路径。
    pub fn add_candidate(&mut self, candidate: impl Into<String>) {
        self.candidates.push(candidate.into());
    }

    /// 按模式解析资源。
    ///
    /// 对标 Spring `getResources(String locationPattern)`：
    /// 非模式位置直接委托加载器；模式位置按来源（文件系统 / 类路径）匹配。
    ///
    /// # 错误
    ///
    /// 位置非法或文件系统遍历失败时返回 [`std::io::Error`]。
    pub fn get_resources(&self, location_pattern: &str) -> io::Result<Vec<Box<dyn Resource>>> {
        if !ResourcePatternUtils::is_pattern(location_pattern) {
            // 对标 Spring：无模式时返回单资源（即使不存在）
            return Ok(vec![self.resource_loader.get_resource(location_pattern)?]);
        }
        if let Some(rest) = location_pattern.strip_prefix("classpath*:") {
            return self.find_classpath_matching(rest);
        }
        if let Some(rest) = location_pattern.strip_prefix("classpath:") {
            return self.find_classpath_matching(rest);
        }
        self.find_file_matching(location_pattern)
    }

    /// 类路径模式匹配：对候选路径逐一做 Ant 匹配。
    fn find_classpath_matching(&self, sub_pattern: &str) -> io::Result<Vec<Box<dyn Resource>>> {
        let mut result = Vec::new();
        for candidate in &self.candidates {
            if self.path_matcher.matches(sub_pattern, candidate) {
                result.push(
                    self.resource_loader
                        .get_resource(&format!("classpath:{candidate}"))?,
                );
            }
        }
        Ok(result)
    }

    /// 文件系统模式匹配：遍历根目录并匹配相对路径。
    fn find_file_matching(&self, location_pattern: &str) -> io::Result<Vec<Box<dyn Resource>>> {
        let pattern = location_pattern
            .strip_prefix("file:")
            .unwrap_or(location_pattern);
        let (root, sub_pattern) = Self::extract_root_dir(pattern);
        let root_path = if root.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(root.trim_end_matches('/'))
        };
        let mut files = Vec::new();
        Self::walk_dir(&root_path, &mut files)?;

        let mut result = Vec::new();
        for file in files {
            if let Ok(relative) = file.strip_prefix(&root_path) {
                let relative_str = relative.to_string_lossy();
                if self.path_matcher.matches(&sub_pattern, &relative_str) {
                    result.push(
                        self.resource_loader
                            .get_resource(&format!("file:{}", file.display()))?,
                    );
                }
            }
        }
        Ok(result)
    }

    /// 提取模式的根目录与剩余子模式（对标 Spring `findPathMatchingResources` 的前置拆分）。
    fn extract_root_dir(pattern: &str) -> (String, String) {
        let first_wildcard = pattern.find(['*', '?']).unwrap_or(pattern.len());
        let prefix = &pattern[..first_wildcard];
        let last_slash = prefix.rfind('/').map_or(0, |i| i + 1);
        (
            pattern[..last_slash].to_string(),
            pattern[last_slash..].to_string(),
        )
    }

    /// 递归收集目录下全部文件。
    fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                Self::walk_dir(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
}

impl Default for PathMatchingResourcePatternResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLoader for PathMatchingResourcePatternResolver {
    fn load(&self, location: &str) -> io::Result<Box<dyn Resource>> {
        self.resource_loader.get_resource(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 创建并填充唯一临时目录（std-only，测试后清理）。
    fn make_temp_dir(tag: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("vernal-pmrr-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("a.txt"), "A").unwrap();
        fs::write(dir.join("b.properties"), "B").unwrap();
        fs::write(dir.join("nested").join("c.txt"), "C").unwrap();
        dir
    }

    #[test]
    fn matches_filesystem_pattern() {
        // A 类（合同对齐）：对标 Spring 文件系统模式匹配
        let dir = make_temp_dir("fs");
        let resolver = PathMatchingResourcePatternResolver::new();
        let pattern = format!("file:{}/a.txt", dir.display());
        let resources = resolver.get_resources(&pattern).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].read_string().unwrap(), "A");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn recursive_wildcard_spans_directories() {
        // A 类（合同对齐）：`**` 跨目录匹配（对标 Spring `**/*.txt`）
        let dir = make_temp_dir("recursive");
        let resolver = PathMatchingResourcePatternResolver::new();
        let pattern = format!("file:{}/*.txt", dir.display());
        let flat = resolver.get_resources(&pattern).unwrap();
        assert_eq!(flat.len(), 1);

        let pattern = format!("file:{}/**/*.txt", dir.display());
        let recursive = resolver.get_resources(&pattern).unwrap();
        assert_eq!(recursive.len(), 2);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unmatched_filesystem_pattern_yields_empty() {
        // B 类（边界行为）：无匹配文件时返回空集合
        let dir = make_temp_dir("unmatched");
        let resolver = PathMatchingResourcePatternResolver::new();
        let pattern = format!("file:{}/*.xml", dir.display());
        assert!(resolver.get_resources(&pattern).unwrap().is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn matches_classpath_candidates() {
        // A 类（合同对齐）：对标 Spring 类路径模式匹配
        let mut resolver = PathMatchingResourcePatternResolver::new();
        resolver.set_candidates(vec![
            "config/a.properties".to_string(),
            "config/b.properties".to_string(),
            "other/c.txt".to_string(),
        ]);
        let resources = resolver
            .get_resources("classpath*:config/*.properties")
            .unwrap();
        assert_eq!(resources.len(), 2);
        let names: Vec<String> = resources
            .iter()
            .map(|r| r.filename().unwrap_or("").to_string())
            .collect();
        assert_eq!(names, vec!["a.properties", "b.properties"]);
    }

    #[test]
    fn unmatched_classpath_pattern_yields_empty() {
        // B 类（边界行为）：候选不匹配时为空
        let mut resolver = PathMatchingResourcePatternResolver::new();
        resolver.set_candidates(vec!["config/a.properties".to_string()]);
        assert!(
            resolver
                .get_resources("classpath*:zzz/*.txt")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn non_pattern_location_returns_single_resource() {
        // D 类（生命周期/重构安全）：对标 Spring 非模式单资源返回
        let resolver = PathMatchingResourcePatternResolver::new();
        let resources = resolver
            .get_resources("file:/tmp/vernal-single.txt")
            .unwrap();
        assert_eq!(resources.len(), 1);
        assert!(!resources[0].exists());
    }

    #[test]
    fn implements_resource_loader() {
        // D 类（重构安全）：委托底层加载器
        let dir = make_temp_dir("loader");
        let resolver = PathMatchingResourcePatternResolver::new();
        let resource = resolver
            .load(&format!("file:{}/a.txt", dir.display()))
            .unwrap();
        assert_eq!(resource.read_string().unwrap(), "A");
        fs::remove_dir_all(&dir).unwrap();
    }
}

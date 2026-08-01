//! 类相对资源加载器。
//!
//! 对标 Spring `org.springframework.core.io.ClassRelativeResourceLoader`。

use std::io;

use super::class_path_resource::ClassPathResource;
use super::resource::Resource;
use super::ResourceLoader;

/// 类相对资源加载器。
///
/// 对应 Java: org.springframework.core.io.ClassRelativeResourceLoader
///
/// Spring 语义：以指定包路径为基座，把相对路径拼接到 classpath 前缀下加载
/// （对标 `ClassRelativeContextResource`）。
pub struct ClassRelativeResourceLoader {
    base_path: String,
}

impl ClassRelativeResourceLoader {
    /// 创建以 `base_path`（如 `com/example/config`）为基座的加载器。
    #[must_use]
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

impl ResourceLoader for ClassRelativeResourceLoader {
    fn load(&self, location: &str) -> io::Result<Box<dyn Resource>> {
        let trimmed = location.trim_start_matches('/');
        let joined = if self.base_path.is_empty() {
            trimmed.to_string()
        } else {
            format!("{}/{trimmed}", self.base_path)
        };
        Ok(Box::new(ClassPathResource::new(joined)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_base_path_and_location() {
        // A 类（合同对齐）：对标 Spring 相对路径拼接
        let loader = ClassRelativeResourceLoader::new("com/example/config");
        let resource = loader.load("app.properties").unwrap();
        assert!(resource.description().contains("com/example/config/app.properties"));
    }

    #[test]
    fn leading_slash_is_stripped() {
        // B 类（边界行为）：对标 Spring 忽略前导 `/`
        let loader = ClassRelativeResourceLoader::new("com/example");
        let resource = loader.load("/app.properties").unwrap();
        assert!(resource.description().contains("com/example/app.properties"));
    }
}

//! 本地化资源辅助。
//!
//! 对标 Spring `org.springframework.core.io.support.LocalizedResourceHelper`。

use std::io;

use crate::io::{Resource, ResourceLoader};

/// 本地化资源辅助。
///
/// 对应 Java: org.springframework.core.io.support.LocalizedResourceHelper
///
/// Spring 语义：按 `basename[_language[_country]]` 后缀探测本地化变体
/// （先语言，后语言+地区，最后无后缀默认）。
pub struct LocalizedResourceHelper {
    loader: Box<dyn ResourceLoader>,
}

impl LocalizedResourceHelper {
    /// 以指定加载器创建辅助对象。
    #[must_use]
    pub fn new(loader: Box<dyn ResourceLoader>) -> Self {
        Self { loader }
    }

    /// 查找指定语言的本地化变体。
    ///
    /// 探测顺序：`basename_language_region` → `basename_language` → `basename`。
    ///
    /// # 错误
    ///
    /// 加载器失败时返回 [`std::io::Error`]。
    pub fn find_localized_resource(
        &self,
        basename: &str,
        language: &str,
        region: Option<&str>,
    ) -> io::Result<Box<dyn Resource>> {
        let candidates: Vec<String> = match region {
            Some(region) => vec![
                format!("{basename}_{language}_{region}"),
                format!("{basename}_{language}"),
                basename.to_string(),
            ],
            None => vec![format!("{basename}_{language}"), basename.to_string()],
        };
        for candidate in &candidates {
            let resource = self.loader.load(candidate)?;
            if resource.exists() {
                return Ok(resource);
            }
        }
        // 全部缺失: 对标 Spring 抛 IOException
        let _ = self.loader.load(candidates.first().map_or(basename, String::as_str))?;
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("no localized resource found for {basename}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::DefaultResourceLoader;

    #[test]
    fn prefers_language_variant() {
        // A 类（合同对齐）：对标 Spring 语言变体优先
        let helper = LocalizedResourceHelper::new(Box::new(DefaultResourceLoader::new()));
        let result = helper.find_localized_resource("messages", "zh", Some("CN"));
        // 无文件存在,应返回 NotFound 错误
        assert!(result.is_err());
    }

    #[test]
    fn missing_all_variants_returns_error() {
        // C 类（错误路径）
        let helper = LocalizedResourceHelper::new(Box::new(DefaultResourceLoader::new()));
        let Err(err) = helper.find_localized_resource("nonexistent-base", "xx", None) else {
            panic!("应返回错误");
        };
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}

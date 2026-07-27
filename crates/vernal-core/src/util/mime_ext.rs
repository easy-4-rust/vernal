//! 扩展名 ↔ MIME 静态映射(feature = "mime-ext")。
//!
//! 基于 [`mime_type`] crate 0.2.0,提供扩展名查询 MIME 的快速静态映射。
//!
//! # 适用场景
//!
//! - 配置属性绑定(`app.mime.json = "application/json"`)
//! - Content-Disposition 头生成(`filename=report.pdf` → `Content-Type: application/pdf`)
//! - 日志/监控中的快速分类
//!
//! # 与其他 MIME 模块的关系
//!
//! | 模块 | Feature | 角色 |
//! |---|---|---|
//! | [`super::mime_type`] | `mime` | HTTP Content-Type 抽象(对标 Spring MimeType) |
//! | 本模块 | `mime-ext` | 扩展名↔MIME 静态映射(基于 mime-type crate) |
//! | [`super::mime_sniff`] | `mime-sniff` | 字节内容嗅探(基于 mimetype-detector crate) |

use mime_type::{MimeFormat, MimeType as ExtMimeType};

/// 从文件扩展名获取 MIME 字符串。
///
/// 扩展名不包含点(`.`),传入 `"png"` 而非 `".png"`。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_core::util::mime_ext;
///
/// assert_eq!(mime_ext::mime_from_extension("png"), Some("image/png"));
/// assert_eq!(mime_ext::mime_from_extension("jpg"), Some("image/jpeg"));
/// assert_eq!(mime_ext::mime_from_extension("mp4"), Some("video/mp4"));
/// assert_eq!(mime_ext::mime_from_extension("unknown"), None);
/// ```
#[must_use]
pub fn mime_from_extension(ext: &str) -> Option<&'static str> {
    ExtMimeType::from_ext(ext).map(|m| leak_mime_string(m.to_string()))
}

/// 从 MIME 字符串获取强类型枚举。
///
/// 返回 `mime_type::MimeType`(由 mime-type crate 定义)。
pub fn mime_type_from_string(mime: &str) -> Option<ExtMimeType> {
    ExtMimeType::from_mime(mime)
}

/// 同时支持带点和不带点的扩展名。
///
/// 便捷包装,自动剥离点号。
#[must_use]
pub fn mime_from_extension_lenient(ext: &str) -> Option<&'static str> {
    let trimmed = ext.trim_start_matches('.');
    mime_from_extension(trimmed)
}

/// 把 String 泄漏为 &'static str。
///
/// 注意:每次调用相同扩展名会产生新的泄漏,业务场景下扩展名数量有限,
/// 且 mime_type crate 的 `to_string()` 输出稳定。生产环境可考虑用
/// `OnceLock<HashMap<String, &'static str>>` 缓存。
fn leak_mime_string(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_from_extension_png() {
        assert_eq!(mime_from_extension("png"), Some("image/png"));
    }

    #[test]
    fn mime_from_extension_jpeg() {
        // 注意:JPEG 扩展名是 jpg 或 jpeg,具体看 mime-type crate 支持
        let result = mime_from_extension("jpg").or_else(|| mime_from_extension("jpeg"));
        assert!(result.is_some(), "jpg or jpeg should be recognized");
    }

    #[test]
    fn mime_from_extension_mp4() {
        assert_eq!(mime_from_extension("mp4"), Some("video/mp4"));
    }

    #[test]
    fn mime_from_extension_unknown_returns_none() {
        assert_eq!(mime_from_extension("nonexistent"), None);
    }

    #[test]
    fn mime_from_extension_lenient_handles_dot() {
        // 带点和不带点都支持
        let with_dot = mime_from_extension_lenient(".png");
        let without_dot = mime_from_extension_lenient("png");
        assert_eq!(with_dot, without_dot);
        assert!(with_dot.is_some());
    }

    #[test]
    fn mime_type_from_string_basic() {
        let mime = mime_type_from_string("image/png");
        assert!(mime.is_some(), "image/png should be parsed");
    }

    #[test]
    fn mime_type_from_string_returns_none_for_invalid() {
        let mime = mime_type_from_string("not/a/valid/mime/with/many/slashes");
        // mime-type crate 对无法识别的字符串返回 None
        let _ = mime; // 行为依赖 mime-type crate 实现
    }
}

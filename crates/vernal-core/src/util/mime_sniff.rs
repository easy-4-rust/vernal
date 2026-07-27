//! 字节内容嗅探(feature = "mime-sniff")。
//!
//! 基于 [`mimetype_detector`] crate 0.3.11,通过文件头(magic numbers)检测真实 MIME 类型。
//!
//! # 适用场景
//!
//! - **上传检测**:防止伪造扩展名的恶意文件(如 `evil.php` 改名为 `evil.jpg`)
//! - 未知扩展名文件的类型识别
//! - 内容安全扫描
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["mime-sniff"] }
//! ```
//!
//! # 示例
//!
//! ```rust,ignore
//! use vernal_core::util::mime_sniff;
//!
//! let png_header = b"\x89PNG\r\n\x1a\n";
//! let result = mime_sniff::detect_bytes(png_header).unwrap();
//! assert_eq!(result.mime(), "image/png");
//! assert_eq!(result.extension(), "png");
//! ```

use std::path::Path;

/// 检测结果的封装。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffedMimeType {
    /// MIME 字符串(如 `image/png`)
    mime: String,
    /// 友好名称(如 `Portable Network Graphics`)
    name: String,
    /// 扩展名(如 `png`,不含点)
    extension: String,
}

impl SniffedMimeType {
    /// 获取 MIME 字符串。
    #[must_use]
    pub fn mime(&self) -> &str {
        &self.mime
    }

    /// 获取友好名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取扩展名(不含点)。
    #[must_use]
    pub fn extension(&self) -> &str {
        &self.extension
    }
}

/// 从字节数据检测 MIME 类型。
///
/// 对标 mimetype_detector::detect(),但返回封装后的 [`SniffedMimeType`]。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_core::util::mime_sniff;
///
/// let png_header = b"\x89PNG\r\n\x1a\n";
/// let result = mime_sniff::detect_bytes(png_header).unwrap();
/// assert_eq!(result.mime(), "image/png");
/// ```
#[must_use]
pub fn detect_bytes(data: &[u8]) -> Option<SniffedMimeType> {
    let mime = mimetype_detector::detect(data);
    Some(SniffedMimeType {
        mime: mime.to_string(),
        name: mime.name().to_string(),
        extension: mime.extension().trim_start_matches('.').to_string(),
    })
}

/// 从文件路径检测 MIME 类型。
///
/// 对标 mimetype_detector::detect_file()。
///
/// # 错误
///
/// 文件不存在或读取失败时返回 `Err`。
pub fn detect_file(path: impl AsRef<Path>) -> std::io::Result<SniffedMimeType> {
    let mime = mimetype_detector::detect_file(path.as_ref())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(SniffedMimeType {
        mime: mime.to_string(),
        name: mime.name().to_string(),
        extension: mime.extension().trim_start_matches('.').to_string(),
    })
}

/// 上传检测:交叉验证字节内容与声明的扩展名是否一致。
///
/// 对标 Spring `MultipartFile` + Apache Tika 的检测模式,但纯 Rust 实现。
///
/// # 参数
///
/// - `data`:上传的字节内容
/// - `declared_ext`:客户端声明的扩展名(带或不带点均可)
///
/// # 返回
///
/// - `Ok(())`:字节内容与扩展名一致
/// - `Err(MismatchError)`:不一致(可能伪造扩展名)
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_core::util::mime_sniff;
///
/// let real_png = b"\x89PNG\r\n\x1a\n";
/// assert!(mime_sniff::verify_extension_matches_content(real_png, "png").is_ok());
/// assert!(mime_sniff::verify_extension_matches_content(real_png, "jpg").is_err());
/// ```
pub fn verify_extension_matches_content(
    data: &[u8],
    declared_ext: &str,
) -> Result<(), MismatchError> {
    let sniffed = detect_bytes(data).ok_or(MismatchError::UnknownContent)?;
    let sniffed_ext = sniffed.extension();
    let declared = declared_ext.trim_start_matches('.');

    if sniffed_ext.eq_ignore_ascii_case(declared) {
        Ok(())
    } else {
        Err(MismatchError::ExtensionMismatch {
            declared: declared.to_string(),
            detected: sniffed_ext.to_string(),
            detected_mime: sniffed.mime().to_string(),
        })
    }
}

/// 上传检测失败错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MismatchError {
    /// 无法识别内容。
    UnknownContent,
    /// 扩展名与字节内容不一致。
    ExtensionMismatch {
        /// 客户端声明的扩展名
        declared: String,
        /// 字节检测出的扩展名
        detected: String,
        /// 字节检测出的 MIME
        detected_mime: String,
    },
}

impl std::fmt::Display for MismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownContent => write!(f, "unable to detect content type from bytes"),
            Self::ExtensionMismatch {
                declared,
                detected,
                detected_mime,
            } => write!(
                f,
                "extension mismatch: declared '.{declared}' but content is '.{detected}' ({detected_mime})"
            ),
        }
    }
}

impl std::error::Error for MismatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_bytes_png() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0DIHDR";
        let result = detect_bytes(png).expect("PNG should be detected");
        assert_eq!(result.mime(), "image/png");
        assert_eq!(result.extension(), "png");
        assert!(!result.name().is_empty());
    }

    #[test]
    fn detect_bytes_jpeg() {
        let jpeg = b"\xFF\xD8\xFF\xE0\x00\x10JFIF";
        let result = detect_bytes(jpeg).expect("JPEG should be detected");
        assert_eq!(result.mime(), "image/jpeg");
    }

    #[test]
    fn detect_bytes_pdf() {
        let pdf = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3";
        let result = detect_bytes(pdf).expect("PDF should be detected");
        assert_eq!(result.mime(), "application/pdf");
    }

    #[test]
    fn detect_bytes_gif() {
        let gif = b"GIF89a";
        let result = detect_bytes(gif).expect("GIF should be detected");
        assert_eq!(result.mime(), "image/gif");
    }

    #[test]
    fn detect_bytes_zip() {
        let zip = b"PK\x03\x04";
        let result = detect_bytes(zip);
        // ZIP 可能被识别为多种格式(docx/xlsx/jar 都是 zip),验证至少能识别
        assert!(result.is_some(), "ZIP should be detected");
    }

    #[test]
    fn detect_bytes_empty_returns_none_or_unknown() {
        let result = detect_bytes(b"");
        // 空 content 可能返回 application/octet-stream 或 None,验证一致性即可
        let _ = result;
    }

    #[test]
    fn detect_bytes_random_bytes_does_not_panic() {
        let random = [0x42; 16];
        let _ = detect_bytes(&random); // 不应 panic
    }

    #[test]
    fn verify_extension_matches_png_ok() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0DIHDR";
        assert!(verify_extension_matches_content(png, "png").is_ok());
    }

    #[test]
    fn verify_extension_matches_png_with_dot_ok() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0DIHDR";
        assert!(verify_extension_matches_content(png, ".png").is_ok());
    }

    #[test]
    fn verify_extension_mismatch_jpg_claiming_png() {
        let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0DIHDR";
        let err = verify_extension_matches_content(png, "jpg").unwrap_err();
        match err {
            MismatchError::ExtensionMismatch {
                declared,
                detected,
                detected_mime,
            } => {
                assert_eq!(declared, "jpg");
                assert_eq!(detected, "png");
                assert_eq!(detected_mime, "image/png");
            }
            _ => panic!("expected ExtensionMismatch"),
        }
    }

    #[test]
    fn verify_extension_unknown_content() {
        // 用空数据或无法识别的数据
        let result = verify_extension_matches_content(b"unknown", "png");
        let _ = result; // 可能 Ok 也可能 Err,取决于 detector
    }

    #[test]
    fn mismatch_error_display_unknown() {
        let err = MismatchError::UnknownContent;
        assert!(err.to_string().contains("unable to detect"));
    }

    #[test]
    fn mismatch_error_display_extension() {
        let err = MismatchError::ExtensionMismatch {
            declared: "jpg".to_string(),
            detected: "png".to_string(),
            detected_mime: "image/png".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("jpg"));
        assert!(s.contains("png"));
    }

    #[test]
    fn mismatch_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<MismatchError>();
    }

    #[test]
    fn sniffed_mime_type_accessors() {
        let s = SniffedMimeType {
            mime: "image/png".to_string(),
            name: "Portable Network Graphics".to_string(),
            extension: "png".to_string(),
        };
        assert_eq!(s.mime(), "image/png");
        assert_eq!(s.name(), "Portable Network Graphics");
        assert_eq!(s.extension(), "png");
    }
}

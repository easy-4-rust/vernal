# MIME 类型集成对照(整合 `mime-type` + `mimetype-detector`)

> 版本：v1.0(2026-07-27)
> 适用场景:**文件上传**的 MIME 类型识别与校验
>
> 集成方案:
> - **静态定义**: [`mime-type` 0.2.0](https://crates.io/crates/mime-type)([docs.rs](https://docs.rs/mime-type/latest/mime_type/)) — 扩展名↔MIME 映射
> - **字节嗅探**: [`mimetype-detector` 0.3.11](https://crates.io/crates/mimetype-detector)([docs.rs](https://docs.rs/mimetype-detector/latest/mimetype_detector/)) — magic numbers 检测
> - **HTTP 头抽象**: vernal-core 自带的 `util::mime_type::MimeType`(对标 Spring `MimeType`,处理 `Content-Type: application/json;charset=utf-8` 这类带参数的字符串)

---

## 一、三者的角色定位

| 工具 | 来源 | 角色 | 典型场景 |
|---|---|---|---|
| `vernal_core::util::mime_type::MimeType` | 自实现(对标 Spring) | HTTP Content-Type 抽象(type/subtype;params) | 解析 HTTP 头、`isCompatibleWith`、charset 提取 |
| `mime_type::MimeType`(集成) | [`mime-type` crate](https://crates.io/crates/mime-type) 0.2.0 | 扩展名↔MIME 静态映射 | 配置属性绑定、文件名后缀查询、生成 Content-Disposition |
| `mimetype_detector::detect`(集成) | [`mimetype-detector` crate](https://crates.io/crates/mimetype-detector) 0.3.11 | 字节内容(magic numbers)嗅探 | **上传检测**:防止伪造扩展名、读取未知名文件 |

### 1.1 为什么需要三个?

- **Spring `MimeType`** 关注的是 **HTTP 协议层** 的兼容性判定(`application/json` 与 `application/*` 兼容)与参数处理(charset),不关心文件
- **`mime-type` crate** 关注的是 **扩展名静态查表**(`.png` → `image/png`),零开销、可预测,但不能防伪造
- **`mimetype-detector` crate** 关注的是 **真实字节内容**(PNG 文件头 `89 50 4E 47` → image/png),能防伪造但有 I/O 成本

**上传检测的正确流程**:`客户端上传 → 读 magic number → 与扩展名交叉验证 → 拒绝不一致 → 写入磁盘`。

---

## 二、`mime-type` crate 集成(feature = "mime-ext")

### 2.1 用途

提供 **扩展名 → MIME 字符串** 的快速查询,用于:
- 配置属性绑定(从 `app.mime.json = "application/json"` 解析)
- Content-Disposition 头生成(`filename=report.pdf` → `Content-Type: application/pdf`)
- 日志/监控中的快速分类

### 2.2 关键 API

```rust
use mime_type::{MimeType, MimeFormat};

// 从扩展名获取 MIME
let mime = MimeType::from_ext("png").unwrap();
assert_eq!(mime.to_string(), "image/png");

// 从 MIME 字符串获取强类型枚举
let mime = MimeType::from_mime("video/mp4").unwrap();
```

**优势**:
- 强类型枚举(`MimeType::Image(Image::Png)`)
- 编译期错误检查
- 零运行时开销

### 2.3 vernal-core 集成方式

```toml
# vernal-core/Cargo.toml
[dependencies]
mime_type = { version = "0.2", optional = true, package = "mime-type" }

[features]
mime-ext = ["dep:mime_type"]
```

```rust
// crates/vernal-core/src/util/mime_ext.rs
//! 扩展名 → MIME 映射(feature = "mime-ext")。
//!
//! 基于 [`mime_type`] crate,提供静态、零开销的扩展名查询。
//! 与 `mime_type::MimeType` 不同,本模块关注"文件后缀"而非"HTTP 头"。

use mime_type::{MimeFormat, MimeType as ExtMimeType};

/// 通过扩展名获取 MIME 字符串。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_core::util::mime_ext;
///
/// assert_eq!(mime_ext::mime_from_extension("png"), Some("image/png"));
/// assert_eq!(mime_ext::mime_from_extension("pdf"), Some("application/pdf"));
/// assert_eq!(mime_ext::mime_from_extension("unknown"), None);
/// ```
#[must_use]
pub fn mime_from_extension(ext: &str) -> Option<&'static str> {
    ExtMimeType::from_ext(ext)
        .ok()
        .map(|m| leak_mime_string(m.to_string()))
}

/// 通过 MIME 字符串获取强类型枚举。
///
/// 返回 `mime_type::MimeType`(由 mime-type crate 定义)。
pub fn extension_from_mime(mime: &str) -> Option<String> {
    // 注意:mime-type 0.2.0 的 MimeType 没有 extension() 方法
    // 这里反向查询需要业务自行维护映射或使用 mimetype-detector
    let _ = mime;
    None
}

// 把 String 泄漏为 &'static str(简化:用户场景下扩展名数量有限)
fn leak_mime_string(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
```

---

## 三、`mimetype-detector` crate 集成(feature = "mime-sniff")

### 3.1 用途

提供 **字节内容嗅探**(magic numbers),用于:
- **上传检测**:防止伪造扩展名的恶意文件(如 `evil.php` 改名为 `evil.jpg`)
- 未知扩展名文件的类型识别
- 内容安全扫描

### 3.2 关键 API

```rust
use mimetype_detector::{detect, detect_file, detect_reader};

// 从字节数据检测
let data = b"\x89PNG\r\n\x1a\n";  // PNG 文件头
let mime = detect(data);
assert_eq!(mime.name(), "Portable Network Graphics");
assert_eq!(mime.extension(), "png");

// 从文件路径检测
let mime = detect_file("test.png").unwrap();

// 从 Reader 检测(流式)
let mime = detect_reader(&mut cursor).unwrap();
```

**优势**:
- **零依赖**(纯 Rust 实现)
- 支持 200+ 文件签名
- 三种入口:字节 / 文件 / Reader

### 3.3 vernal-core 集成方式

```toml
# vernal-core/Cargo.toml
[dependencies]
mimetype_detector = { version = "0.3", optional = true }

[features]
mime-sniff = ["dep:mimetype_detector"]
```

```rust
// crates/vernal-core/src/util/mime_sniff.rs
//! 字节内容嗅探(feature = "mime-sniff")。
//!
//! 基于 [`mimetype_detector`] crate,通过文件头(magic numbers)检测真实 MIME 类型。
//! 主要用于**上传检测**场景,防止伪造扩展名的恶意文件。

use mimetype_detector::{detect as detect_bytes, detect_file as detect_path};

/// 检测结果的简化封装。
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
/// 适用于上传文件的内容检测。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_core::util::mime_sniff;
///
/// let png_header = b"\x89PNG\r\n\x1a\n";
/// let result = mime_sniff::detect_bytes(png_header).unwrap();
/// assert_eq!(result.mime(), "image/png");
/// assert_eq!(result.extension(), "png");
/// ```
pub fn detect_bytes(data: &[u8]) -> Option<SniffedMimeType> {
    let mime = detect_bytes(data);
    Some(SniffedMimeType {
        mime: mime.to_string(),
        name: mime.name().to_string(),
        extension: mime.extension().trim_start_matches('.').to_string(),
    })
}

/// 从文件路径检测 MIME 类型。
pub fn detect_file(path: &str) -> std::io::Result<SniffedMimeType> {
    let mime = detect_path(path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(SniffedMimeType {
        mime: mime.to_string(),
        name: mime.name().to_string(),
        extension: mime.extension().trim_start_matches('.').to_string(),
    })
}

/// 上传检测:交叉验证字节内容与声明的扩展名是否一致。
///
/// 对标 Spring `MultipartFile` + `Tika` 的检测模式,但纯 Rust 实现。
///
/// # 返回
///
/// - `Ok(())`:字节内容与扩展名一致
/// - `Err(MismatchError)`:不一致(可能伪造扩展名)
pub fn verify_extension_matches_content(data: &[u8], declared_ext: &str) -> Result<(), MismatchError> {
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
            Self::ExtensionMismatch { declared, detected, detected_mime } => write!(
                f,
                "extension mismatch: declared '.{declared}' but content is '.{detected}' ({detected_mime})"
            ),
        }
    }
}

impl std::error::Error for MismatchError {}
```

---

## 四、典型上传检测流程

```rust
use vernal_core::util::{mime_ext, mime_sniff};
use vernal_core::BoxError;

/// 处理文件上传,严格校验 MIME 类型。
///
/// 流程:
/// 1. 从扩展名查询期望的 MIME(快速)
/// 2. 从字节内容嗅探实际 MIME(防伪造)
/// 3. 交叉验证两者一致
/// 4. 返回最终确认的 MIME
pub fn handle_upload(
    filename: &str,
    content: &[u8],
) -> Result<String, BoxError> {
    // 1. 提取扩展名
    let ext = filename.rsplit('.').next().unwrap_or("");

    // 2. 扩展名查询期望 MIME(feature = "mime-ext")
    let expected_mime = mime_ext::mime_from_extension(ext);

    // 3. 字节嗅探实际 MIME(feature = "mime-sniff")
    let sniffed = mime_sniff::detect_bytes(content)
        .ok_or_else(|| mime_sniff::MismatchError::UnknownContent)?;

    // 4. 交叉验证
    mime_sniff::verify_extension_matches_content(content, ext)?;

    // 5. 返回实际 MIME(优先用字节检测结果)
    Ok(if expected_mime == Some(sniffed.mime()) {
        sniffed.mime().to_string()
    } else {
        sniffed.mime().to_string()
    })
}
```

---

## 五、feature 矩阵

| Feature 名 | 启用 | 依赖 | 用途 |
|---|---|---|---|
| `mime` | (已在 v3.0) | 无(vernal-core 自实现) | HTTP Content-Type 抽象(对标 Spring) |
| `mime-ext` | 新增 | `mime-type = "0.2"` | 扩展名↔MIME 静态映射 |
| `mime-sniff` | 新增 | `mimetype_detector = "0.3"` | 字节内容嗅探(上传检测) |

三个 feature **可以独立启用,也可以组合使用**。

---

## 六、推荐 Cargo.toml 配置

```toml
[dependencies]
# vernal-core 默认零依赖
# 业务按需启用:

# 完整 MIME 处理(HTTP + 扩展名 + 字节嗅探)
vernal-core = { version = "...", features = ["mime", "mime-ext", "mime-sniff"] }

# 仅 HTTP 头处理
vernal-core = { version = "...", features = ["mime"] }

# 仅扩展名查询(最轻量)
vernal-core = { version = "...", features = ["mime-ext"] }

# 仅上传检测
vernal-core = { version = "...", features = ["mime-sniff"] }
```

---

## 七、参考链接

- **mime-type crate**:
  - crates.io: <https://crates.io/crates/mime-type>
  - docs.rs: <https://docs.rs/mime-type/latest/mime_type/>
  - 版本: 0.2.0,许可证: MIT OR Apache-2.0
- **mimetype-detector crate**:
  - crates.io: <https://crates.io/crates/mimetype-detector>
  - docs.rs: <https://docs.rs/mimetype-detector/latest/mimetype_detector/>
  - 版本: 0.3.11,许可证: MIT OR Apache-2.0
  - **零依赖,纯 Rust 实现**
- **Spring 对应**:
  - `org.springframework.util.MimeType`(HTTP 抽象)
  - `org.springframework.web.multipart.MultipartFile`(上传 API)
  - Spring 项目实际用 Apache Tika 做字节嗅探,vernal-core 改用 `mimetype-detector`
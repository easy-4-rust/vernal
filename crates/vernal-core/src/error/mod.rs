//! 框架统一错误体系。
//!
//! 提供跨子系统的结构化错误类型 [`VernalError`]，支持：
//!
//! - **零分配的业务错误码**（domain / code / message）
//! - **带动态上下文的增强错误**（WithContext 变体）
//! - **第三方错误的透明包装**（Infrastructure 变体）
//!
//! ## 与现有错误类型的关系
//!
//! | 类型 | 用途 | 关系 |
//! |------|------|------|
//! | `BoxError` | `Result` 的通用错误类型 | 保留，`VernalError` 可 `From<BoxError>` |
//! | `SharedError` | 需要共享的错误 | 保留，`VernalError::Infrastructure` 包装它 |
//! | `VernalError` | 结构化错误，支持程序化匹配 | **新增** |
//!
//! ## 设计来源
//!
//! 对标 tx_di 的 `AppError`（domain / code / message）+ `CodeMsg` 模式，
//! 适配 vernal 的类型体系和异步模型。

// ─── 子模块声明 ───
mod vernal_error;
mod error_code;
mod error_domain;
mod error_kind;
mod error_context;
mod error_report;

// ─── 公开导出 ───
pub use vernal_error::VernalError;
pub use error_code::ErrorCode;
pub use error_domain::ErrorDomain;
pub use error_kind::ErrorKind;
pub use error_context::ErrorContext;
pub use error_report::ErrorReport;

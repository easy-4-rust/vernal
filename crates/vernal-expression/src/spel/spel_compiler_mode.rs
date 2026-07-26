//! SpEL 编译模式枚举。
//!
//! 对标 Spring 的 `SpelCompilerMode`。

/// SpEL 编译模式。
///
/// 对标 Spring 的 `org.springframework.expression.spel.SpelCompilerMode`。
/// Rust 不需要字节码编译，此枚举保留用于 API 兼容。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpelCompilerMode {
    /// 关闭编译（默认）
    Off,
    /// 立即编译
    Immediate,
    /// 混合模式
    Mixed,
}

//! Option Trait 构造注入测试端口。

/// 由可选 Trait Binding 提供的原生客户端端口。
pub(crate) trait OptionalDerivedPort: Send + Sync {
    /// 返回客户端稳定名称。
    fn name(&self) -> &str;
}

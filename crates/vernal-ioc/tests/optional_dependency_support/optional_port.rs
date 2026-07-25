//! 立即可选 Trait 依赖测试端口。

/// 用于证明可选 Trait Binding 仍保持类型安全投影的端口。
pub(crate) trait OptionalPort: Send + Sync {
    /// 返回端口实现持有的稳定值。
    fn value(&self) -> &str;
}

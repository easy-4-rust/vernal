//! 组件作用域对象。

/// 组件实例的创建与缓存语义。
///
/// 作用域状态归属于单个 [`crate::Container`]，不会通过进程级全局变量在多个
/// 应用上下文或并行测试之间共享。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Scope {
    /// 每个容器惰性创建并缓存一个实例。
    #[default]
    Singleton,
    /// 每次解析都创建新实例。
    Transient,
}

impl Scope {
    /// 返回适合稳定诊断和序列化输出的作用域名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Singleton => "singleton",
            Self::Transient => "transient",
        }
    }
}

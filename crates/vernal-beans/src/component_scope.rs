//! 组件作用域声明对象。

use crate::ScopeKey;

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
    /// 在匹配类型身份的显式 [`crate::ScopeContext`] 内惰性创建并缓存。
    Custom(ScopeKey),
}

impl Scope {
    /// 创建由 Rust 标记类型 `S` 识别的自定义作用域声明。
    #[must_use]
    pub fn custom<S: 'static>() -> Self {
        Self::Custom(ScopeKey::of::<S>())
    }

    /// 返回适合稳定诊断和序列化输出的作用域名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Singleton => "singleton",
            Self::Transient => "transient",
            Self::Custom(key) => key.type_name(),
        }
    }

    /// 返回该定义是否属于容器级 Singleton。
    #[must_use]
    pub const fn is_singleton(self) -> bool {
        matches!(self, Self::Singleton)
    }

    /// 返回该定义是否在每次解析时重新构造。
    #[must_use]
    pub const fn is_transient(self) -> bool {
        matches!(self, Self::Transient)
    }

    /// 返回自定义作用域身份；内建作用域返回 `None`。
    #[must_use]
    pub const fn custom_key(self) -> Option<ScopeKey> {
        match self {
            Self::Custom(key) => Some(key),
            Self::Singleton | Self::Transient => None,
        }
    }
}

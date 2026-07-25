//! 自定义组件作用域标识对象。

use std::{
    any::{TypeId, type_name},
    fmt,
};

/// 由 Rust 标记类型确定的自定义作用域身份。
///
/// `ScopeKey` 不依赖字符串注册表：相同标记类型在同一进程中具有相同 `TypeId`，
/// 完整类型名只用于诊断。Request、Task、Tenant 等语义由消费方定义标记类型，
/// `vernal-beans` 不引入任何 HTTP 或业务概念。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScopeKey {
    pub(crate) type_id: TypeId,
    type_name: &'static str,
}

impl ScopeKey {
    /// 为作用域标记类型 `S` 创建稳定键。
    #[must_use]
    pub fn of<S: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<S>(),
            type_name: type_name::<S>(),
        }
    }

    /// 返回作用域标记的完整 Rust 类型名。
    #[must_use]
    pub const fn type_name(self) -> &'static str {
        self.type_name
    }
}

impl fmt::Display for ScopeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.type_name)
    }
}

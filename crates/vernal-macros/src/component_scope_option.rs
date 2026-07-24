//! 组件派生宏作用域选项对象。

use syn::Type;

/// 保存内建作用域或调用方声明的类型化自定义作用域。
pub(crate) enum ComponentScopeOption {
    /// 每个 Container 一个实例。
    Singleton,
    /// 每次解析创建实例。
    Transient,
    /// 在匹配标记类型的显式 `ScopeContext` 中缓存实例。
    Custom(Box<Type>),
}

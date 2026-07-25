//! 配置字段缺失值策略。

use syn::Expr;

/// 表示派生宏为普通配置字段选择的缺失值处理方式。
pub(crate) enum ConfigurationDefaultOption {
    /// 属性必须存在。
    Required,
    /// 属性缺失时调用字段类型的 `Default`。
    Default,
    /// 属性缺失时求值调用方声明的 Rust 表达式。
    Expression(Expr),
}

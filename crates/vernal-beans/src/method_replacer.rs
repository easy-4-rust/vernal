//! MethodReplacer — Spring 风格的方法替换器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MethodReplacer`。
//!
//! 配合 `ReplaceOverride` 使用：实现该 trait 的 Bean 可以替换另一个 Bean 中
//! 指定方法的实现。

use std::any::Any;

/// 方法替换器 trait。
///
/// 对应 Spring 的 `MethodReplacer`。
///
/// 当一个方法被 `ReplaceOverride` 覆盖时，容器会定位到对应的 `MethodReplacer`
/// Bean，并调用 `reimplement()` 执行替换逻辑。
///
/// ## 与 Rust 的映射
///
/// Java 版本签名：`Object reimplement(Object obj, Method method, Object[] args) throws Throwable`。
/// Rust 中使用 `&dyn Any` 替代 `Object`，`&str` 替代方法名，`&[dyn Any]` 替代参数数组。
pub trait MethodReplacer: Send + Sync {
    /// 重新实现目标方法。
    ///
    /// # 参数
    ///
    /// - `obj` — 原始对象（被替换方法所属的实例），类型擦除为 `Any`
    /// - `method` — 被替换的方法名
    /// - `args` — 调用参数列表（类型擦除）
    ///
    /// # 返回
    ///
    /// 成功返回重新实现的返回值（装箱为 `Any`），失败返回错误。
    fn reimplement(
        &self,
        obj: &dyn Any,
        method: &str,
        args: &[&dyn Any],
    ) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>>;
}

//! 属性访问器 trait。
//!
//! 对标 Spring 的 `PropertyAccessor` 和 `IndexAccessor`。

use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypedValue;

/// 属性访问器 trait。
///
/// 读取和写入对象的属性。
/// 对标 Spring 的 `org.springframework.expression.PropertyAccessor`。
pub trait PropertyAccessor: Send + Sync {
    /// 是否可以读取指定属性。
    fn can_read(&self, context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool;

    /// 读取属性值。
    fn read(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException>;

    /// 是否可以写入指定属性。
    fn can_write(&self, context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool;

    /// 写入属性值。
    fn write(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
        value: &TypedValue,
    ) -> Result<(), AccessException>;

    /// 获取此访问器支持的目标类型（空表示通用）。
    fn specific_target_classes(&self) -> &[&str] {
        &[]
    }
}

/// 索引访问器 trait。
///
/// 对标 Spring 的 `org.springframework.expression.IndexAccessor`（7.0 新增）。
pub trait IndexAccessor: Send + Sync {
    /// 是否可以读取指定索引。
    fn can_read(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        index: &TypedValue,
    ) -> bool;

    /// 读取索引值。
    fn read(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        index: &TypedValue,
    ) -> Result<TypedValue, AccessException>;

    /// 是否可以写入指定索引。
    fn can_write(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        index: &TypedValue,
    ) -> bool;

    /// 写入索引值。
    fn write(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        index: &TypedValue,
        value: &TypedValue,
    ) -> Result<(), AccessException>;
}

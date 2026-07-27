//! TypeConverter — Spring 风格的类型转换器接口。
//!
//! 对应 Java 类：`org.springframework.beans.TypeConverter`。
//!
//! 提供类型转换能力：将值从一种类型转换为另一种类型。

use std::any::Any;

/// Spring 风格的类型转换器接口。
///
/// 对应 Spring 的 `TypeConverter`。
///
/// 提供类型转换能力：
/// - 将字符串转换为目标类型
/// - 将一个数值类型转换为另一个数值类型
/// - 将 `Option<T>` 转换为 `T`
///
/// ## 与 ConversionService 的关系
///
/// - `TypeConverter` 是类型转换的统一入口
/// - `ConversionService` 是具体的转换器注册表
/// - `TypeConverter` 可以委托 `ConversionService` 执行转换
pub trait TypeConverter: Send + Sync + 'static {
    /// 将值转换为目标类型。
    ///
    /// 对应 Spring 的 `<T> T convertIfNecessary(String propertyName, Object value, Class<T> requiredType)`。
    ///
    /// # 参数
    ///
    /// - `property_name` — 属性名（用于错误报告）
    /// - `value` — 源值
    /// - `target_type` — 目标类型（类型擦除后使用 TypeId）
    ///
    /// # 返回
    ///
    /// - `Ok(converted)` — 转换后的值
    /// - `Err` — 转换失败
    fn convert_if_necessary(
        &self,
        property_name: Option<&str>,
        value: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 将值转换为目标类型（如果需要）。
    ///
    /// 对应 Spring 的 `<T> T convertIfNecessary(Object value, Class<T> requiredType)`。
    fn convert_value(
        &self,
        value: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.convert_if_necessary(None, value, target_type)
    }
}

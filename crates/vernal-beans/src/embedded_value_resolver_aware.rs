//! EmbeddedValueResolverAware — Spring 风格的 EmbeddedValueResolver 感知接口。
//!
//! 对应 Java 类：`org.springframework.context.EmbeddedValueResolverAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_embedded_value_resolver`，
//! 将一个字符串值解析器（例如占位符 `${...}` 解析器）注入 Bean。

use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 EmbeddedValueResolver 感知接口。
///
/// 对应 Spring 的 `EmbeddedValueResolverAware.setEmbeddedValueResolver(StringValueResolver resolver)`。
///
/// 容器在实例化 Bean 后，调用此方法将字符串值解析器注入。
/// 解析器负责解析嵌入式值（如占位符 `${...}`），返回解析后的字符串。
pub trait EmbeddedValueResolverAware: Aware {
    /// 将嵌入式值解析器注入 Bean。
    ///
    /// 对应 Spring 的 `EmbeddedValueResolverAware.setEmbeddedValueResolver(StringValueResolver resolver)`。
    ///
    /// `resolver` 是一个接收字符串并返回解析后字符串的函数。
    /// 例如，它可以解析 `${property.name}` 占位符为对应的配置值。
    fn set_embedded_value_resolver(&mut self, resolver: Arc<dyn Fn(&str) -> String + Send + Sync>);
}

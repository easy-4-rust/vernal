//! 类型安全配置对象契约。

use std::any::Any;

use vernal_core::BoxError;
use vernal_ioc::ComponentDefinition;

use crate::{ApplicationEnvironment, ConfigurationPropertiesError};

/// 把一组带共同前缀的 Environment 属性绑定成 Rust 原生配置对象。
///
/// 该契约不拥有 TOML、YAML、`.setting`、环境变量或远程配置中心；这些格式只需
/// 提供 [`crate::PropertySource`]。派生实现按已知字段精确读取属性，因此不要求
/// `PropertySource` 暴露所有键，也不会把凭证复制到全局配置树。
///
/// 配置对象满足 `Any + Send + Sync` 后即可通过 [`Self::component_definition`]
/// 注册为普通 Singleton，继续被业务组件、Tokio 任务和 Web Adapter 类型安全注入。
pub trait ConfigurationProperties: Any + Send + Sync + Sized {
    /// 根配置对象使用的属性前缀。
    const PREFIX: &'static str;

    /// 使用类型声明的 [`Self::PREFIX`] 绑定完整配置对象。
    ///
    /// # Errors
    ///
    /// 必填字段缺失、属性类型错误、占位符错误或来源读取失败时返回脱敏的
    /// [`ConfigurationPropertiesError`]。
    fn bind(environment: &ApplicationEnvironment) -> Result<Self, ConfigurationPropertiesError> {
        Self::bind_with_prefix(environment, Self::PREFIX)
    }

    /// 使用调用方提供的前缀绑定配置对象。
    ///
    /// 该入口由嵌套配置字段调用，使同一配置类型可以在不同父级路径复用。
    ///
    /// # Errors
    ///
    /// 字段绑定失败时返回 [`ConfigurationPropertiesError`]。
    fn bind_with_prefix(
        environment: &ApplicationEnvironment,
        prefix: &str,
    ) -> Result<Self, ConfigurationPropertiesError>;

    /// 创建一个依赖当前 Context `ApplicationEnvironment` 的 Singleton 定义。
    ///
    /// 定义沿用 Vernal 标准依赖图和受限 Resolver，不读取进程级全局状态。配置对象
    /// 在纯 `Container` 用法中首次解析时绑定；`ApplicationContext::refresh()` 会
    /// 预热全部 Singleton，因此应用模式会在进入 `Refreshed` 前完成绑定并快速失败。
    /// 成功结果在该 Container 中保持 Singleton 身份。
    #[must_use]
    fn component_definition() -> ComponentDefinition {
        ComponentDefinition::try_singleton::<Self, _>(|resolver| -> Result<Self, BoxError> {
            let environment = resolver.resolve::<ApplicationEnvironment>()?;
            Self::bind(&environment).map_err(|source| Box::new(source) as BoxError)
        })
        .depends_on::<ApplicationEnvironment>()
    }
}

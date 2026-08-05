//! 可配置属性解析器契约。
//!
//! 对标 Spring `org.springframework.core.env.ConfigurablePropertyResolver`。

use super::MissingRequiredPropertiesException;
use super::PropertyResolver;

/// 可配置属性解析器契约。
///
/// 对应 Java: org.springframework.core.env.ConfigurablePropertyResolver
///
/// Spring 语义：在 `PropertyResolver` 之上增加占位符语法配置（前缀/后缀/
/// 值分隔符）、必填属性校验与转换服务接入。
pub trait ConfigurablePropertyResolver: PropertyResolver {
    /// 设置占位符前缀（默认 `${`）。
    fn set_placeholder_prefix(&mut self, prefix: &str);

    /// 设置占位符后缀（默认 `}`）。
    fn set_placeholder_suffix(&mut self, suffix: &str);

    /// 设置值分隔符（默认 `:`，`None` 表示禁用默认值语法）。
    fn set_value_separator(&mut self, separator: Option<String>);

    /// 设置是否忽略未解析的嵌套占位符。
    fn set_ignore_unresolvable_nested_placeholders(&mut self, ignore: bool);

    /// 设置必填属性列表。
    fn set_required_properties(&mut self, required: Vec<String>);

    /// 校验全部必填属性已定义。
    ///
    /// # 错误
    ///
    /// 存在缺失属性时返回 [`MissingRequiredPropertiesException`]。
    fn validate_required_properties(&self) -> Result<(), MissingRequiredPropertiesException>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MutablePropertySources;
    use crate::env::PropertySourcesPropertyResolver;

    #[test]
    fn resolver_satisfies_contract() {
        // D 类（重构安全）：`PropertySourcesPropertyResolver` 实现该契约
        fn assert_resolver<T: ConfigurablePropertyResolver>() {}
        assert_resolver::<PropertySourcesPropertyResolver>();
        let _ = MutablePropertySources::new();
    }
}

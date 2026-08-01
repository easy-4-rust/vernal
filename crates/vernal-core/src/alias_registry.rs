//! 别名注册契约。
//!
//! 对标 Spring `org.springframework.core.AliasRegistry`。

/// 别名注册契约。
///
/// 对应 Java: org.springframework.core.AliasRegistry
///
/// Spring 语义：Bean 别名的注册/查询/移除（对标 `BeanDefinitionRegistry` 的
/// 别名能力）。
pub trait AliasRegistry: Send + Sync {
    /// 注册别名。
    ///
    /// # 错误
    ///
    /// 别名已存在或与规范名相同时返回错误信息。
    fn register_alias(&mut self, name: &str, alias: &str) -> Result<(), String>;

    /// 移除别名。
    fn remove_alias(&mut self, alias: &str);

    /// 判断是否为已注册别名。
    fn is_alias(&self, name: &str) -> bool;

    /// 返回指定名称的全部别名。
    fn get_aliases(&self, name: &str) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleAliasRegistry;

    #[test]
    fn registry_satisfies_contract() {
        // D 类（重构安全）：`SimpleAliasRegistry` 实现该契约
        fn assert_registry<T: AliasRegistry>() {}
        assert_registry::<SimpleAliasRegistry>();
    }
}

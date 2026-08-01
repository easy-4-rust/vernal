//! 属性源集合契约。
//!
//! 对标 Spring `org.springframework.core.env.PropertySources`。

use super::PropertySource;

/// 属性源集合契约。
///
/// 对应 Java: org.springframework.core.env.PropertySources
///
/// Spring 语义：按优先级顺序持有多个属性源，`iterator()` 提供遍历。
pub trait PropertySources: Send + Sync {
    /// 按优先级顺序返回属性源视图（首个为最高优先级）。
    fn sources(&self) -> Vec<&dyn PropertySource>;

    /// 返回属性源数量。
    fn len(&self) -> usize;

    /// 是否为空。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::MutablePropertySources;

    #[test]
    fn mutable_sources_satisfy_contract() {
        // D 类（重构安全）：`MutablePropertySources` 实现该契约
        fn assert_sources<T: PropertySources>() {}
        assert_sources::<MutablePropertySources>();
        let sources = MutablePropertySources::new();
        assert!(sources.is_empty());
    }
}

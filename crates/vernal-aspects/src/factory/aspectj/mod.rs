//! 对标 `org.springframework.beans.factory.aspectj` 包（按审计脚本"保留末两层"规则）。
//!
//! 提供可配置对象 DI 切面的 Rust 等价实现，覆盖：
//! - `ConfigurableObject`：标记 trait
//! - `ConfigurableObjectWithLifetime`：带生命周期的标记 trait

mod configurable_object;

pub use configurable_object::ConfigurableObject;
pub use configurable_object::ConfigurableObjectWithLifetime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configurable_object() {
        struct Test;
        impl ConfigurableObject for Test {}
        fn assert_impl<T: ConfigurableObject>() {}
        assert_impl::<Test>();
    }
}

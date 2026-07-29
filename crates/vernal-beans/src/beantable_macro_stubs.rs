//! beantable 宏设计占位 — 用于 S14 的 `#[beantable]` 和 `#[bean]` 过程宏 API。
//!
//! ## 设计思路
//!
//! 此模块定义了 `#[beantable]` 和 `#[bean]` 过程宏将在未来生成的 trait 代码。
//! 这两个宏的定位对标 Spring Boot 的 `@Configuration` + `@Bean` 注解模式：
//!
//! - `#[beantable]` — 标记一个模块或结构体为 Bean 定义表（类似 Spring `@Configuration` 类）。
//!   被标记的项会被展开为一个模块级 `Beantable` 实现。
//!
//! - `#[bean]` — 标记一个函数为 Bean 工厂方法（类似 Spring `@Bean` 方法）。
//!   被标记的函数会被包装为一个 `BeanMethod` 实现，生成对应的 `ComponentDefinition`。
//!
//! ## 宏展开示例（预期）
//!
//! ```rust,ignore
//! // 输入：
//! // #[beantable]
//! // mod my_config {
//! //     #[bean]
//! //     fn my_service() -> MyService { MyService::new() }
//! // }
//!
//! // 输出（大致）：
//! // impl Beantable for my_config {
//! //     fn bean_definitions() -> Vec<ComponentDefinition> {
//! //         vec![my_service::bean_definition()]
//! //     }
//! // }
//! // impl BeanMethod for my_service {
//! //     fn bean_definition() -> ComponentDefinition {
//! //         ComponentDefinition { ... }
//! //     }
//! // }
//! ```
//!
//! ## 未来计划
//!
//! - S14: 实现 `#[beantable]` 和 `#[bean]` 过程宏（proc-macro crate）
//! - S15: 支持 `#[bean(name = "...")]` 自定义名称、`#[bean(scope = "prototype")]` 等属性
//! - S16: 支持 `#[bean(init_method = "...", destroy_method = "...")]` 生命周期回调
//! - S17: 与 `vernal-context` 集成，实现 `@Configuration` 风格的自动注册

use crate::component_definition::ComponentDefinition;

/// `#[beantable]` 宏生成的 trait。
///
/// 对应 Spring 的 `@Configuration` 类语义：一个模块或结构体包含多个 `#[bean]` 定义。
///
/// 宏展开后，被 `#[beantable]` 标记的项会实现此 trait，
/// 返回该表中所有 Bean 定义的集合。
///
/// # 用法（宏展开后）
///
/// ```rust,ignore
/// impl Beantable for my_config_mod {
///     fn bean_definitions() -> Vec<ComponentDefinition> {
///         vec![
///             bean_method_1::bean_definition(),
///             bean_method_2::bean_definition(),
///         ]
///     }
/// }
/// ```
pub trait Beantable {
    /// 返回此 beantable 表中所有 Bean 的 `ComponentDefinition` 列表。
    ///
    /// 容器在初始化时调用此方法，将返回的所有定义注册到容器中。
    fn bean_definitions() -> Vec<ComponentDefinition>;
}

/// `#[bean]` 宏生成的 trait。
///
/// 对应 Spring 的 `@Bean` 方法语义：一个工厂方法定义了一个或多个 Bean。
///
/// 宏展开后，被 `#[bean]` 标记的函数会实现此 trait，
/// 返回单个 Bean 的 `ComponentDefinition`。
///
/// # 用法（宏展开后）
///
/// ```rust,ignore
/// impl BeanMethod for my_service_fn {
///     fn bean_definition() -> ComponentDefinition {
///         ComponentDefinition::builder()
///             .name("myService")
///             .factory(|resolver| Ok(Arc::new(MyService::new())))
///             .build()
///     }
/// }
/// ```
pub trait BeanMethod {
    /// 返回此 bean 方法对应的 `ComponentDefinition`。
    ///
    /// 容器在注册时调用此方法，获取 Bean 的定义信息（名称、工厂、依赖等）。
    fn bean_definition() -> ComponentDefinition;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_definitions_exist() {
        // 验证 trait 定义可以被编译
        // 实际行为在宏展开后通过集成测试验证
        struct DummyBeantable;
        impl Beantable for DummyBeantable {
            fn bean_definitions() -> Vec<ComponentDefinition> {
                vec![]
            }
        }

        struct DummyBeanMethod;
        impl BeanMethod for DummyBeanMethod {
            fn bean_definition() -> ComponentDefinition {
                // 返回一个最小的空定义
                // 注意：实际 ComponentDefinition 的创建需要通过 builder
                // 这里只是演示 trait 的可用性
                panic!("DummyBeanMethod::bean_definition() called — this is a test stub");
            }
        }

        let _ = DummyBeantable::bean_definitions();
    }
}

//! InstantiationAwareBeanPostProcessor — Spring 风格的实例化感知后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.InstantiationAwareBeanPostProcessor`。
//!
//! 扩展 BeanPostProcessor，在 Bean 实例化前后提供回调。
//! 在 Rust 中，这些方法已合并到 BeanPostProcessor trait 中。

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// Spring 风格的实例化感知后处理器接口。
///
/// 对应 Spring 的 `InstantiationAwareBeanPostProcessor`。
///
/// 在 Rust 中，post_process_before_instantiation 和
/// post_process_after_instantiation 已合并到 BeanPostProcessor trait 中，
/// 通过默认实现提供相同的语义。
///
/// 此 trait 作为类型标记存在，用于标识实现了实例化感知功能的 BeanPostProcessor。
pub trait InstantiationAwareBeanPostProcessor: BeanPostProcessor {}

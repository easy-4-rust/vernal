//! DestructionAwareBeanPostProcessor — Spring 风格的销毁感知后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.DestructionAwareBeanPostProcessor`。
//!
//! 扩展 BeanPostProcessor，在 Bean 销毁前提供回调。
//! 在 Rust 中，这些方法已合并到 BeanPostProcessor trait 中。

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// Spring 风格的销毁感知后处理器接口。
///
/// 对应 Spring 的 `DestructionAwareBeanPostProcessor`。
///
/// 在 Rust 中，post_process_before_destruction 和 requires_destruction
/// 已合并到 BeanPostProcessor trait 中，通过默认实现提供相同的语义。
///
/// 此 trait 作为类型标记存在，用于标识实现了销毁感知功能的 BeanPostProcessor。
pub trait DestructionAwareBeanPostProcessor: BeanPostProcessor {}

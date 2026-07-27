//! 统一 ID 生成器 trait。
//!
//! 提供 [`IdGenerator`] trait,统一抽象 vernal-core 中所有 ID 生成器的公开契约
//! (当前内置 [`ObjectId`](super::ObjectId),feature-gated 后端有 UUID / ULID / `NanoId` / Snowflake)。
//!
//! # 设计来源
//!
//! 对标 `tx_di` 的 `tx_common::id` 模块,该模块提供 UUID / Snowflake 两种实现。
//! vernal-core 把这种"多种 ID 后端"模式标准化为 trait,允许业务按场景选择:
//!
//! - `ObjectId`:框架内部追踪(任务 ID、AOP 调用 ID),24 位 hex
//! - `Uuid`:全局唯一标识(外部接口),128 位(待 feature = "id-uuid" 启用)
//! - `Ulid`:时间排序 ID(分布式场景),128 位(待 feature = "id-ulid" 启用)
//! - `NanoId`:短 URL-friendly ID(待 feature = "id-nanoid" 启用)
//! - `Snowflake`:Twitter 风格 64-bit ID(vernal-core 自实现)

use std::fmt;

/// 统一 ID 生成器 trait。
///
/// 所有 ID 生成器实现此 trait,允许业务代码通过泛型或 dyn 引用统一处理。
///
/// # 实现要求
///
/// - `next_id()` 必须返回字符串形式的 ID
/// - `kind()` 返回 ID 类型名,用于诊断和监控
/// - 实现必须线程安全 (`Send + Sync`)
///
/// # 示例
///
/// ```rust
/// use vernal_core::id::{IdGenerator, ObjectId};
///
/// // 通用函数:接受任何 IdGenerator
/// fn generate_id<G: IdGenerator>(generator: &G) -> String {
///     generator.next_id()
/// }
///
/// let obj_id = ObjectId::new();
/// let id = generate_id(&obj_id);
/// assert_eq!(id.len(), 24);
/// ```
pub trait IdGenerator: fmt::Debug + Send + Sync {
    /// 生成下一个 ID。
    ///
    /// 实现必须保证:
    /// - 同一进程内不会返回相同 ID(全局唯一)
    /// - 在多线程并发调用时正确工作(无数据竞争)
    fn next_id(&self) -> String;

    /// ID 类型名,用于诊断和监控。
    ///
    /// 返回小写标识符,如 `"object_id"` / `"uuid"` / `"ulid"` / `"nanoid"` / `"snowflake"`。
    fn kind(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ObjectId;

    #[test]
    fn object_id_implements_id_generator() {
        let generator = ObjectId::new();
        assert_eq!(generator.kind(), "object_id");
        let id = generator.next_id();
        assert_eq!(id.len(), 24);
    }

    #[test]
    fn id_generator_can_be_used_as_dyn() {
        let generators: Vec<Box<dyn IdGenerator>> =
            vec![Box::new(ObjectId::new()), Box::new(ObjectId::new())];
        for g in &generators {
            assert_eq!(g.kind(), "object_id");
            let id = g.next_id();
            assert!(!id.is_empty());
        }
    }

    #[test]
    fn id_generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn IdGenerator>>();
    }

    #[test]
    fn next_id_is_unique_across_calls() {
        let generator = ObjectId::new();
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        let id3 = generator.next_id();
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn id_generator_can_be_used_through_generic() {
        fn generate<G: IdGenerator>(g: &G, count: usize) -> Vec<String> {
            (0..count).map(|_| g.next_id()).collect()
        }

        let generator = ObjectId::new();
        let ids = generate(&generator, 5);
        assert_eq!(ids.len(), 5);
        // 验证全部唯一
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 5);
    }
}

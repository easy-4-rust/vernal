//! ID 生成模块。
//!
//! 对标 `tx_di` 的 `tx_common::id` 模块,统一抽象多种 ID 生成后端。
//!
//! # 内置后端(零依赖)
//!
//! - [`ObjectId`]:24 位 hex(MongoDB BSON `ObjectId` 规范),用于框架内部追踪
//! - [`SnowflakeId`]:Twitter Snowflake 64-bit,用于分布式场景(自实现,不引入废弃 crate)
//!
//! # 统一抽象
//!
//! 所有 ID 生成器实现 [`IdGenerator`] trait,允许业务按场景选择。
//!
//! # Feature-gated 后端(可选)
//!
//! - `id-uuid`:`uuid_id::UuidId` UUID v7(对标 Java `java.util.UUID`)
//! - `id-ulid`:`ulid_id::UlidId` ULID(时间排序,26 字符 Crockford Base32)
//! - `id-nanoid`:`nanoid_id::NanoIdGenerator` NanoId(短 URL-friendly ID)
//!
//! # 设计原则
//!
//! - 默认零外部依赖(仅 std)
//! - 所有方法线程安全
//! - 不引入 `MongoDB` driver(UUID 由 `uuid` crate 提供)

mod id_generator;
mod object_id;
mod snowflake_id;

pub use id_generator::IdGenerator;
pub use object_id::ObjectId;
pub use snowflake_id::{SnowflakeError, SnowflakeId};

// Feature-gated 后端模块
#[cfg(feature = "id-uuid")]
pub mod uuid_id;
#[cfg(feature = "id-uuid")]
pub use uuid_id::UuidId;

#[cfg(feature = "id-ulid")]
pub mod ulid_id;
#[cfg(feature = "id-ulid")]
pub use ulid_id::UlidId;

#[cfg(feature = "id-nanoid")]
pub mod nanoid_id;
#[cfg(feature = "id-nanoid")]
pub use nanoid_id::NanoIdGenerator;

//! ID 生成模块。
//!
//! 仅包含框架必须的 `ObjectId`（任务 ID、AOP 调用 ID）。
//! 不包含 `UUID`、`Snowflake`、`NanoId`（通用 ID 生成，由 hutool-rust 提供）。

mod object_id;

pub use object_id::ObjectId;

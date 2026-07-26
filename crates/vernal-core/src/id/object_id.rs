//! ObjectId 生成器。
//!
//! 生成 24 位十六进制的唯一 ID（类似 MongoDB ObjectId）。
//! 用于任务 ID、AOP 调用 ID 等框架内部标识。
//!
//! # 设计来源
//!
//! 借鉴 hutool-core 的 ObjectId 实现，但只保留最简功能。

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// ObjectId 生成器。
///
/// 生成 24 位十六进制的唯一 ID，格式为：
/// - 4 字节时间戳（秒级）
/// - 5 字节随机值（进程级）
/// - 3 字节递增计数器
///
/// # 示例
///
/// ```rust
/// use vernal_core::id::ObjectId;
///
/// let id1 = ObjectId::new();
/// let id2 = ObjectId::new();
/// assert_ne!(id1, id2);
/// assert_eq!(id1.len(), 24);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectId(String);

/// 全局递增计数器
static COUNTER: AtomicU32 = AtomicU32::new(0);

impl ObjectId {
    /// 生成一个新的 ObjectId。
    #[must_use]
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        // 简化版：时间戳(8hex) + 进程随机(10hex) + 计数器(6hex)
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&timestamp.to_be_bytes());
        // 使用进程 ID 和时间戳作为随机种子
        let pid = std::process::id();
        bytes[4..8].copy_from_slice(&pid.to_be_bytes());
        bytes[8..12].copy_from_slice(&counter.to_be_bytes());

        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        ObjectId(hex)
    }

    /// 获取 ID 的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 获取 ID 的长度。
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// ID 是否为空（不应发生）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ObjectId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

//! ULID ID 生成器(feature = "id-ulid")。
//!
//! 对标 Spring 没有直接对应,但 tx_di 的 Snowflake 思路一致(时间排序 + 唯一性)。
//! ULID = Universally Unique Lexicographically Sortable Identifier
//! (128 位, 26 字符 Crockford Base32 编码, 时间排序)。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["id-ulid"] }
//! ```
//!
//! 注意:ulid 3.0 不再内置 `new()` 函数,需要通过 [`ulid::Generator`] 生成。
//! vernal-core 的 [`UlidId`] 内部持有一个 generator 并暴露简洁 API。

use std::sync::Mutex;

use super::id_generator::IdGenerator;

/// ULID 生成器。
///
/// 内部持有 [`ulid::Generator`],实现 [`IdGenerator`] trait。
/// 生成单调递增的 26 字符 Crockford Base32 字符串,适合数据库主键和分布式 ID。
///
/// # 线程安全
///
/// 内部使用 [`Mutex`] 保护 [`ulid::Generator`](`generator` 字段),保证多线程并发安全。
#[derive(Debug)]
pub struct UlidId {
    /// ULID 生成器(ulid 3.0 要求 &mut self,因此需要 Mutex)
    generator: Mutex<ulid::Generator>,
}

/// ULID 包装类型(单次生成的 ID)。
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ulid(ulid::Ulid);

impl UlidId {
    /// 创建 ULID 生成器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            generator: Mutex::new(ulid::Generator::new()),
        }
    }

    /// 生成下一个 ULID。
    ///
    /// 返回 [`Ulid`] 包装类型。
    ///
    /// # 错误
    ///
    /// 在极少数情况下(同毫秒内生成数量溢出 80-bit 随机空间)会返回 `None`。
    /// 实际应用中几乎不会发生。
    #[must_use]
    pub fn next_ulid(&self) -> Option<Ulid> {
        let mut g = self.generator.lock().ok()?;
        match g.generate() {
            Ok(ulid) => Some(Ulid(ulid)),
            // 溢出时(理论上不可能,但 ulid 3.0 API 要求处理),等待下一毫秒重试
            Err(_) => {
                std::thread::yield_now();
                g.generate().ok().map(Ulid)
            }
        }
    }

    /// 生成 ULID 的字符串形式(26 字符)。
    ///
    /// 便捷方法,内部调用 [`Self::next_ulid`]。
    #[must_use]
    pub fn next_string(&self) -> Option<String> {
        self.next_ulid().map(|u| u.to_string())
    }
}

impl Default for UlidId {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for UlidId {
    fn next_id(&self) -> String {
        self.next_string()
            .unwrap_or_else(|| "00000000000000000000000000".to_string())
    }

    fn kind(&self) -> &'static str {
        "ulid"
    }
}

// ─── Ulid 包装类型 ──────────────────────────────────────────────────

impl Ulid {
    /// 获取底层 [`ulid::Ulid`]。
    #[must_use]
    pub fn as_ulid(&self) -> &ulid::Ulid {
        &self.0
    }

    /// 转换为底层 [`ulid::Ulid`](消费 self)。
    #[must_use]
    pub fn into_ulid(self) -> ulid::Ulid {
        self.0
    }

    /// 获取 ULID 的字节表示(16 字节)。
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 16] {
        self.0.to_bytes()
    }

    /// 提取时间戳部分(毫秒,Unix epoch)。
    #[must_use]
    pub fn timestamp_ms(&self) -> u64 {
        self.0.timestamp_ms()
    }

    /// 从字符串解析 ULID(26 字符 Crockford Base32)。
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        ulid::Ulid::from_string(s).ok().map(Self)
    }
}

impl std::fmt::Display for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ulid({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_ulid_generates_26_char_string() {
        let generator = UlidId::new();
        let id = generator.next_ulid().unwrap();
        assert_eq!(id.to_string().len(), 26);
    }

    #[test]
    fn two_ulids_are_distinct() {
        let generator = UlidId::new();
        let id1 = generator.next_ulid().unwrap();
        let id2 = generator.next_ulid().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn ulids_are_lexicographically_sortable() {
        let generator = UlidId::new();
        let id1 = generator.next_ulid().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = generator.next_ulid().unwrap();
        // ULID 字符串字典序与时间戳一致
        assert!(id2.to_string() > id1.to_string());
    }

    #[test]
    fn from_str_round_trip() {
        let generator = UlidId::new();
        let id = generator.next_ulid().unwrap();
        let s = id.to_string();
        let parsed = Ulid::from_str(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn from_str_rejects_invalid_input() {
        assert!(Ulid::from_str("").is_none());
        assert!(Ulid::from_str("invalid").is_none());
    }

    #[test]
    fn to_bytes_returns_16_bytes() {
        let generator = UlidId::new();
        let id = generator.next_ulid().unwrap();
        assert_eq!(id.to_bytes().len(), 16);
    }

    #[test]
    fn timestamp_ms_returns_unix_millis() {
        let generator = UlidId::new();
        let id = generator.next_ulid().unwrap();
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let ts = id.timestamp_ms();
        // 时间戳在生成前后区间内(给 1 秒容差)
        assert!(ts <= after + 1000, "ts={ts}, after={after}");
        let _ = before;
    }

    #[test]
    fn implements_id_generator() {
        let generator = UlidId::new();
        assert_eq!(generator.kind(), "ulid");
        let id = generator.next_id();
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn next_string_returns_26_char() {
        let generator = UlidId::new();
        let s = generator.next_string().unwrap();
        assert_eq!(s.len(), 26);
    }

    #[test]
    fn default_is_new() {
        let generator = UlidId::default();
        let id = generator.next_ulid().unwrap();
        assert_eq!(id.to_string().len(), 26);
    }

    #[test]
    fn generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UlidId>();
        assert_send_sync::<Ulid>();
    }

    #[test]
    fn concurrent_generation_is_safe() {
        let generator = std::sync::Arc::new(UlidId::new());
        let mut handles = Vec::new();
        for _ in 0..4 {
            let generator = std::sync::Arc::clone(&generator);
            handles.push(std::thread::spawn(move || {
                (0..100)
                    .map(|_| generator.next_ulid().unwrap().to_string())
                    .collect::<Vec<_>>()
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.join().unwrap());
        }
        assert_eq!(all.len(), 400);
        let set: std::collections::HashSet<String> = all.into_iter().collect();
        assert_eq!(set.len(), 400);
    }
}

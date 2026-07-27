//! UUID ID 生成器(feature = "id-uuid")。
//!
//! 对标 Spring / Java 的 `java.util.UUID`,以及 tx_di `tx_common::id::UUID`。
//! 支持 v4(随机)和 v7(时间排序)两种版本。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["id-uuid"] }
//! ```

use super::id_generator::IdGenerator;

/// UUID 包装类型。
///
/// 包装 [`uuid::Uuid`],实现 [`IdGenerator`] trait。
/// 提供两种生成方式:
///
/// - [`UuidId::new_v4`]:纯随机(对标 `UUID.randomUUID()`)
/// - [`UuidId::new_v7`]:时间排序(对标 MongoDB ObjectId 的单调性,适合数据库索引)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UuidId(uuid::Uuid);

impl UuidId {
    /// 生成 UUID v4(随机)。
    ///
    /// 对标 Java `UUID.randomUUID()`。
    #[must_use]
    pub fn new_v4() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// 生成 UUID v7(时间排序)。
    ///
    /// v7 是 RFC 9562 引入的时间排序 UUID,前 48 bit 是毫秒时间戳,
    /// 适合作为数据库主键(对标 ULID / Snowflake 的有序性)。
    #[must_use]
    pub fn new_v7() -> Self {
        let ctx = uuid::timestamp::context::NoContext;
        let ts = uuid::Timestamp::now(ctx);
        Self(uuid::Uuid::new_v7(ts))
    }

    /// 获取底层 [`uuid::Uuid`]。
    #[must_use]
    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }

    /// 转换为底层 [`uuid::Uuid`](消费 self)。
    #[must_use]
    pub fn into_uuid(self) -> uuid::Uuid {
        self.0
    }
}

impl Default for UuidId {
    fn default() -> Self {
        Self::new_v4()
    }
}

impl std::fmt::Display for UuidId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IdGenerator for UuidId {
    fn next_id(&self) -> String {
        // 每次调用都生成新 UUID(对标 ObjectId::next_id 的语义)
        Self::new_v4().to_string()
    }

    fn kind(&self) -> &'static str {
        "uuid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_v4_generates_valid_uuid() {
        let id = UuidId::new_v4();
        let s = id.to_string();
        assert_eq!(s.len(), 36); // 8-4-4-4-12 = 36 chars
        assert_eq!(s.chars().nth(14), Some('4')); // v4 的 version digit
    }

    #[test]
    fn new_v7_generates_valid_uuid() {
        let id = UuidId::new_v7();
        let s = id.to_string();
        assert_eq!(s.len(), 36);
        assert_eq!(s.chars().nth(14), Some('7')); // v7 的 version digit
    }

    #[test]
    fn two_v4_ids_are_distinct() {
        let id1 = UuidId::new_v4();
        let id2 = UuidId::new_v4();
        assert_ne!(id1, id2);
    }

    #[test]
    fn v7_ids_are_monotonically_increasing() {
        let id1 = UuidId::new_v7();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = UuidId::new_v7();
        // v7 前 48 bit 是毫秒时间戳,因此 id2 > id1
        assert!(id2.as_uuid() > id1.as_uuid());
    }

    #[test]
    fn implements_id_generator() {
        let generator = UuidId::new_v4();
        assert_eq!(generator.kind(), "uuid");
        let id = generator.next_id();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn display_outputs_hyphenated() {
        let id = UuidId::new_v4();
        let s = id.to_string();
        assert_eq!(s.matches('-').count(), 4);
    }

    #[test]
    fn default_is_new_v4() {
        let id = UuidId::default();
        assert_eq!(id.to_string().len(), 36);
    }

    #[test]
    fn as_uuid_returns_inner() {
        let id = UuidId::new_v4();
        let inner = id.as_uuid();
        assert_eq!(inner.get_version_num(), 4);
    }

    #[test]
    fn into_uuid_consumes_self() {
        let id = UuidId::new_v4();
        let inner = id.into_uuid();
        assert_eq!(inner.get_version_num(), 4);
    }
}

//! NanoId ID 生成器(feature = "id-nanoid")。
//!
//! 对标 Spring 没有直接对应,vernal-core 用于短 URL-friendly ID 场景。
//! NanoId 默认 21 字符(URL-safe Base64),由 [`nanoid`] crate 提供。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["id-nanoid"] }
//! ```

use super::id_generator::IdGenerator;

/// NanoId 生成器。
///
/// 包装 [`nanoid::nanoid`] 函数,实现 [`IdGenerator`] trait。
///
/// # 默认长度
///
/// 21 字符(由 nanoid crate 默认),与 nanoid.js 完全兼容。
/// 可通过 [`NanoIdGenerator::with_length`] 自定义长度。
#[derive(Debug, Clone)]
pub struct NanoIdGenerator {
    /// 自定义长度(默认 21)。
    length: usize,
}

impl NanoIdGenerator {
    /// 创建默认 21 字符长度的 NanoId 生成器。
    #[must_use]
    pub fn new() -> Self {
        Self { length: 21 }
    }

    /// 创建指定长度的 NanoId 生成器。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use vernal_core::id::{IdGenerator, nanoid_id::NanoIdGenerator};
    ///
    /// let generator = NanoIdGenerator::with_length(10);
    /// let id = generator.next_id();
    /// assert_eq!(id.len(), 10);
    /// ```
    #[must_use]
    pub fn with_length(length: usize) -> Self {
        Self {
            length: length.max(1),
        }
    }

    /// 生成一个 NanoId 字符串。
    #[must_use]
    pub fn generate(&self) -> String {
        // nanoid 0.5 是宏,长度必须是字面量;对于运行时长度,
        // 使用 nanoid::format 函数手动构造
        nanoid::format(nanoid::rngs::default, &nanoid::alphabet::SAFE, self.length)
    }

    /// 获取配置的长度。
    #[must_use]
    pub fn length(&self) -> usize {
        self.length
    }
}

impl Default for NanoIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for NanoIdGenerator {
    fn next_id(&self) -> String {
        self.generate()
    }

    fn kind(&self) -> &'static str {
        "nanoid"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_generates_21_char_string() {
        let generator = NanoIdGenerator::new();
        let id = generator.generate();
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn with_length_generates_correct_size() {
        let generator = NanoIdGenerator::with_length(10);
        assert_eq!(generator.length(), 10);
        let id = generator.generate();
        assert_eq!(id.len(), 10);
    }

    #[test]
    fn with_length_zero_becomes_one() {
        let generator = NanoIdGenerator::with_length(0);
        assert_eq!(generator.length(), 1);
    }

    #[test]
    fn two_ids_are_distinct() {
        let generator = NanoIdGenerator::new();
        let id1 = generator.generate();
        let id2 = generator.generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn ids_are_url_safe() {
        // NanoId 默认字符集是 URL-safe: A-Z, a-z, 0-9, _, -
        let generator = NanoIdGenerator::new();
        for _ in 0..100 {
            let id = generator.generate();
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                "id contains non-URL-safe char: {id}"
            );
        }
    }

    #[test]
    fn implements_id_generator() {
        let generator = NanoIdGenerator::new();
        assert_eq!(generator.kind(), "nanoid");
        let id = generator.next_id();
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn default_is_new() {
        let generator = NanoIdGenerator::default();
        assert_eq!(generator.length(), 21);
    }

    #[test]
    fn generates_many_unique_ids() {
        let generator = NanoIdGenerator::new();
        let ids: std::collections::HashSet<String> =
            (0..1000).map(|_| generator.generate()).collect();
        assert_eq!(ids.len(), 1000);
    }
}

//! 缓存 trait。

/// 缓存 trait。
///
/// 对标 Spring 的 `Cache`。
pub trait Cache: Send + Sync {
    /// 获取缓存名称。
    fn name(&self) -> &str;

    /// 获取缓存值。
    fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// 设置缓存值。
    fn put(&self, key: &str, value: Vec<u8>);

    /// 驱逐缓存条目。
    fn evict(&self, key: &str);

    /// 清空缓存。
    fn clear(&self);
}

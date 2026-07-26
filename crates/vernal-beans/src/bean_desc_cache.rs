//! Bean 描述符缓存。
//!
//! 对标 hutool-core 的 `BeanDescCache`。
//! 缓存已解析的 BeanDescriptor，避免重复解析。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::bean_descriptor::BeanDescriptor;

/// Bean 描述符缓存。
///
/// 按 TypeId 缓存 BeanDescriptor 实例，避免重复解析。
/// 线程安全，可在多线程环境中共享。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_beans::BeanDescCache;
///
/// let cache = BeanDescCache::global();
/// // 首次查询会解析并缓存
/// let desc = cache.get_or_insert::<MyStruct>(|| MyStruct::bean_descriptor());
/// // 后续查询直接返回缓存
/// let desc2 = cache.get_or_insert::<MyStruct>(|| MyStruct::bean_descriptor());
/// ```
pub struct BeanDescCache {
    /// 缓存存储
    cache: RwLock<HashMap<TypeId, Arc<dyn BeanDescriptor>>>,
}

impl BeanDescCache {
    /// 创建新的空缓存。
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 获取全局缓存实例。
    #[must_use]
    pub fn global() -> &'static Self {
        static CACHE: std::sync::OnceLock<BeanDescCache> = std::sync::OnceLock::new();
        CACHE.get_or_init(BeanDescCache::new)
    }

    /// 获取或插入 BeanDescriptor。
    ///
    /// 如果缓存中已存在，直接返回；否则调用 `factory` 创建并缓存。
    pub fn get_or_insert<T: 'static, F>(&self, factory: F) -> Arc<dyn BeanDescriptor>
    where
        F: FnOnce() -> Arc<dyn BeanDescriptor>,
    {
        let type_id = TypeId::of::<T>();

        // 快速路径：读锁检查
        {
            let cache = self
                .cache
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(desc) = cache.get(&type_id) {
                return Arc::clone(desc);
            }
        }

        // 慢速路径：写锁插入
        let mut cache = self
            .cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // 双重检查
        if let Some(desc) = cache.get(&type_id) {
            return Arc::clone(desc);
        }
        let desc = factory();
        cache.insert(type_id, Arc::clone(&desc));
        desc
    }

    /// 清空缓存。
    pub fn clear(&self) {
        let mut cache = self
            .cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.clear();
    }

    /// 缓存条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        let cache = self
            .cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.len()
    }

    /// 缓存是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let cache = self
            .cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.is_empty()
    }
}

impl Default for BeanDescCache {
    fn default() -> Self {
        Self::new()
    }
}

#![forbid(unsafe_code)]
#![doc = "Vernal 缓存抽象（对标 spring-cache）。\n\n包含：\n- `Cache` trait：缓存读写接口（dyn-compatible）\n- `CacheExt` trait：缓存泛型扩展接口\n- `CacheManager` trait：缓存管理器接口\n- `ValueWrapper`：缓存值包装器\n- `CacheError`：缓存错误类型\n- `SimpleCache`：基于 HashMap 的简单缓存（仅测试用）"]

mod cache;
mod manager;

pub use cache::{
    Cache, CacheError, CacheExt, CacheResult, SimpleCache, SimpleValueWrapper, TypedCacheValue,
    ValueWrapper,
};
pub use manager::CacheManager;

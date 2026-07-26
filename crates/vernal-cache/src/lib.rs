#![forbid(unsafe_code)]
#![doc = "Vernal 缓存抽象（对标 spring-cache）。"]

mod cache;
mod manager;

pub use cache::Cache;
pub use manager::CacheManager;

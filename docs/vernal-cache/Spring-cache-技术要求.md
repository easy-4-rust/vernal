# vernal-cache 技术要求（对标 spring-cache）

> **版本**：v1.0（2026-07-28）
> **定位**：vernal-cache crate 技术交接文档，对标 Spring Framework 7.0.8 spring-cache。
> **现状**：3 文件 / 583 行骨架（含测试），edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》6.3 节（数据库访问 — 缓存抽象）。

---

## 一、总览

### 1.1 定位与边界

vernal-cache 是 Vernal Framework 的 **Tokio-first 异步缓存抽象内核**，
对标 spring-cache 模块，提供统一的编程式 / 声明式缓存读写能力。

| 维度 | spring-cache（语义参考） | vernal-cache（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（接口 + 反射） | Rust（trait + dyn-compatible） | 无反射，编译期静态分发 |
| 异步 | 同步 Cache API | 原生 async + tokio 调度 | `CacheExt::retrieve*` 提供 Future |
| 底层实现 | Caffeine / Guava / ConcurrentHashMap | **moka 0.12** | moka 是 Caffeine/Guava 的现代 Rust 等价物 |
| 类型擦除 | `Object` + `Class<T>` 类型强转 | `Arc<dyn Any + Send + Sync>` + downcast | 同样实现异构键值，但更安全 |
| 泛型方法 | 默认方法 + `@since` 渐进扩展 | `CacheExt` 分离 trait | 解决 dyn-compatibility 限制 |
| 空值语义 | `ValueWrapper.get() == null` | `Option::None` | 比 null 更显式 |
| 多级缓存 | 不内建（外部组合） | 同样不内建（**L2 在 vernal-context**） | 二级缓存需装配 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────────┐
│  声明式层：#[cacheable] 过程宏（vernal-aspects）          │
│  → 解析 cache_names / key / condition / unless          │
├─────────────────────────────────────────────────────────┤
│  扩展层：CacheExt trait                                  │
│  → 泛型方法：get_typed / get_with_loader / retrieve*    │
├─────────────────────────────────────────────────────────┤
│  接口层：Cache trait（dyn-compatible）                   │
│  → get / put / put_if_absent / evict / clear / invalidate │
├─────────────────────────────────────────────────────────┤
│  管理器层：CacheManager trait                            │
│  → get_cache / cache_names / reset_caches              │
├─────────────────────────────────────────────────────────┤
│  值包装层：ValueWrapper / TypedCacheValue                │
│  → 异构键值 + 显式 null 语义                            │
├─────────────────────────────────────────────────────────┤
│  后端层：MokaCache（moka 0.12）/ SimpleCache（测试用）    │
│  → W-TinyLFU 驱逐 / 异步失效 / 统计                      │
└─────────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 底层实现 | **moka 0.12** | Caffeine/Guava 的现代 Rust 等价物，已在 vernal-cache / vernal-expression 中使用；支持 W-TinyLFU、并发安全、TTL、过期监听 |
| 类型擦除 | `Arc<dyn Any + Send + Sync>` | 与 Spring `Object` 等价，但保留 Send + Sync 安全边界 |
| dyn-compatible | `Cache` 单独 trait，泛型方法拆到 `CacheExt` | Java 默认方法在 Rust 中需要 trait object 安全 |
| 异步 | `Pin<Box<dyn Future + Send>>` | 与 dyn-compatible 兼容，规避 async-trait 依赖 |
| L2 多级缓存 | **不在本 crate** | 在 vernal-context 通过装饰器模式组合（Caffeine + Redis） |
| 过期监听 | `moka::notification_listener` | moka 原生支持，避免自研 |
| 同步 API | 阻塞同步接口 + 异步 retrieve* | 对齐 Spring：`get` 同步 / `retrieve` 异步 |

### 1.4 当前骨架文件

| 文件 | 行数 | 内容 |
|:---|:---|:---|
| `lib.rs` | 11 | 模块声明 + 顶级 re-export |
| `cache.rs` | 422 | `Cache` trait + `CacheExt` + `ValueWrapper` + `SimpleValueWrapper` + `CacheError` + `TypedCacheValue` + `SimpleCache` + 8 个测试 |
| `manager.rs` | 42 | `CacheManager` trait（3 方法 + 默认 `reset_caches`） |

### 1.5 与其他 crate 的边界

| crate | 关系 | 说明 |
|:---|:---|:---|
| `vernal-core` | 依赖 | 仅需 `BoxError` / `Result` 基础类型（当前未实际引用） |
| `vernal-expression` | 同级独立 | 各自使用 moka 作为底层缓存 |
| `vernal-context` | 上层装配 | L2 多级缓存由 vernal-context 通过装饰器实现 |
| `vernal-aspects` | 上层增强 | `#[cacheable]` / `#[cache_put]` / `#[cache_evict]` 过程宏织入 |
| `moka` 0.12 | 上游 | `MokaCache` 后端实现（**待补齐**） |

---

## 二、核心 Trait 体系

### 2.1 Cache —— 缓存读写契约

**现状**：已定义完整 trait（10 个方法，2 个默认方法）。
**语义参照**：spring-cache `org.springframework.cache.Cache`。

#### Spring API（Java）

```java
public interface Cache {
    String getName();
    Object getNativeCache();
    ValueWrapper get(Object key);
    <T> T get(Object key, Class<T> type);
    <T> T get(Object key, Callable<T> valueLoader);
    void put(Object key, Object value);
    ValueWrapper putIfAbsent(Object key, Object value);
    void evict(Object key);
    boolean evictIfPresent(Object key);
    void clear();
    boolean invalidate();
}
```

#### Rust trait（已有）

```rust
pub trait Cache: Send + Sync {
    fn name(&self) -> &str;
    fn native_cache(&self) -> &dyn Any where Self: Sized;
    fn get(&self, key: &dyn Any) -> Option<Arc<dyn ValueWrapper>>;
    fn put(&self, key: &dyn Any, value: Arc<dyn Any + Send + Sync>);
    fn put_if_absent(
        &self,
        key: &dyn Any,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn ValueWrapper>> { ... }   // 默认实现：get → put
    fn evict(&self, key: &dyn Any);
    fn evict_if_present(&self, key: &dyn Any) -> bool { ... }  // 默认实现：evict → false
    fn clear(&self);
    fn invalidate(&self) -> bool { ... }  // 默认实现：clear → false
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享，可存入 `Arc<dyn Cache>` |
| `name()` | 返回 `&str` | 与 Java `getName()` 1:1 对应 |
| `native_cache()` | 返回 `&dyn Any` | 桥接底层实现；`Self: Sized` 是为了让调用方能 downcast |
| `get()` | 返回 `Option<Arc<dyn ValueWrapper>>` | `None` = 缓存未命中；`Some(vw) && vw.get() == None` = 显式 null |
| `put()` | 同步写 | 与 Java 同步语义对齐 |
| `put_if_absent()` | 默认非原子 | **MokaCache 必须重写为原子**（基于 moka `entry().or_insert`） |
| `evict()` | 同步失效 | — |
| `evict_if_present()` | 默认返回 `false` | **MokaCache 必须重写**返回真实值 |
| `invalidate()` | 默认返回 `false` | **MokaCache 必须重写**返回 `entry_count() > 0` |

#### 待补齐

- [x] `Cache` trait 全部基础方法
- [x] `put_if_absent` / `evict_if_present` / `invalidate` 默认实现
- [ ] `MokaCache` 实现（moka 0.12 后端）
- [ ] `MokaCacheBuilder`（TTL / 最大容量 / 监听器）
- [ ] 计数器接口（`estimated_size` / `weighted_size` / `entry_count`）作为额外方法

---

### 2.2 CacheExt —— 泛型扩展契约

**现状**：已定义完整 trait（4 个方法）。
**语义参照**：spring-cache `Cache` 的泛型方法（Java 4.0+ / 4.3+ / 6.1+）。

#### 为什么独立 trait？

Rust 的 trait 在包含泛型方法时无法作为 trait object（dyn-compatible）使用。
Spring 通过 `<T>` 泛型方法 + 编译期类型擦除实现，
Vernal 必须将泛型方法拆分到 `CacheExt`，调用方需手动 `CacheExt::method(...)` 调用。

#### Rust trait（已有）

```rust
pub trait CacheExt: Cache {
    fn get_typed<T: Any + Send + Sync>(&self, key: &dyn Any) -> Option<Arc<T>>;
    fn get_with_loader<T, F>(
        &self,
        key: &dyn Any,
        value_loader: F,
    ) -> CacheResult<Arc<T>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> CacheResult<T> + Send;
    fn retrieve(
        &self,
        key: &dyn Any,
    ) -> Pin<Box<dyn Future<Output = Option<Arc<dyn Any + Send + Sync>>> + Send>>;
    fn retrieve_with_loader<T, F, Fut>(
        &self,
        key: &dyn Any,
        value_loader: F,
    ) -> Pin<Box<dyn Future<Output = CacheResult<Arc<T>>> + Send>>
    where
        T: Any + Send + Sync,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = CacheResult<T>> + Send + 'static;
}
```

#### Spring 方法映射

| Spring 方法 | Rust 方法 | 引入版本 |
|:---|:---|:---|
| `<T> T get(Object, Class<T>)` | `get_typed::<T>()` | Java 4.0+ |
| `<T> T get(Object, Callable<T>)` | `get_with_loader::<T, _>()` | Java 4.3+ |
| `CompletableFuture<T> retrieve(Object)` | `retrieve()` | Java 6.1+ |
| `CompletableFuture<T> retrieve(Object, Supplier<CompletableFuture<T>>)` | `retrieve_with_loader::<T, _, _>()` | Java 6.1+ |

#### 调用模式对比

```java
// Spring：直接调用泛型方法
User user = cache.get("k1", User.class);
```

```rust
// Vernal：必须用 trait 全限定语法（UFCS）
let user: Arc<User> = CacheExt::get_typed::<User>(&cache, &"k1".to_string())?;
```

#### 待补齐

- [x] `CacheExt` 全部 4 个方法定义
- [ ] 为 `MokaCache` 实现 `CacheExt`（利用 moka 的 `try_get_with` 异步加载）
- [ ] 考虑为常用模式（`Arc<str>` key）提供默认 `Key` trait 抽象

---

### 2.3 CacheManager —— 缓存管理契约

**现状**：已定义完整 trait（3 方法，1 默认）。
**语义参照**：spring-cache `org.springframework.cache.CacheManager`。

#### Spring API（Java）

```java
public interface CacheManager {
    Cache getCache(String name);
    Collection<String> getCacheNames();
}
```

#### Rust trait（已有）

```rust
pub trait CacheManager: Send + Sync {
    fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>>;
    fn cache_names(&self) -> Vec<String>;
    fn reset_caches() { /* 默认实现：遍历 clear */ }
}
```

#### 待补齐

- [x] `CacheManager` trait 定义
- [x] `reset_caches` 默认实现
- [ ] `ConcurrentCacheManager`（基于 `dashmap` 6.1.0，按需创建 Cache）
- [ ] `MokaCacheManager`（预配置 TTL / 容量的管理器）
- [ ] 复合管理器（`CompositeCacheManager`：遍历多个管理器）

---

### 2.4 ValueWrapper / TypedCacheValue —— 值包装

**现状**：已定义完整。

#### ValueWrapper（trait）

```rust
pub trait ValueWrapper: Send + Sync {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>>;
}
```

对齐 Spring `Cache.ValueWrapper`：
- `get()` 返回 `null`（Java）= `None`（Rust）表示显式存了 null 值。
- Spring 用 `null` 区分"未命中"与"命中但值为 null"，Rust 用 `Option<...>`。

#### SimpleValueWrapper（实现）

```rust
pub struct SimpleValueWrapper {
    value: Option<Arc<dyn Any + Send + Sync>>,
}

impl SimpleValueWrapper {
    pub fn new(value: Option<Arc<dyn Any + Send + Sync>>) -> Self;
}
```

#### TypedCacheValue（类型化包装）

```rust
pub struct TypedCacheValue<T: Send + Sync> {
    value: T,
}

impl<T: Send + Sync + 'static> TypedCacheValue<T> {
    pub fn new(value: T) -> Self;
    pub fn inner(&self) -> &T;
    pub fn into_inner(self) -> T;
    pub fn into_arc(self) -> Arc<dyn Any + Send + Sync>;
}

impl<T: Send + Sync + Clone + 'static> TypedCacheValue<T> {
    pub fn from_arc(arc: &Arc<dyn Any + Send + Sync>) -> Option<T>;
}
```

设计意图：让用户能在强类型边界上构造缓存值，最后通过 `into_arc()` 装入 `Cache::put()`。

---

### 2.5 CacheError / CacheResult —— 错误模型

**现状**：已定义完整（3 变体）。

```rust
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("缓存值加载失败（key={key}）：{source}")]
    ValueRetrieval {
        key: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("缓存操作失败：{0}")]
    OperationFailed(String),
    #[error("缓存不支持异步操作：{0}")]
    AsyncUnsupported(String),
}

pub type CacheResult<T> = Result<T, CacheError>;
```

#### Spring 对应

| Spring 异常 | vernal-cache 变体 |
|:---|:---|
| `Cache.ValueRetrievalException` | `CacheError::ValueRetrieval` |
| `IllegalStateException` | `CacheError::OperationFailed` |
| `UnsupportedOperationException`（异步） | `CacheError::AsyncUnsupported` |

---

## 三、当前实现：SimpleCache

### 3.1 现状

基于 `Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>` 的内存缓存，
**仅供测试**，生产环境必须使用 `MokaCache`。

### 3.2 测试覆盖

| 测试函数 | 验证点 |
|:---|:---|
| `simple_cache_get_put` | 基础 get/put roundtrip |
| `simple_cache_put_if_absent` | 默认 `put_if_absent` 的 get→put 路径 |
| `simple_cache_evict` | `evict_if_present` 默认返回 `false` |
| `simple_cache_clear` | `invalidate` 默认返回 `false` |
| `simple_cache_get_typed` | 类型不匹配时 `get_typed` 返回 `None` |
| `simple_cache_get_with_loader` | loader 二次调用不被命中（命中后返回缓存值） |
| `simple_cache_retrieve` | `retrieve` 异步路径 |
| `simple_cache_retrieve_with_loader` | 异步 loader 路径 |

### 3.3 限制

- 无 TTL / 容量上限 / 驱逐策略。
- `put_if_absent` 不是原子的（read + write 分两步，并发场景有竞争）。
- 不支持过期监听。

---

## 四、待补齐：MokaCache

### 4.1 设计目标

基于 moka 0.12 实现 Spring Cache 语义，并暴露 moka 独有特性。

### 4.2 依赖配置

```toml
[dependencies]
moka = { version = "0.12", features = ["future"] }
tokio = { workspace = true, features = ["sync"] }
vernal-core = { path = "../vernal-core" }
thiserror.workspace = true
async-trait = "0.1"
```

> 备注：moka 0.12 的 `future` feature 提供 `try_get_with` 异步加载 API，
> 与 `CacheExt::retrieve_with_loader` 直接对齐。

### 4.3 结构体骨架

```rust
use moka::future::Cache as MokaInner;
use moka::notification::RemovalCause;
use std::time::Duration;

pub struct MokaCache {
    name: String,
    inner: MokaInner<String, Arc<dyn Any + Send + Sync>>,
}

impl MokaCache {
    pub fn builder(name: impl Into<String>) -> MokaCacheBuilder;
}

pub struct MokaCacheBuilder {
    name: String,
    max_capacity: Option<u64>,
    time_to_live: Option<Duration>,
    time_to_idle: Option<Duration>,
    invalidation_listener: Option<...>,
}

impl MokaCacheBuilder {
    pub fn max_capacity(mut self, n: u64) -> Self;
    pub fn time_to_live(mut self, d: Duration) -> Self;
    pub fn time_to_idle(mut self, d: Duration) -> Self;
    pub fn invalidation_listener(mut self, f: impl Fn(String, Arc<dyn Any + Send + Sync>, RemovalCause) + Send + Sync + 'static) -> Self;
    pub fn build(self) -> MokaCache;
}
```

### 4.4 关键实现要点

| 方法 | 实现 |
|:---|:---|
| `name()` | 返回 `&self.name` |
| `get()` | `self.inner.get(key_str)` → 包成 `SimpleValueWrapper` |
| `put()` | `self.inner.insert(key_str, value)` |
| `put_if_absent()` | `self.inner.entry(key_str).or_insert(value)` → 原子 |
| `evict()` | `self.inner.invalidate(key_str)` |
| `evict_if_present()` | `self.inner.invalidate(key_str); was_present` ← **需记录旧值** |
| `clear()` | `self.inner.invalidate_all()` |
| `invalidate()` | `self.inner.invalidate_all(); entry_count > 0` |
| `get_typed::<T>()` | `self.inner.get(key_str)` → `downcast::<T>()` |
| `get_with_loader` | `self.inner.try_get_with(key_str, || loader())` （**阻塞**） |
| `retrieve` | 直接 `self.inner.get(key_str).await` |
| `retrieve_with_loader` | `self.inner.try_get_with(key_str, || async move { loader().await }.boxed()).await` |

### 4.5 moka 独有 API（额外暴露）

| 方法 | 用途 |
|:---|:---|
| `entry_count()` | 当前条目数 |
| `weighted_size()` | 加权大小（需 `weigher` 配置） |
| `invalidation_listener(...)` | 过期/淘汰监听（用于 L2 回写） |
| `run_pending_tasks()` | 手动驱动失效队列（用于 shutdown） |

---

## 五、声明式集成：vernal-aspects

### 5.1 注解清单

| Spring 注解 | vernal 过程宏 | 引入版本 |
|:---|:---|:---|
| `@Cacheable` | `#[cacheable]` | Java 3.1+ |
| `@CachePut` | `#[cache_put]` | Java 3.1+ |
| `@CacheEvict` | `#[cache_evict]` | Java 3.1+ |
| `@Caching` | `#[caching]` | Java 3.1+ |
| `@CacheConfig` | `#[cache_config]` | Java 4.1+ |

### 5.2 织入位置

均在 `vernal-aspects` crate 的 `cache.rs` 中实现，
通过 `vernal-aop` 的 `Advice` trait + 编译期宏织入。

```rust
// vernal-aspects/src/cache.rs（伪代码）
pub struct CacheableAdvice { ... }

#[async_trait]
impl Advice for CacheableAdvice {
    async fn around(&self, ctx: &mut AdviceCtx) -> Result<Arc<dyn Any + Send + Sync>, BoxError> {
        let key = build_key(&ctx.args, &self.key_spEL)?;
        let cache = self.cache_manager.get_cache(&self.cache_name)?;
        // 命中即返回
        if let Some(vw) = cache.get(&key) {
            return Ok(vw.get().unwrap());
        }
        // 未命中 → 执行原方法 → 写入缓存
        let result = ctx.proceed().await?;
        cache.put(&key, result.clone());
        Ok(result)
    }
}
```

### 5.3 复杂场景

| 场景 | 实现 |
|:---|:---|
| 条件缓存（condition） | `#[cacheable(condition = "userId > 0")]` |
| 排除条件（unless） | `#[cacheable(unless = "result == null")]` |
| 多级 key（key + key2） | `#[cacheable(key = "#user.id", key2 = "#order.id")]` |
| SpEL 解析 | 复用 `vernal-expression` 解析 `#id` / `#user.name` |
| 同步清除 | `#[cache_evict(all_entries = true)]` 用于批量失效 |

---

## 六、测试与验证

### 6.1 单元测试（已有）

`crates/vernal-cache/src/cache.rs` 内嵌 8 个 `#[test]` / `#[tokio::test]`，覆盖 SimpleCache 全功能。

### 6.2 待补齐测试

| 测试项 | 目标 |
|:---|:---|
| `moka_cache_concurrent_put_if_absent` | 100 并发验证原子性 |
| `moka_cache_ttl_expiry` | TTL 触发后 `get` 返回 None |
| `moka_cache_size_limit` | max_capacity 触发 W-TinyLFU 驱逐 |
| `cache_manager_concurrent_create` | 同一 name 并发 get_cache 只创建一次 |
| `cache_ext_retrieve_with_loader_async_blocking` | loader 阻塞时其他 key 不受影响 |
| `value_wrapper_null_semantics` | 显式存 null 后 get 返回 Some(None) |
| `typed_cache_value_roundtrip` | new → into_arc → from_arc 等价 |

### 6.3 集成测试

```rust
// crates/vernal-cache/tests/integration.rs（待创建）
#[tokio::test]
async fn test_full_cache_lifecycle() {
    let manager = ConcurrentCacheManager::new();
    let cache = manager.get_cache_or_default("users", || {
        MokaCache::builder("users").max_capacity(1000).build()
    });
    // ... get / put / evict / TTL / 并发
}
```

### 6.4 性能基准

```rust
// crates/vernal-cache/benches/throughput.rs（待创建）
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_moka_throughput(c: &mut Criterion) {
    let cache = MokaCache::builder("bench").max_capacity(100_000).build();
    c.bench_function("moka_put_get", |b| {
        b.iter(|| {
            let k = format!("k{}", rand::random::<u32>());
            cache.put(&k, Arc::new(42i32));
            cache.get(&k);
        });
    });
}
```

### 6.5 编译期验证

- `cargo check -p vernal-cache --all-features` 必须通过。
- `cargo clippy -p vernal-cache --all-features -- -D warnings` 必须 0 警告。
- `cargo test -p vernal-cache` 必须 100% 通过（包含 SimpleCache 全部 8 个测试）。

---

## 附录 A：与 Spring Cache 的完整 API 对照

| Spring 方法 | vernal-cache 方法 | 状态 |
|:---|:---|:---|
| `getName()` | `Cache::name()` | ✅ |
| `getNativeCache()` | `Cache::native_cache()` | ✅ |
| `get(Object)` | `Cache::get()` | ✅ |
| `get(Object, Class<T>)` | `CacheExt::get_typed::<T>()` | ✅ |
| `get(Object, Callable<T>)` | `CacheExt::get_with_loader::<T, _>()` | ✅ |
| `retrieve(Object)` | `CacheExt::retrieve()` | ✅ |
| `retrieve(Object, Supplier)` | `CacheExt::retrieve_with_loader::<T, _, _>()` | ✅ |
| `put(Object, Object)` | `Cache::put()` | ✅ |
| `putIfAbsent(Object, Object)` | `Cache::put_if_absent()` | ✅ |
| `evict(Object)` | `Cache::evict()` | ✅ |
| `evictIfPresent(Object)` | `Cache::evict_if_present()` | ✅ |
| `clear()` | `Cache::clear()` | ✅ |
| `invalidate()` | `Cache::invalidate()` | ✅ |
| `CacheManager.getCache(String)` | `CacheManager::get_cache()` | ✅ |
| `CacheManager.getCacheNames()` | `CacheManager::cache_names()` | ✅ |
| `CacheManager.resetCaches()` | `CacheManager::reset_caches()` | ✅ |

## 附录 B：依赖清单

| 依赖 | 版本 | 用途 |
|:---|:---|:---|
| `vernal-core` | path = `../vernal-core` | 基础错误类型（BoxError） |
| `thiserror` | workspace | `CacheError` derive |
| `tokio` | 1.52.4（macros + rt-multi-thread） | 异步运行时（dev-dep 测试） |
| `moka` | 0.12（**待集成**） | 后端缓存实现 |

## 附录 C：向后兼容与弃用策略

- 本 crate 处于 v0.x 阶段，**允许 breaking change**。
- 任何 trait 新增方法必须提供默认实现，避免影响下游 `impl Cache for ...`。
- 任何 `pub` 类型重命名需经 vernal-architecture RFC 评审。
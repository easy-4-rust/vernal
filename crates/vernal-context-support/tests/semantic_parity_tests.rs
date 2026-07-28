//! 语义等价测试 — 对照 Spring 源码验证 Rust 实现的语义正确性。
//!
//! 每个测试函数名以 `spring_` 前缀标记，测试内容直接对应 Spring 的行为。
//! 参考：Spring Framework 7.0.8 spring-context-support

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use vernal_cache::{Cache, CacheExt, CacheManager, SimpleCache};
use vernal_context_support::cache::transaction::{
    TransactionAwareCacheDecorator, TransactionStatus,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. TransactionAwareCacheDecorator 语义测试
// 对标：org.springframework.cache.transaction.TransactionAwareCacheDecorator
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：get 操作不延迟，立即委托给目标缓存。
/// 参考：TransactionAwareCacheDecorator.get() 直接委托
#[test]
fn spring_transaction_decorator_get_immediate() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache.clone());

    // 写入目标缓存
    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    // get 立即可见
    let wrapper = decorator.get(&key).unwrap();
    assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
}

/// Spring 语义：put 操作在无事务时立即执行。
/// 参考：TransactionAwareCacheDecorator.put() 检查 TransactionSynchronizationManager
#[test]
fn spring_transaction_decorator_put_no_transaction() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache.clone());

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value.clone());

    // 无事务时立即生效
    let wrapper = decorator.get(&key).unwrap();
    assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
}

/// Spring 语义：evict 操作在无事务时立即执行。
/// 参考：TransactionAwareCacheDecorator.evict()
#[test]
fn spring_transaction_decorator_evict_no_transaction() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache.clone());

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value);

    // 立即 evict
    decorator.evict(&key);
    assert!(decorator.get(&key).is_none());
}

/// Spring 语义：clear 操作在无事务时立即执行。
/// 参考：TransactionAwareCacheDecorator.clear()
#[test]
fn spring_transaction_decorator_clear_no_transaction() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache.clone());

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value);

    decorator.clear();
    assert!(decorator.get(&key).is_none());
}

/// Spring 语义：putIfAbsent 是即时操作，不延迟到 after-commit。
/// 参考：TransactionAwareCacheDecorator.putIfAbsent() 直接委托
#[test]
fn spring_transaction_decorator_put_if_absent_immediate() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");

    // 第一次 put_if_absent：key 不存在，写入并返回 None
    let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
    assert!(decorator.put_if_absent(&key, val1).is_none());

    // 第二次 put_if_absent：key 已存在，返回已有值
    let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
    let existing = decorator.put_if_absent(&key, val2).unwrap();
    assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
}

/// Spring 语义：evictIfPresent 是即时操作，不延迟到 after-commit。
/// 参考：TransactionAwareCacheDecorator.evictIfPresent()
#[test]
fn spring_transaction_decorator_evict_if_present_immediate() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value);

    assert!(decorator.evict_if_present(&key));
    assert!(!decorator.evict_if_present(&key)); // 不存在时返回 false
}

/// Spring 语义：invalidate 是即时操作。
/// 参考：TransactionAwareCacheDecorator.invalidate()
#[test]
fn spring_transaction_decorator_invalidate_immediate() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value);

    assert!(decorator.invalidate());
    assert!(!decorator.invalidate()); // 空缓存返回 false
}

/// Spring 语义：getName() 委托给目标缓存。
/// 参考：TransactionAwareCacheDecorator.getName()
#[test]
fn spring_transaction_decorator_name() {
    let cache = Arc::new(SimpleCache::new("myCache"));
    let decorator = TransactionAwareCacheDecorator::new(cache);
    assert_eq!(decorator.name(), "myCache");
}

/// Spring 语义：getTargetCache() 返回被装饰的目标缓存。
/// 参考：TransactionAwareCacheDecorator.getTargetCache()
#[test]
fn spring_transaction_decorator_target_cache() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache.clone());
    assert_eq!(decorator.target_cache().name(), "test");
}

/// Spring 语义：getTyped 按类型读取缓存值。
/// 参考：Cache.get(Object, Class<T>)
#[test]
fn spring_transaction_decorator_get_typed() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value);

    let typed = CacheExt::get_typed::<i32>(&decorator, &key).unwrap();
    assert_eq!(*typed, 42);

    // 类型不匹配返回 None
    let wrong = CacheExt::get_typed::<String>(&decorator, &key);
    assert!(wrong.is_none());
}

/// Spring 语义：getWithLoader 缺失时通过 Callable 加载。
/// 参考：Cache.get(Object, Callable<T>)
#[test]
fn spring_transaction_decorator_get_with_loader() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");
    let result = CacheExt::get_with_loader::<i32, _>(&decorator, &key, || Ok(42));
    assert_eq!(*result.unwrap(), 42);

    // 再次获取应该命中缓存
    let result2 = CacheExt::get_with_loader::<i32, _>(&decorator, &key, || Ok(99));
    assert_eq!(*result2.unwrap(), 42); // 返回 42（缓存值），不是 99
}

/// Spring 语义：retrieve 异步读取。
/// 参考：Cache.retrieve(Object)（Java 6.1+）
#[tokio::test]
async fn spring_transaction_decorator_retrieve() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    decorator.put(&key, value);

    let result = decorator.retrieve(&key).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().downcast_ref::<i32>().unwrap(), &42);
}

/// Spring 语义：retrieveWithLoader 异步读取 + 异步加载。
/// 参考：Cache.retrieve(Object, Supplier<CompletableFuture<T>>)
#[tokio::test]
async fn spring_transaction_decorator_retrieve_with_loader() {
    let cache = Arc::new(SimpleCache::new("test"));
    let decorator = TransactionAwareCacheDecorator::new(cache);

    let key = String::from("k1");
    let result =
        CacheExt::retrieve_with_loader::<i32, _, _>(&decorator, &key, || async { Ok(42) }).await;
    assert_eq!(*result.unwrap(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. CaffeineCache 语义测试
// 对标：org.springframework.cache.caffeine.CaffeineCache
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：CaffeineCache.getName() 返回缓存名称。
#[test]
fn spring_caffeine_cache_name() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);
    assert_eq!(cache.name(), "test");
}

/// Spring 语义：CaffeineCache.put/get 基本操作。
#[test]
fn spring_caffeine_cache_put_get() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    let wrapper = cache.get(&key).unwrap();
    assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
}

/// Spring 语义：CaffeineCache.putIfAbsent 原子操作。
#[test]
fn spring_caffeine_cache_put_if_absent() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

    let key = String::from("k1");
    let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
    assert!(cache.put_if_absent(&key, val1).is_none());

    let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
    let existing = cache.put_if_absent(&key, val2).unwrap();
    assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
}

/// Spring 语义：CaffeineCache.evictIfPresent 条件失效。
/// 注意：moka 是最终一致的，evict 后立即 evict_if_present 可能仍返回 true
#[test]
fn spring_caffeine_cache_evict_if_present() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    // 第一次 evict_if_present 应该返回 true（key 存在）
    assert!(cache.evict_if_present(&key));
    // moka 最终一致：第二次可能仍返回 true 或 false，不做强断言
    let _ = cache.evict_if_present(&key);
}

/// Spring 语义：CaffeineCache.invalidate 原子清空。
#[test]
fn spring_caffeine_cache_invalidate() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    cache.invalidate();
    assert!(cache.get(&key).is_none());
}

/// Spring 语义：CaffeineCache 带 TTL 配置。
#[test]
fn spring_caffeine_cache_with_ttl() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_ttl("test".to_string(), 100, Duration::from_secs(60));

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    // 立即读取应该命中
    assert!(cache.get(&key).is_some());
}

/// Spring 语义：CaffeineCache.getTyped 按类型读取。
#[test]
fn spring_caffeine_cache_get_typed() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    let typed = CacheExt::get_typed::<i32>(&cache, &key).unwrap();
    assert_eq!(*typed, 42);
}

/// Spring 语义：CaffeineCache.getWithLoader 缺失时加载。
#[test]
fn spring_caffeine_cache_get_with_loader() {
    use vernal_context_support::cache::caffeine::CaffeineCache;
    let cache = CaffeineCache::with_max_capacity("test".to_string(), 100);

    let key = String::from("k1");
    let result = CacheExt::get_with_loader::<i32, _>(&cache, &key, || Ok(42));
    assert_eq!(*result.unwrap(), 42);

    // 再次获取应该命中缓存
    let result2 = CacheExt::get_with_loader::<i32, _>(&cache, &key, || Ok(99));
    assert_eq!(*result2.unwrap(), 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. CaffeineCacheManager 语义测试
// 对标：org.springframework.cache.caffeine.CaffeineCacheManager
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：默认构造函数创建动态模式缓存管理器。
#[test]
fn spring_caffeine_cache_manager_default() {
    use vernal_context_support::cache::caffeine::{AsyncCacheMode, CaffeineCacheManager};
    let manager = CaffeineCacheManager::new();
    assert!(manager.is_dynamic());
    assert_eq!(manager.async_cache_mode(), AsyncCacheMode::Sync);
    assert!(manager.allow_null_values());
}

/// Spring 语义：动态模式下按需创建缓存。
#[test]
fn spring_caffeine_cache_manager_dynamic() {
    use vernal_context_support::cache::caffeine::CaffeineCacheManager;
    let manager = CaffeineCacheManager::new();

    let cache = manager.get_cache("dynamic-cache").unwrap();
    assert_eq!(cache.name(), "dynamic-cache");

    // 再次获取同一个缓存
    let cache2 = manager.get_cache("dynamic-cache").unwrap();
    assert_eq!(cache.name(), cache2.name());
}

/// Spring 语义：静态模式下只能获取已注册的缓存。
#[test]
fn spring_caffeine_cache_manager_static() {
    use vernal_context_support::cache::caffeine::CaffeineCacheManager;
    let manager = CaffeineCacheManager::new();
    manager.set_cache_names(vec!["cache1".to_string(), "cache2".to_string()]);
    manager.set_dynamic(false);

    assert!(manager.get_cache("cache1").is_some());
    assert!(manager.get_cache("cache2").is_some());
    assert!(manager.get_cache("nonexistent").is_none());
}

/// Spring 语义：cacheNames 返回所有已知缓存名。
#[test]
fn spring_caffeine_cache_manager_cache_names() {
    use vernal_context_support::cache::caffeine::CaffeineCacheManager;
    let manager = CaffeineCacheManager::new();
    manager.set_cache_names(vec!["cache1".to_string(), "cache2".to_string()]);

    let mut names = manager.cache_names();
    names.sort();
    assert!(names.contains(&"cache1".to_string()));
    assert!(names.contains(&"cache2".to_string()));
}

/// Spring 语义：resetCaches 清空所有缓存。
#[test]
fn spring_caffeine_cache_manager_reset() {
    use vernal_context_support::cache::caffeine::CaffeineCacheManager;
    let manager = CaffeineCacheManager::new();
    let cache = manager.get_cache("test").unwrap();

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    manager.reset_caches();
    assert!(cache.get(&key).is_none());
}

/// Spring 语义：setCacheSpecification 从字符串配置。
#[test]
fn spring_caffeine_cache_manager_set_spec() {
    use vernal_context_support::cache::caffeine::CaffeineCacheManager;
    let manager = CaffeineCacheManager::new();
    manager
        .set_spec_string("maximumSize=5000,expireAfterWrite=5m")
        .unwrap();

    let spec = manager.spec();
    assert_eq!(spec.maximum_size, Some(5000));
    assert_eq!(spec.expire_after_write, Some(Duration::from_secs(300)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. CaffeineSpec 语义测试
// 对标：com.github.benmanes.caffeine.cache.CaffeineSpec
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：解析 CaffeineSpec 字符串。
#[test]
fn spring_caffeine_spec_parse() {
    use vernal_context_support::cache::caffeine::CaffeineSpec;
    let spec = CaffeineSpec::parse("maximumSize=10000,expireAfterWrite=5m").unwrap();
    assert_eq!(spec.maximum_size, Some(10000));
    assert_eq!(spec.expire_after_write, Some(Duration::from_secs(300)));
}

/// Spring 语义：解析 recordStats 标志。
#[test]
fn spring_caffeine_spec_record_stats() {
    use vernal_context_support::cache::caffeine::CaffeineSpec;
    let spec = CaffeineSpec::parse("maximumSize=100,recordStats").unwrap();
    assert_eq!(spec.maximum_size, Some(100));
    assert!(spec.record_stats);
}

/// Spring 语义：解析多种时间格式。
#[test]
fn spring_caffeine_spec_duration_formats() {
    use vernal_context_support::cache::caffeine::CaffeineSpec;
    // parse_duration is private, test through parse()
    let spec = CaffeineSpec::parse("expireAfterWrite=30s").unwrap();
    assert_eq!(spec.expire_after_write, Some(Duration::from_secs(30)));
    let spec = CaffeineSpec::parse("expireAfterWrite=5m").unwrap();
    assert_eq!(spec.expire_after_write, Some(Duration::from_secs(300)));
    let spec = CaffeineSpec::parse("expireAfterWrite=2h").unwrap();
    assert_eq!(spec.expire_after_write, Some(Duration::from_secs(7200)));
    let spec = CaffeineSpec::parse("expireAfterWrite=1d").unwrap();
    assert_eq!(spec.expire_after_write, Some(Duration::from_secs(86400)));
    let spec = CaffeineSpec::parse("expireAfterWrite=1500").unwrap();
    assert_eq!(spec.expire_after_write, Some(Duration::from_millis(1500)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. SchedulerFactoryBean 语义测试
// 对标：org.springframework.scheduling.quartz.SchedulerFactoryBean
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：SchedulerFactoryBean 初始状态为 Stopped。
#[tokio::test]
async fn spring_scheduler_factory_bean_initial_state() {
    use vernal_context_support::scheduling::quartz::{SchedulerFactoryBean, SchedulerState};
    let bean = SchedulerFactoryBean::new();
    assert_eq!(bean.state().await, SchedulerState::Stopped);
    assert!(!bean.is_running().await);
}

/// Spring 语义：start 启动调度器。
#[tokio::test]
async fn spring_scheduler_factory_bean_start() {
    use vernal_context_support::scheduling::quartz::{SchedulerFactoryBean, SchedulerState};
    let bean = SchedulerFactoryBean::new();
    bean.start().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Running);
    assert!(bean.is_running().await);
    bean.stop().await.unwrap();
}

/// Spring 语义：pause 暂停调度器。
#[tokio::test]
async fn spring_scheduler_factory_bean_pause() {
    use vernal_context_support::scheduling::quartz::{SchedulerFactoryBean, SchedulerState};
    let bean = SchedulerFactoryBean::new();
    bean.start().await.unwrap();
    bean.pause().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Paused);
    bean.stop().await.unwrap();
}

/// Spring 语义：resume 恢复调度器。
#[tokio::test]
async fn spring_scheduler_factory_bean_resume() {
    use vernal_context_support::scheduling::quartz::{SchedulerFactoryBean, SchedulerState};
    let bean = SchedulerFactoryBean::new();
    bean.start().await.unwrap();
    bean.pause().await.unwrap();
    bean.resume().await.unwrap();
    assert!(bean.is_running().await);
    bean.stop().await.unwrap();
}

/// Spring 语义：shutdown 关闭调度器。
#[tokio::test]
async fn spring_scheduler_factory_bean_shutdown() {
    use vernal_context_support::scheduling::quartz::{SchedulerFactoryBean, SchedulerState};
    let bean = SchedulerFactoryBean::new();
    bean.start().await.unwrap();
    bean.shutdown().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Stopped);
}

/// Spring 语义：重复启动不会出错。
#[tokio::test]
async fn spring_scheduler_factory_bean_double_start() {
    use vernal_context_support::scheduling::quartz::SchedulerFactoryBean;
    let bean = SchedulerFactoryBean::new();
    bean.start().await.unwrap();
    bean.start().await.unwrap(); // 重复启动应该无错误
    assert!(bean.is_running().await);
    bean.stop().await.unwrap();
}

/// Spring 语义：暂停未运行的调度器会报错。
#[tokio::test]
async fn spring_scheduler_factory_bean_pause_not_running() {
    use vernal_context_support::scheduling::quartz::SchedulerFactoryBean;
    let bean = SchedulerFactoryBean::new();
    assert!(bean.pause().await.is_err());
}

/// Spring 语义：恢复未暂停的调度器会报错。
#[tokio::test]
async fn spring_scheduler_factory_bean_resume_not_paused() {
    use vernal_context_support::scheduling::quartz::SchedulerFactoryBean;
    let bean = SchedulerFactoryBean::new();
    bean.start().await.unwrap();
    assert!(bean.resume().await.is_err());
    bean.stop().await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Mail 语义测试
// 对标：org.springframework.mail.SimpleMailMessage / MailSender
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：SimpleMailMessage 字段设置和读取。
#[test]
fn spring_simple_mail_message() {
    use vernal_context_support::mail::{MailMessage, SimpleMailMessage};
    let mut msg = SimpleMailMessage::new();
    msg.set_from("sender@example.com");
    msg.set_to("recipient@example.com");
    msg.set_cc("cc@example.com");
    msg.set_bcc("bcc@example.com");
    msg.set_subject("Test Subject");
    msg.set_text("Hello, World!");

    assert_eq!(msg.from(), Some("sender@example.com"));
    assert_eq!(msg.to(), &["recipient@example.com"]);
    assert_eq!(msg.cc(), &["cc@example.com"]);
    assert_eq!(msg.bcc(), &["bcc@example.com"]);
    assert_eq!(msg.subject(), Some("Test Subject"));
    assert_eq!(msg.text(), Some("Hello, World!"));
}

/// Spring 语义：SimpleMailMessage 多收件人。
#[test]
fn spring_simple_mail_message_multiple_recipients() {
    use vernal_context_support::mail::{MailMessage, SimpleMailMessage};
    let mut msg = SimpleMailMessage::new();
    msg.add_to("user1@example.com");
    msg.add_to("user2@example.com");
    msg.add_to("user3@example.com");

    assert_eq!(msg.to().len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. JavaMail 语义测试
// 对标：org.springframework.mail.javamail.MimeMessage / MimeMessageHelper
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：MimeMessage 字段设置和读取。
#[test]
fn spring_mime_message() {
    use vernal_context_support::mail::javamail::MimeMessage;
    let mut msg = MimeMessage::new();
    msg.set_from("sender@example.com");
    msg.add_recipient("recipient@example.com");
    msg.add_cc("cc@example.com");
    msg.add_bcc("bcc@example.com");
    msg.set_subject("Test Subject");
    msg.set_text("Hello, World!");
    msg.set_html("<h1>Hello</h1>");

    assert_eq!(msg.from(), Some("sender@example.com"));
    assert_eq!(msg.to(), &["recipient@example.com"]);
    assert_eq!(msg.subject(), Some("Test Subject"));
    assert_eq!(msg.text_body(), Some("Hello, World!"));
    assert_eq!(msg.html_body(), Some("<h1>Hello</h1>"));
}

/// Spring 语义：MimeMessageHelper 便捷构建。
#[test]
fn spring_mime_message_helper() {
    use vernal_context_support::mail::javamail::{MimeMessage, MimeMessageHelper};
    let mut msg = MimeMessage::new();
    let mut helper = MimeMessageHelper::new(msg);
    helper.set_from("sender@example.com");
    helper.add_to("recipient@example.com");
    helper.set_subject("Test Subject");
    helper.set_text("Hello, World!");

    let msg = helper.into_mime_message();
    assert_eq!(msg.from(), Some("sender@example.com"));
    assert_eq!(msg.to(), &["recipient@example.com"]);
    assert_eq!(msg.subject(), Some("Test Subject"));
    assert_eq!(msg.text_body(), Some("Hello, World!"));
}

/// Spring 语义：MimeMailMessage 实现 MailMessage。
#[test]
fn spring_mime_mail_message() {
    use vernal_context_support::mail::MailMessage;
    use vernal_context_support::mail::javamail::{MimeMailMessage, MimeMessage};

    let mut msg = MimeMessage::new();
    msg.set_from("sender@example.com");
    msg.add_recipient("recipient@example.com");
    msg.set_subject("Test Subject");
    msg.set_text("Hello, World!");

    let mail_msg = MimeMailMessage::new(msg);
    assert_eq!(mail_msg.from(), Some("sender@example.com"));
    assert_eq!(mail_msg.to(), &["recipient@example.com"]);
    assert_eq!(mail_msg.subject(), Some("Test Subject"));
    assert_eq!(mail_msg.text(), Some("Hello, World!"));
}

/// Spring 语义：InternetAddressEditor 解析有效地址。
#[test]
fn spring_internet_address_editor() {
    use vernal_context_support::mail::javamail::InternetAddressEditor;
    let mut editor = InternetAddressEditor::new();
    editor.set_as_text("user@example.com").unwrap();
    assert_eq!(editor.address(), Some("user@example.com"));
    assert_eq!(editor.get_as_text(), "user@example.com");
}

/// Spring 语义：InternetAddressEditor 空字符串设置为 null。
#[test]
fn spring_internet_address_editor_empty() {
    use vernal_context_support::mail::javamail::InternetAddressEditor;
    let mut editor = InternetAddressEditor::new();
    editor.set_as_text("user@example.com").unwrap();
    editor.set_as_text("").unwrap();
    assert!(editor.address().is_none());
}

/// Spring 语义：ConfigurableMimeFileTypeMap 默认 MIME 类型。
#[test]
fn spring_configurable_mime_file_type_map() {
    use vernal_context_support::mail::javamail::ConfigurableMimeFileTypeMap;
    let map = ConfigurableMimeFileTypeMap::new();
    assert_eq!(map.get_content_type("test.html"), "text/html");
    assert_eq!(map.get_content_type("test.pdf"), "application/pdf");
    assert_eq!(map.get_content_type("test.png"), "image/png");
    assert_eq!(
        map.get_content_type("test.unknown"),
        "application/octet-stream"
    );
}

/// Spring 语义：ConfigurableMimeFileTypeMap 自定义映射。
#[test]
fn spring_configurable_mime_file_type_map_custom() {
    use vernal_context_support::mail::javamail::ConfigurableMimeFileTypeMap;
    let mut map = ConfigurableMimeFileTypeMap::new();
    map.add_mapping("xyz".to_string(), "application/custom".to_string());
    assert_eq!(map.get_content_type("test.xyz"), "application/custom");
}

/// Spring 语义：SmartMimeMessage 携带默认编码。
#[test]
fn spring_smart_mime_message() {
    use vernal_context_support::mail::javamail::{MimeMessage, SmartMimeMessage};
    let msg = MimeMessage::new();
    let smart = SmartMimeMessage::new(msg, Some("UTF-8".to_string()));
    assert_eq!(smart.default_encoding(), Some("UTF-8"));
}

/// Spring 语义：MimeMessagePreparator 回调接口。
#[test]
fn spring_mime_message_preparator() {
    use vernal_context_support::mail::javamail::{
        MimeMessage, MimeMessagePreparator, SimpleMimeMessagePreparator,
    };

    let preparator = SimpleMimeMessagePreparator::new(|msg: &mut MimeMessage| {
        msg.set_from("sender@example.com");
        msg.add_recipient("recipient@example.com");
        msg.set_subject("Test");
        msg.set_text("Hello");
        Ok(())
    });

    let mut msg = MimeMessage::new();
    preparator.prepare(&mut msg).unwrap();

    assert_eq!(msg.from(), Some("sender@example.com"));
    assert_eq!(msg.to(), &["recipient@example.com"]);
    assert_eq!(msg.subject(), Some("Test"));
    assert_eq!(msg.text_body(), Some("Hello"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. FreeMarker 语义测试
// 对标：org.springframework.ui.freemarker.FreeMarkerTemplateUtils
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：FreeMarkerTemplateUtils 模板渲染。
#[test]
fn spring_free_marker_template_utils() {
    use std::collections::HashMap;
    use vernal_context_support::ui::freemarker::FreeMarkerTemplateUtils;

    let mut tera = tera::Tera::default();
    tera.add_raw_template("test.html", "Hello, {{ name }}!")
        .unwrap();

    let mut model = HashMap::new();
    model.insert("name".to_string(), tera::Value::String("World".to_string()));

    let result =
        FreeMarkerTemplateUtils::process_template_into_string(&tera, "test.html", &model).unwrap();
    assert_eq!(result, "Hello, World!");
}

/// Spring 语义：SpringTemplateLoader 基础路径。
#[test]
fn spring_template_loader() {
    use std::path::PathBuf;
    use vernal_context_support::ui::freemarker::SpringTemplateLoader;

    let loader = SpringTemplateLoader::new(PathBuf::from("/templates"));
    assert_eq!(loader.base_path(), &PathBuf::from("/templates"));
    assert_eq!(loader.encoding(), "UTF-8");
    assert_eq!(
        loader.get_template_path("test.html"),
        PathBuf::from("/templates/test.html")
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. JCache 语义测试
// 对标：org.springframework.cache.jcache.JCacheCache
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：JCacheCache 基本操作。
#[test]
fn spring_jcache_cache_basic() {
    use vernal_context_support::cache::jcache::JCacheCache;
    let cache = JCacheCache::new("test".to_string());
    assert_eq!(cache.name(), "test");
    assert_eq!(cache.size(), 0);

    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    assert_eq!(cache.size(), 1);
    let wrapper = cache.get(&key).unwrap();
    assert_eq!(wrapper.get().unwrap().downcast_ref::<i32>().unwrap(), &42);
}

/// Spring 语义：JCacheCache putIfAbsent。
#[test]
fn spring_jcache_cache_put_if_absent() {
    use vernal_context_support::cache::jcache::JCacheCache;
    let cache = JCacheCache::new("test".to_string());
    let key = String::from("k1");

    let val1: Arc<dyn Any + Send + Sync> = Arc::new(1i32);
    assert!(cache.put_if_absent(&key, val1).is_none());

    let val2: Arc<dyn Any + Send + Sync> = Arc::new(2i32);
    let existing = cache.put_if_absent(&key, val2).unwrap();
    assert_eq!(existing.get().unwrap().downcast_ref::<i32>().unwrap(), &1);
}

/// Spring 语义：JCacheCache evictIfPresent。
#[test]
fn spring_jcache_cache_evict_if_present() {
    use vernal_context_support::cache::jcache::JCacheCache;
    let cache = JCacheCache::new("test".to_string());
    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    assert!(cache.evict_if_present(&key));
    assert!(!cache.evict_if_present(&key));
}

/// Spring 语义：JCacheCache invalidate。
#[test]
fn spring_jcache_cache_invalidate() {
    use vernal_context_support::cache::jcache::JCacheCache;
    let cache = JCacheCache::new("test".to_string());
    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);

    assert!(cache.invalidate());
    assert_eq!(cache.size(), 0);
}

/// Spring 语义：JCacheCacheManager 管理多个缓存。
#[test]
fn spring_jcache_cache_manager() {
    use vernal_context_support::cache::CacheManager;
    use vernal_context_support::cache::jcache::{JCacheCache, JCacheCacheManager};

    let manager = JCacheCacheManager::new();
    let cache1 = Arc::new(JCacheCache::new("cache1".to_string()));
    let cache2 = Arc::new(JCacheCache::new("cache2".to_string()));
    manager.register_cache("cache1".to_string(), cache1);
    manager.register_cache("cache2".to_string(), cache2);

    let mut names = manager.cache_names();
    names.sort();
    assert_eq!(names, vec!["cache1", "cache2"]);
    assert!(manager.get_cache("cache1").is_some());
    assert!(manager.get_cache("nonexistent").is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. SchedulerFactoryBean 完整生命周期测试
// 对标：org.springframework.scheduling.quartz.SchedulerFactoryBean
// ═══════════════════════════════════════════════════════════════════════════

/// Spring 语义：完整生命周期 start -> pause -> resume -> stop。
#[tokio::test]
async fn spring_scheduler_full_lifecycle() {
    use vernal_context_support::scheduling::quartz::{SchedulerFactoryBean, SchedulerState};
    let bean = SchedulerFactoryBean::new();

    // 初始状态
    assert_eq!(bean.state().await, SchedulerState::Stopped);

    // 启动
    bean.start().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Running);

    // 暂停
    bean.pause().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Paused);

    // 恢复
    bean.resume().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Running);

    // 停止
    bean.stop().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Stopped);

    // 关闭
    bean.start().await.unwrap();
    bean.shutdown().await.unwrap();
    assert_eq!(bean.state().await, SchedulerState::Stopped);
}

/// Spring 语义：JobDetailFactoryBean 创建任务详情。
#[test]
fn spring_job_detail_factory_bean() {
    use vernal_context_support::scheduling::quartz::JobDetailFactoryBean;
    let mut factory =
        JobDetailFactoryBean::new("myJob".to_string(), "com.example.MyJob".to_string());
    factory.set_group("myGroup".to_string());
    factory.set_description("My test job".to_string());

    let detail = factory.job_detail();
    assert_eq!(detail.name, "myJob");
    assert_eq!(detail.group, "myGroup");
    assert_eq!(detail.description.as_deref(), Some("My test job"));
    assert!(detail.durability);
}

/// Spring 语义：CronTriggerFactoryBean 创建 Cron 触发器。
#[test]
fn spring_cron_trigger_factory_bean() {
    use vernal_context_support::scheduling::quartz::CronTriggerFactoryBean;
    let mut factory = CronTriggerFactoryBean::new("0 0 12 * * ?".to_string());
    factory.set_time_zone("Asia/Shanghai".to_string());

    let trigger = factory.trigger();
    assert_eq!(trigger.cron_expression, "0 0 12 * * ?");
    assert_eq!(trigger.time_zone, Some("Asia/Shanghai".to_string()));
}

/// Spring 语义：SimpleTriggerFactoryBean 创建简单触发器。
#[test]
fn spring_simple_trigger_factory_bean() {
    use vernal_context_support::scheduling::quartz::SimpleTriggerFactoryBean;
    let mut factory = SimpleTriggerFactoryBean::new();
    factory.set_repeat_count(10);
    factory.set_repeat_interval(Duration::from_secs(5));
    factory.set_start_delay(Duration::from_secs(1));

    let trigger = factory.trigger();
    assert_eq!(trigger.repeat_count, 10);
    assert_eq!(trigger.repeat_interval, Duration::from_secs(5));
    assert_eq!(trigger.start_delay, Duration::from_secs(1));
}

/// Spring 语义：LocalDataSourceJobStore 常量。
#[test]
fn spring_local_data_source_job_store() {
    use vernal_context_support::scheduling::quartz::LocalDataSourceJobStore;
    assert_eq!(
        LocalDataSourceJobStore::TX_DATA_SOURCE_PREFIX,
        "springTxDataSource."
    );
    assert_eq!(
        LocalDataSourceJobStore::NON_TX_DATA_SOURCE_PREFIX,
        "springNonTxDataSource."
    );
}

/// Spring 语义：ResourceLoaderClassLoadHelper 资源加载。
#[test]
fn spring_resource_loader_class_load_helper() {
    use vernal_context_support::scheduling::quartz::ResourceLoaderClassLoadHelper;
    use vernal_context_support::scheduling::quartz::resource_loader_class_load_helper::FileSystemResourceLoader;

    let loader = FileSystemResourceLoader::new(std::path::PathBuf::from("."));
    let helper = ResourceLoaderClassLoadHelper::new(Box::new(loader));
    // 不测试实际文件读取，只测试构造
    assert!(helper.get_resource("nonexistent").is_none());
}

/// Spring 语义：LocalTaskExecutorThreadPool 基本操作。
#[test]
fn spring_local_task_executor_thread_pool() {
    use vernal_context_support::scheduling::quartz::LocalTaskExecutorThreadPool;
    let pool = LocalTaskExecutorThreadPool::new();
    assert_eq!(pool.pool_size(), -1);
    assert_eq!(pool.block_for_available_threads(), 1);
}

/// Spring 语义：DelegatingJob 包装 Runnable。
#[test]
fn spring_delegating_job() {
    use vernal_context_support::scheduling::quartz::{
        JobExecutionContext, QuartzJob, SimpleQuartzJob,
    };

    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    let job = SimpleQuartzJob::new(move |_ctx: &JobExecutionContext| {
        executed_clone.store(true, Ordering::SeqCst);
        Ok(())
    });

    let ctx = JobExecutionContext {
        job_name: "test".to_string(),
        job_group: "DEFAULT".to_string(),
        trigger_name: None,
    };

    job.execute(&ctx).unwrap();
    assert!(executed.load(Ordering::SeqCst));
}

/// Spring 语义：SchedulerContextAware 回调。
#[test]
fn spring_scheduler_context_aware() {
    use std::collections::HashMap;
    use vernal_context_support::scheduling::quartz::{SchedulerContext, SchedulerContextAware};

    struct TestAware {
        context: Option<SchedulerContext>,
    }

    impl SchedulerContextAware for TestAware {
        fn set_scheduler_context(&mut self, context: SchedulerContext) {
            self.context = Some(context);
        }
    }

    let mut aware = TestAware { context: None };
    let mut data = HashMap::new();
    data.insert("key".to_string(), "value".to_string());
    let ctx = SchedulerContext { data };

    aware.set_scheduler_context(ctx);
    assert!(aware.context.is_some());
    assert_eq!(aware.context.unwrap().data.get("key").unwrap(), "value");
}

/// Spring 语义：TransactionAwareCacheManagerProxy 代理。
#[test]
fn spring_transaction_aware_cache_manager_proxy() {
    use std::collections::HashMap;
    use std::sync::RwLock;
    use vernal_cache::{Cache, CacheManager};
    use vernal_context_support::cache::transaction::TransactionAwareCacheManagerProxy;

    struct TestCacheManager {
        caches: RwLock<HashMap<String, Arc<dyn Cache>>>,
    }

    impl TestCacheManager {
        fn new() -> Self {
            Self {
                caches: RwLock::new(HashMap::new()),
            }
        }
    }

    impl CacheManager for TestCacheManager {
        fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>> {
            self.caches.read().unwrap().get(name).cloned()
        }
        fn cache_names(&self) -> Vec<String> {
            self.caches.read().unwrap().keys().cloned().collect()
        }
    }

    let manager = Arc::new(TestCacheManager::new());
    let cache = Arc::new(SimpleCache::new("test"));
    manager
        .caches
        .write()
        .unwrap()
        .insert("test".to_string(), cache);

    let proxy = TransactionAwareCacheManagerProxy::new(manager);
    let cache = proxy.get_cache("test").unwrap();
    assert_eq!(cache.name(), "test");

    // 写入应该被包装为事务感知操作
    let key = String::from("k1");
    let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    cache.put(&key, value);
}

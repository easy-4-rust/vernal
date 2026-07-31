<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-aspects 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> 当前权威要求。统一遵循[迁移验收规范](../迁移验收规范.md)，Spring 基线为 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。

## 范围与事实

`spring-aspects` 是对象和包结构主线，`vernal-aop`/`aspect-rs` 只提供织入或拦截底座，不能因此豁免 Spring 对象。自动审计统计为 9 个 Java 业务对象：0 个严格完成、2 个 `MISPLACED`、6 个 `MISSING`、1 个 `UNVERIFIED`。详见[自动对象审计](../migration-audit/vernal-aspects.md)。

## 目标目录

按去除 `org.springframework` 后的包路径保留末两层：

| Spring 来源 | 目标 Rust |
|---|---|
| `beans/factory/aspectj/AnnotationBeanConfigurerAspect.java` | `factory/aspectj/annotation_bean_configurer_aspect.rs` |
| `cache/aspectj/AnnotationCacheAspect.java` | `cache/aspectj/annotation_cache_aspect.rs` |
| `scheduling/aspectj/AnnotationAsyncExecutionAspect.java` | `scheduling/aspectj/annotation_async_execution_aspect.rs` |
| `transaction/aspectj/AnnotationTransactionAspect.java` | `transaction/aspectj/annotation_transaction_aspect.rs` |

`lib.rs`、`mod.rs` 仅声明和重导出；一个 `.rs` 文件只承载一个 Spring 对象。

## 核心语义

- 事务切面必须把匹配结果交给事务属性解析与提交/回滚链，不能只做 pointcut 布尔判断。
- 缓存切面必须保持条件、key、同步加载、提前/延后淘汰和异常传播语义。
- 异步切面必须保持执行器选择、返回值、拒绝与未捕获异常语义。
- Bean 配置切面必须保持注入时点与对象生命周期，不允许以普通构造器注入替代后宣称等价。
- AspectJ 编译/类加载织入若判为 `PLATFORM_NA`，必须逐对象登记 JVM 证据；过程宏只是候选实现。

## 验收门禁

只有 `IMPLEMENTED`、有精确证据的 `DEPENDENCY_REUSED`、逐对象证明的 `PLATFORM_NA` 算已处理。还需通过路径、中文来源注释、语义测试、无 stub 和无生产 wildcard import 检查。

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-aspects 技术交接文档

> **版本**：v1.0（2026-07-28）
> **定位**：Vernal Framework 内建 AOP 切面 crate，对标 Spring 的 `spring-aspects` 模块。
> **现状**：43 文件 / 8585 行。已实现 @Transactional / @Cacheable / @Async / @Configurable 四大内建切面。
> **工具链**：edition 2024 / rustc 1.88。
> **约束**：零 unsafe（`#![forbid(unsafe_code)]`）；AspectJ 字节码织入禁止使用，以过程宏 + `vernal-aop` Interceptor 替代。
> **选型依据**：引用《Spring 组件替换约定》8.5 节。

---

## 一、定位与概述

### 1.1 与 Spring spring-aspects 的关系

Spring 的 `spring-aspects` 模块通过 AspectJ 编译期/加载期织入，为 Spring 应用提供四个内建切面：

| Spring 切面 | 切入点 | 作用 |
|:---|:---|:---|
| `AnnotationTransactionAspect` | `@Transactional` 方法/类型 | 声明式事务管理 |
| `AnnotationCacheAspect` | `@Cacheable` / `@CacheEvict` / `@CachePut` 方法/类型 | 声明式缓存管理 |
| `AnnotationAsyncExecutionAspect` | `@Async` 方法/类型 | 声明式异步执行 |
| `AnnotationBeanConfigurerAspect` | `@Configurable` 类 | 非 Spring 管理对象的 DI 注入 |

vernal-aspects 对这四个切面进行 Rust 等价实现。核心差异：

- **织入机制**：Spring 使用 AspectJ 字节码织入（`aop.xml` + `ajc` 编译器），vernal 使用 `vernal-aop::Interceptor` trait + 过程宏 `#[intercept]` 实现编译期织入。
- **运行时**：Spring 基于 JVM 线程模型，vernal 基于 Tokio 异步运行时。
- **安全约束**：vernal 全局 `#![forbid(unsafe_code)]`，不允许任何 unsafe 代码。

### 1.2 在 Vernal 架构中的位置

```
vernal-framework workspace
├── vernal-core          ── 基础类型、错误类型、工具
├── vernal-aop           ── AOP 内核（Interceptor trait、Invocation、Advisor）
├── vernal-aspects  ◀── 本 crate：内建切面实现
├── vernal-macros        ── 过程宏（#[derive(Component)]、#[intercept]）
├── vernal-context       ── ApplicationContext、BeanFactory
├── vernal-tx            ── 事务管理器（PlatformTransactionManager）
├── vernal-cache         ── 缓存管理器（CacheManager、moka 封装）
└── vernal-context-indexer ── linkme 分布式切面注册
```

vernal-aspects 处于 AOP 层的上层，依赖 `vernal-aop` 提供的 `Interceptor` trait 来定义切面行为，
通过 `vernal-tx` / `vernal-cache` 等底层 crate 提供事务/缓存的实际实现。

### 1.3 当前依赖状态

```toml
# crates/vernal-aspects/Cargo.toml
[package]
name = "vernal-aspects"
description = "Vernal 内建 AOP 切面：@Transactional / @Cacheable / @Async / @Scheduled"

[dependencies]
vernal-aop = { path = "../vernal-aop" }
# vernal-context = { path = "../vernal-context" }  # 暂时禁用：依赖 vernal-aop
# vernal-core = { path = "../vernal-core" }      # 暂时禁用：当前阶段不需要
```

**关键说明**：vernal-aop 依赖已启用，提供 `Interceptor` trait 实现。
vernal-context 和 vernal-core 依赖暂时禁用，待相关 crate 就绪后启用。

---

## 二、模块架构

### 2.1 目录结构（100% 镜像 Spring 包路径）

vernal-aspects 的目录命名完全镜像 Spring 5 个 `aspectj` 子包路径：

```
src/
├── lib.rs                              ── crate 入口，模块声明 + 集成测试
├── transactional_aspect.rs             ── 简化版事务切面（vernal-aop Interceptor 实现）
├── cacheable_aspect.rs                 ── 简化版缓存切面（vernal-aop Interceptor 实现）
├── async_aspect.rs                     ── 简化版异步切面（vernal-aop Interceptor 实现）
├── scheduled_aspect.rs                 ── 简化版定时调度切面（vernal-aop Interceptor 实现）
│
├── transaction/aspectj/                ── 对标 org.springframework.transaction.aspectj
│   ├── mod.rs                          ── 模块入口 + 公共 re-export
│   ├── propagation.rs                  ── Propagation 枚举（7 种传播行为）
│   ├── isolation.rs                    ── Isolation 枚举（5 种隔离级别）
│   ├── transaction_attribute.rs        ── TransactionAttribute 事务属性模型
│   ├── transaction_attribute_source.rs ── TransactionAttributeSource trait + 注解实现
│   ├── transaction_aspect_support.rs   ── TransactionAspectSupport 核心执行引擎
│   ├── abstract_transaction_aspect.rs  ── AbstractTransactionAspect 抽象基类
│   ├── annotation_transaction_aspect.rs── AnnotationTransactionAspect @Transactional 驱动
│   ├── jta_annotation_transaction_aspect.rs ── JtaAnnotationTransactionAspect JTA 驱动
│   ├── aspectj_transaction_management_configuration.rs ── @Configuration 等价
│   ├── aspectj_jta_transaction_management_configuration.rs ── JTA @Configuration 等价
│   └── rethrower.rs                    ── checked 异常透传辅助
│
├── cache/aspectj/                      ── 对标 org.springframework.cache.aspectj
│   ├── mod.rs                          ── 模块入口
│   ├── cache_operation.rs              ── CacheOperation 缓存操作元数据
│   ├── cache_operation_source.rs       ── CacheOperationSource trait + 注解实现
│   ├── cache_aspect_support.rs         ── CacheAspectSupport 核心执行引擎
│   ├── abstract_cache_aspect.rs        ── AbstractCacheAspect 抽象基类
│   ├── annotation_cache_aspect.rs      ── AnnotationCacheAspect @Cacheable 驱动
│   ├── jcache_cache_aspect.rs          ── JCacheCacheAspect JSR-107 驱动
│   ├── aspectj_caching_configuration.rs── @Configuration 等价
│   ├── aspectj_jcache_configuration.rs ── JCache @Configuration 等价
│   └── any_throw.rs                    ── checked 异常透传辅助
│
├── scheduling/aspectj/                 ── 对标 org.springframework.scheduling.aspectj
│   ├── mod.rs                          ── 模块入口
│   ├── async_task_executor.rs          ── AsyncTaskExecutor trait
│   ├── async_uncaught_exception_handler.rs ── 异常处理器 trait
│   ├── abstract_async_execution_aspect.rs ── AbstractAsyncExecutionAspect 抽象基类
│   ├── annotation_async_execution_aspect.rs ── AnnotationAsyncExecutionAspect @Async 驱动
│   └── aspectj_async_configuration.rs  ── @Configuration 等价
│
├── beans/factory/aspectj/              ── 对标 org.springframework.beans.factory.aspectj
│   ├── mod.rs                          ── 模块入口
│   ├── configurable_object.rs          ── ConfigurableObject 标记 trait
│   ├── abstract_dependency_injection_aspect.rs ── DI 切面抽象基类
│   └── annotation_bean_configurer_aspect.rs ── AnnotationBeanConfigurerAspect @Configurable 驱动
│
├── context/annotation/aspectj/         ── 对标 org.springframework.context.annotation.aspectj
│   ├── mod.rs                          ── 模块入口
│   └── spring_configured_configuration.rs ── SpringConfiguredConfiguration
│
├── weaver/                             ── 切面织入机制（对标 META-INF/aop.xml）
│   ├── mod.rs                          ── 模块入口
│   ├── advice_kind.rs                  ── AdviceKind 枚举（4 种 advice 类型）
│   └── pointcut_matcher.rs             ── PointcutMatcher（AspectJ 表达式 → Rust DSL）
│
└── support/                            ── vernal-aop ↔ aspect-rs 桥接
    └── mod.rs                          ── AopBridge 适配器
```

### 2.2 Spring 包 → Rust 模块映射表

| Spring Java 包 | Rust 模块路径 | 文件数 | 行数 |
|:---|:---|:---|:---|
| `org.springframework.transaction.aspectj` | `transaction::aspectj` | 12 | ~3,880 |
| `org.springframework.cache.aspectj` | `cache::aspectj` | 10 | ~1,935 |
| `org.springframework.scheduling.aspectj` | `scheduling::aspectj` | 6 | ~1,010 |
| `org.springframework.beans.factory.aspectj` | `beans::factory::aspectj` | 4 | ~550 |
| `org.springframework.context.annotation.aspectj` | `context::annotation::aspectj` | 2 | ~127 |
| `META-INF/aop.xml` + advice 类型 | `weaver/` | 3 | ~420 |
| 简化版切面（vernal-aop Interceptor） | 顶层 `*_aspect.rs` | 4 | ~370 |
| 桥接层 | `support/` | 1 | ~58 |
| **合计** | | **43** | **~8,585** |

### 2.3 模块分层

vernal-aspects 内部采用三层架构：

```
┌─────────────────────────────────────────────────────┐
│  第一层：简化版切面（vernal-aop Interceptor 实现）     │
│  transactional_aspect.rs / cacheable_aspect.rs /    │
│  async_aspect.rs / scheduled_aspect.rs              │
│  ── 直接实现 Interceptor trait，提供快速集成入口       │
├─────────────────────────────────────────────────────┤
│  第二层：完整 Aspect 模块（镜像 Spring aspectj 包）    │
│  transaction::aspectj / cache::aspectj / ...        │
│  ── 提供与 Spring 完全对标的类型体系和执行引擎         │
├─────────────────────────────────────────────────────┤
│  第三层：织入基础设施                                  │
│  weaver/ (AdviceKind + PointcutMatcher)             │
│  support/ (AopBridge)                               │
│  ── 提供 advice 类型定义和 pointcut 匹配能力          │
└─────────────────────────────────────────────────────┘
```

---

## 三、核心组件详解

### 3.1 transaction::aspectj — 事务管理切面

这是 vernal-aspects 中最复杂的模块（~3,880 行），完整对标 Spring 的事务切面体系。

#### 3.1.1 Propagation — 7 种传播行为

```rust
// 文件：transaction/aspectj/propagation.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Propagation {
    #[default]
    Required,      // 默认：支持当前事务，不存在则新建
    Supports,      // 支持当前事务，不存在则以非事务方式运行
    Mandatory,     // 必须在现有事务中运行，否则抛异常
    RequiresNew,   // 总是新建事务，挂起现有事务
    NotSupported,  // 以非事务方式运行，挂起现有事务
    Never,         // 不允许事务，存在则抛异常
    Nested,        // 嵌套事务（SavePoint）
}
```

完全对标 Spring 的 7 种 `Propagation` 枚举值，语义一致。

#### 3.1.2 Isolation — 5 种隔离级别

```rust
// 文件：transaction/aspectj/isolation.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Isolation {
    #[default]
    Default,           // 数据库默认
    ReadUncommitted,   // READ UNCOMMITTED（脏读）
    ReadCommitted,     // READ COMMITTED（不可重复读）
    RepeatableRead,    // REPEATABLE READ（幻读）
    Serializable,      // SERIALIZABLE（完全串行）
}
```

提供 `as_jdbc_value()` / `from_jdbc_value()` 方法，用于 JDBC 连接池配置时传递隔离级别参数。

#### 3.1.3 TransactionAttribute — 事务属性模型

包含 9 个字段：`propagation`、`isolation`、`read_only`、`timeout`、`rollback_for`、`no_rollback_for`、`name`、`qualifier`、`labels`。完全对标 Spring 的 `TransactionAttribute` 接口。

核心方法 `should_rollback()` 实现 Spring 的回滚规则：
1. 检查 `no_rollback_for` 列表
2. 检查 `rollback_for` 列表（非空时只有列表中的异常才回滚）
3. 默认规则：RuntimeException 和 Error 回滚，checked 异常不回滚

#### 3.1.4 TransactionAttributeSource — 事务属性源

`TransactionAttributeSource` trait 定义 `get_transaction_attribute()` 和 `is_candidate_class()` 两个方法。`AnnotationTransactionAttributeSource` 实现了方法级属性优先、类级属性回退的查找逻辑。

`MethodMetadata` 结构体（`type_name`、`method_name`、`parameter_types`、`return_type`）弥补 Rust 缺少运行时反射的不足。

#### 3.1.5 TransactionAspectSupport — 核心执行引擎

```rust
// 文件：transaction/aspectj/transaction_aspect_support.rs
pub struct TransactionAspectSupport<S: TransactionAttributeSource> {
    attribute_source: Arc<S>,
    transaction_manager: Option<Arc<dyn TransactionManager>>,
    transaction_manager_cache: RwLock<HashMap<String, Arc<dyn TransactionManager>>>,
}
```

核心方法 `invoke_within_transaction()` 实现完整的事务生命周期：

```
1. 获取事务属性（TransactionAttribute）
2. 如果属性为 null，直接执行目标方法（无事务）
3. 根据传播行为决定事务行为：
   ├── Required    → 有事务则加入，无事务则新建
   ├── Supports    → 有事务则加入，无事务则非事务执行
   ├── Mandatory   → 有事务则加入，无事务则抛异常
   ├── RequiresNew → 挂起现有事务，新建事务
   ├── NotSupported→ 挂起现有事务，非事务执行
   ├── Never       → 有事务则抛异常，无事务则非事务执行
   └── Nested      → 有事务则嵌套（SavePoint），无事务则新建
4. 执行目标方法
5. 根据结果决定提交或回滚
```

#### 3.1.6 AnnotationTransactionAspect — @Transactional 驱动

```rust
// 文件：transaction/aspectj/annotation_transaction_aspect.rs
pub struct AnnotationTransactionAspect<S: TransactionAttributeSource> {
    inner: AbstractTransactionAspect<S>,
    attribute_source: Arc<S>,
}
```

提供两种 pointcut 的匹配方法：
- `matches_execution_of_any_public_method_in_at_transactional_type()` — 类型级 `@Transactional`
- `matches_execution_of_transactional_method()` — 方法级 `@Transactional`

组合 pointcut 通过 `transactional_method_execution()` 方法实现 `||` 组合。

#### 3.1.7 辅助类型

| 类型 | 文件 | 说明 |
|:---|:---|:---|
| `TransactionResult` | `transaction_aspect_support.rs` | `Ok(Box<dyn Any>)` / `Err(TransactionError)` |
| `TransactionError` | `transaction_aspect_support.rs` | 包含 message、exception_type、is_runtime、is_error、is_checked |
| `TransactionManager` trait | `transaction_aspect_support.rs` | 对标 `PlatformTransactionManager`，提供 `get_name()` / `get_type()` |
| `NoOpTransactionManager` | `transaction_aspect_support.rs` | 占位实现，用于测试 |
| `SuspendedTransactionInfo` | `transaction_aspect_support.rs` | 挂起事务信息 |
| `Rethrower` | `rethrower.rs` | checked 异常透传辅助 |
| `JtaAnnotationTransactionAspect` | `jta_annotation_transaction_aspect.rs` | JTA 1.2 `@Transactional` 驱动 |
| `AspectJTransactionManagementConfiguration` | `aspectj_transaction_management_configuration.rs` | Spring @Configuration 等价 |
| `AspectJJtaTransactionManagementConfiguration` | `aspectj_jta_transaction_management_configuration.rs` | JTA @Configuration 等价 |

---

### 3.2 cache::aspectj — 缓存管理切面

~1,935 行，完整对标 Spring 的缓存切面体系。

#### 3.2.1 CacheOperation — 缓存操作元数据

三种操作类型：`Cacheable`（@Cacheable 读取缓存）、`CachePut`（@CachePut 更新缓存）、`CacheEvict`（@CacheEvict 驱逐缓存）。

`CacheOperationMetadata` 包含完整配置：`operation`、`cache_names`、`key`（SpEL 表达式）、`condition`（条件表达式）、`unless`（排除表达式）、`sync`、`before_invocation`（CacheEvict 专用）、`all_entries`（CacheEvict 专用）。

#### 3.2.2 CacheAspectSupport — 核心执行引擎

泛型结构体 `CacheAspectSupport<S: CacheOperationSource>`，包含 `cache_operation_source`、`cache_manager`、`default_cache_manager_name`、`error_handler` 四个字段。

定义了三个核心 trait：
- `CacheManager`：`get_name()` / `get_cache()` 方法
- `Cache`：`get()` / `put()` / `evict()` / `clear()` 方法
- `CacheOperationInvoker`：`invoke()` 方法

`execute()` 方法按操作类型分发：`Cacheable` 先查缓存命中返回，未命中执行目标并缓存结果；`CachePut` 执行目标并更新缓存；`CacheEvict` 驱逐缓存（before/after 可配置）。

#### 3.2.3 AnnotationCacheAspect — @Cacheable 驱动

提供 8 个内部 pointcut 的组合匹配：
- 4 个类型级：`@Cacheable` / `@CacheEvict` / `@CachePut` / `@Caching` 类型
- 4 个方法级：同上注解的方法

通过 `cache_method_execution()` 方法实现 `||` 组合 + `this(cachedObject)` 约束。

#### 3.2.4 缓存后端选型

根据《Spring 组件替换约定》8.5 节及模式 B（直接封装）：
- **moka** 替代 Caffeine 作为本地缓存后端
- `vernal-cache` crate 封装 moka 提供 Spring Cache 语义
- `CacheManager` trait 允许多种缓存后端实现

---

### 3.3 scheduling::aspectj — 异步执行切面

~1,010 行，对标 Spring 的异步执行切面体系。

#### 3.3.1 AsyncTaskExecutor — 异步任务执行器

定义 `submit()` / `submit_all()` / `get_executor_name()` 三个方法。`DefaultAsyncTaskExecutor` 为占位实现（直接在当前线程执行），生产环境应使用 `tokio::spawn`。

#### 3.3.2 AbstractAsyncExecutionAspect — 抽象基类

包含 `executor`、`exception_handler`、`default_executor_name` 三个字段。核心方法 `execute_async()` 通过 `determine_async_executor()` 确定执行器，有则异步执行，无则同步回退。

#### 3.3.3 AnnotationAsyncExecutionAspect — @Async 驱动

组合两个 pointcut：`asyncMarkedMethod()`（方法级 `@Async`）和 `asyncTypeMarkedMethod()`（类型级 `@Async`），返回类型限制为 `void || Future+`。`get_executor_qualifier()` 方法级优先，回退到类级。

#### 3.3.4 异步运行时选型

根据《Spring 组件替换约定》8.8 节：
- **tokio** 1.52.4 作为全局异步运行时
- `tokio::spawn` 替代 Spring 的 `ThreadPoolTaskExecutor`
- `tokio-util` 提供 `CancellationToken`、`TaskTracker` 等高级能力

---

### 3.4 beans::factory::aspectj — @Configurable DI 切面

~550 行，对标 Spring 的 `@Configurable` 依赖注入切面。

#### 3.4.1 ConfigurableObject — 标记 trait

`ConfigurableObject: Send + Sync + 'static` 标记需要 DI 注入的对象。同时提供 `ConfigurableObjectWithLifetime` 变体支持带生命周期参数的类型。

#### 3.4.2 AbstractDependencyInjectionAspect — DI 抽象基类

泛型结构体 `AbstractDependencyInjectionAspect<T: ConfigurableObject>`，包含 `pre_construction_enabled` 和 `bean_name` 字段。提供三个 advice 方法：`before_construction()`（pre-construction，仅在启用时执行）、`after_construction()`（post-construction，默认行为）、`after_deserialization()`（反序列化后注入）。

#### 3.4.3 AnnotationBeanConfigurerAspect — @Configurable 驱动

委托 `AbstractDependencyInjectionAspect<T>` 执行实际的 DI 注入逻辑。提供 `configure_bean()` / `set_bean_name()` 等方法。

---

### 3.5 weaver/ — 织入基础设施

~420 行，对标 AspectJ 的 `META-INF/aop.xml` + advice 类型系统。

#### 3.5.1 AdviceKind — 4 种 advice 类型

```rust
// 文件：weaver/advice_kind.rs
pub enum AdviceKind {
    Before,      // 前置 advice
    After,       // 后置 advice
    Around,      // 环绕 advice
    AfterError,  // 异常 advice
}
```

注意：Spring AspectJ 的 `AfterReturning` + `AfterThrowing` 在 vernal 中合并为 `After` + `AfterError`。

#### 3.5.2 PointcutMatcher — Pointcut 模式匹配

```rust
// 文件：weaver/pointcut_matcher.rs
pub struct MethodMetadata {
    pub type_name: &'static str,
    pub method_name: &'static str,
    pub is_public: bool,
    pub type_annotations: Vec<&'static str>,
    pub method_annotations: Vec<&'static str>,
}
```

`PointcutMatcher` 提供 AspectJ 表达式的 Rust 等价匹配：

| 方法 | 对应 AspectJ 表达式 |
|:---|:---|
| `match_execution_public()` | `execution(public * *(..))` |
| `match_execution_with_annotation()` | `execution(@Transactional * *(..))` |
| `match_within_annotation()` | `within(@Transactional *)` |
| `match_this()` | `this(Object)` |
| `match_transactional_type()` | `execution(public * ((@Transactional *)+).*(..)) && within(@Transactional *)` |
| `match_transactional_method()` | `execution(@Transactional * *(..))` |
| `match_cacheable_method()` | `execution(@Cacheable * *(..))` |
| `match_cache_evict_method()` | `execution(@CacheEvict * *(..))` |
| `match_cache_put_method()` | `execution(@CachePut * *(..))` |
| `match_async_method()` | `execution(@Async (void \|\| Future+) *(..))` |
| `match_configurable_method()` | `execution(@Configurable * *(..))` |

#### 3.5.3 support/ — AopBridge 桥接

```rust
// 文件：support/mod.rs
pub struct AopBridge;
```

提供 `vernal-aop` 与 `aspect-rs` 之间的适配层。当前为骨架实现，待 vernal-aop 编译问题修复后完善。

---

## 四、简化版切面（vernal-aop Interceptor 实现）

顶层提供 4 个简化版切面文件，直接实现 `vernal-aop::Interceptor` trait，
作为快速集成入口。这些切面与第二层的完整 Aspect 模块互补：

### 4.1 TransactionalAspect

`TransactionalAspect { config: TransactionConfig }` 实现 `Interceptor` trait。Builder 模式配置：`with_propagation()` / `with_isolation()` / `with_read_only()` / `with_timeout()`。`intercept()` 中 TODO：集成 `vernal-tx` 的 `PlatformTransactionManager`，根据 `config.rollback_for` 和结果决定回滚。

### 4.2 CacheableAspect

`CacheableAspect { config: CacheConfig }` 实现 `Interceptor` trait。`intercept()` 中 TODO：集成 `vernal-cache` 的 `CacheManager`，按 `config.operation` 分发 Cacheable/CachePut/CacheEvict 逻辑。

### 4.3 AsyncAspect

`AsyncAspect { config: AsyncConfig }` 实现 `Interceptor` trait。`intercept()` 中 TODO：集成 `vernal-async` 的 `AsyncTaskExecutor`，将目标方法提交到 `tokio::spawn`。

### 4.4 ScheduledAspect

`ScheduledAspect { config: ScheduleConfig }` 实现 `Interceptor` trait。`ScheduleConfig` 包含 `cron` / `fixed_delay_ms` / `fixed_rate_ms` / `initial_delay_ms` 四个调度参数。`intercept()` 中 TODO：集成 `vernal-context` 的 `ScheduledTask`。

---

## 五、关键设计决策与约束

### 5.1 零 unsafe 约束

```rust
// lib.rs 第一行
#![forbid(unsafe_code)]
```

全 crate 禁止任何 unsafe 代码。所有并发安全通过 `Send + Sync + 'static` trait bound 保证，
使用 `Arc` / `RwLock` 等标准库安全抽象。

### 5.2 AspectJ 字节码织入 → 过程宏 + Interceptor

| 维度 | Spring AspectJ | vernal-aspects |
|:---|:---|:---|
| 织入时机 | 编译期（ajc）或加载期（LTW） | 编译期（过程宏 `#[intercept]`） |
| 织入机制 | 字节码改写 | `vernal-aop::Interceptor` trait 实现 |
| Pointcut 定义 | AspectJ 表达式语言 | Rust 方法调用（`PointcutMatcher`） |
| Advice 绑定 | `aop.xml` + `@Aspect` 注解 | `vernal-macros` 过程宏 + linkme 注册 |
| 运行时代理 | 不需要 | 不需要（编译期确定） |

### 5.3 方法元数据的 Rust 等价

Java 的反射能力（`Method`、`Class`、`Annotation`）在 Rust 中不存在。
vernal-aspects 通过静态 `MethodMetadata` 结构体弥补：

- `type_name: &'static str` — 类型全限定名（编译期确定）
- `method_name: &'static str` — 方法名（编译期确定）
- `parameter_types: Vec<&'static str>` — 参数类型名列表
- `return_type: &'static str` — 返回类型名
- `method_annotations: Vec<&'static str>` — 方法注解列表
- `type_annotations: Vec<&'static str>` — 类型注解列表

这些元数据由 `vernal-macros` 过程宏在编译期生成，通过 `vernal-context-indexer` 的
linkme 分布式 slice 注册到全局注册表。

### 5.4 Send + Sync + 'static 约束

所有核心 trait 均要求 `Send + Sync + 'static`：`TransactionAttributeSource`、`TransactionManager`、`CacheManager`、`Cache`、`AsyncTaskExecutor`、`AsyncUncaughtExceptionHandler`、`ConfigurableObject`。确保所有切面组件可以在 Tokio 多线程运行时中安全使用。

### 5.5 泛型参数化

事务和缓存切面使用泛型参数化属性源（`TransactionAspectSupport<S>`、`AnnotationTransactionAspect<S>`、`CacheAspectSupport<S>`、`AnnotationCacheAspect<S>`），允许不同的属性源实现（注解驱动、XML 配置、编程式注册等）。

---

## 六、集成与使用指南

### 6.1 典型使用流程

```rust
use vernal_aspects::transaction::aspectj::*;
use std::sync::Arc;

// 1. 创建事务属性源
let mut source = AnnotationTransactionAttributeSource::new(false);

// 2. 注册方法级事务属性
source.register_method(
    "com.example.UserService#transfer(String, String, BigDecimal)".to_string(),
    TransactionAttribute {
        propagation: Propagation::Required,
        isolation: Isolation::ReadCommitted,
        rollback_for: vec![Cow::Borrowed("java.lang.RuntimeException")],
        ..Default::default()
    },
);

// 3. 创建事务切面
let source = Arc::new(source);
let aspect = AnnotationTransactionAspect::new(source);

// 4. 在方法执行时调用
let method = MethodMetadata::new(
    "com.example.UserService",
    "transfer",
    vec!["String", "String", "BigDecimal"],
    "void",
);

let result = aspect.invoke_within_transaction(&method, "com.example.UserService", || {
    // 执行业务逻辑
    Ok(Box::new(()) as Box<dyn std::any::Any + Send + Sync>)
});
```

### 6.2 简化版切面使用

```rust
use vernal_aspects::{TransactionalAspect, TransactionConfig, Propagation};

// 创建事务切面（Builder 模式）
let aspect = TransactionalAspect::with_defaults()
    .with_propagation(Propagation::RequiresNew)
    .with_read_only(true)
    .with_timeout(30);

// 注册到 AOP Advisor（通过 vernal-aop）
```

### 6.3 与 vernal-aop 的集成点

```rust
// 通过 Interceptor trait 实现 AOP 拦截
use vernal_aop::{Interceptor, Invocation, InvocationFuture, Next};

impl Interceptor for TransactionalAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            // 前置逻辑（开启事务）
            let result = next.run(invocation).await;
            // 后置逻辑（提交/回滚事务）
            result
        })
    }
}
```

### 6.4 与 veranl-context 的集成

切面通过 `ApplicationModule` 注册到 `ApplicationContext`：

```rust
// SpringConfiguredConfiguration 注册 Bean 配置切面
let config = SpringConfiguredConfiguration::new();
config.register(); // 注册 AnnotationBeanConfigurerAspect 单例 Bean
```

Bean 名称常量：

```rust
pub const BEAN_CONFIGURER_ASPECT_BEAN_NAME: &'static str =
    "org.springframework.context.config.internalBeanConfigurerAspect";
```

### 6.5 待完成集成项

| 集成项 | 依赖 | 状态 | 说明 |
|:---|:---|:---|:---|
| vernal-aop Interceptor 真实集成 | vernal-aop 编译问题修复 | 阻塞中 | 当前为 mock 引用 |
| vernal-tx PlatformTransactionManager | vernal-tx crate 就绪 | 待集成 | 事务切面的核心依赖 |
| vernal-cache CacheManager | vernal-cache crate 就绪 | 待集成 | 缓存切面的核心依赖 |
| tokio::spawn 异步执行 | tokio 运行时就绪 | 待集成 | @Async 切面的核心依赖 |
| vernal-macros #[intercept] 宏 | vernal-macros 就绪 | 待集成 | 编译期织入的核心依赖 |
| linkme 分布式切面注册 | vernal-context-indexer 就绪 | 待集成 | 切面自动发现的核心依赖 |
| SpEL 表达式求值 | vernal-expression 就绪 | 待集成 | 缓存 key / condition / unless 表达式 |
| cron 表达式解析 | cron crate 选型 | 待集成 | @Scheduled cron 调度 |

### 6.6 测试覆盖

当前 crate 包含全面的单元测试，覆盖：
- 所有枚举类型的变体、默认值、序列化
- 所有结构体的创建、Builder 模式、Debug/Clone/Hash
- 所有 trait 的 Send + Sync 约束验证
- 核心执行引擎的 7 种传播行为分支
- 缓存切面的 8 种 pointcut 组合
- 异步切面的方法级/类型级 @Async 匹配
- DI 切面的 pre/post-construction advice

运行测试：

```bash
cargo test -p vernal-aspects
```

---

## 附录：行数统计汇总

| 模块 | 文件数 | 行数 | 核心文件 |
|:---|:---|:---|:---|
| 顶层简化切面 | 5 | ~567 | `transactional_aspect.rs`(195)、`cacheable_aspect.rs`(99)、`async_aspect.rs`(85)、`scheduled_aspect.rs`(85)、`lib.rs`(103) |
| `transaction::aspectj` | 12 | ~3,880 | `transaction_aspect_support.rs`(1158)、`transaction_attribute.rs`(577)、`transaction_attribute_source.rs`(431)、`annotation_transaction_aspect.rs`(370) |
| `cache::aspectj` | 10 | ~1,935 | `cache_aspect_support.rs`(421)、`annotation_cache_aspect.rs`(315)、`cache_operation.rs`(233)、`cache_operation_source.rs`(226) |
| `scheduling::aspectj` | 6 | ~1,010 | `abstract_async_execution_aspect.rs`(346)、`annotation_async_execution_aspect.rs`(235)、`async_task_executor.rs`(201) |
| `beans::factory::aspectj` | 4 | ~550 | `abstract_dependency_injection_aspect.rs`(265)、`configurable_object.rs`(123)、`annotation_bean_configurer_aspect.rs`(122) |
| `context::annotation::aspectj` | 2 | ~127 | `spring_configured_configuration.rs`(109) |
| `weaver/` | 3 | ~420 | `pointcut_matcher.rs`(293)、`advice_kind.rs`(99) |
| `support/` | 1 | ~58 | `mod.rs`(58) |
| 各 `mod.rs` | — | ~38 | 模块入口声明 |
| **合计** | **43** | **~8,585** | |

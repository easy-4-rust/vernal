# Vernal 完全整合设计方案

> 版本：0.1 | 日期：2026-07-26 | 状态：设计稿

## 一、设计目标

将 vernal 打造成 **完全整合 tx_di 设计哲学 + 对标 spring-framework 模块命名 + 吸收 hutool-rust 实现细节** 的 Rust 生态最完整的企业级 IoC/AOP/Context 框架。

## 二、当前仓库全景（基于 codegraph 分析）

### vernal（22 crates，3667 节点）
- 架构骨架优于 tx_di，但功能完整度落后
- 缺 inner_init、async_run、shutdown、SpEL、事务、缓存抽象、内建 AOP 切面
- 已有 hutool-vernal/sa-token-vernal 桥接

### tx_di（508 文件，5292 节点）
- 设计完整但实现简单：Component trait + linkme 自动注册 + AppError + init_sort + TraitImplMap + Interceptor 链
- 11 个插件完整：日志/缓存/文件/任务/认证/注册中心/SIP/GB28181

### hutool-rust（26 crates）
- 947 文件 hutool-core（含 ID/日期/反射/类型转换/Bean）
- 完整日志、缓存、Cron、Web、数据库、加密、JWT、AI 生态
- hutool-extra::spring 已定义 ApplicationContext / ConfigurableBeanFactory trait

### spring-framework（691 文件，3936 节点）
- 23 步 refresh 生命周期
- 15 种核心设计模式
- 完整事务/缓存/异步/调度/消息/Web 体系

## 三、vernal 完全整合后的目标架构（Spring 对标）

```
vernal/
├── crates/
│   │
│   ├── ─── 基础合同层 ───
│   ├── vernal-core              // 对标 spring-core（大幅充实）
│   │   ├── BoxError / SharedError
│   │   ├── error/（VernalError 体系）
│   │   ├── id/（Snowflake/UUID/ObjectId/NanoId — 来自 hutool-core）
│   │   ├── time/（DateUtil/StopWatch — 来自 hutool-core）
│   │   ├── reflect/（ReflectUtil/MethodHandleUtil）
│   │   ├── convert/（Convert/ConverterRegistry — 对标 ConversionService）
│   │   ├── mirror/（AnnotationSynthesizer — 来自 hutool-macro）
│   │   └── ordered（排序契约）
│   │
│   ├── ─── IoC 内核 ───
│   ├── vernal-beans             // 对标 spring-beans
│   │   ├── Component trait + definition()
│   │   ├── inner_init（来自 tx_di）
│   │   ├── init/async_init/async_run/shutdown（来自 tx_di）
│   │   ├── ComponentDefinition（支持 Transient/Instance/Factory/Shutdown）
│   │   ├── BeanDescriptor（属性/方法反射 — 来自 hutool-core::BeanDesc）
│   │   ├── RegistryBuilder / Registry / Container
│   │   ├── Resolver（受限依赖）
│   │   ├── Scope / ScopeContext
│   │   └── GraphPlanner（拓扑排序 + init_order）
│   │
│   ├── ─── AOP 内核 ───
│   ├── vernal-aop               // 对标 spring-aop
│   │   ├── Operation / Pointcut / Advisor / Interceptor
│   │   ├── InvocationPlan / InvocationPlanCatalog
│   │   ├── LocalInterceptor（非 Send）
│   │   ├── SimpleInterceptor（tx_di 风格 before/after/around）
│   │   └── TimeIntervalInterceptor（来自 hutool-aop）
│   │
│   ├── ─── 应用上下文层 ───
│   ├── vernal-context           // 对标 spring-context
│   │   ├── ApplicationContext / Builder
│   │   ├── ApplicationModule（Spring @Configuration 等价）
│   │   ├── ApplicationEnvironment / PropertySource
│   │   ├── ConfigurationProperties + bind_with_prefix
│   │   ├── EventBus / ApplicationEventListener
│   │   ├── Lifecycle / initialize/start/stop
│   │   ├── ApplicationRunner
│   │   ├── ManagedTaskSupervisor / spawn_with_timeout
│   │   ├── ScheduledTask / ManagedScheduledTask
│   │   ├── AsyncTask（新增，P0-2）
│   │   ├── ComponentCondition / ConditionalModule
│   │   └── ComponentScanModule（对标 @ComponentScan）
│   │
│   ├── vernal-discovery         // 对标 spring-context-indexer
│   │   └── LINKED_COMPONENT_REGISTRATIONS
│   │
│   ├── vernal-expression        // 对标 spring-expression（新增）
│   │   ├── ExpressionParser / Expression
│   │   ├── StandardEvaluationContext / SimpleEvaluationContext
│   │   ├── SpEL 兼容语法子集
│   │   └── 条件装配用 ConditionExpression
│   │
│   ├── vernal-aspects           // 对标 spring-aspects（新增）
│   │   ├── TransactionalAspect
│   │   ├── CacheableAspect
│   │   ├── AsyncAspect
│   │   └── ScheduledAspect
│   │
│   ├── ─── 高级抽象层 ───
│   ├── vernal-tx                // 对标 spring-tx（新增）
│   │   ├── PlatformTransactionManager
│   │   ├── @Transactional 宏
│   │   ├── 7 种传播行为
│   │   └── ReactiveTransactionManager
│   │
│   ├── vernal-cache             // 对标 spring-cache（整合 hutool-cache）
│   │   ├── Cache / CacheManager
│   │   ├── @Cacheable/@CachePut/@CacheEvict 宏
│   │   ├── InMemory + Redis 后端
│   │   └── KeyGenerator
│   │
│   ├── vernal-scheduling        // 对标 spring-scheduling（增强）
│   │   ├── TaskScheduler
│   │   ├── @Scheduled 宏
│   │   ├── CronTrigger / PeriodicTrigger
│   │   └── TaskExecutor
│   │
│   ├── vernal-async             // 对标 spring-async（新增）
│   │   ├── AsyncTask trait
│   │   ├── @Async 宏
│   │   ├── TaskExecutor 抽象
│   │   └── ManagedTaskSupervisor 增强
│   │
│   ├── vernal-db                // 对标 spring-jdbc（整合 hutool-db）
│   │   ├── Db / DaoTemplate
│   │   ├── PageResult
│   │   ├── 多方言支持
│   │   └── 集成 vernal-tx 事务
│   │
│   ├── vernal-log               // 对标 spring-jcl（整合 hutool-log）
│   │   ├── init/env_filter
│   │   ├── LogFactory / LogLevel
│   │   └── tracing 适配
│   │
│   ├── vernal-observability     // 对标 spring-actuator（整合 hutool-observability）
│   │   ├── /health 端点
│   │   ├── /metrics 端点
│   │   └── DiagnosticAuthorizer
│   │
│   ├── vernal-messaging         // 对标 spring-messaging（新增）
│   │   ├── MessageChannel
│   │   ├── @EventListener 增强
│   │   └── 消息路由
│   │
│   ├── vernal-websocket         // 对标 spring-websocket（新增）
│   │   └── WebSocket 协议支持
│   │
│   ├── vernal-test              // 对标 spring-test（新增）
│   │   ├── TestContext
│   │   ├── MockMvc 等价物
│   │   └── 测试工具
│   │
│   ├── ─── 过程宏 ───
│   ├── vernal-macros            // 过程宏入口
│   │   ├── #[derive(Component)] + inner_init + async_run
│   │   ├── #[derive(ConfigurationProperties)]
│   │   ├── #[derive(ErrorCode)]
│   │   ├── #[intercept]
│   │   ├── #[transactional] / #[cacheable] / #[async] / #[scheduled]
│   │   └── operation!()
│   │
│   ├── ─── Web & RPC ───
│   ├── vernal-web               // Web 契约（已有）
│   ├── vernal-http              // HTTP 契约（已有）
│   ├── vernal-tower             // Tower 集成（已有）
│   ├── vernal-hyper             // Hyper 桥接（已有）
│   │
│   ├── vernal-axum              // adapter（已有）
│   ├── vernal-actix-web         // adapter（已有）
│   ├── vernal-rocket            // adapter（已有）
│   ├── vernal-warp              // adapter（已有）
│   ├── vernal-salvo             // adapter（已有）
│   ├── vernal-poem              // adapter（已有）
│   ├── vernal-ntex              // adapter（已有）
│   ├── vernal-gotham            // adapter（已有）
│   ├── vernal-tide              // adapter（已有）
│   ├── vernal-tonic             // adapter（已有）
│   │
│   ├── vernal-web-testkit       // Web adapter 一致性测试（已有）
│   │
│   ├── ─── 桥接层 ───
│   ├── hutool-vernal            // Hutool 桥接（独立仓库）
│   ├── sa-token-vernal          // Sa-Token 桥接（独立仓库）
│   │
│   └── ─── 门面 ───
│   └── vernal                   // 统一门面 re-exports
```

## 四、整合的核心模式（来自 tx_di）

### 1. Component trait 完整化

```rust
pub trait Component: Send + Sync + Sized {
    type Deps: DepsTuple;
    fn build(deps: Self::Deps, store: &Store) -> Self;
    fn inner_init(&mut self, store: &Store) -> Result<(), BoxError> { Ok(()) }
    const SCOPE: Scope = Scope::Singleton;
    fn init_sort() -> i32 { INIT_SORT_BUSINESS }
    fn init(app: &Arc<App>) -> Result<(), BoxError> { Ok(()) }
    fn async_init(app: &Arc<App>) -> BoxFuture<Result<(), BoxError>> { Box::pin(async { Ok(()) }) }
    fn async_run(app: &Arc<App>, token: CancellationToken) -> BoxFuture<Result<(), BoxError>> { Box::pin(async { Ok(()) }) }
    fn shutdown(&self) {}
    fn has_async_run() -> bool { false }
    fn trait_impls() -> &'static [fn() -> TypeId] { &[] }
}
```

### 2. lifecycle 5 阶段

```
build → inner_init → init → async_init → async_run → shutdown（逆序）
```

### 3. AppError 结构化错误

```rust
pub enum AppError {
    ErrCode { domain, code, message },
    WithContext { domain, code, message, context },
    Internal(SharedError),
}
```

### 4. init_sort 标准常量

```rust
pub const INIT_SORT_INFRASTRUCTURE: i32 = i32::MIN + 1;  // 日志、配置
pub const INIT_SORT_BUSINESS: i32 = 0;                    // 业务组件
pub const INIT_SORT_APPLICATION: i32 = i32::MAX - 1;       // 应用层
```

### 5. AppAllConfig 启动期强类型绑定

```rust
// BuildContext 自动构造，组件无需自己 build
impl Component for AppAllConfig {
    fn build(_: Self::Deps, _: &Store) -> Self {
        panic!("AppAllConfig 由 BuildContext 特殊构造")
    }
}
```

### 6. ComponentMeta linkme 注册

```rust
#[linkme::distributed_slice]
pub static COMPONENT_REGISTRY: [ComponentMeta] = [..];

pub struct ComponentMeta {
    type_id, name, dep_type_ids, factory, scope,
    impl_traits, trait_impls,
    init_sort, init_fn, async_init_fn, async_run_fn, shutdown_fn,
    has_async_run: bool,
}
```

## 五、整合 hutool-rust 的实现细节

### 整合到 vernal-core（8 个工具）

| hutool 模块 | vernal-core 子模块 | 用途 |
|------------|-------------------|------|
| `hutool-core::id` | `vernal-core::id` | Snowflake/UUID/ObjectId/NanoId |
| `hutool-core::date` | `vernal-core::time` | DateUtil/StopWatch |
| `hutool-core::reflect_util` | `vernal-core::reflect` | 类型反射工具 |
| `hutool-core::convert` | `vernal-core::convert` | 类型转换 ConversionService |
| `hutool-core::bean` | `vernal-core::bean` | BeanDesc 属性描述符 |
| `hutool-macro` | `vernal-core::mirror` | 注解镜像基础 |
| `hutool-core::builder` | `vernal-core::builder` | EqualsBuilder 等 |
| `hutool-extra::spring` | `vernal-context::traits` | ApplicationContext trait |

### 整合到专用 crate

| hutool 模块 | vernal 新增 crate |
|------------|-------------------|
| `hutool-cache` | `vernal-cache` |
| `hutool-cron` | 增强 `vernal-context::scheduled_task` |
| `hutool-db` | `vernal-db` |
| `hutool-log` | `vernal-log` |
| `hutool-observability` | `vernal-observability` |
| `hutool-aop::TimeIntervalAspect` | `vernal-aop::timing` |
| `hutool-extra::validation` | `vernal-validation` |

## 六、对标 Spring 的完整模块清单

| Spring 6.2 | Vernal 现状 | Vernal 目标 | 整合来源 |
|-----------|-----------|----------|---------|
| spring-core | 薄 | **大幅充实**（8 个工具子模块）| hutool-core, hutool-macro |
| spring-beans | vernal-beans | 增强（inner_init, shutdown, async_run）| tx_di |
| spring-aop | vernal-aop | 增强（TimeIntervalAspect）| hutool-aop |
| spring-aspects | ❌ 无 | **新增 vernal-aspects** | 自实现 |
| spring-context | vernal-context | 增强（AsyncTask, ComponentScanModule）| tx_di |
| spring-context-indexer | vernal-discovery | 保持（已完成重构）| linkme |
| spring-context-support | ❌ 无 | hutool-vernal 已覆盖 | hutool-rust |
| spring-core-test | ❌ 无 | 未来 vernal-test | 自实现 |
| spring-expression | ❌ 无 | **新增 vernal-expression** | 自实现 |
| spring-instrument | ❌ 无 | 不需要 | — |
| spring-jcl | ❌ 无 | **新增 vernal-log** | hutool-log |
| spring-jdbc | ❌ 无 | **新增 vernal-db** | hutool-db |
| spring-jms | ❌ 无 | vernal-messaging 统一 | — |
| spring-messaging | ❌ 无 | **新增 vernal-messaging** | 自实现 |
| spring-orm | ❌ 无 | 未来 vernal-orm | rbatis |
| spring-oxm | ❌ 无 | 不需要（serde 替代）| — |
| spring-r2dbc | ❌ 无 | 未来 vernal-r2dbc | sqlx |
| spring-test | ❌ 无 | **新增 vernal-test** | 自实现 |
| spring-tx | ❌ 无 | **新增 vernal-tx** | 自实现 |
| spring-web | vernal-web | 保持 | 已有 |
| spring-webflux | 不需要 | 天然 async | — |
| spring-webmvc | 10 adapters | 保持 | 已有 |
| spring-websocket | ❌ 无 | **新增 vernal-websocket** | 自实现 |

## 七、实施路线图

### 阶段 A：核心补全（1 周）

**目标**：吸纳 tx_di 的 Component trait 完整化

1. `vernal-beans`: `Component::inner_init`, `init_sort`, `has_async_run`, `shutdown`
2. `vernal-beans`: 接收 `&Store` 参数到工厂和 inner_init
3. `vernal-macros`: 增强 `Component` derive 生成 inner_init 调用
4. `vernal-core`: `INIT_SORT_INFRASTRUCTURE/BUSINESS/APPLICATION` 常量

### 阶段 B：异步任务（1 周）

**目标**：吸纳 tx_di 的 `async_run` 设计

1. `vernal-context`: `AsyncTask` trait
2. `vernal-context`: `ManagedTaskSupervisor` 自动派生 AsyncTask 组件
3. `vernal-context`: `ApplicationContext::start()` 后激活 AsyncTask
4. `vernal-macros`: `#[component(async_run = "fn")]` 属性

### 阶段 C：Spring 对标核心（2 周）

**目标**：vernal-core 大幅充实

1. `vernal-core::id`（Snowflake/UUID/ObjectId/NanoId）
2. `vernal-core::time`（DateUtil/StopWatch）
3. `vernal-core::convert`（Convert/ConverterRegistry）
4. `vernal-core::reflect`（ReflectUtil）
5. `vernal-core::bean`（BeanDesc）
6. `vernal-core::mirror`（AnnotationSynthesizer）
7. `vernal-context::traits`（ApplicationContext trait，对标 hutool-extra::spring）

### 阶段 D：横切关注点（3 周）

**目标**：vernal-aspects / vernal-tx / vernal-async / vernal-cache

1. `vernal-aspects` crate（@Transactional/@Cacheable/@Async/@Scheduled 内建切面）
2. `vernal-tx` crate（PlatformTransactionManager + 7 种传播行为）
3. `vernal-async` crate（AsyncTaskExecutor 抽象）
4. `vernal-cache` crate（整合 hutool-cache）

### 阶段 E：高级抽象（2 周）

**目标**：vernal-expression / vernal-log / vernal-db

1. `vernal-expression` crate（简化版 SpEL）
2. `vernal-log` crate（整合 hutool-log）
3. `vernal-db` crate（整合 hutool-db）

### 阶段 F：测试与文档（1 周）

1. `vernal-test` crate（TestContext 框架）
2. 完善所有中文注释
3. 架构文档更新

## 八、合并 hutool-rust 实现到 vernal 的具体策略

### 策略 A：源代码借鉴（推荐）

**不直接复制**，而是从 hutool-rust 的设计中学习：
1. 读取 hutool-core/src/convert/converter_registry.rs
2. 理解 Convert trait + ConverterRegistry 模式
3. 在 vernal-core/src/convert/ 中用纯 Rust 实现

**优点**：保持 vernal 的 idiomatic Rust 风格
**缺点**：需要重新实现，可能引入差异

### 策略 B：直接依赖（备选）

在 vernal-core/Cargo.toml 添加：
```toml
hutool-core = { git = "https://github.com/easy-4-rust/hutool-rust.git", rev = "..." }
```

**优点**：立即可用
**缺点**：引入重量级依赖，违反"对标 Spring 的独立模块化"原则

### 策略 C：可选依赖（最终方案）

```toml
[dependencies]
hutool-core = { version = "0.1", optional = true }

[features]
hutool = ["dep:hutool-core"]
```

**优点**：用户可选，不强制依赖
**缺点**：需要维护两套实现

**最终决定**：策略 A + 部分策略 C
- vernal-core 核心工具用纯 Rust 实现
- hutool-cache/cron/db/log 通过桥接 crate 集成

## 九、跨仓库整合路线

```
vernal 主仓库
├── 吸收 tx_di 的设计模式（inner_init, async_run, shutdown）
├── 吸收 hutool-rust 的实现细节（convert, id, time, reflect）
└── 新增 Spring 对标模块（tx, aspects, expression, cache, async, db, log）

hutool-vernal 桥接仓库
├── 桥接 hutool-cache → vernal-cache
├── 桥接 hutool-cron → vernal-context::ScheduledTask
├── 桥接 hutool-db → vernal-db
└── 桥接 hutool-log → vernal-log

sa-token-vernal 桥接仓库
├── 桥接 sa-token-core → vernal-web 安全拦截器
└── 多租户支持 → vernal-context
```

## 十、风险评估

| 风险 | 严重程度 | 缓解措施 |
|------|---------|---------|
| 重命名 inner_init 破坏现有 API | 🟡 中等 | 默认为空实现，向后兼容 |
| 新增 AsyncTask 增加复杂度 | 🟢 低 | 仿照 ScheduledTask 设计 |
| hutool-rust 依赖与 vernal 设计冲突 | 🟡 中等 | 使用纯 Rust 借鉴，不直接依赖 |
| 完整 Spring 对标需要长期投入 | 🟡 中等 | 分阶段实施，P0/P1 优先 |
| 文档维护负担 | 🟡 中等 | 模块级 rustdoc + 中文注释 |

## 十一、预期成果

完成整合后，vernal 将成为 **Rust 生态最完整的 IoC/AOP/Context 框架**：

| 维度 | 当前 | 整合后 |
|------|------|--------|
| 与 Spring 7.x 功能对标度 | 65% | **90%+** |
| 与 tx_di 设计一致性 | 70% | **95%** |
| 与 hutool-rust 工具复用度 | 0% | **80%** |
| 中文文档完整度 | 70% | **95%** |
| 测试覆盖率 | 高 | 更高 |
| 架构清晰度 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

## 十二、立即可执行的下一步

**P0（必须立刻）**：阶段 A + 阶段 B（吸纳 tx_di 的 inner_init + async_run）
**P1（核心）**：阶段 C（vernal-core 大幅充实）
**P2（增强）**：阶段 D（横切关注点）
**P3（完整）**：阶段 E + 阶段 F

要我现在开始执行吗？建议从 **阶段 A（inner_init）** 开始，这是吸纳 tx_di 设计的关键一步。
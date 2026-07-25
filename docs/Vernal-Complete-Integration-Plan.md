# Vernal 完全整合设计方案

> 版本：0.2 | 日期：2026-07-26 | 状态：待办清单

## 一、设计目标

将 vernal 打造成 **完全对标 spring-framework 模块化设计 + 整合 tx_di 设计哲学 + 吸收 hutool-rust 实现细节** 的 Rust 生态最完整的企业级 IoC/AOP/Context 框架。

**核心定位**：Spring 是 Java 生态最流行的应用框架，vernal 应该成为 Rust 生态同等地位的框架。

## 二、当前仓库全景（基于 codegraph 分析）

### vernal（598 文件，3667 节点，31404 条边，361 个执行流）
- 架构骨架优于 tx_di，但功能完整度落后
- 缺 inner_init、async_run、shutdown、SpEL、事务、缓存抽象、内建 AOP 切面
- 已有 hutool-vernal/sa-token-vernal 桥接

### tx_di（508 文件，5292 节点，43892 条边，782 个执行流）
- 设计完整但实现简单：Component trait + linkme 自动注册 + AppError + init_sort + TraitImplMap + Interceptor 链
- 11 个插件完整：日志/缓存/文件/任务/认证/注册中心/SIP/GB28181

### hutool-rust（26 crates）
- 947 文件 hutool-core（含 ID/日期/反射/类型转换/Bean）
- 完整日志、缓存、Cron、Web、数据库、加密、JWT、AI 生态
- hutool-extra::spring 已定义 ApplicationContext / ConfigurableBeanFactory trait

### spring-framework（691 文件，3936 节点，39371 条边，3771 个执行流，15 个社区）
- 23 步 refresh 生命周期
- 15 种核心设计模式
- 完整事务/缓存/异步/调度/消息/Web 体系

## 三、vernal 完全整合后的目标架构（Spring 对标）

### 3.1 整体分层（从底层到顶层）

```
┌─────────────────────────────────────────────────────────┐
│  应用层（业务代码 + ApplicationModule + Configuration）    │
├─────────────────────────────────────────────────────────┤
│  高级抽象层（tx / cache / async / scheduling / messaging） │
├─────────────────────────────────────────────────────────┤
│  横切层（aspects / expression / aop）                      │
├─────────────────────────────────────────────────────────┤
│  Context 层（application context / lifecycle / events）   │
├─────────────────────────────────────────────────────────┤
│  IoC 层（beans / discovery）                                │
├─────────────────────────────────────────────────────────┤
│  基础设施层（core：error / id / time / convert / reflect）│
└─────────────────────────────────────────────────────────┘
```

### 3.2 完整 crate 清单（31 个）

```
vernal/
├── crates/
│   │
│   ├── ─── 基础设施层（对标 spring-core）───
│   ├── vernal-core              ← 🔄 增强
│   ├── vernal-macros            ← 已有
│   │
│   ├── ─── IoC 内核层（对标 spring-beans）───
│   ├── vernal-beans             ← 🔄 增强（inner_init/async_run/shutdown）
│   ├── vernal-discovery         ← ✅ 已完成重构
│   │
│   ├── ─── AOP 内核层（对标 spring-aop）───
│   ├── vernal-aop               ← 🔄 增强（TimeIntervalAspect）
│   │
│   ├── ─── Context 层（对标 spring-context）───
│   ├── vernal-context           ← 🔄 增强（AsyncTask/ComponentScanModule）
│   │
│   ├── ─── 横切层（对标 spring-aspects + spring-expression）───
│   ├── vernal-aspects           ← ❌ 新增
│   ├── vernal-expression        ← ❌ 新增
│   │
│   ├── ─── 高级抽象层（对标 spring-tx/cache/async/scheduling/messaging）───
│   ├── vernal-tx                ← ❌ 新增
│   ├── vernal-cache             ← ❌ 新增
│   ├── vernal-async             ← ❌ 新增
│   ├── vernal-scheduling        ← 🔄 增强
│   ├── vernal-messaging         ← ❌ 新增
│   │
│   ├── ─── 数据层（对标 spring-jdbc/orm/r2dbc）───
│   ├── vernal-db                ← ❌ 新增
│   ├── vernal-orm               ← ❌ 未来
│   ├── vernal-r2dbc             ← ❌ 未来
│   │
│   ├── ─── 横切基础设施（对标 spring-jcl/observability）───
│   ├── vernal-log               ← ❌ 新增
│   ├── vernal-observability     ← ❌ 新增
│   │
│   ├── ─── 测试与文档（对标 spring-test）───
│   ├── vernal-test              ← ❌ 新增
│   │
│   ├── ─── Web & RPC ───
│   ├── vernal-web               ← 已有
│   ├── vernal-http              ← 已有
│   ├── vernal-tower             ← 已有
│   ├── vernal-hyper             ← 已有
│   ├── vernal-websocket         ← ❌ 新增
│   │
│   ├── ─── 框架适配器（10 个）───
│   ├── vernal-axum              ← 已有
│   ├── vernal-actix-web         ← 已有
│   ├── vernal-rocket            ← 已有
│   ├── vernal-warp              ← 已有
│   ├── vernal-salvo             ← 已有
│   ├── vernal-poem              ← 已有
│   ├── vernal-ntex              ← 已有
│   ├── vernal-gotham            ← 已有
│   ├── vernal-tide              ← 已有
│   ├── vernal-tonic             ← 已有
│   ├── vernal-web-testkit       ← 已有
│   │
│   └── ─── 门面 ───
│   └── vernal                   ← 已有
```

## 四、完整待办清单

### 4.1 阶段 A：基础设施补全（吸纳 tx_di + hutool-rust）🔴 P0

#### A-1：vernal-beans Component trait 完整化

- [ ] **A-1-1** 添加 `inner_init(&mut self, &Store)` 钩子到 Component trait
  - 来源：`tx-di-core/src/component.rs:34-97`
  - 位置：`crates/vernal-beans/src/component_contract.rs`
  - 工作量：0.5 天
  - 测试：inner_init 在 build 后、init 前调用

- [ ] **A-1-2** 添加 `fn init_sort() -> i32 { 10000 }` 默认实现
  - 来源：`tx-di-core/src/component.rs:74`
  - 位置：`crates/vernal-beans/src/component_contract.rs`
  - 工作量：0.5 天

- [ ] **A-1-3** 添加 `fn has_async_run() -> bool { false }` 优化钩子
  - 来源：`tx-di-core/src/registry.rs:22-65`
  - 位置：`crates/vernal-beans/src/component_contract.rs`
  - 工作量：0.5 天

- [ ] **A-1-4** 添加 `fn shutdown(&self) {}` 生命周期钩子
  - 来源：`tx-di-core/src/component.rs:70`
  - 位置：`crates/vernal-beans/src/component_contract.rs`
  - 工作量：0.5 天

- [ ] **A-1-5** 添加 `Deps` 关联类型 + `DepsTuple` trait
  - 来源：`tx-di-core/src/component.rs:102-108`
  - 位置：`crates/vernal-beans/src/deps_tuple.rs`（新增文件）
  - 工作量：2 天
  - 实现：支持 1-16 个元素的 `Arc<T>` 元组
  - 测试：每个元组大小的解析测试

- [ ] **A-1-6** 修改 `ComponentDefinition::create` 接受 `&Store` 而非 `&Resolver`
  - 来源：tx_di 工厂模式
  - 位置：`crates/vernal-beans/src/component_definition.rs`
  - 工作量：1 天

#### A-2：vernal-macros 增强

- [ ] **A-2-1** 宏生成 `inner_init` 调用代码
  - 来源：`tx-di-macros/src/codegen/inner_init.rs`
  - 位置：`crates/vernal-macros/src/component_derive.rs`
  - 工作量：1 天
  - 接受 `#[component(inner_init = "fn_name")]` 属性

- [ ] **A-2-2** 宏生成 `shutdown` 调用代码
  - 位置：`crates/vernal-macros/src/component_derive.rs`
  - 工作量：0.5 天
  - 接受 `#[component(shutdown = "fn_name")]` 属性

- [ ] **A-2-3** 宏生成 `init_sort` 元数据
  - 位置：`crates/vernal-macros/src/component_options.rs`
  - 工作量：0.5 天
  - 接受 `#[component(init_sort = N)]` 属性

- [ ] **A-2-4** 宏生成 `has_async_run` 标记
  - 位置：`crates/vernal-macros/src/component_derive.rs`
  - 工作量：0.5 天
  - 接受 `#[component(async_run = "fn_name")]` 属性

#### A-3：vernal-core 充实

- [ ] **A-3-1** `vernal-core::id` 模块（Snowflake/UUID/ObjectId/NanoId）
  - 来源：`hutool-core/src/id.rs`, `lang/snowflake.rs`, `lang/object_id.rs`
  - 位置：`crates/vernal-core/src/id/`（新建目录）
  - 文件：`mod.rs`, `snowflake.rs`, `object_id.rs`, `uuid_fast.rs`, `nano_id.rs`
  - 工作量：2 天
  - 对标：Spring 的 `IdGenerator` + `JdkIdGenerator`

- [ ] **A-3-2** `vernal-core::time` 模块（DateUtil/StopWatch）
  - 来源：`hutool-core/src/date/`, `date/stop_watch/`
  - 位置：`crates/vernal-core/src/time/`（新建目录）
  - 文件：`mod.rs`, `date_util.rs`, `local_date_time_util.rs`, `stop_watch.rs`
  - 工作量：2 天
  - 对标：Spring 的 `StopWatch`

- [ ] **A-3-3** `vernal-core::convert` 模块（Convert/ConverterRegistry）
  - 来源：`hutool-core/src/convert/`
  - 位置：`crates/vernal-core/src/convert/`（新建目录）
  - 文件：`mod.rs`, `convert.rs`, `converter_registry.rs`, `impl/`（31 个具体转换器）
  - 工作量：3 天
  - 对标：Spring 的 `ConversionService` + `DefaultConversionService`

- [ ] **A-3-4** `vernal-core::reflect` 模块（ReflectUtil）
  - 来源：`hutool-core/src/util/reflect_util.rs`
  - 位置：`crates/vernal-core/src/reflect.rs`
  - 工作量：1 天
  - 对标：Spring 的 `ReflectionUtils`

- [ ] **A-3-5** `vernal-core::ordered` 模块（INIT_SORT_* 常量）
  - 位置：`crates/vernal-core/src/ordered.rs`
  - 工作量：0.5 天
  - 内容：`INIT_SORT_INFRASTRUCTURE = i32::MIN + 1`, `INIT_SORT_BUSINESS = 0`, `INIT_SORT_APPLICATION = i32::MAX - 1`

- [ ] **A-3-6** `vernal-core::id` 集成到 `ManagedTaskSupervisor`（任务 ID 生成）
  - 位置：`crates/vernal-context/src/managed_task_supervisor.rs`
  - 工作量：0.5 天

### 4.2 阶段 B：异步任务体系（吸纳 tx_di async_run）🔴 P0

#### B-1：AsyncTask trait

- [ ] **B-1-1** 定义 `AsyncTask` trait
  - 来源：tx_di 的 `Component::async_run`
  - 位置：`crates/vernal-context/src/async_task.rs`（新增）
  - 工作量：1 天
  - 签名：`async fn async_task(&self, token: CancellationToken) -> Result<(), BoxError>`
  - 与 `Lifecycle` 分离：`Lifecycle::start` 是一次性，`AsyncTask` 是长期后台

- [ ] **B-1-2** `AsyncTask` 注册到 `ManagedTaskSupervisor`
  - 位置：`crates/vernal-context/src/managed_task_supervisor.rs`
  - 工作量：1 天
  - 在 `ApplicationContext::start()` 后激活所有 AsyncTask 组件

- [ ] **B-1-3** 宏支持 `#[component(async_run = "fn")]`
  - 位置：`crates/vernal-macros/src/component_derive.rs`
  - 工作量：1 天
  - 自动生成 `AsyncTask` impl

### 4.3 阶段 C：横切层补全（对标 spring-aspects + spring-expression）🟡 P1

#### C-1：vernal-aspects（对标 spring-aspects）

- [ ] **C-1-1** crate 结构搭建
  - 位置：`crates/vernal-aspects/`（新建）
  - Cargo.toml + lib.rs
  - 工作量：0.5 天

- [ ] **C-1-2** `TransactionalAspect`（@Transactional 实现）
  - 位置：`crates/vernal-aspects/src/transactional.rs`
  - 工作量：3 天
  - 包含事务属性解析、传播行为决策、commit/rollback 流程

- [ ] **C-1-3** `CacheableAspect`（@Cacheable/@CachePut/@CacheEvict）
  - 位置：`crates/vernal-aspects/src/cacheable.rs`
  - 工作量：3 天

- [ ] **C-1-4** `AsyncAspect`（@Async 方法拦截）
  - 位置：`crates/vernal-aspects/src/async_aspect.rs`
  - 工作量：2 天

- [ ] **C-1-5** `ScheduledAspect`（@Scheduled 元数据处理）
  - 位置：`crates/vernal-aspects/src/scheduled.rs`
  - 工作量：2 天

- [ ] **C-1-6** 集成测试（与 vernal-tx, vernal-cache 联合）
  - 工作量：2 天

#### C-2：vernal-expression（对标 spring-expression）

- [ ] **C-2-1** crate 结构搭建
  - 位置：`crates/vernal-expression/`（新建）
  - 工作量：0.5 天

- [ ] **C-2-2** `ExpressionParser` trait
  - 位置：`crates/vernal-expression/src/parser.rs`
  - 工作量：1 天

- [ ] **C-2-3** `Expression` trait + `StandardEvaluationContext`
  - 位置：`crates/vernal-expression/src/expression.rs`
  - 工作量：2 天

- [ ] **C-2-4** 简化版 SpEL 语法解析（属性访问、方法调用、字面量、运算符）
  - 位置：`crates/vernal-expression/src/spel/`
  - 工作量：4 天

- [ ] **C-2-5** `SimpleEvaluationContext`（受限安全上下文）
  - 位置：`crates/vernal-expression/src/context.rs`
  - 工作量：1 天

- [ ] **C-2-6** 集成到 `ComponentCondition`
  - 位置：`crates/vernal-context/src/conditional_component_module.rs`
  - 工作量：1 天

### 4.4 阶段 D：高级抽象层（对标 spring-tx/cache/async）🟡 P1

#### D-1：vernal-tx（对标 spring-tx）

- [ ] **D-1-1** crate 结构搭建
  - 位置：`crates/vernal-tx/`（新建）
  - 工作量：0.5 天

- [ ] **D-1-2** `PlatformTransactionManager` trait
  - 位置：`crates/vernal-tx/src/manager.rs`
  - 工作量：1 天
  - 签名：`getTransaction/commit/rollback`

- [ ] **D-1-3** `TransactionDefinition` + 7 种传播行为
  - 位置：`crates/vernal-tx/src/definition.rs`
  - 工作量：1.5 天
  - REQUIRED/REQUIRES_NEW/MANDATORY/NESTED/SUPPORTS/NOT_SUPPORTED/NEVER

- [ ] **D-1-4** `TransactionInterceptor`（AOP 拦截器）
  - 位置：`crates/vernal-tx/src/interceptor.rs`
  - 工作量：2 天

- [ ] **D-1-5** `ReactiveTransactionManager`（异步事务）
  - 位置：`crates/vernal-tx/src/reactive.rs`
  - 工作量：2 天

- [ ] **D-1-6** `#[transactional]` 过程宏
  - 位置：`crates/vernal-tx/src/macros.rs`（或 vernal-macros）
  - 工作量：1 天

- [ ] **D-1-7** 集成测试
  - 工作量：1.5 天

#### D-2：vernal-cache（对标 spring-cache，整合 hutool-cache）

- [ ] **D-2-1** crate 结构搭建
  - 位置：`crates/vernal-cache/`（新建）
  - 工作量：0.5 天

- [ ] **D-2-2** `Cache<K, V>` trait + `CacheManager` trait
  - 位置：`crates/vernal-cache/src/cache.rs`
  - 工作量：1 天

- [ ] **D-2-3** `InMemoryCache` 实现（moka 后端）
  - 来源：`hutool-cache/src/lib.rs`
  - 位置：`crates/vernal-cache/src/memory.rs`
  - 工作量：2 天

- [ ] **D-2-4** `CacheModule`（ApplicationModule 集成）
  - 来源：增强 `HutoolCacheModule`
  - 位置：`crates/vernal-cache/src/module.rs`
  - 工作量：1 天

- [ ] **D-2-5** `#[cacheable]`/`#[cache_put]`/`#[cache_evict]` 过程宏
  - 位置：`crates/vernal-cache/src/macros.rs`
  - 工作量：2 天

#### D-3：vernal-async（对标 spring-async）

- [ ] **D-3-1** crate 结构搭建
  - 位置：`crates/vernal-async/`（新建）
  - 工作量：0.5 天

- [ ] **D-3-2** `AsyncTaskExecutor` trait
  - 来源：`spring-core/src/task/TaskExecutor`
  - 位置：`crates/vernal-async/src/executor.rs`
  - 工作量：1 天

- [ ] **D-3-3** `TokioTaskExecutor` 实现
  - 位置：`crates/vernal-async/src/tokio_executor.rs`
  - 工作量：1.5 天

- [ ] **D-3-4** `#[async]` 过程宏
  - 位置：`crates/vernal-async/src/macros.rs`
  - 工作量：1.5 天

### 4.5 阶段 E：数据与基础设施层 🟢 P2

#### E-1：vernal-db（对标 spring-jdbc，整合 hutool-db）

- [ ] **E-1-1** crate 结构搭建
  - 位置：`crates/vernal-db/`（新建）
  - 工作量：0.5 天

- [ ] **E-1-2** `Db` trait + `DaoTemplate<T>`
  - 来源：`hutool-db/src/lib.rs::Db/DaoTemplate`
  - 位置：`crates/vernal-db/src/db.rs`
  - 工作量：2 天

- [ ] **E-1-3** 多方言支持（MySQL/PostgreSQL/Oracle/SQL Server/SQLite）
  - 来源：`hutool-db/src/dialect/`
  - 位置：`crates/vernal-db/src/dialect/`
  - 工作量：3 天

- [ ] **E-1-4** `DbConfig: ConfigurationProperties`
  - 位置：`crates/vernal-db/src/config.rs`
  - 工作量：1 天

- [ ] **E-1-5** `DbModule`（ApplicationModule 集成）
  - 位置：`crates/vernal-db/src/module.rs`
  - 工作量：1 天

- [ ] **E-1-6** 集成 vernal-tx（事务感知的数据访问）
  - 位置：`crates/vernal-db/src/transactional_db.rs`
  - 工作量：2 天

#### E-2：vernal-log（对标 spring-jcl，整合 hutool-log）

- [ ] **E-2-1** crate 结构搭建
  - 位置：`crates/vernal-log/`（新建）
  - 工作量：0.5 天

- [ ] **E-2-2** `init()` / `env_filter()` 函数
  - 来源：`hutool-log/src/lib.rs::init/env_filter`
  - 位置：`crates/vernal-log/src/init.rs`
  - 工作量：1 天

- [ ] **E-2-3** `LogFactory` + `LogLevel`（tracing 适配）
  - 来源：`hutool-log/src/compat/`
  - 位置：`crates/vernal-log/src/factory.rs`
  - 工作量：2 天

- [ ] **E-2-4** 多方言兼容（Log4j/SLF4J/JDK/TinyLog）
  - 来源：`hutool-log/src/dialect/`
  - 位置：`crates/vernal-log/src/dialect/`
  - 工作量：2 天

#### E-3：vernal-observability（对标 spring-actuator）

- [ ] **E-3-1** crate 结构搭建
  - 位置：`crates/vernal-observability/`（新建）
  - 工作量：0.5 天

- [ ] **E-3-2** `/health` 端点
  - 来源：`hutool-observability/src/health/`
  - 位置：`crates/vernal-observability/src/health.rs`
  - 工作量：2 天

- [ ] **E-3-3** `/metrics` 端点（Prometheus）
  - 来源：`hutool-observability/src/metrics/`
  - 位置：`crates/vernal-observability/src/metrics.rs`
  - 工作量：2 天

- [ ] **E-3-4** `DiagnosticAuthorizer`（访问授权）
  - 来源：`hutool-observability/src/diagnostics/`
  - 位置：`crates/vernal-observability/src/diagnostics.rs`
  - 工作量：1 天

### 4.6 阶段 F：通信与协议层 🟢 P2

#### F-1：vernal-messaging（对标 spring-messaging）

- [ ] **F-1-1** crate 结构搭建
  - 位置：`crates/vernal-messaging/`（新建）
  - 工作量：0.5 天

- [ ] **F-1-2** `Message<T>` + `MessageChannel` trait
  - 来源：`spring-messaging/MessageChannel`
  - 位置：`crates/vernal-messaging/src/message.rs`
  - 工作量：2 天

- [ ] **F-1-3** `@EventListener` 增强（已存在，扩展异步、事务、条件）
  - 来源：`spring-context/event/EventListener`
  - 位置：`crates/vernal-context/src/event_bus.rs`
  - 工作量：1.5 天

#### F-2：vernal-websocket（对标 spring-websocket）

- [ ] **F-2-1** crate 结构搭建
  - 位置：`crates/vernal-websocket/`（新建）
  - 工作量：0.5 天

- [ ] **F-2-2** WebSocket Handler 适配
  - 来源：`spring-websocket`
  - 位置：`crates/vernal-websocket/src/handler.rs`
  - 工作量：3 天

### 4.7 阶段 G：测试与质量保证 🟢 P2

#### G-1：vernal-test（对标 spring-test）

- [ ] **G-1-1** crate 结构搭建
  - 位置：`crates/vernal-test/`（新建）
  - 工作量：0.5 天

- [ ] **G-1-2** `TestContext` 框架
  - 来源：`spring-test/TestContextManager`
  - 位置：`crates/vernal-test/src/context.rs`
  - 工作量：3 天

- [ ] **G-1-3** 测试注解（`@TestComponent`, `@MockBean`）
  - 位置：`crates/vernal-test/src/annotations.rs`
  - 工作量：2 天

- [ ] **G-1-4** MockMvc 等价物（HTTP 适配测试）
  - 来源：`spring-test/MockMvc`
  - 位置：`crates/vernal-test/src/mock_http.rs`
  - 工作量：3 天

#### G-2：代码质量提升

- [ ] **G-2-1** 所有新增代码补充中文文档注释
  - 工作量：贯穿整个项目

- [ ] **G-2-2** 所有新增模块添加 rustdoc 模块级文档
  - 工作量：贯穿整个项目

- [ ] **G-2-3** 集成测试覆盖率达到 80%+
  - 工作量：贯穿整个项目

### 4.8 阶段 H：横切基础设施 🟢 P2

#### H-1：vernal-beans BeanDescriptor 增强（吸纳 hutool-core::bean）

- [ ] **H-1-1** `BeanDescriptor<T>`（属性/方法描述）
  - 来源：`hutool-core/src/bean/bean_desc.rs`
  - 位置：`crates/vernal-beans/src/bean_descriptor.rs`
  - 工作量：2 天
  - 用途：`ConfigurationProperties::bind_with_prefix` 的属性绑定

- [ ] **H-1-2** `BeanDescCache`（缓存反射元数据）
  - 来源：`hutool-core/src/bean/bean_desc_cache.rs`
  - 位置：`crates/vernal-beans/src/bean_descriptor_cache.rs`
  - 工作量：1.5 天

- [ ] **H-1-3** `BeanUtil::copy_properties`（属性复制）
  - 来源：`hutool-core/src/bean/bean_util.rs`
  - 位置：`crates/vernal-beans/src/bean_util.rs`
  - 工作量：1.5 天

#### H-2：vernal-mirror（对标 spring 注解处理）

- [ ] **H-2-1** crate 结构搭建
  - 位置：`crates/vernal-mirror/`（新建）
  - 工作量：0.5 天

- [ ] **H-2-2** `AnnotationSynthesizer`（注解合成）
  - 来源：`hutool-macro/src/`
  - 位置：`crates/vernal-mirror/src/synthesizer.rs`
  - 工作量：2 天

- [ ] **H-2-3** 集成到 vernal-macros 宏
  - 位置：`crates/vernal-macros/src/component_options.rs`
  - 工作量：1.5 天

## 五、依赖方向（不可违反）

```
vernal-core ← vernal-beans ← vernal-context ← vernal
                ↑                ↑
           vernal-aop            │
                ↑                │
                └────────────────┘

vernal-core ← vernal-expression
vernal-core ← vernal-aspects
vernal-core ← vernal-tx
vernal-core ← vernal-cache
vernal-core ← vernal-db
vernal-core ← vernal-async
vernal-core ← vernal-log
vernal-core ← vernal-observability
vernal-core ← vernal-messaging
vernal-core ← vernal-websocket
vernal-core ← vernal-test
vernal-core ← vernal-mirror
```

### 禁止的依赖方向
- `vernal-core` 不得依赖任何内核模块
- `vernal-beans` 不得依赖 `vernal-aop`、`vernal-context`
- `vernal-aop` 不得依赖 `vernal-beans`、`vernal-context`
- `vernal-context` 不得依赖任何具体 Web 框架
- 任何 adapter 不得依赖另一个 adapter

## 六、实施路线图

| 阶段 | 工作量 | 周数 | 优先级 | 状态 |
|------|--------|------|--------|------|
| A：基础设施补全 | 18.5 天 | 3 | 🔴 P0 | 待开始 |
| B：异步任务体系 | 3 天 | 0.5 | 🔴 P0 | 待开始 |
| C：横切层补全 | 17 天 | 3 | 🟡 P1 | 待开始 |
| D：高级抽象层 | 19.5 天 | 3 | 🟡 P1 | 待开始 |
| E：数据与基础设施 | 16 天 | 2.5 | 🟢 P2 | 待开始 |
| F：通信与协议 | 7.5 天 | 1 | 🟢 P2 | 待开始 |
| G：测试与质量 | 10 天 | 1.5 | 🟢 P2 | 待开始 |
| H：横切基础设施 | 9 天 | 1.5 | 🟢 P2 | 待开始 |
| **总计** | **100.5 天** | **~16 周** | — | — |

## 七、核心整合来源详细映射

### 7.1 来自 tx_di 的设计模式

| tx_di 设计 | vernal 实现位置 | 工作量 |
|-----------|----------------|--------|
| `Component` trait + `Deps` 关联类型 | A-1-1, A-1-5 | 2.5 天 |
| `inner_init` 工厂内初始化 | A-1-1, A-2-1 | 1.5 天 |
| `init_sort` 优先级排序 | A-1-2, A-3-5, A-2-3 | 1.5 天 |
| `async_run` 后台任务 | B-1 全套 | 3 天 |
| `shutdown` 关闭钩子 | A-1-4, A-2-2 | 1 天 |
| `has_async_run` 优化 | A-1-3, A-2-4 | 1 天 |
| `linkme` 自动注册 | ✅ 已完成 | — |
| `AppError` 结构化错误 | ✅ 已完成 | — |
| `ArgValue` 参数序列化 | 未来 AOP 增强 | — |
| `TraitImplMap` | ✅ 已完成 | — |
| `has_async_run` 优化 | A-1-3 | 0.5 天 |

### 7.2 来自 spring-framework 的设计模式

| Spring 设计 | vernal 实现位置 | 工作量 |
|-----------|----------------|--------|
| 23 步 refresh 生命周期 | ✅ 已实现 | — |
| `BeanPostProcessor` 链 | `ApplicationModule` 替代 | — |
| `BeanFactoryPostProcessor` | `ApplicationEnvironment` | — |
| `@ConfigurationProperties` | ✅ 已实现 | — |
| `@Transactional` | C-1-2, D-1 全套 | 8 天 |
| `@Cacheable` | C-1-3, D-2 全套 | 6 天 |
| `@Async` | C-1-4, D-3 全套 | 4 天 |
| `@Scheduled` | C-1-5 | 2 天 |
| SpEL 表达式 | C-2 全套 | 10 天 |
| `ConversionService` | A-3-3 | 3 天 |
| `ApplicationEventPublisher` | ✅ `EventBus` | — |
| `MessageChannel` | F-1-2 | 2 天 |
| `TaskScheduler` | 增强 ScheduledTask | — |
| `StopWatch` | A-3-2 | 2 天 |
| `BeanInfo` | H-1-1 | 2 天 |

### 7.3 来自 hutool-rust 的实现细节

| hutool-rust 模块 | vernal 实现位置 | 工作量 |
|-----------------|----------------|--------|
| `hutool-core::id` (Snowflake/UUID/ObjectId) | A-3-1 | 2 天 |
| `hutool-core::date` (DateUtil/StopWatch) | A-3-2 | 2 天 |
| `hutool-core::convert` (Convert/ConverterRegistry) | A-3-3 | 3 天 |
| `hutool-core::reflect_util` | A-3-4 | 1 天 |
| `hutool-core::bean` (BeanDesc) | H-1-1 | 2 天 |
| `hutool-macro` (AnnotationSynthesizer) | H-2-2 | 2 天 |
| `hutool-cache` (moka 集成) | D-2-3 | 2 天 |
| `hutool-cron` (cron 解析) | 增强 ScheduledTask | — |
| `hutool-db` (Db/DaoTemplate) | E-1-2 | 2 天 |
| `hutool-log` (tracing 适配) | E-2-3 | 2 天 |
| `hutool-observability` (health/metrics) | E-3-2, E-3-3 | 4 天 |
| `hutool-aop::TimeIntervalAspect` | vernal-aop 增强 | — |
| `hutool-extra::spring` (ApplicationContext trait) | vernal-context traits | — |

## 八、风险评估

| 风险 | 严重程度 | 缓解措施 |
|------|---------|---------|
| Component trait 变更破坏现有 API | 🟡 中等 | 默认空实现，向后兼容 |
| 新增 AsyncTask 引入复杂度 | 🟢 低 | 与 ScheduledTask 保持一致设计 |
| hutool-rust 依赖与 vernal 设计冲突 | 🟡 中等 | 纯 Rust 借鉴，不直接依赖 |
| 完整 Spring 对标需要长期投入 | 🟡 中等 | 分阶段实施，P0/P1 优先 |
| 文档维护负担 | 🟡 中等 | 模块级 rustdoc + 中文注释 |
| 测试覆盖率难以保证 | 🟡 中等 | 每阶段必须有测试覆盖 |
| 模块边界划分冲突 | 🟡 中等 | 严格遵循依赖方向 |
| AOP 编织复杂度增加 | 🟡 中等 | 优先支持编译期织入 |
| 跨异步运行时错误 | 🟡 中等 | 使用 Tokio 抽象层 |
| WebSocket/STOMP 集成复杂度 | 🟢 低 | 复用 vernal-aop 拦截器 |

## 九、预期成果

完成整合后，vernal 将成为 **Rust 生态最完整的 IoC/AOP/Context 框架**：

| 维度 | 当前 | 整合后 |
|------|------|--------|
| 与 Spring 7.x 功能对标度 | 65% | **90%+** |
| 与 tx_di 设计一致性 | 70% | **95%** |
| 与 hutool-rust 工具复用度 | 0% | **80%** |
| 中文文档完整度 | 70% | **95%** |
| 测试覆盖率 | 高 | 更高 |
| 架构清晰度 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 模块数量 | 22 | **31** |

## 十、立即可执行的下一步

**P0（必须立刻）**：阶段 A + 阶段 B（吸纳 tx_di 的 inner_init + async_run）
**P1（核心）**：阶段 C（vernal-core 大幅充实）
**P2（增强）**：阶段 D（横切关注点）
**P3（完整）**：阶段 E + 阶段 F + 阶段 G + 阶段 H

## 十一、详细文件路径索引

### 11.1 已有 vernal crates 路径（`/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal/`）

```
crates/vernal-core/src/{lib.rs, error/, failure.rs}
crates/vernal-beans/src/{component_contract.rs, component_definition.rs, ...}
crates/vernal-aop/src/{interceptor.rs, simple_*.rs, ...}
crates/vernal-context/src/{application_context.rs, application_context_builder.rs, ...}
crates/vernal-discovery/src/{linked_component_*.rs, ...}
crates/vernal-macros/src/{component_derive.rs, component_options.rs, ...}
crates/vernal-web/src/{lib.rs, ...}
crates/vernal-http/src/{lib.rs, ...}
crates/vernal-tower/src/{...}
crates/vernal-hyper/src/{...}
crates/vernal-{axum,actix-web,rocket,warp,salvo,poem,ntex,gotham,tide,tonic}/src/{...}
crates/vernal-web-testkit/src/{...}
crates/vernal/src/{lib.rs, ...}
```

### 11.2 新增 crates 路径规划

```
crates/vernal-expression/src/{parser.rs, expression.rs, context.rs, spel/}
crates/vernal-aspects/src/{transactional.rs, cacheable.rs, async_aspect.rs, scheduled.rs}
crates/vernal-tx/src/{manager.rs, definition.rs, interceptor.rs, reactive.rs, macros.rs}
crates/vernal-cache/src/{cache.rs, memory.rs, module.rs, macros.rs}
crates/vernal-async/src/{executor.rs, tokio_executor.rs, macros.rs}
crates/vernal-db/src/{db.rs, dialect/, config.rs, module.rs, transactional_db.rs}
crates/vernal-log/src/{init.rs, factory.rs, dialect/}
crates/vernal-observability/src/{health.rs, metrics.rs, diagnostics.rs}
crates/vernal-messaging/src/{message.rs}
crates/vernal-websocket/src/{handler.rs}
crates/vernal-test/src/{context.rs, annotations.rs, mock_http.rs}
crates/vernal-mirror/src/{synthesizer.rs}
```

## 十二、立即行动

要我现在开始执行阶段 A（核心补全）吗？建议从 **A-1-1 到 A-1-4（Component trait 完整化）** 开始，这是吸纳 tx_di 设计的关键一步，预计 2 天工作量。
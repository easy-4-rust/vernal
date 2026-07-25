# Vernal 完全整合设计方案（修订版）

> 版本：0.3 | 日期：2026-07-26 | 状态：待办清单

## 一、核心设计原则

### 1.1 框架边界原则

| 类型 | 归属 | 说明 |
|------|------|------|
| **框架能力** | vernal | IoC、AOP、Context、生命周期、模块、事件 |
| **通用工具** | hutool-rust | 日期、字符串、ID、HTTP 客户端、JSON、加密 |
| **桥接适配** | hutool-vernal | 将 hutool 工具适配为 vernal 组件 |
| **vernal 必须的工具** | vernal-core | 仅保留框架运转**不可或缺**的工具 |

### 1.2 与 Spring/Hutool 的关系对标

**Spring Framework 的做法**：
- Spring 自带：`StopWatch`（AOP 计时）、`IdGenerator`、`TypeFilter`、`ResolvableType` 等**框架必须**的工具
- Spring 不提供：`DateUtil`、`StrUtil`、`JSONUtil`、HTTP 客户端、加密 — 由第三方工具库提供
- Spring 通过 Spring Boot Starter 桥接第三方工具

**vernal 的对标做法**：
- vernal-core 自带：仅框架**必须**的工具（参见第三章）
- vernal 不实现：`DateUtil`、`StrUtil`、`JSONUtil`、HTTP 客户端、加密
- vernal 通过 `hutool-vernal` 桥接 hutool 工具
- 用户可以选择不引入 hutool-vernal，使用其他工具库（serde_json、reqwest、uuid）

### 1.3 为什么不能让框架重叠

如果 vernal 内置了大量工具：
1. **职责混乱**：vernal 既是框架又是工具库
2. **重复实现**：hutool 已经实现的工具 vernal 又写一遍
3. **生态割裂**：用户不知道该用哪个
4. **违反单一职责**：Spring 不包含 Hutool，Hutool 也不包含 Spring

## 二、当前仓库全景（基于 codegraph 分析）

### 2.1 vernal（598 文件，3667 节点）
- 架构骨架优于 tx_di，但功能完整度落后
- 缺 inner_init、async_run、shutdown、SpEL、事务、缓存抽象、内建 AOP 切面
- 已有 hutool-vernal/sa-token-vernal 桥接

### 2.2 tx_di（508 文件，5292 节点）
- 设计完整但实现简单：Component trait + linkme 自动注册 + AppError + init_sort + TraitImplMap + Interceptor 链
- 11 个插件完整：日志/缓存/文件/任务/认证/注册中心/SIP/GB28181

### 2.3 hutool-rust（26 crates，947+ 文件）
- **工具库定位**：通用工具的 Rust 实现
- 不应该被 vernal 内置

### 2.4 spring-framework（691 文件，3936 节点）
- 23 步 refresh 生命周期
- 15 种核心设计模式
- 完整事务/缓存/异步/调度/消息/Web 体系
- **不内嵌任何通用工具库**

## 三、vernal 与 hutool-rust 的职责划分

### 3.1 vernal-core **必须**包含的工具（仅限框架运转不可或缺）

| 工具 | 理由 | 对标 Spring |
|------|------|-----------|
| `error`（VernalError、ErrorCode） | 框架自身错误体系 | `NestedRuntimeException` |
| `ordered`（INIT_SORT_* 常量） | 拓扑排序需要 | `Ordered` 接口 |
| `lifecycle` 阶段枚举 | 框架生命周期管理 | `Lifecycle.Phase` |
| `convert` **精简版**（仅 5-6 个核心转换器） | 配置属性绑定需要 | `ConversionService` |
| `time` **StopWatch 唯一**（不带 DateUtil） | AOP 计时需要 | `StopWatch` |
| `id` **ObjectId 唯一**（不带 Snowflake/UUID） | 任务 ID 生成需要 | 无（hutool 自带） |

**原则**：vernal-core 工具 ≤ 6 个，每个工具都有**强框架需求**。

### 3.2 vernal-core **绝不**包含的工具

| 工具 | 归属 | 理由 |
|------|------|------|
| `DateUtil`（完整日期工具） | hutool-core | 通用日期处理 |
| `StrUtil`（字符串工具） | hutool-core | 通用字符串处理 |
| `JSONUtil`（JSON 处理） | hutool-json | 通用 JSON 处理 |
| `HTTP Client` | hutool-http | 通用 HTTP 客户端 |
| `SecureUtil`（加密） | hutool-crypto | 通用加密处理 |
| `IdUtil`（UUID/Snowflake） | hutool-core | 通用 ID 生成 |
| `MapUtil`（集合工具） | hutool-core | 通用集合处理 |
| `FileUtil`（文件工具） | hutool-core | 通用文件处理 |

**原则**：任何可以独立于 vernal 框架使用的工具，都应该留在 hutool-rust 中。

### 3.3 hutool-vernal 桥接的职责

**只做"适配"，不做"实现"**：
- ✅ 将 `hutool-http` 的 `HttpClient` 适配为 vernal 组件
- ✅ 将 `hutool-setting` 的 `.setting` 文件适配为 vernal PropertySource
- ✅ 将 `hutool-cache` 的 Cache 适配为 vernal CacheManager
- ❌ **不**在 hutool-vernal 中重新实现任何 hutool 功能
- ❌ **不**让 vernal 核心代码直接调用 hutool

## 四、vernal 完全整合后的目标架构（Spring 对标 + 边界严格）

### 4.1 整体分层

```
┌─────────────────────────────────────────────────────────┐
│  应用层（业务代码 + ApplicationModule + Configuration）    │
├─────────────────────────────────────────────────────────┤
│  高级抽象层（tx / cache / async / scheduling / messaging） │
│  ← 桥接 hutool-cache/hutool-cron/hutool-db                │
├─────────────────────────────────────────────────────────┤
│  横切层（aspects / expression / aop）                      │
├─────────────────────────────────────────────────────────┤
│  Context 层（application context / lifecycle / events）   │
├─────────────────────────────────────────────────────────┤
│  IoC 层（beans / discovery）                                │
├─────────────────────────────────────────────────────────┤
│  基础设施层（core：仅 6 个必须工具）                        │
│  ← 吸收 hutool-core 的设计思想（不直接依赖）              │
└─────────────────────────────────────────────────────────┘
```

### 4.2 完整 crate 清单（31 个，但严格区分"框架"和"桥接"）

```
vernal/                                    # 框架核心（不含任何工具实现）
├── crates/
│   │
│   ├── ─── 基础设施层（对标 spring-core）───
│   ├── vernal-core              ← 🔄 精简（仅 6 个必须工具）
│   ├── vernal-macros            ← 已有
│   │
│   ├── ─── IoC 内核层（对标 spring-beans）───
│   ├── vernal-beans             ← 🔄 增强（inner_init/async_run/shutdown）
│   ├── vernal-discovery         ← ✅ 已完成重构
│   │
│   ├── ─── AOP 内核层（对标 spring-aop）───
│   ├── vernal-aop               ← 🔄 增强（StopWatchAspect）
│   │
│   ├── ─── Context 层（对标 spring-context）───
│   ├── vernal-context           ← 🔄 增强（AsyncTask/ComponentScanModule）
│   │
│   ├── ─── 横切层（对标 spring-aspects + spring-expression）───
│   ├── vernal-aspects           ← ❌ 新增（仅框架能力）
│   ├── vernal-expression        ← ❌ 新增（仅条件装配子集）
│   │
│   ├── ─── 高级抽象层（对标 spring-tx/cache/async/scheduling/messaging）───
│   ├── vernal-tx                ← ❌ 新增（框架能力，不含 JDBC 实现）
│   ├── vernal-cache             ← 🔄 改为桥接，依赖 hutool-cache
│   ├── vernal-async             ← ❌ 新增（框架抽象）
│   ├── vernal-scheduling        ← 🔄 增强（含 cron 桥接）
│   ├── vernal-messaging         ← ❌ 新增
│   │
│   ├── ─── 数据层（对标 spring-jdbc/orm/r2dbc）───
│   ├── vernal-db                ← ❌ 新增（框架抽象，不含 SQL 方言）
│   │
│   ├── ─── 横切基础设施（对标 spring-jcl/observability）───
│   ├── vernal-log               ← ❌ 新增（框架门面，底层 tracing）
│   ├── vernal-actuator         ← ❌ 新增（对标 spring-actuator）
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
│   ├── vernal-{axum,actix-web,rocket,warp,salvo,poem,ntex,gotham,tide,tonic}
│   │
│   └── ─── 门面 ───
│   └── vernal                   ← 已有

hutool-vernal/                            # 桥接层（独立仓库）
├── 将 hutool-rust 工具适配为 vernal 组件
├── 不修改 vernal 核心代码
└── 用户可选：可使用其他工具库替代

hutool-rust/                              # 工具库（独立仓库，不变）
├── 通用工具集合
├── 不依赖 vernal
└── 可以独立使用
```

## 五、vernal-core 精简工具清单（仅 6 个，必须有）

| 工具 | 行数估计 | 来源 | 框架需求 |
|------|---------|------|---------|
| `error`（VernalError + ErrorCode） | 350 行 | 自实现 | ✅ 框架必须 |
| `ordered`（INIT_SORT_* 常量） | 30 行 | 自实现 | ✅ 拓扑排序必须 |
| `lifecycle_phase`（阶段枚举） | 50 行 | 自实现 | ✅ Lifecycle 必须 |
| `convert` 精简版（5 个核心转换器） | 150 行 | 借鉴 hutool 设计 | ✅ 配置属性必须 |
| `time::StopWatch`（仅计时） | 100 行 | 借鉴 hutool 设计 | ✅ AOP 计时必须 |
| `id::ObjectId`（仅生成 ID） | 100 行 | 借鉴 hutool 设计 | ✅ 任务 ID 必须 |
| **合计** | **~780 行** | — | — |

**对比**：hutool-core 是 947 文件、几万行；vernal-core 应该是 **1 个文件、几百行**。

## 六、hutool-vernal 桥接清单（用户可选）

| 桥接模块 | 桥接的 hutool 功能 | vernal 侧能力 |
|---------|-------------------|--------------|
| `HutoolApplicationModule` | 整体集成入口 | 组合所有桥接 |
| `HutoolHttpComponents` | hutool-http::HttpClient | Web 客户端组件 |
| `HutoolSettingPropertySource` | hutool-setting::Setting | 配置属性源 |
| `HutoolCacheModule` | hutool-cache::Cache | 缓存组件 |
| `HutoolFileStorageModule` | hutool-extra FileStorage | 文件存储组件 |
| `HutoolCronModule` | hutool-cron::Scheduler | 定时任务组件 |
| `HutoolDbModule` | hutool-db::Db | 数据库组件 |
| `HutoolLogModule` | hutool-log::LogFactory | 日志门面 |
| `HutoolObservabilityModule` | hutool-observability | 健康/指标组件 |

**原则**：hutool-vernal 只是"适配层"，**不实现任何 hutool 功能**。

## 七、完整待办清单

### 7.1 阶段 A：基础设施精简补全 🔴 P0

#### A-1：vernal-core 精简（从 7 文件压到 6 个必须工具）

- [ ] **A-1-1** `error/` 保留（已存在，仅优化）
  - 工作量：0.5 天
  - 来源：vernal-core 已有的 VernalError

- [ ] **A-1-2** 添加 `ordered.rs`（仅 INIT_SORT_* 常量）
  - 来源：tx_di 的 `init_sort` 设计
  - 位置：`crates/vernal-core/src/ordered.rs`
  - 工作量：0.5 天
  - 内容：`INIT_SORT_INFRASTRUCTURE = i32::MIN + 1`, `INIT_SORT_BUSINESS = 0`, `INIT_SORT_APPLICATION = i32::MAX - 1`

- [ ] **A-1-3** 添加 `lifecycle_phase.rs`（LifecyclePhase 枚举）
  - 来源：tx_di 的 `init/async_init/async_run` 阶段
  - 位置：`crates/vernal-core/src/lifecycle_phase.rs`
  - 工作量：0.5 天

- [ ] **A-1-4** 添加 `convert/` 精简版（**仅 5 个转换器**）
  - 来源：借鉴 hutool-core::convert 的设计，但**不依赖**
  - 位置：`crates/vernal-core/src/convert/`
  - 工作量：2 天
  - 内容（**仅 5 个**，不为通用性而增加）：
    - `StringConverter`（字符串 ↔ 其他类型）
    - `NumberConverter`（字符串 ↔ 数字）
    - `BooleanConverter`（字符串 ↔ bool）
    - `EnumConverter`（字符串 ↔ 枚举）
    - `OptionConverter`（可选值处理）

- [ ] **A-1-5** 添加 `time/stop_watch.rs`（**仅 StopWatch**，无 DateUtil）
  - 来源：借鉴 hutool-core::date::StopWatch 设计
  - 位置：`crates/vernal-core/src/time/stop_watch.rs`
  - 工作量：0.5 天

- [ ] **A-1-6** 添加 `id/object_id.rs`（**仅 ObjectId**，无 UUID/Snowflake）
  - 来源：借鉴 hutool-core::lang::ObjectId 设计
  - 位置：`crates/vernal-core/src/id/object_id.rs`
  - 工作量：0.5 天

- [ ] **A-1-7** **删除** 不应该出现的工具（如果之前误添加）
  - 审查：vernal-core 不应包含 DateUtil、StrUtil、JSONUtil、HTTP 客户端等

#### A-2：vernal-beans Component trait 完整化（吸纳 tx_di）

- [ ] **A-2-1** 添加 `inner_init(&mut self, &Store)` 钩子
  - 来源：`tx-di-core/src/component.rs:34-97`
  - 位置：`crates/vernal-beans/src/component_contract.rs`
  - 工作量：0.5 天

- [ ] **A-2-2** 添加 `init_sort() -> i32` 默认 0
  - 工作量：0.5 天

- [ ] **A-2-3** 添加 `shutdown(&self)` 生命周期钩子
  - 工作量：0.5 天

- [ ] **A-2-4** 添加 `has_async_run() -> bool` 优化钩子
  - 工作量：0.5 天

#### A-3：vernal-macros 增强

- [ ] **A-3-1** 宏支持 `#[component(inner_init = "fn")]`
  - 工作量：1 天

- [ ] **A-3-2** 宏支持 `#[component(shutdown = "fn")]`
  - 工作量：0.5 天

- [ ] **A-3-3** 宏支持 `#[component(init_sort = N)]`
  - 工作量：0.5 天

### 7.2 阶段 B：异步任务体系 🔴 P0

#### B-1：AsyncTask trait

- [ ] **B-1-1** 定义 `AsyncTask` trait
  - 来源：tx_di 的 `Component::async_run`
  - 位置：`crates/vernal-context/src/async_task.rs`
  - 工作量：1 天

- [ ] **B-1-2** `ManagedTaskSupervisor` 自动派生 AsyncTask
  - 工作量：1.5 天

- [ ] **B-1-3** 宏支持 `#[component(async_run = "fn")]`
  - 工作量：1 天

### 7.3 阶段 C：横切层补全 🟡 P1

#### C-1：vernal-aspects（对标 spring-aspects）

- [ ] **C-1-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **C-1-2** `TransactionalAspect`（@Transactional）
  - 工作量：3 天

- [ ] **C-1-3** `CacheableAspect`（@Cacheable）
  - 工作量：3 天

- [ ] **C-1-4** `AsyncAspect`（@Async）
  - 工作量：2 天

- [ ] **C-1-5** `ScheduledAspect`（@Scheduled）
  - 工作量：2 天

#### C-2：vernal-expression（对标 spring-expression，**精简版**）

- [ ] **C-2-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **C-2-2** 简化 SpEL 子集（**仅 4 种表达式**）
  - 工作量：4 天
  - **仅支持**：
    - 字面量（字符串、数字、布尔）
    - 属性访问（`env.get('key')`）
    - 比较运算（`==`, `!=`, `<`, `>`）
    - 逻辑运算（`&&`, `||`, `!`）
  - **不支持**：方法调用、复杂 SpEL 语法

- [ ] **C-2-3** 集成到 `ComponentCondition`
  - 工作量：1 天

### 7.4 阶段 D：高级抽象层 🟡 P1

#### D-1：vernal-tx（对标 spring-tx，**仅框架抽象**）

- [ ] **D-1-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **D-1-2** `PlatformTransactionManager` trait
  - 工作量：1 天

- [ ] **D-1-3** `TransactionDefinition` + 7 种传播行为
  - 工作量：1.5 天

- [ ] **D-1-4** `TransactionInterceptor`（AOP 拦截器）
  - 工作量：2 天

- [ ] **D-1-5** `#[transactional]` 过程宏
  - 工作量：1 天

- [ ] **D-1-6** `ReactiveTransactionManager`
  - 工作量：2 天

#### D-2：vernal-cache（对标 spring-cache，**桥接 hutool-cache**）

- [ ] **D-2-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **D-2-2** `CacheManager` trait（**框架抽象**）
  - 工作量：1 天

- [ ] **D-2-3** **桥接** `HutoolCacheManager`（实现 `CacheManager`，内部用 hutool-cache）
  - 来源：**不实现**，**桥接** hutool-cache
  - 工作量：2 天

- [ ] **D-2-4** `#[cacheable]`/`#[cache_put]`/`#[cache_evict]` 宏
  - 工作量：2 天

#### D-3：vernal-async（对标 spring-async）

- [ ] **D-3-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **D-3-2** `AsyncTaskExecutor` trait
  - 工作量：1 天

- [ ] **D-3-3** `TokioAsyncTaskExecutor` 实现
  - 工作量：1.5 天

- [ ] **D-3-4** `#[async]` 宏
  - 工作量：1.5 天

### 7.5 阶段 E：数据与基础设施 🟢 P2

#### E-1：vernal-db（对标 spring-jdbc，**桥接 hutool-db**）

- [ ] **E-1-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **E-1-2** `DataSource` trait（**框架抽象**）
  - 工作量：1 天

- [ ] **E-1-3** **桥接** `HutoolDataSource`（实现 `DataSource`，内部用 hutool-db）
  - 工作量：2 天

- [ ] **E-1-4** **桥接** `HutoolDbModule`（配置驱动）
  - 工作量：1 天

- [ ] **E-1-5** 集成 vernal-tx
  - 工作量：1.5 天

#### E-2：vernal-log（对标 spring-jcl，**仅框架门面**）

- [ ] **E-2-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **E-2-2** `Log` trait（**框架门面**，底层 tracing）
  - 工作量：1 天

- [ ] **E-2-3** `LogFactory`（桥接 tracing）
  - 工作量：1.5 天

#### E-3：vernal-actuator（对标 spring-actuator，**桥接 hutool-observability**）

- [ ] **E-3-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **E-3-2** `/health` 端点（**框架契约**）
  - 工作量：2 天

- [ ] **E-3-3** `/metrics` 端点（**桥接** hutool-observability）
  - 工作量：2 天

### 7.6 阶段 F：通信与协议 🟢 P2

#### F-1：vernal-messaging

- [ ] **F-1-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **F-1-2** `MessageChannel` trait
  - 工作量：2 天

#### F-2：vernal-websocket

- [ ] **F-2-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **F-2-2** WebSocket Handler
  - 工作量：3 天

### 7.7 阶段 G：测试与质量 🟢 P2

#### G-1：vernal-test

- [ ] **G-1-1** crate 结构搭建
  - 工作量：0.5 天

- [ ] **G-1-2** `TestContext` 框架
  - 工作量：3 天

- [ ] **G-1-3** 测试注解
  - 工作量：2 天

### 7.8 阶段 H：横切基础设施 🟢 P2

#### H-1：BeanDescriptor 增强（**仅在 vernal-beans 内**）

- [ ] **H-1-1** `BeanDescriptor<T>`（**框架必须**，用于配置属性绑定）
  - 工作量：2 天

- [ ] **H-1-2** `BeanDescCache`
  - 工作量：1.5 天

- [ ] **H-1-3** `BeanUtil::copy_properties`（**框架必须**）
  - 工作量：1.5 天

## 八、依赖方向（严格）

```
vernal-core ← vernal-beans ← vernal-context ← vernal
                ↑                ↑
           vernal-aop            │
                ↑                │
                └────────────────┘

hutool-rust → hutool-vernal → vernal-context（桥接层，外部依赖）
                              ↓
                          （不依赖）
```

### 禁止的依赖方向
- `vernal-core` **绝不**依赖 hutool-rust
- `vernal-beans` **绝不**依赖 hutool-rust
- `vernal-aop` **绝不**依赖 hutool-rust
- `vernal-context` **绝不**依赖 hutool-rust（仅通过 hutool-vernal 桥接）
- `vernal-macros` **绝不**依赖 hutool-rust

## 九、hutool-vernal 桥接实现（**重点**）

### 9.1 桥接实现原则

```rust
// ✅ 正确：vernal-cache 只定义抽象 trait
// crates/vernal-cache/src/manager.rs
pub trait CacheManager: Send + Sync {
    fn get_cache(&self, name: &str) -> Arc<dyn Cache>;
}

// ✅ 正确：hutool-vernal-cache 实现 trait，内部用 hutool
// crates/hutool-vernal-cache/src/lib.rs（独立仓库）
pub struct HutoolCacheManager { ... }
impl CacheManager for HutoolCacheManager {
    fn get_cache(&self, name: &str) -> Arc<dyn Cache> {
        // 内部调用 hutool_cache::Cache::new(...)
        Arc::new(HutoolCacheAdapter { ... })
    }
}

// ❌ 错误：vernal-cache 直接调用 hutool-cache
// crates/vernal-cache/src/hutool_impl.rs
pub struct HutoolCache { ... }
// 这会让 vernal-cache 依赖 hutool-cache，违反框架边界！
```

### 9.2 桥接的命名规范

| vernal 框架（抽象） | hutool-vernal（实现） |
|-------------------|---------------------|
| `vernal_cache::Cache` (trait) | `HutoolCache` (struct) |
| `vernal_cache::CacheManager` (trait) | `HutoolCacheManager` (struct) |
| `vernal_db::DataSource` (trait) | `HutoolDataSource` (struct) |
| `vernal_db::Db` (trait) | `HutoolDb` (struct) |

## 十、实施路线图

| 阶段 | 工作量 | 周数 | 优先级 | 状态 |
|------|--------|------|--------|------|
| A：基础设施精简 | 8.5 天 | 1.5 | 🔴 P0 | 待开始 |
| B：异步任务体系 | 3.5 天 | 0.5 | 🔴 P0 | 待开始 |
| C：横切层补全 | 12.5 天 | 2 | 🟡 P1 | 待开始 |
| D：高级抽象层 | 15.5 天 | 2.5 | 🟡 P1 | 待开始 |
| E：数据与基础设施 | 10 天 | 1.5 | 🟢 P2 | 待开始 |
| F：通信与协议 | 5.5 天 | 1 | 🟢 P2 | 待开始 |
| G：测试与质量 | 5.5 天 | 1 | 🟢 P2 | 待开始 |
| H：横切基础设施 | 5 天 | 1 | 🟢 P2 | 待开始 |
| **总计** | **66 天** | **~11 周** | — | — |

**对比修订前**：100+ 天 → 66 天（精简了约 35% 工作量，避免重复造轮子）

## 十一、核心边界规则总结

### 11.1 绝对禁止

| 行为 | 原因 |
|------|------|
| vernal-core 内置 DateUtil、StrUtil、JSONUtil | hutool 已有 |
| vernal-beans 直接调用 hutool-cache | 违反依赖方向 |
| vernal-context 直接调用 hutool-db | 违反依赖方向 |
| vernal-cache 提供 moka 实现 | 应在 hutool-vernal |
| 在 vernal 内重新实现 hutool 的任何工具 | 违反单一职责 |

### 11.2 必须遵循

| 行为 | 原因 |
|------|------|
| vernal-core 仅含 6 个必须工具 | 保持框架精简 |
| hutool-vernal 仅做适配，不实现 | 保持桥接层纯粹 |
| hutool-rust 不依赖 vernal | 保持工具库独立 |
| 用户可选 hutool-vernal | 不强制依赖 |

### 11.3 借鉴（设计思想）

| vernal 可以借鉴 hutool | vernal 不可以拷贝 hutool |
|---------------------|------------------------|
| hutool convert 的 Convert trait 设计 | hutool convert 的 31 个具体实现 |
| hutool StopWatch 的 API 设计 | hutool StopWatch 的全部功能 |
| hutool ObjectId 的生成算法 | hutool ObjectId 的完整实现 |
| hutool BeanDesc 的反射模式 | hutool BeanDesc 的所有 getter |

## 十二、立即可执行

**P0（必须立刻）**：阶段 A + 阶段 B（精简基础设施 + 异步任务）

要现在开始执行吗？建议从 **A-1-2（INIT_SORT 常量）** + **A-2（Component trait 完整化）** 开始，这是吸纳 tx_di 设计的关键一步。
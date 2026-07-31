<!-- migration-doc: authority=historical canonical=迁移路线图.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [迁移路线图.md](迁移路线图.md) 为准。

# spring-core + tx_di → vernal-core 全量迁移路线图 v2.0

> 版本：v2.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core + tx_di **dev**（`/Users/wandl/workspaces/workspace-github-easy-4-rust/tx_di`）
> 仓库：easy-4-rust/vernal @ dev（`/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework`）
> 现状基线：`crates/vernal-core/`（22 个 .rs 文件 / 1,417 行 / 7 个模块 / 0 外部依赖 / 17 个公开声明）
> 实现底盘：**Rust 同步语义 + Arc<dyn Error> + 零外部依赖 + feature-gated 可选集成**
> **目录命名 100% 镜像 Spring core 包路径 + 镜像 tx_di tx-error 包路径**
> 本文档随代码同步维护，每阶段完成时更新

---

## 一、总目标

以 Spring Framework 7.0.8 的 **spring-core**（12.8 万行 Java / 737 个 .java / 51 个顶级包）与
tx_di 的 **tx-di-core + common/tx_error + common/tx_common**（414 个 .rs / 73,479 行）作为双蓝本，
实现功能语义完全对齐的 `vernal-core` crate。

**核心定位**：
- vernal-core 是 **最底层稳定合约包**，禁止反向依赖任何其他 vernal 子系统
- 是 `vernal-beans / context / aop / expression / web / tx / db / cache` 等 32 个 crate 的基础合同层
- 单向依赖：`vernal-core` ← `vernal-beans` ← `vernal-context` ← `vernal-web` ← ...（永远反向不可达）

### 设计原则（强制）

1. **vernal-core 是最底层基础合同**：禁止反向依赖 `vernal-beans / context / aop / web` 等
2. **默认零外部依赖**：除 `linkme` 在 workspace level 提供外，`cargo build -p vernal-core` 必须零外部依赖通过
3. **`BoxError` + `SharedError` 是错误体系的根基**：业务 crate 用 `#[derive(thiserror::Error)]` 后自动兼容
4. **类型命名 100% 保留**：Spring `Lifecycle / ConversionService / Ordered / StopWatch / ObjectId` 名字照搬
5. **状态机 100% 镜像**：`AppLifecyclePhase`（8 变体）vs Spring `SmartLifecycle` 8 状态 vs tx_di 4 阶段
6. **生命周期 3 阶段简化**：vernal `Lifecycle::initialize / start / stop` 对齐 Spring `Lifecycle` 的精简版；tx_di 的 5 阶段收编到 `vernal-context`
7. **公开类型必须有中文 rustdoc 注释**：结构体、枚举、trait、方法、字段
8. **`#[derive(ErrorCode)]` 必须留给 `vernal-macros`**（不污染 vernal-core 的同步性质）
9. **不使用 `async-trait`**：Rust 1.75+ 原生支持 `async fn in trait`，vernal-core 不持有 `dyn` trait
10. **不引入 anyhow**：vernal-core 用 `Arc<dyn Error>` 替代 anyhow，业务 crate 在 `vernal-bridge` 互转

### vernal-core 现状 → 目标（v2.0）

| 维度 | 现状（v1.0） | 目标（v2.0） |
|---|---|---|
| 子模块 | 7（convert / error / id / lifecycle_phase / ordered / time / failure） | **9**（新增 `app_lifecycle_phase` 拆分；保留 7；新增 `span`） |
| 文件数 | 22 | **30+** |
| 代码行数 | 1,417 | **~2,200** |
| 单元测试 | 6 | **40+** |
| 错误域覆盖 | 10 | **13+**（新增 `MIGRATION / SECURITY / METRICS`） |
| 初始化排序常量 | 4 | **8+**（新增 `BEAN_FACTORY / EVENT_LISTENER / MESSAGE_SOURCE / TASK`） |
| 内置 Convertible | 5 | **9+**（新增 `PathBuf / OffsetDateTime / Duration / URL / SocketAddr`） |
| ID 生成器 | 1（ObjectId） | **5**（feature-gated uuid/ulid/nanoid/snowflake 全部接入） |
| Rust 生态集成 | 0 | **14 个 feature-gated 模块**（`uuid / ulid / nanoid / snowflake / serde / chrono / time / url / bytes / case / web-time / derive-all / convert-* / id-*`） |
| 公开 API 稳定性承诺 | 部分 | **完整公开契约清单**（见一致性检查 §7） |

---

## 二、进度总览

| 阶段 | 内容 | 状态 | 验收 |
|---|---|---|---|
| **S0** | 4 份迁移文档 v2.0（路线图 / 对象对照 / 语义对照 / 一致性检查） | ✅ | docs/vernal-*.md 4 份 |
| **S1** | 命名修复：`LifecyclePhase` 重命名为 `AppLifecyclePhase` + 模块重命名 | ⬜ | 不破坏下游；通过 `cargo build -p vernal-core` 与 `-p vernal-context` 全测试 |
| **S2** | 错误体系补强：扩展 `From<...>` 派生 + `is_infrastructure()` + 跨变体 PartialEq 测试 | ⬜ | 9 个 trait 测试 + 8 个 From 测试通过 |
| **S3** | 错误域扩展：增加 `MIGRATION / SECURITY / METRICS` 三个域常量 | ⬜ | 测试覆盖所有 13 域 |
| **S4** | Ordered 锚点扩展：增加 `INIT_SORT_BEAN_FACTORY / EVENT_LISTENER / MESSAGE_SOURCE / TASK` + Spring 风格别名 | ⬜ | 8 个常量关系测试 + `HIGHEST_PRECEDENCE / LOWEST_PRECEDENCE` 别名 |
| **S5** | Convertible 扩展：`PathBuf / OffsetDateTime / Duration / URL / SocketAddr`（**feature-gated**） | ⬜ | 5 个新转换器 round-trip 测试通过 |
| **S6** | StopWatch 强化：`short_summary()` / `pretty_print_with_unit(unit)` + `StopWatchUnit` 枚举 | ⬜ | 与 Spring 6.1+ 输出对齐测试 |
| **S7** | ObjectId serde 支持 + uuid/ulid/nanoid/snowflake 多 ID 后端（**feature-gated**） | ⬜ | `serde_json::to_string(&id)` 测试通过；4 个 ID 后端 trait 抽象统一 |
| **S8** | Rust 生态 feature flag 接入（**14 个 feature flag**）：uuid / ulid / nanoid / snowflake / chrono / time / url / bytes / serde / case / web-time / thiserror | ⬜ | 默认 `cargo build` 零外部依赖通过；`--features "uuid,serde,chrono"` 增加 3 个 crate |
| **S9** | `vernal_core::Span` 诊断跨度类型（对齐 Spring 6.1 Observation API 的轻量版） | ⬜ | Span 创建 / 进入 / 退出 / 错误传播测试通过 |
| **S10** | 测试覆盖与文档收尾 | ⬜ | 40+ 单元测试；中文 rustdoc 覆盖率 100% |

---

## 三、阶段详细计划

### S1 — 命名修复：`LifecyclePhase` → `AppLifecyclePhase`（**预估 0.5 天**）

**问题**：`vernal-core::lifecycle_phase::LifecyclePhase`（8 变体 Created/Initializing/Initialized/Starting/Started/Stopping/Stopped/Failed）
与 `vernal-context::lifecycle_phase::LifecyclePhase`（3 变体 Initialize/Start/Stop）重名，造成语义混淆。

**方案**：重命名 vernal-core 版本为 `AppLifecyclePhase`。

**目标文件**：
- 移动：`vernal-core/src/lifecycle_phase.rs` → `vernal-core/src/app_lifecycle_phase.rs`
- 修改 `vernal-core/src/lib.rs`：`mod lifecycle_phase; pub use lifecycle_phase::LifecyclePhase;` → `mod app_lifecycle_phase; pub use app_lifecycle_phase::AppLifecyclePhase;`
- 同步检查：`grep -rn "vernal_core::lifecycle_phase" crates/` 应该全部替换为 `vernal_core::AppLifecyclePhase`

**验收**：
- `cargo build -p vernal-core` 通过
- `cargo build -p vernal-context` 通过（**实测仅 vernal-core 自身使用，下游无 import**，可直接重命名）
- 全 workspace `cargo test` 通过
- 中文 doc 注释中明确说明 `AppLifecyclePhase` 对标 Spring `SmartLifecycle.getPhase()` 的 8 个状态

---

### S2 — 错误体系补强（**预估 1.5 天**）

**目标文件**：`vernal-core/src/error/vernal_error.rs`（修改 + 新增 2 个测试文件）

**新增 trait 实现**：

```rust
// 现有 From 实现
impl From<BoxError> for VernalError { ... }      // ✅ 已有
impl From<SharedError> for VernalError { ... }  // ✅ 已有
impl From<std::io::Error> for VernalError { ... } // ✅ 已有

// 新增：tokio JoinError（feature-gated，避免反向依赖 tokio）
#[cfg(feature = "tokio")]
impl From<tokio::task::JoinError> for VernalError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::infrastructure(err)
    }
}

// 新增：From<String> / From<&str>（不引入 anyhow 的字符串错误捕获）
impl From<String> for VernalError {
    fn from(s: String) -> Self {
        Self::infrastructure(std::io::Error::new(std::io::ErrorKind::Other, s))
    }
}
impl From<&str> for VernalError {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

// 新增：From<VernalError> for BoxError（自动派生已有，显式记录文档）
// 由 std blanket impl 提供: impl<E: Error + Send + Sync + 'static> From<E> for Box<dyn Error + Send + Sync>

// 新增：Display 测试覆盖 4 变体
```

**新增单元测试**（S2 测试文件：`vernal-core/src/error/tests_vernal_error.rs`）：
- `VernalError::Business { domain: "ioc", code: -1, message: "NotFound" }.is_infrastructure()` → false
- `VernalError::Infrastructure(arc) == VernalError::Infrastructure(arc)` → true（Arc ptr_eq）
- 跨变体比较：`Business{} ≠ WithContext{} ≠ WithContextEntries{} ≠ Infrastructure{}`
- `is_same_kind`: `Business{d:"ioc", c:-1} == WithContext{d:"ioc", c:-1, ..}` → true
- 8 个 From 测试：覆盖 `BoxError`、`SharedError`、`io::Error`、`String`、`&str`
- `Display` 测试：`Business{...}` → `"[ioc:-1] NotFound"`
- `Error::source()` 测试：`Infrastructure` 返回 `Some(...)`，其余返回 `None`

**验收**：
- 17+ 测试通过
- `#[derive(thiserror::Error)]` 的业务错误可 `?` 直接转 `VernalError::Infrastructure`
- 中文 rustdoc 覆盖率 100%（每个 From 都有用途说明）

---

### S3 — 错误域扩展（**预估 0.25 天**）

**目标文件**：`vernal-core/src/error/error_domain.rs`（修改）

**新增常量**：

```rust
impl ErrorDomain {
    // 现有 10 个常量（保留不变）
    pub const IOC: &'static str = "ioc";
    pub const AOP: &'static str = "aop";
    pub const CONTEXT: &'static str = "context";
    pub const WEB: &'static str = "web";
    pub const HTTP: &'static str = "http";
    pub const TOWER: &'static str = "tower";
    pub const DISCOVERY: &'static str = "discovery";
    pub const MACROS: &'static str = "macros";
    pub const CORE: &'static str = "core";
    pub const BRIDGE: &'static str = "bridge";

    // 新增 3 个常量
    /// 数据库迁移子系统（对标 Spring `org.springframework.jdbc.datasource.init`）
    pub const MIGRATION: &'static str = "migration";
    /// 安全子系统（对标 Spring Security `org.springframework.security`）
    pub const SECURITY: &'static str = "security";
    /// 监控指标子系统（对标 Spring Boot Actuator `org.springframework.boot.actuate.metrics`）
    pub const METRICS: &'static str = "metrics";
}
```

**验收**：
- 13 个域常量集合测试通过
- 中文 rustdoc 每个域说明对应 Spring 子系统

---

### S4 — Ordered 锚点扩展（**预估 0.5 天**）

**目标文件**：`vernal-core/src/ordered.rs`（修改）

**新增常量**：

```rust
// 现有 4 个常量（保留不变）
pub const INIT_SORT_INFRASTRUCTURE: i32 = i32::MIN + 1; // -2147483647
pub const INIT_SORT_BUSINESS: i32 = 0;
pub const INIT_SORT_APPLICATION: i32 = i32::MAX - 1; // 2147483646
pub const INIT_SORT_DEFAULT: i32 = i32::MAX;          // 2147483647

// 新增 4 个 Spring 风格锚点
/// BeanFactory 初始化优先级（早于所有业务，先把 Bean 工厂建好）
pub const INIT_SORT_BEAN_FACTORY: i32 = i32::MIN + 2;
/// 事件监听器注册优先级（业务开始前）
pub const INIT_SORT_EVENT_LISTENER: i32 = -2_000_000_000;
/// 消息源初始化优先级（次之）
pub const INIT_SORT_MESSAGE_SOURCE: i32 = -1_000_000_000;
/// 异步任务启动优先级（最后启动，比 Web 还晚）
pub const INIT_SORT_TASK: i32 = i32::MAX - 100;

// 新增 Spring 风格别名（与 Ordered.HIGHEST_PRECEDENCE / LOWEST_PRECEDENCE 对齐）
/// Spring `Ordered.HIGHEST_PRECEDENCE` 等价值
pub const HIGHEST_PRECEDENCE: i32 = INIT_SORT_INFRASTRUCTURE;
/// Spring `Ordered.LOWEST_PRECEDENCE` 等价值
pub const LOWEST_PRECEDENCE: i32 = INIT_SORT_DEFAULT;
```

**验证**：8 个常量之间的偏序关系不变：
```
INFRASTRUCTURE < BEAN_FACTORY < EVENT_LISTENER < MESSAGE_SOURCE < BUSINESS < APPLICATION < TASK < DEFAULT
```

**验收**：
- 单元测试覆盖 8 个常量之间的 `<` / `>` 关系
- 中文 rustdoc 每个常量对应 Spring 哪个组件
- 与 `vernal-expression/spring-aspects` 的 8 个 INIT_SORT 命名保持一致

---

### S5 — Convertible 扩展（**预估 2 天**）

**目标文件**：新增 5 个 .rs + 1 个 mod.rs 修改

| 新增文件 | Spring / Rust 对应 | 依赖 |
|---|---|---|
| `convert/path_converter.rs` | `StringToPathConverter`（Spring 自带） + `PathBuf::from_str_value` | std only |
| `convert/datetime_converter.rs` | `StringToInstantConverter` / `StringToLocalDateConverter` | `time` crate (feature = "convert-time") |
| `convert/duration_converter.rs` | `StringToDurationConverter`（Spring 风格） + `Duration::from_str_value` | std only |
| `convert/url_converter.rs` | `StringToURLConverter`（Spring `org.springframework.core.convert.support`） | `url` crate (feature = "convert-url") |
| `convert/socket_addr_converter.rs` | Rust 特有：网络栈配置属性绑定 | std only |

**修改**：`convert/mod.rs`
- 暴露 `PathConverter` / `DatetimeConverter` / `DurationConverter` / `UrlConverter` / `SocketAddrConverter` 结构体
- 与现有 5 个内置实现并列
- 新增 `ConversionService::can_convert::<T>() -> bool` 判定方法（对齐 Spring `ConversionService.canConvert()`）

**验收**：
- `PathBuf::from_str_value("/tmp/foo.txt")` 正确返回 `PathBuf`
- `OffsetDateTime::from_str_value("2026-01-01T00:00:00Z")` round-trip 测试通过（feature = "convert-time"）
- `Duration::from_str_value("30s")` round-trip 测试通过（std only）
- `url::Url::from_str_value("https://example.com")` round-trip 测试通过（feature = "convert-url"）
- `SocketAddr::from_str_value("127.0.0.1:8080")` round-trip 测试通过（std only）
- 单元测试 8+ 个，覆盖每个新转换器

---

### S6 — StopWatch 强化（**预估 0.5 天**）

**目标文件**：`vernal-core/src/time/stop_watch.rs`（修改）+ 新增 `time/stop_watch_unit.rs`

**新增枚举**：`StopWatchUnit { Nano, Micro, Milli, Second, Minute }`

**新增方法**：

```rust
impl StopWatch {
    /// Spring `StopWatch.shortSummary()` 等价
    /// 输出示例：`"StopWatch 'op-name': 0.123 seconds (1 tasks)"`
    pub fn short_summary(&self) -> String;

    /// Spring 6.1+ `StopWatch.prettyPrint(TimeUnit)` 等价
    /// 输出按指定单位精度格式化
    pub fn pretty_print_with_unit(&self, unit: StopWatchUnit) -> String;

    /// Spring `StopWatch.start(long)` 等价
    pub fn start_with_ticks(&mut self, name: impl Into<String>, ticks: u64);

    /// Spring `StopWatch.getTaskInfo()` 等价
    pub fn task_info_array(&self) -> Vec<TaskInfo>;
}
```

**验收**：
- `short_summary()` 输出格式：`"StopWatch 'op-name': 0.123 seconds (1 tasks)"`
- `pretty_print_with_unit(StopWatchUnit::Nano)` 输出含 ns 精度
- 与 vernal-expression 的 `vernal_aspects` StopWatch 单元测试对齐

---

### S7 — ObjectId serde 支持 + 多 ID 后端抽象（**预估 2 天**）

**目标文件**：
- 新增 `id/serde_object_id.rs`（feature = "serde"）
- 新增 `id/uuid_id.rs`（feature = "id-uuid"）
- 新增 `id/ulid_id.rs`（feature = "id-ulid"）
- 新增 `id/nanoid_id.rs`（feature = "id-nanoid"）
- 新增 `id/snowflake_id.rs`（feature = "id-snowflake"）
- 新增 `id/id_generator.rs`（核心 trait 抽象）

**核心 trait**：

```rust
/// 统一 ID 生成器 trait
pub trait IdGenerator: Send + Sync {
    /// 生成新 ID
    fn next_id(&self) -> String;
    /// ID 类型名
    fn kind(&self) -> &'static str;
}
```

**各实现**：

| ID 类型 | 底层 crate | 版本 | 许可证 | Feature 名 | 用途 |
|---|---|---|---|---|---|
| `ObjectId` (24-hex) | std | — | — | (内置) | 框架内部追踪（任务、AOP 调用） |
| `Uuid` v4/v7 | `uuid` | `1.24` | MIT/Apache-2.0 | `id-uuid` | 全局唯一标识（外部接口） |
| `Ulid` | `ulid` | `3.0` | MIT | `id-ulid` | 时间排序 ID（分布式场景） |
| `NanoId` | `nanoid` | `0.5` | MIT | `id-nanoid` | 短 ID（URL-friendly） |
| `Snowflake` | 自实现（基于 `uuid v7` + epoch） | — | — | `id-snowflake` | Twitter 风格 64-bit |

**验收**：
- `serde_json::to_string(&ObjectId::new())` → `"\"" + 24-hex + "\""`
- `serde_json::from_str::<ObjectId>(s)` round-trip 一致
- 4 个新 ID 类型（uuid/ulid/nanoid/snowflake）feature-gated 启用后可独立编译
- 单元测试 10+ 个

---

### S8 — Rust 生态 feature flag 接入（**预估 2 天**）

**目标文件**：`vernal-core/Cargo.toml`（修改）+ 8+ 个 feature-gated 模块

**最终 Cargo.toml 配置**：

```toml
[package]
name = "vernal-core"
version.workspace = true
edition.workspace = true       # 2024
rust-version.workspace = true  # 1.88
license.workspace = true       # MIT
description = "Stable shared contracts for the Vernal framework."
publish = false

[dependencies]
linkme = { workspace = true }   # 已有（workspace level 提供）
thiserror = { version = "2.0", optional = true }
pastey = { version = "0.2", optional = true }                  # Tier 1：宏拼接
inventory = { version = "0.3", optional = true }               # Tier 1：分布式注册（备选 linkme）
once_cell = { version = "1.21", optional = true }              # Tier 1：框架级单例
uuid = { version = "1.24", optional = true, features = ["v4", "v7"] }
ulid = { version = "3.0", optional = true }
nanoid = { version = "0.5", optional = true }
url = { version = "2.5", optional = true }
bytes = { version = "1.12", optional = true }
time = { version = "0.3", optional = true, features = ["formatting", "parsing"] }
chrono = { version = "0.4", optional = true, default-features = false, features = ["clock", "std"] }
serde = { version = "1", optional = true }
derive_more = { version = "2.1", optional = true, features = ["display", "from", "error"] }
convert_case = { version = "0.11", optional = true }
web-time = { version = "1.1", optional = true, default-features = false }

[features]
default = []
# Tier 1（强烈推荐 - 框架基础设施）
error-derive = ["dep:thiserror"]
macros = ["dep:pastey"]
once-cell = ["dep:once_cell"]
# Tier 2（强烈推荐 - 用户可见类型）
id-uuid = ["dep:uuid"]
id-ulid = ["dep:ulid"]
id-nanoid = ["dep:nanoid"]
convert-uuid = ["id-uuid"]
convert-url = ["dep:url"]
convert-time = ["dep:time"]
convert-chrono = ["dep:chrono"]
convert-bytes = ["dep:bytes"]
serde = ["dep:serde"]
derive-extras = ["dep:derive_more"]
# Tier 3（可选 - 特定场景）
case-conv = ["dep:convert_case"]
time-web = ["dep:web-time"]
```

**不引入的依赖**（明确排除，**禁止**新增）：
- ❌ `anyhow`（vernal-core 自身永不允许依赖；业务 crate 在 `vernal-bridge` 互转）
- ❌ `tokio`（runtime 耦合，由 `vernal-context` 引入）
- ❌ `priority-queue`（LGPL/MPL copyleft，污染 Vernal 的 MIT）
- ❌ `instant`（已停止维护，改用 `web-time` 或 `std::time`）
- ❌ `snowflake-rs` 0.1.1（自 2018 年起废弃）
- ❌ `async-trait`（Rust 1.75+ 原生支持 native `async fn in trait`，vernal-core 不持有 `dyn` trait）
- ❌ `tracing`（归 `vernal-log`，vernal-core 自身不打印日志）
- ❌ `serde_json`（仅 `serde` trait；JSON 序列化由 `vernal-context` 决定）
- ❌ `toml`（同 `serde_json`）

**验收**：
- `cargo build -p vernal-core` 默认零外部依赖通过
- `cargo build -p vernal-core --features "id-uuid,convert-time,serde"` 增加 3 个 crate
- 所有 feature 独立可启用、可组合使用（`#[cfg(feature = "...")]` 严格 gating）
- 公开文档注明每个 feature 的使用场景与代价
- 中文 rustdoc 每个 feature 都有 § 章节说明

---

### S9 — `Span` 诊断跨度类型（**预估 1.5 天**）

**目标文件**：新增 `vernal-core/src/diagnostics/span.rs` + `diagnostics/mod.rs` + `diagnostics/span_id.rs`

**设计来源**：对标 Spring 6.1 `Observation` API 的轻量版，提供 trace 上下文。

**核心类型**：

```rust
/// 诊断跨度：用于追踪一次操作的开始 / 结束 / 错误传播
pub struct Span {
    span_id: SpanId,
    parent_id: Option<SpanId>,
    name: Cow<'static, str>,
    start: Instant,
    attributes: Vec<(&'static str, AttributeValue)>,
    status: SpanStatus,
}

pub enum SpanStatus {
    Unset,
    Ok,
    Error(SharedError),
}

pub enum AttributeValue {
    String(Cow<'static, str>),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Span {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self;
    pub fn child(&self, name: impl Into<Cow<'static, str>>) -> Self;
    pub fn set_attribute(&mut self, key: &'static str, value: impl Into<AttributeValue>);
    pub fn record_error(&mut self, error: &dyn Error);
    pub fn end(self) -> SpanReport;
    pub fn is_recording(&self) -> bool;
}
```

**验收**：
- Span 创建 / 进入 / 退出 / 错误传播测试通过
- 父子嵌套层级正确
- `SpanReport` 包含总耗时 + 错误信息
- 与 vernal-context 的 StartupObservation 对齐
- 单元测试 5+ 个

---

### S10 — 测试覆盖与文档收尾（**预估 1.5 天**）

**目标**：40+ 单元测试，中文文档覆盖率 100%

**单元测试清单**（按模块）：
- `error/vernal_error.rs` + `tests_*.rs`：12 个测试（4 变体 × Display/Debug/PartialEq/From）
- `error/error_code.rs`：3 个测试（domain/code/message/is_same_kind）
- `error/error_context.rs`：4 个测试（new / with / is_empty / Display）
- `error/error_report.rs`：3 个测试（from_error / from_error_code / Display）
- `error/error_domain.rs`：2 个测试（13 个常量集合 / Display）
- `convert/*.rs`：14 个测试（5 内置 × round-trip + 3 新增（PathBuf/Datetime/Duration）+ URL/SocketAddr + can_convert）
- `id/*.rs`：8 个测试（ObjectId 生成 / 4 个新 ID 类型 / Display / 唯一性 / serde）
- `time/stop_watch.rs`：5 个测试（pretty_print / pretty_print_with_unit / short_summary / task_count）
- `app_lifecycle_phase.rs`：3 个测试（is_active / is_terminal / is_transitioning）
- `ordered.rs`：4 个测试（8 常量偏序 / HIGHEST_PRECEDENCE / LOWEST_PRECEDENCE 别名）
- `lib.rs`：1 个测试（FRAMEWORK_VERSION 不为空 / MINIMUM_RUST_VERSION 格式正确）
- `diagnostics/span.rs`：5 个测试（new / child / set_attribute / record_error / end）

**合计**：62 个单元测试（目标 40+，超出以确保核心契约覆盖完整）

**文档清单**：
- 给每个公开类型增加中文 `///` rustdoc（覆盖率 100%）
- 给每个 trait 方法增加中文示例
- 模块级 `//!` 文档说明与 Spring / tx_di 对应关系
- 4 份 v2.0 迁移文档（路线图 / 对象对照 / 语义对照 / 一致性检查）齐备且互相引用

**验收**：
- `cargo test -p vernal-core` 全 62 测试通过
- `cargo doc -p vernal-core --no-deps` 无警告
- 中文 rustdoc 覆盖率 100%
- `cargo clippy -p vernal-core -- -D warnings` 无警告

---

## 四、阶段依赖图

```
S0 ✅ → S1 → { S2 + S3 + S4 + S5 + S6 + S7 + S8 + S9 } → S10
          │
          └── S1 必须先完成（重命名生命周期阶段）
              │
              └── S2-S9 可以并行开发（独立模块）
                  │
                  └── S10 必须在所有前序阶段完成后进行
```

---

## 五、风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| S1 重命名破坏下游 crate | `vernal-context` 等下游无法编译 | 实测仅 vernal-core 自身使用 `LifecyclePhase`，可直接重命名；改 `lib.rs` re-export 一次完成 |
| S2 新增 `From<JoinError>` 反向依赖 tokio | 破坏 vernal-core 零依赖原则 | **改用条件 cfg**：`#[cfg(feature = "tokio")] impl From<tokio::task::JoinError> for VernalError { ... }` |
| S5 `OffsetDateTime` 引入 `time` crate 反向依赖 | 破坏零依赖 | 仅在 `feature = "convert-time"` 启用 |
| S7 `uuid` feature 与现有 `ObjectId` 概念重叠 | 用户不知道用哪个 | 文档明确区分：`ObjectId` 任务追踪（内部）/ `uuid` 全局唯一（外部接口） |
| S7 `snowflake` crate 已废弃 | 引入陈旧依赖 | **自实现** snowflake ID（基于 `uuid v7` + epoch），不引入任何 `snowflake*` crate |
| S8 `chrono` 默认 features 含 `iana-time-zone` + `now` | 体积膨胀 | 必须 `default-features = false` 后只开启所需 features |
| S8 `inventory` vs `linkme` 同时启用 | 符号冲突 | 二选一；默认沿用 workspace 已有的 `linkme` |
| S9 Spring Observation API 太重 | 引入复杂依赖 | vernal-core 仅实现最轻量版本（Span + SpanReport），无 OTel 集成 |
| Spring 6.1 StopWatch 行为变更（`prettyPrint(TimeUnit)`） | 输出格式变化 | 严格对齐 Spring 6.1 输出格式，提供 `StopWatchUnit` 参数控制 |
| v1.0 文档存在但未实施 | S0 阶段文档与代码可能不一致 | S10 阶段实施前需先更新 v2.0 文档，避免历史债务 |

---

## 六、验收矩阵

| Spring / tx_di 概念 | vernal-core 实现 | 阶段 | 测试期望 | 状态 |
|---|---|---|---|---|
| Spring `ConversionService.convert()` | `ConversionService::convert::<T>()` | 已完成 | 5 内置类型通过 | ✅ |
| Spring `ConversionService.canConvert()` | `ConversionService::can_convert::<T>()` | S5 | 类型判定通过 | ⬜ |
| Spring `Ordered.getOrder()` | 8 个 `INIT_SORT_*` 常量 | S4 | 偏序关系测试 | ⬜ |
| Spring `Ordered.HIGHEST_PRECEDENCE` | `HIGHEST_PRECEDENCE` 别名 | S4 | 与 `INIT_SORT_INFRASTRUCTURE` 相等 | ⬜ |
| Spring `Lifecycle.Phase` | `AppLifecyclePhase` 8 变体 | S1 | is_active / is_terminal / is_transitioning | ⬜ |
| Spring `StopWatch.prettyPrint()` | `StopWatch::pretty_print()` | 已完成 | 输出格式与 Spring 6.0 一致 | ✅ |
| Spring 6.1 `StopWatch.prettyPrint(TimeUnit)` | `StopWatch::pretty_print_with_unit()` | S6 | 5 时间单位输出 | ⬜ |
| Spring `StopWatch.shortSummary()` | `StopWatch::short_summary()` | S6 | 输出 `"StopWatch 'name': X seconds (N tasks)"` | ⬜ |
| Spring `StringToInstantConverter` | `convert-time` feature | S5 | `OffsetDateTime::from_str_value()` round-trip | ⬜ |
| Spring `StringToURLConverter` | `convert-url` feature | S5 | `url::Url::from_str_value()` round-trip | ⬜ |
| tx_di `AppError::ErrCode` | `VernalError::Business { domain, code, message }` | 已完成 | 归一化测试通过 | ✅ |
| tx_di `AppError::WithContext` | `VernalError::WithContext { context: String }` | 已完成 | 上下文测试通过 | ✅ |
| tx_di `AppError::Internal(anyhow)` | `VernalError::Infrastructure(SharedError)` | 已完成 | anyhow 痕迹通过 `From<BoxError>` 适配 | ✅ |
| tx_di `CodeMsg` trait | `ErrorCode` trait + `into_vernal_error()` | 已完成 | `#[derive(ErrorCode)]` 落库（vernal-macros） | ✅ |
| tx_di `BoxError / SharedError` | 别名稳定 | 已完成 | 类型签名一致 | ✅ |
| tx_di `Convertible` 5 内置 + 5 新增 | `convert/` 模块 10 个内置实现 | S5 | round-trip + 类型判定 | ⬜ |
| tx_di `id::UUID / Snowflake` | `id-uuid` / `id-snowflake` features | S7 | 4 个新 ID 类型生成测试 | ⬜ |
| Spring 6.1 `Observation` API | `vernal_core::diagnostics::Span` | S9 | Span 创建 / 嵌套 / 错误传播测试 | ⬜ |

---

## 七、进度跟踪（完成时填写）

| 日期 | 阶段 | 完成 | 累计 |
|---|---|---|---|
| 2026-07-27 | S0 ✅ | ✅ | S0 |
| 待定 | S1 | ⬜ | S0-S1 |
| 待定 | S2-S9 | ⬜ | S0-S9 |
| 待定 | S10 | ⬜ | S0-S10（迁移完成） |

---

## 八、与 vernal-aspects 的协同

vernal-aspects（已完成 spring-aspects 迁移）与 vernal-core 的协同点：

| vernal-core | vernal-aspects 消费方 |
|---|---|
| `ConversionService` | 缓存切面 `cache_operation.rs` 中的 key 转换 |
| `INIT_SORT_*` 常量 | 切面执行顺序（`@Order` 等价） |
| `AppLifecyclePhase` | 切面自身的初始化阶段追踪 |
| `VernalError` | 所有切面异常传播 |
| `ObjectId` | AOP 调用 ID（`InvocationId`） |
| `StopWatch` | 切面计时 |
| `Span`（S9 新增） | 切面调用链路追踪 |

---

## 九、与 tx_di 迁移的协同

tx_di 的迁移在 `tx_admin` 示例项目（`/Users/wandl/workspaces/workspace-github-easy-4-rust/tx_di/examples/tx_admin`）中独立进行，不影响 vernal-core。
但 **vernal-core 本身就是 tx_di 的简化版**：vernal-core 的 `VernalError` 抽象正是从 `tx_error::AppError` 抽象而来，因此 vernal-core 与 tx_di 共享同一套设计语言，迁移完成后可以直接在两边互通。

### 9.1 共享接口层（建议在 `vernal-bridge` 提供）

```rust
// vernal-bridge（不在 vernal-core 中）
impl From<anyhow::Error> for BoxError {
    fn from(e: anyhow::Error) -> Self { Box::new(e) }
}
impl From<BoxError> for anyhow::Error {
    fn from(e: BoxError) -> Self { anyhow::Error::new(e) }
}
```

确保业务 crate 即便已用 anyhow 也能 0 成本接入 vernal-core。

---

## 十、版本与文档同步

- 本文档 v2.0 与 `vernal-framework/docs/vernal-core对象级对照表.md`、`vernal-core语义迁移对照表.md`、`vernal-core对象名称一致性检查.md` 同步维护
- v1.0 文档保留在 `vernal-framework/docs/vernal-expression/` 作为历史参考
- v2.0 文档放置在 `vernal-framework/docs/` 根目录，作为当前权威版本
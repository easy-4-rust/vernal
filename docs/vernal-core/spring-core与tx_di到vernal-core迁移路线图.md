# spring-core + tx_di → vernal-core 全量迁移路线图

> 版本：v1.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core + tx_di **dev**
> 仓库：easy-4-rust/vernal @ dev（`/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework`）
> 底盘：**Rust 同步语义 + thiserror 派生态** + Arc/dyn Error + 零外部依赖（feature-gated 集成）
> **目录命名 100% 镜像 Spring + 镜像 tx_di 路径**
> 本文档随代码同步维护，每阶段完成时更新

## 一、总目标

以 Spring Framework 7.0.8 的 spring-core 与 tx_di 的 tx-di-core/common/tx_error 为蓝本，
实现功能语义完全对齐的 `vernal-core` crate。

**目标**：
- 7 → 13+ 个子模块
- 19 → ~26+ 个 Rust 文件
- 1100 → ~1800 行代码
- 单元测试：6 → 30+
- 满足 vernal 各子系统（beans / context / aop / db / tx / web）作为基础合同的需求

### 设计原则

1. **vernal-core 是最底层基础合同**：禁止反向依赖 `vernal-beans / context / aop / web` 等
2. **零外部依赖原则**（除可选 feature flag）：不引入 anyhow / serde_json / chrono，与 spring-core 同级
3. **`BoxError` + `SharedError` 是错误体系的根基**：业务 crate 用 `#[derive(thiserror::Error)]` 后自动兼容
4. **类型命名 100% 保留**：Spring `Lifecycle / ConversionService / Ordered` 名字照搬，Rust 用常量代替 trait getter
5. **状态机 100% 镜像**：`AppLifecyclePhase`（原 `LifecyclePhase`）8 变体 vs Spring `SmartLifecycle` 8 状态
6. **生命周期 3 阶段简化**：vernal `Lifecycle::initialize / start / stop` 对齐 Spring `Lifecycle` 的精简版；tx_di 的 5 阶段收编到 `vernal-context`
7. **公开类型必须有中文 rustdoc 注释**：结构体、枚举、trait、方法、字段
8. **`#[derive(ErrorCode)]` 必须留给 `vernal-macros`**（不污染 vernal-core 的同步性质）

### vernal-core 现有 → 目标

| 维度 | 现状 | 目标 |
|---|---|---|
| 子模块 | 7（convert / error / id / lifecycle_phase / ordered / time / failure） | **8+**（新增 `app_lifecycle_phase` 拆分；保留 7） |
| 文件数 | 14 | **~24** |
| 代码行数 | ~1100 | ~1800 |
| 单元测试 | 6 | **30+** |
| 错误域覆盖 | 10 | **12+**（新增 `MIGRATION / SECURITY / METRICS`） |
| 初始化排序常量 | 4 | **8+**（新增 `BEAN_FACTORY / EVENT_LISTENER / MESSAGE_SOURCE / TASK`） |
| 内置 Convertible | 5 | **7+**（新增 `PathBuf / OffsetDateTime/Duration`） |
| Rust 生态集成 | 0 | **4 个 feature-gated 模块**（`uuid / chrono / strum / serde`） |

---

## 二、进度总览

| 阶段 | 内容 | 状态 | 验收 |
|---|---|---|---|
| **S0** | 4 份迁移文档（对象对照 / 语义对照 / 路线图 / 一致性检查） | ✅ | 本文档 + 配套 3 份 |
| **S1** | 命名修复：`LifecyclePhase` 重命名为 `AppLifecyclePhase` | ⬜ | 不破坏 `vernal-context` 引用；通过 `cargo build -p vernal-core` 与 `-p vernal-context` 全测试 |
| **S2** | 错误体系补强：`From<io::Error>` + `From<tokio::task::JoinError>` + `is_infrastructure()` 测试 | ⬜ | 9 个 trait 测试 + 8 个 From 测试通过 |
| **S3** | 错误域扩展：增加 `MIGRATION / SECURITY / METRICS` 三个域 | ⬜ | 测试覆盖所有 13 域 |
| **S4** | Ordered 锚点扩展：增加 `INIT_SORT_BEAN_FACTORY / EVENT_LISTENER / MESSAGE_SOURCE / TASK` | ⬜ | 8 个常量关系测试 |
| **S5** | Convertible 扩展：`PathBuf / OffsetDateTime / Duration`（**feature-gated**） | ⬜ | 至少 3 个新转换器 round-trip 测试通过 |
| **S6** | StopWatch 强化：`short_summary()` / `pretty_print(unit)` | ⬜ | 与 Spring 6.1 输出对齐测试 |
| **S7** | ObjectId serde 支持（**feature-gated**） | ⬜ | `serde_json::to_string(&id)` 测试通过 |
| **S8** | Rust 生态 feature flag：`uuid` / `chrono` / `strum` | ⬜ | 默认 `cargo build` 零外部依赖通过；`--features uuid` 增加 UUID 生成器 |
| **S9** | 测试覆盖与文档收尾 | ⬜ | 30+ 单元测试；中文 rustdoc 覆盖率 100%；新增 4 份迁移文档全 ✅ |

---

## 三、阶段详细计划

### S1 — 命名修复：`LifecyclePhase` → `AppLifecyclePhase`（**预估 0.5 天**）

**问题**：`vernal-core::lifecycle_phase::LifecyclePhase`（8 变体）与 `vernal-context::lifecycle_phase::LifecyclePhase`（3 变体）重名，造成语义混淆。

**方案**：重命名 vernal-core 的版本为 `AppLifecyclePhase`。

**目标文件**：
- 移动：`vernal-core/src/lifecycle_phase.rs` → `vernal-core/src/app_lifecycle_phase.rs`
- 修改 `vernal-core/src/lib.rs`：`mod lifecycle_phase;` → `mod app_lifecycle_phase; pub use app_lifecycle_phase::AppLifecyclePhase;`
- 修改 `vernal-context`：`use vernal_core::lifecycle_phase::LifecyclePhase` → `use vernal_core::AppLifecyclePhase`（按上下文判断）

**验收**：
- `cargo build -p vernal-core` 通过
- `cargo build -p vernal-context` 通过（可能需要同步修改）
- 全测试通过

---

### S2 — 错误体系补强（**预估 1 天**）

**目标文件**：`vernal-core/src/error/vernal_error.rs`（修改）

**新增 trait 实现**：
```rust
impl From<std::io::Error> for VernalError { /* Infrastructure */ }
impl From<tokio::task::JoinError> for VernalError { /* Infrastructure */ }
impl From<BoxError> for VernalError { /* Infrastructure(Arc::from(...)) */ }
impl From<SharedError> for VernalError { /* Infrastructure(arc) */ }
```

**新增单元测试**：
- `VernalError::Business { domain: "ioc", code: -1, message: "NotFound" }.is_infrastructure()` → false
- `VernalError::Infrastructure(arc) == VernalError::Infrastructure(arc)` → true（Arc ptr_eq）
- 跨变体比较：`Business{} ≠ WithContext{}`
- `is_same_kind`: `Business{d:"ioc", c:-1} == WithContext{d:"ioc", c:-1, ..}` → true
- 8 个 From 测试：覆盖 `BoxError`、`SharedError`、`io::Error`、`JoinError`
- `Display` 测试：`Business{...}` → `"[ioc:-1] NotFound"`

**验收**：
- 17+ 测试通过
- `#[derive(thiserror::Error)]` 的业务错误可 `?` 直接转 `VernalError::Infrastructure`

---

### S3 — 错误域扩展（**预估 0.25 天**）

**目标文件**：`vernal-core/src/error/error_domain.rs`（修改）

**新增常量**：
```rust
pub const MIGRATION: &'static str = "migration";   // Spring `org.springframework.jdbc.datasource.init`
pub const SECURITY: &'static str = "security";       // Spring `org.springframework.security`
pub const METRICS: &'static str = "metrics";         // Spring `org.springframework.boot.actuate.metrics`
```

**验收**：
- 13 个域常量集合测试通过
- 文档注释每个域说明对应 Spring 子系统

---

### S4 — Ordered 锚点扩展（**预估 0.25 天**）

**目标文件**：`vernal-core/src/ordered.rs`（修改）

**新增常量**：
```rust
pub const INIT_SORT_BEAN_FACTORY: i32 = i32::MIN + 2;       // 早于所有业务，先把 Bean 工厂建好
pub const INIT_SORT_EVENT_LISTENER: i32 = -2_000_000_000;   // 业务开始前，事件监听器先注册
pub const INIT_SORT_MESSAGE_SOURCE: i32 = -1_000_000_000;   // 消息源次之
pub const INIT_SORT_TASK: i32 = i32::MAX - 100;             // 异步任务最后启动（比 Web 还晚）
```

**验证**：8 个常量之间的偏序关系不变：
```
INFRASTRUCTURE < BEAN_FACTORY < EVENT_LISTENER < MESSAGE_SOURCE < BUSINESS < APPLICATION < TASK < DEFAULT
```

**验收**：
- 单元测试覆盖 8 个常量之间的 `<` / `>` 关系
- 文档注释每个常量对应 Spring 哪个组件

---

### S5 — Convertible 扩展（**预估 1 天**）

**目标文件**：新增 2 个 .rs

**新增**：
- `convert/path_converter.rs`：给 `PathBuf` 实现 `Convertible`
- `convert/datetime_converter.rs`：给 `time::OffsetDateTime` 和 `std::time::Duration` 实现 `Convertible`（**feature = "time"**）

**修改**：`convert/mod.rs`
- 暴露 `PathConverter` / `DatetimeConverter` 结构体
- 与现有 5 个内置实现并列

**验收**：
- `PathBuf::from_str_value("/tmp/foo.txt")` 正确返回 `PathBuf`
- `OffsetDateTime::from_str_value("2026-01-01T00:00:00Z")` round-trip 测试通过
- `Duration::from_str_value("30s")` round-trip 测试通过（需要 trait 字典解析，简化为秒）

---

### S6 — StopWatch 强化（**预估 0.5 天**）

**目标文件**：`vernal-core/src/time/stop_watch.rs`（修改）

**新增方法**：
```rust
pub fn short_summary(&self) -> String;                                  // 类似 Spring `shortSummary()`
pub fn pretty_print_with_unit(&self, unit: StopWatchUnit) -> String;    // 类似 Spring `prettyPrint(TimeUnit)`
```

**新增枚举**：`StopWatchUnit { Nano, Micro, Milli, Second, Minute }`

**验收**：
- `short_summary()` 输出：`"StopWatch 'op-name': 0.123 seconds (1 tasks)"`
- `pretty_print_with_unit(StopWatchUnit::Nano)` 输出含 ns 精度

---

### S7 — ObjectId serde 支持（**预估 0.25 天**）

**目标文件**：新增 `id/serde_object_id.rs`（**feature = "serde"**）

**新增**：
```rust
#[cfg(feature = "serde")]
impl serde::Serialize for ObjectId { ... }

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ObjectId { ... }
```

**验收**：
- `serde_json::to_string(&ObjectId::new())` → `"\"" + 24-hex + "\""`
- `serde_json::from_str::<ObjectId>(s)` round-trip 一致

---

### S8 — Rust 生态 feature flag 集成（**预估 1 天**）

**目标文件**：`vernal-core/Cargo.toml`（修改）+ 新增子模块 + `lib.rs`

**新增 feature flags**（经过 crates.io 实际调研）：
```toml
[dependencies]
linkme = { workspace = true }                # 已有
thiserror = { version = "2.0", optional = true }
web-time = { version = "1.1", optional = true, default-features = false }
convert_case = { version = "0.11", optional = true }
uuid = { version = "1.24", optional = true, features = ["v7"] }
ulid = { version = "2.0", optional = true }
nanoid = { version = "0.5", optional = true }
snowflake = { version = "1.3", optional = true }
serde = { version = "1", optional = true }
chrono = { version = "0.4", optional = true, default-features = false, features = ["clock", "serde"] }

[features]
default = []
uuid       = ["dep:uuid"]
ulid       = ["dep:ulid"]
nanoid     = ["dep:nanoid"]
snowflake  = ["dep:snowflake"]
serde      = ["dep:serde"]
chrono     = ["dep:chrono"]
case       = ["dep:convert_case"]
web-time   = ["dep:web-time"]
derive-all = ["dep:thiserror"]
```

**新增子模块**：
- `id/uuid_id.rs`（feature = `"uuid"`）：生成 UUID v7，时间排序与 ObjectId 一致
- `id/ulid_id.rs`（feature = `"ulid"`）：ULID
- `id/nanoid_id.rs`（feature = `"nanoid"`）：short URL-friendly ID
- `id/snowflake_id.rs`（feature = `"snowflake"`）：Twitter-style 64-bit
- `id/serde_object_id.rs`（feature = `"serde"`）：ObjectId 的 Serde 支持
- `time/datetime.rs`（feature = `"chrono"`）：基于 chrono 0.4 的 `OffsetDateTime` Convertible 实现
- `convert/case_converter.rs`（feature = `"case"`）：基于 convert_case 的 enum 大小写无关转换

**不引入的依赖**（明确排除）：
- ❌ `anyhow`（vernal-core 自身不引；`vernal-bridge` 提供 `From<anyhow>` 互转）
- ❌ `priority-queue`（LGPL/MPL copyleft，污染 Vernal 的 MIT）
- ❌ `instant`（已停止维护，改用 `web-time`）
- ❌ `snowflake-rs` 0.1.1（自 2018 年起废弃）
- ❌ `async-trait`（Rust 1.75+ 原生支持 native `async fn in trait`，vernal-core 不持有 `dyn` trait）
- ❌ `tracing`（归 `vernal-log`，vernal-core 自身不打印日志）

**验收**：
- `cargo build -p vernal-core` 默认零外部依赖通过
- `cargo build -p vernal-core --features "uuid,serde,chrono"` 增加 3 个 crate
- 所有 feature 独立可启用、可组合使用（`#[cfg(feature = "...")]` 严格 gating）
- 公开文档注明每个 feature 的使用场景与代价

---

### S9 — 测试覆盖与文档收尾（**预估 1 天**）

**目标**：30+ 单元测试，中文文档覆盖率 100%

**单元测试清单**：
- error/ 模块：12 个测试（4 个变体 × Display/Debug/PartialEq/From）
- convert/ 模块：8 个测试（5 内置 × round-trip + 3 PathBuf/Datetime/Duration）
- id/ 模块：3 个测试（生成唯一性 / len / Display）
- time/ 模块：3 个测试（pretty_print / pretty_print_with_unit / short_summary）
- lifecycle_phase.rs（旧）→ app_lifecycle_phase.rs（新）：3 个测试（is_active / is_terminal / is_transitioning）
- ordered.rs：3 个测试（8 常量偏序）
- lib.rs：1 个测试（FRAMEWORK_VERSION 不为空）

合计：**33 个单元测试**

**文档清单**：
- 给每个公开类型增加中文 `///` rustdoc
- 给每个 trait 方法增加中文示例
- 模块级 `//!` 文档说明与 Spring / tx_di 对应关系

**验收**：
- `cargo test -p vernal-core` 全 33 测试通过
- `cargo doc -p vernal-core --no-deps` 无警告
- 中文 rustdoc 覆盖率 100%

---

## 四、阶段依赖图

```
S0 ✅ → S1 → { S2 + S3 + S4 + S5 + S6 + S7 + S8 } → S9
          │
          └── S1 必须先完成（重命名生命周期阶段）
              │
              └── S2-S8 可以并行开发
                  │
                  └── S9 必须在所有前序阶段完成后进行
```

---

## 五、风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| S1 重命名破坏下游 crate | `vernal-context` 等下游无法编译 | 同步修改 `vernal-context` 中的 `use vernal_core::lifecycle_phase::LifecyclePhase` → `use vernal_core::AppLifecyclePhase`；一次性提交 |
| S2 新增 `From<JoinError>` 反向依赖 tokio | 破坏 vernal-core 零依赖原则 | **改用条件 cfg**：`#[cfg(feature = "tokio")] impl From<tokio::task::JoinError> for VernalError { ... }` |
| S5 `OffsetDateTime` 引入 `time` crate 反向依赖 | 破坏零依赖 | 仅在 `feature = "time"` 启用 |
| S8 `uuid` feature 与现有 `ObjectId` 概念重叠 | 用户不知道用哪个 | 文档明确区分：`ObjectId` 任务追踪（内部）、`uuid` 全局唯一（外部接口） |
| Spring 6.1 StopWatch 行为变更（`prettyPrint(TimeUnit)`） | 输出格式变化 | 严格对齐 Spring 6.1 输出格式，提供 `StopWatchUnit` 参数控制 |

---

## 六、验收矩阵

| Spring / tx_di 概念 | 阶段 | 测试期望 | 实际 |
|---|---|---|---|
| `ConversionService.convert()` | 已完成 | 5 内置类型通过 | ✅ |
| `Ordered.getOrder()` | S4 | 8 常量偏序关系通过 | ⬜ |
| `Lifecycle.Phase` | S1 | `AppLifecyclePhase` 8 变体映射 Spring 状态 | ⬜ |
| `StopWatch.prettyPrint()` | 已完成 | 输出格式与 Spring 6.0 一致 | ✅ |
| `StopWatch.prettyPrint(TimeUnit)` | S6 | 5 时间单位输出 | ⬜ |
| `AppError::ErrCode` | 已完成 | `domain/code/message` 归一化 | ✅ |
| `AppError::WithContext` | 已完成 | `context()` 单段字符串 | ✅ |
| `AppError::Internal(anyhow)` | S2 | 改为 `Infrastructure(SharedError)`，**不引入 anyhow** | ⬜ |
| `CodeMsg` trait + 派生宏 | 已完成 / 委托给 `vernal-macros` | `#[derive(ErrorCode)]` 落库 | ⬜ |
| `BoxError / SharedError` | 已完成 | 别名稳定 | ✅ |
| `Convertible` 5 内置 + 3 新增 | S5 | `PathBuf / OffsetDateTime / Duration` | ⬜ |

---

## 七、进度跟踪（完成时填写）

| 日期 | 阶段 | 完成 | 累计 |
|---|---|---|---|
| 2026-07-27 | S0 ✅ | ✅ | S0 |
| 待定 | S1 | ⬜ | S0-S1 |
| 待定 | S2-S8 | ⬜ | S0-S8 |
| 待定 | S9 | ⬜ | S0-S9（迁移完成） |

---

## 八、与 tx_di 迁移的协同

tx_di 的迁移在 `tx_admin` 示例项目（`/Users/wandl/workspaces/workspace-github-easy-4-rust/tx_di/examples/tx_admin`）中独立进行，不影响 vernal-core。
但 **vernal-core 本身就是 tx_di 的简化版**：vernal-core 的 `VernalError` 抽象正是从 `tx_error::AppError` 抽象而来，因此 vernal-core 与 tx_di 共享同一套设计语言，迁移完成后可以直接在两边互通。

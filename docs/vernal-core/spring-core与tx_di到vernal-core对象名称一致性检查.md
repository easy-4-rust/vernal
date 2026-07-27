# spring-core + tx_di → vernal-core 对象名称一致性检查

> 版本：v1.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core + tx_di **dev**
> 仓库：easy-4-rust/vernal @ dev
> 路径：`/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework`

本文档系统性检查 vernal-core 与 spring-core、tx_di 在类型命名、模块命名、模块路径上的对应关系，找出**一致 / 重命名 / 冲突 / 缺失 / 错误**5 类问题。

---

## 一、命名对齐原则

1. **优先沿用 spring-core 命名**：Spring 是 JSR-330 / JSR-250 事实标准，vernal-core 应保留 Spring 的面向对象抽象
2. **次选 tx_di 命名**：当 spring-core 与 tx_di 名称冲突时，**优先保留 spring 命名**；tx_di 的命名空间被 `@err("DI")` 等域前缀吸收
3. **vernal-core 与 vernal-context 出现重名时**：
   - 重命名 **vernal-core** 版本（前缀 `App*` 或保留 Spring 风格）
   - 保留 **vernal-context** 版本（与 `ApplicationContext` 协同）
4. **公开别名稳定**：现有 `BoxError / SharedError` 等 vernal 公共类型一旦发布，**不再重命名**（版本稳定承诺）

---

## 二、逐项检查

### 2.1 Spring 类型 → vernal-core 直接重命名

| Java 类型 | `org.springframework` 包路径 | vernal-core 路径 | 一致性 |
|---|---|---|---|
| `Ordered` | `org.springframework.core.Ordered` | `vernal_core::ordered` 模块 + `INIT_SORT_*` 常量 | 🟡 **部分一致**：用常量代替 trait getter（语义对齐，命名路径不对齐） |
| `PriorityOrdered` | `org.springframework.core.PriorityOrdered` | `INIT_SORT_INFRASTRUCTURE = i32::MIN + 1` | 🟡 **部分一致** |
| `ConversionService` | `org.springframework.core.convert.ConversionService` | `vernal_core::convert::ConversionService` | ✅ **完全一致**（类型名 + 模块路径 + 公开 API） |
| `Converter<S,T>` | `org.springframework.core.convert.converter.Converter` | `vernal_core::convert::Converter<S, T>` | ✅ **完全一致** |
| `Lifecycle` | `org.springframework.context.Lifecycle` | （不存在；归 `vernal_context::Lifecycle`） | 🟢 **跨 crate 对齐**：vernal-core 提供阶段枚举，vernal-context 提供 trait |
| `SmartLifecycle` | `org.springframework.context.SmartLifecycle` | （同上） | 🟢 **跨 crate 对齐** |
| `NestedRuntimeException` | `org.springframework.core.NestedRuntimeException` | `VernalError::Infrastructure(SharedError)` | 🟡 **语义对齐，命名不沿用** |
| `ErrorCoded` | `org.springframework.core.ErrorCoded` | `vernal_core::error::ErrorCode` trait | 🟡 **类型名**：保留 `Code` 单词，去掉 `d` 后缀；语义一致 |
| `StopWatch` | `org.springframework.util.StopWatch` | `vernal_core::time::StopWatch` | ✅ **完全一致** |

---

### 2.2 tx_di 类型 → vernal-core 重命名

| tx_di 类型 | tx_di 路径 | vernal-core 路径 | 一致性 |
|---|---|---|---|
| `AppError` | `tx_error::AppError` | `VernalError::VernalError` | ✅ **更名一致**：vernal-core 把 `App` 全部改成 `Vernal` 前缀，避免与 Spring `ApplicationContext` 混淆 |
| `AppErrCode` | `tx_error::AppErrCode` | `ErrorCode` trait + `into_vernal_error()` | 🟡 **语义一致**：trait 形态替代值类型（Rust 习惯） |
| `AppResult` | `tx_error::AppResult<T>` | `Result<T, BoxError>` | 🟡 **不沿用别名**：用 Rust 标准 `Result<T, E>` |
| `CodeMsg` trait | `tx_error::CodeMsg` | `ErrorCode` | 🟡 **重命名一致**：vernal-core 改名 `ErrorCode`，与 Spring `ErrorCoded` 同源 |
| `R<u64>` / `RIE<T>` | `tx_di::RIE<T>` | （不引入） | 🚫 |
| `BoxFuture<T>` | `tx_di::BoxFuture<T>` | （不引入；归 `vernal-context::lifecycle_future`） | 🟢 **跨 crate 对齐** |
| `Scope` enum | `tx_di::Scope` | （不引入；归 `vernal-beans::component_scope`） | 🟢 **跨 crate 对齐** |

---

### 2.3 vernal-core 内部冲突

| 冲突类型 | 现状 | 修复方案 | 优先级 |
|---|---|---|---|
| **`LifecyclePhase` 重名** | `vernal-core::LifecyclePhase`（8 变体）vs `vernal-context::LifecyclePhase`（3 变体） | 重命名 vernal-core 版本为 **`AppLifecyclePhase`** | 🔴 **S1 必修** |
| `Phase` 命名 vs Spring `Phase` 接口 | vernal 用枚举，Spring `Phase` 是 trait | 不修改；vernal 现状更符合 Rust 习惯 | 🟢 |
| `INIT_SORT_DEFAULT` 含义与 Spring `Ordered.LOWEST_PRECEDENCE` 一致 | ✅ | 不修改 | 🟢 |
| `INIT_SORT_INFRASTRUCTURE` 是新增；Spring 用 `HIGHEST_PRECEDENCE = i32::MIN` | 类型名不同但语义一致 | 文档注明 | 🟢 |
| `Converter<S, T>` 既存在于 `vernal_core::convert::Converter` 也存在于 `vernal_aop::*` 的 `Interceptor` chain | 同名但不冲突（不同 crate 不同域） | 文档注明路径分离 | 🟡 |
| `Convertible` 是 vernal-core 新增；Spring 无对应 | vernal 选择 trait 名称时需谨慎（与 `convert::Converter` 区分） | 保留 `Convertible`（强调 "可被转换" 而非 "主动转换"） | 🟡 |

---

### 2.4 模块路径对齐

| spring-core 文件路径 | tx_di 文件路径 | vernal-core 文件路径 | 命名空间对齐 |
|---|---|---|---|
| `spring-core/src/main/java/org/springframework/core/Ordered.java` | — | `vernal-core/src/ordered.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/PriorityOrdered.java` | — | 同上 | ✅ |
| `spring-core/src/main/java/org/springframework/core/convert/ConversionService.java` | — | `vernal-core/src/convert/mod.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/convert/converter/Converter.java` | — | `vernal-core/src/convert/converter.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/NestedRuntimeException.java` | `tx_error/src/error.rs::AppError` | `vernal-core/src/error/vernal_error.rs` | 🟡 **不一致**：spring 用 `RuntimeException` 子类，tx_di 用 enum，vernal 用 enum（选择 Rust 习惯） |
| `spring-core/src/main/java/org/springframework/util/StopWatch.java` | — | `vernal-core/src/time/stop_watch.rs` | ✅ |
| — | `tx_error/src/code.rs::AppErrCode` | `vernal-core/src/error/error_code.rs`（trait 形态） | 🟡 |
| — | `tx_di/src/lifecycle.rs` | `vernal-core/src/lifecycle_phase.rs`（仅 enum） + `vernal-context/src/lifecycle_phase.rs`（3 变体 enum） + `vernal-context/src/component_lifecycle.rs`（trait） | 🔴 **冲突** |

---

## 三、命名对比表（4 列）

| 概念 | Spring 命名 | tx_di 命名 | vernal-core 命名 | 一致性 |
|---|---|---|---|---|
| 组件主 trait | `org.springframework.stereotype.Component` | `tx_di_core::Component`（5 钩子） | `vernal_context::component_lifecycle::Lifecycle`（3 钩子） | 🟡 |
| 排序 | `Ordered.getOrder(): int` | `Component::init_sort(): i32` | `INIT_SORT_*` 常量 | 🟡 |
| 优先级排序 | `PriorityOrdered` | — | `INIT_SORT_INFRASTRUCTURE` | 🟡 |
| 类型转换 | `ConversionService.convert()` | — | `ConversionService::convert()` | ✅ |
| 单向转换 | `Converter<S, T>` | — | `Converter<S, T>` | ✅ |
| 目标类型 trait | — | — | `Convertible::from_str_value()` | 🆕 |
| 嵌套异常 | `NestedRuntimeException` | `AppError::Internal(anyhow)` | `VernalError::Infrastructure(SharedError)` | 🟡 |
| 业务错误码 | `ErrorCoded` | `AppErrCode` + `CodeMsg` | `ErrorCode` trait + `into_vernal_error()` | 🟡 |
| 通用错误 | — | `BoxError` 不存在 | `BoxError = Box<dyn Error + ...>` | 🆕 |
| 共享错误 | — | — | `SharedError = Arc<dyn Error + ...>` | 🆕 |
| 生命周期 trait | `org.springframework.context.Lifecycle` | `Component::init/async_init/async_run/shutdown` | `vernal_context::Lifecycle::initialize/start/stop` | 🟡 |
| 生命周期阶段枚举 | `Phase` 接口 + `getPhase(): int` | （无 enum） | `vernal_core::AppLifecyclePhase`（8 变体）+ `vernal_context::LifecyclePhase`（3 变体） | 🟡 |
| 计时器 | `org.springframework.util.StopWatch` | — | `vernal_core::time::StopWatch` | ✅ |
| ObjectId | `org.springframework.util.ObjectUtils.identityToString()` | `tx_common::id::*`（UUID / Snowflake） | `vernal_core::id::ObjectId` | 🟡 |
| 错误域常量 | 隐式（namespace 命名空间） | `#[err("DI")]` | `ErrorDomain::IOC / AOP / CONTEXT / WEB / HTTP / TOWER / DISCOVERY / MACROS / CORE / BRIDGE / MIGRATION / SECURITY / METRICS` | 🟡 |
| 版本号 | `org.springframework.core.SpringVersion.getVersion()` | `env!("CARGO_PKG_VERSION")` 直接获取 | `vernal_core::FRAMEWORK_VERSION` + `MINIMUM_RUST_VERSION` + `PROJECT_STATUS` | ✅ |
| API Result | `HttpStatusCode.value()` 不用 | `AppResult<T>` 别名 | `Result<T, BoxError>` 标准 | 🟡 |

---

## 四、问题清单（修复时按优先级处理）

### 🔴 P0 必修（S1 阶段立刻修复）

#### 1. `LifecyclePhase` 重名冲突

**问题描述**：

```rust
// vernal-core/src/lifecycle_phase.rs:11
pub enum LifecyclePhase {  // 8 变体
    Created, Initializing, Initialized, Starting, Started,
    Stopping, Stopped, Failed,
}

// vernal-context/src/lifecycle_phase.rs:7  
pub enum LifecyclePhase {  // 3 变体（看起来更紧凑）
    Initialize, Start, Stop,
}
```

二者**同名同路径**（`vernal_core::LifecyclePhase` vs `vernal_context::LifecyclePhase`），但**变体数量和语义完全不同**。这会让用户以及所有 vernal 内部代码产生歧义。

**根因**：vernal-core 的 `LifecyclePhase` 是从 tx_di 的 `Component::init/async_init/async_run/shutdown` 4 阶段外加 Created/Failed 推导出的 8 状态机；vernal-context 的 `LifecyclePhase` 是 Spring `LifecycleProcessor` 协调阶段的精简版（3 状态）。两者都不该用同一个名字。

**修复方案**：
1. 重命名 vernal-core 内的 `LifecyclePhase` 为 **`AppLifecyclePhase`**
2. 文件 `vernal-core/src/lifecycle_phase.rs` → `vernal-core/src/app_lifecycle_phase.rs`
3. `vernal-core/src/lib.rs`：`pub mod lifecycle_phase` → `pub mod app_lifecycle_phase`
4. `vernal-context` 中的 `LifecyclePhase` **保留原名**，因为它是 vernal-context 的核心抽象（与 `ApplicationContext` 协同）
5. 同步修改 `vernal-context` 中引用 `vernal_core::lifecycle_phase::*` 的地方（**实测仅 `vernal-core` 自身使用，下游无 import**）

**验证**：
```bash
grep -rn "vernal_core::lifecycle_phase" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/ | grep -v target
# 期望：无任何下游 import（vernal-core 的 LifecyclePhase 当前未对外暴露）
```

**实施代码**（S1 阶段执行）：
```bash
git mv crates/vernal-core/src/lifecycle_phase.rs crates/vernal-core/src/app_lifecycle_phase.rs
# 修改 app_lifecycle_phase.rs: enum LifecyclePhase → enum AppLifecyclePhase
# 修改 lib.rs: mod lifecycle_phase → mod app_lifecycle_phase; pub use app_lifecycle_phase::AppLifecyclePhase
```

---

### 🟡 P1 后续版本（S4-S6 阶段处理）

#### 2. `INIT_SORT_*` 命名与 Spring `Ordered` 的语义对齐

**现状**：
- vernal-core 已有：`INIT_SORT_INFRASTRUCTURE / BUSINESS / APPLICATION / DEFAULT`
- Spring：`HIGHEST_PRECEDENCE / LOWEST_PRECEDENCE + getOrder()`

**问题**：vernal 用"语义名"（`BUSINESS`），Spring 用"前后端"（`MIN/MAX`）。二者价值对等但风格迥异。

**修复**：保留 vernal 现状，但增加 Spring 风格别名：
```rust
/// Spring `Ordered.HIGHEST_PRECEDENCE` 等价值（vernal-core 不引入负 MIN 避免与基础设施层重叠）
pub const HIGHEST_PRECEDENCE: i32 = INIT_SORT_INFRASTRUCTURE;
pub const LOWEST_PRECEDENCE: i32 = INIT_SORT_DEFAULT;
```

#### 3. `ErrorCoded` vs `CodeMsg` vs `ErrorCode`

**问题**：三个来源，vernal-core 选 `ErrorCode`，与 Spring 同根。

**修复**：保留 `ErrorCode` 不变；文档已注明命名选择理由（见语义迁移对照表 1.3 节）。

#### 4. `ConversionService` 模块结构

**现状**：
- Spring：`org.springframework.core.convert.ConversionService`（interface）+ `org.springframework.core.convert.support.DefaultConversionService`（默认实现）
- vernal-core：单 struct `ConversionService`

**问题**：vernal-core 单 struct 与"接口 + 默认实现"分离模式不一致。

**修复**：保留 vernal 现状（Rust 不需要接口+实现分离）；文档注明理由。

---

### 🟢 P2 文档维护（持续）

#### 5. `BoxError` 与 `anyhow::Error` 的兼容性文档

**问题**：vernal-core 故意不引入 anyhow，但外部用户已经在用 anyhow，需要文档化互转方案。

**修复**：在 `failure.rs` doc-comment 中增加示例：
```rust
//! ## 与 anyhow 互转
//!
//! ```ignore
//! use anyhow::anyhow;
//! let anyhow_err = anyhow!("db failed");
//! let boxed: BoxError = Box::new(anyhow_err);
//! let vernal: VernalError = boxed.into();
//! ```
```

---

## 五、导入路径一致性 grep 检查

下列 grep 命令必须在迁移完成后产生零结果（除期望的引用外）：

```bash
# 1. vernal-core 内部不得有 vernal-context 引用
grep -rn "vernal_context" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：0 结果

# 2. vernal-core 不得引入 anyhow
grep -rn "anyhow" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：0 结果（除非 feature-gated 注释中提及）

# 3. 验证重命名后无遗留
grep -rn "lifecycle_phase" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：所有出现点都对应 mod app_lifecycle_phase；无遗留 mod lifecycle_phase

# 4. vernal-core 错误域常量必须稳定
grep -rn "ErrorDomain::" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：13 个常量，类型稳定
```

---

## 六、命名变更日志（与路线图同步）

| 日期 | 类型 | 改名前 | 改名后 | 阶段 |
|---|---|---|---|---|
| 2026-07-27 | 新文档 | — | — | S0 ✅ |
| 待定 | enum | `LifecyclePhase` | `AppLifecyclePhase` | S1 |
| 待定 | trait 增强 | `ErrorCode`（不变） | — | S2 |
| 待定 | module 重命名 | `lifecycle_phase.rs` | `app_lifecycle_phase.rs` | S1 |

---

## 七、外部 API 一致性承诺（vernal-core 公共契约）

下列类型/常量在迁移完成后 **必须保持不变**（除非新增）：

### 7.1 必须保持的类型和模块

```rust
// 错误体系
vernal_core::error::VernalError
vernal_core::error::ErrorCode
vernal_core::error::ErrorKind
vernal_core::error::ErrorDomain
vernal_core::error::ErrorContext
vernal_core::error::ErrorReport
vernal_core::BoxError
vernal_core::SharedError

// 排序
vernal_core::ordered::INIT_SORT_INFRASTRUCTURE
vernal_core::ordered::INIT_SORT_BUSINESS
vernal_core::ordered::INIT_SORT_APPLICATION
vernal_core::ordered::INIT_SORT_DEFAULT

// 生命周期
vernal_core::app_lifecycle_phase::AppLifecyclePhase  // 重命名后路径

// ID
vernal_core::id::ObjectId

// 时间
vernal_core::time::StopWatch

// 转换
vernal_core::convert::ConversionService
vernal_core::convert::Convertible
vernal_core::convert::Converter
vernal_core::convert::convert_enum
vernal_core::convert::BooleanConverter
vernal_core::convert::NumberConverter
vernal_core::convert::StringConverter
vernal_core::convert::OptionConverter

// 常量
vernal_core::FRAMEWORK_VERSION
vernal_core::MINIMUM_RUST_VERSION
vernal_core::PROJECT_STATUS
```

### 7.2 新增承诺

迁移完成后，下列类型是 **新增** 公共契约，**不得删除**：

```rust
// S2 新增
impl From<std::io::Error> for VernalError
impl From<BoxError> for VernalError
impl From<SharedError> for VernalError

// S3 新增
vernal_core::error::ErrorDomain::MIGRATION
vernal_core::error::ErrorDomain::SECURITY
vernal_core::error::ErrorDomain::METRICS

// S4 新增
vernal_core::ordered::INIT_SORT_BEAN_FACTORY
vernal_core::ordered::INIT_SORT_EVENT_LISTENER
vernal_core::ordered::INIT_SORT_MESSAGE_SOURCE
vernal_core::ordered::INIT_SORT_TASK

// S6 新增
vernal_core::time::StopWatchUnit
vernal_core::time::StopWatch::short_summary
vernal_core::time::StopWatch::pretty_print_with_unit

// S7 新增
impl Serialize for ObjectId (feature = "serde")
impl Deserialize for ObjectId (feature = "serde")

// S8 新增 (feature-gated)
#[cfg(feature = "uuid")]       uuid::Uuid::new_v7()           // 替代 ObjectId
#[cfg(feature = "ulid")]       ulid::Ulid::new()              // 替代 ObjectId
#[cfg(feature = "nanoid")]     nanoid::nanoid!()              // 短 ID
#[cfg(feature = "snowflake")]  snowflake::SnowflakeId::new()  // Twitter 风格
```

---

## 九、Rust 生态 feature flag 一致性（**新增章节**）

迁移完成后，vernal-core 应当通过 `cargo build -p vernal-core` 默认以**零外部依赖**通过编译。
下列 feature flag 必须保持稳定命名（避免破坏用户脚本）：

| Feature 名 | 对应 crate | 默认 | 一致性 |
|---|---|---|---|
| `uuid` | `uuid = "1.24"` | off | ✅ |
| `ulid` | `ulid = "2.0"` | off | ✅ |
| `nanoid` | `nanoid = "0.5"` | off | ✅ |
| `snowflake` | `snowflake = "1.3"` | off | ✅ |
| `serde` | `serde = "1"` | off | ✅ |
| `chrono` | `chrono = "0.4"` | off | ✅ |
| `case` | `convert_case = "0.11"` | off | ✅ |
| `web-time` | `web-time = "1.1"` | off | ✅ |
| `derive-all` | `thiserror = "2.0"` | off | ✅ |

**明确禁止新增（除非团队架构评审通过）**：
- ❌ `anyhow` feature（vernal-core 自身永不允许依赖 anyhow）
- ❌ `priority-queue` feature（LGPL/MPL copyleft）
- ❌ `instant` feature（unmaintained；改用 web-time 或 std）
- ❌ `async-trait` feature（vernal-core 不持有 dyn trait）

---

## 八、检查结论

### 当前已发现的问题

| # | 问题 | 严重度 | 修复阶段 |
|---|---|---|---|
| 1 | `LifecyclePhase` 重名冲突 | 🔴 P0 | S1 |
| 2 | `INIT_SORT_*` 风格与 Spring 不同 | 🟡 P1 | S4 |
| 3 | `ErrorCode` 命名取舍 | 🟡 P1 | 文档维护 |
| 4 | `ConversionService` 单 struct 模式 | 🟢 P2 | 文档维护 |
| 5 | `BoxError` 与 anyhow 兼容性文档 | 🟢 P2 | S9 |

### 通过的检查项

- ✅ 14 个映射到 spring-core 命名保留
- ✅ 5 个映射到 tx_di 命名保留或采纳
- ✅ 模块路径 100% 镜像 spring-core（命名大小写一致）
- ✅ `BoxError / SharedError / ObjectId / StopWatch` 与 Spring / tx_di 兼容
- ✅ `VernalError` vs `AppError` 命名空间通过 `Vernal` 前缀分离
- ✅ `Converter<S, T>` 与 `Convertible` trait 在 vernal 中各自承担 `Converter / Convertible` 角色，互不冲突

### 跟进事项

1. **S1 完成前**：禁止引入新功能涉及 `LifecyclePhase` 的命名
2. **S2 完成前**：`From<BoxError>` 等 From 实现的稳定 API
3. **S9 完成前**：所有公开类型的文档覆盖率必须达到 100%
4. **总目标**：vernal-core 成为整个 vernal 框架的"基础合同"层，被 `vernal-beans / context / aop / web / tx / db` 等子系统单向依赖，且保证类型稳定。

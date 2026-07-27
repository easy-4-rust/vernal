# spring-core + tx_di → vernal-core 对象名称一致性检查 v2.0

> 版本：v2.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core（737 个 .java / 51 个顶级包）+ tx_di **dev**（414 个 .rs / 73,479 行）
> 仓库：easy-4-rust/vernal @ dev
> 路径：`/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework`
>
> **配套三份文档**：
> - 《vernal-core 迁移路线图 v2.0》
> - 《vernal-core 对象级对照表 v2.0》
> - 《vernal-core 语义迁移对照表 v2.0》

本文档系统性检查 vernal-core 与 spring-core、tx_di 在**类型命名、模块命名、模块路径**上的对应关系，找出**一致 / 重命名 / 冲突 / 缺失 / 错误** 5 类问题，并给出 P0/P1/P2 优先级排序。

---

## 一、命名对齐原则

### 1.1 优先级规则

1. **优先沿用 spring-core 命名**：Spring 是 JSR-330 / JSR-250 事实标准，vernal-core 应保留 Spring 的面向对象抽象
2. **次选 tx_di 命名**：当 spring-core 与 tx_di 名称冲突时，**优先保留 spring 命名**；tx_di 的命名空间被 `@err("DI")` 等域前缀吸收
3. **vernal-core 与 vernal-context 出现重名时**：
   - 重命名 **vernal-core** 版本（前缀 `App*` 或保留 Spring 风格）
   - 保留 **vernal-context** 版本（与 `ApplicationContext` 协同）
4. **公开别名稳定**：现有 `BoxError / SharedError / ConversionService / StopWatch` 等 vernal 公共类型一旦发布，**不再重命名**（版本稳定承诺）

### 1.2 类型命名规范

- **PascalCase**：类型名（Spring `ConversionService` → Rust `ConversionService`）
- **snake_case**：方法名（Spring `getOrder()` → Rust `get_order()` 或 `order()`；Spring `prettyPrint()` → Rust `pretty_print()`）
- **snake_case**：字段名（Spring `errorCode` → Rust `error_code`；带单位后缀如 `timeoutSecs`）
- **模块名 snake_case**：`convert` / `error` / `id` / `time` / `diagnostics`
- **常量名 UPPER_SNAKE_CASE**：`INIT_SORT_INFRASTRUCTURE` / `HIGHEST_PRECEDENCE` / `FRAMEWORK_VERSION`
- **类型枚举变体 PascalCase**：`LifecyclePhase::Created` / `LifecyclePhase::Started`

---

## 二、逐项检查

### 2.1 Spring 类型 → vernal-core 直接重命名

| Java 类型 | `org.springframework` 包路径 | vernal-core 路径 | 一致性 | 阶段 |
|---|---|---|---|---|
| `Ordered` | `org.springframework.core.Ordered` | `vernal_core::ordered` 模块 + `INIT_SORT_*` 常量 | 🟡 **部分一致**：用常量代替 trait getter（语义对齐，命名路径不对齐） | S4 |
| `PriorityOrdered` | `org.springframework.core.PriorityOrdered` | `INIT_SORT_INFRASTRUCTURE = i32::MIN + 1` | 🟡 **部分一致** | S4 |
| `ConversionService` | `org.springframework.core.convert.ConversionService` | `vernal_core::convert::ConversionService` | ✅ **完全一致**（类型名 + 模块路径 + 公开 API） | ✅ |
| `Converter<S,T>` | `org.springframework.core.convert.converter.Converter` | `vernal_core::convert::Converter<S, T>` | ✅ **完全一致** | ✅ |
| `Convertible` (vernal 自创) | — | `vernal_core::convert::Convertible` | 🆕 **新增** | ✅ |
| `Lifecycle` | `org.springframework.context.Lifecycle` | （不存在；归 `vernal_context::Lifecycle`） | 🟢 **跨 crate 对齐**：vernal-core 提供阶段枚举，vernal-context 提供 trait | ✅ |
| `SmartLifecycle` | `org.springframework.context.SmartLifecycle` | （同上） | 🟢 **跨 crate 对齐** | ✅ |
| `Lifecycle.Phase` | `org.springframework.context.Lifecycle.Phase` | `vernal_core::AppLifecyclePhase`（重命名后） | 🔴 **当前错用 `LifecyclePhase`** | S1 |
| `NestedRuntimeException` | `org.springframework.core.NestedRuntimeException` | `VernalError::Infrastructure(SharedError)` | 🟡 **语义对齐，命名不沿用** | ✅ |
| `NestedIOException` | `org.springframework.core.NestedIOException` | `From<io::Error> for VernalError` | 🟡 | ✅ |
| `ErrorCoded` | `org.springframework.core.ErrorCoded` | `vernal_core::error::ErrorCode` trait | 🟡 **类型名**：保留 `Code` 单词，去掉 `d` 后缀；语义一致 | ✅ |
| `StopWatch` | `org.springframework.util.StopWatch` | `vernal_core::time::StopWatch` | ✅ **完全一致** | ✅ |
| `SpringVersion` | `org.springframework.core.SpringVersion` | `vernal_core::FRAMEWORK_VERSION` + `MINIMUM_RUST_VERSION` + `PROJECT_STATUS` | 🟡 **名称不一致**（Spring 是 `Version` 后缀，vernal 用 `FRAMEWORK_VERSION` 全大写） | ✅ |
| `Observation` (6.1+) | `org.springframework Observation API` | `vernal_core::diagnostics::Span`（轻量版） | 🆕（待 S9） | ⬜ |
| `ObservationRegistry` (6.1+) | 同上 | （不引入，由 `vernal-context::startup_observation` 负责） | 🚫 | — |

### 2.2 tx_di 类型 → vernal-core 重命名

| tx_di 类型 | tx_di 路径 | vernal-core 路径 | 一致性 | 阶段 |
|---|---|---|---|---|
| `AppError` | `tx_error::AppError` | `VernalError::VernalError` | ✅ **更名一致**：vernal-core 把 `App` 全部改成 `Vernal` 前缀，避免与 Spring `ApplicationContext` 混淆 | ✅ |
| `AppErrCode` | `tx_error::AppErrCode` | `ErrorCode` trait + `into_vernal_error()` | 🟡 **语义一致**：trait 形态替代值类型（Rust 习惯） | ✅ |
| `AppResult` | `tx_error::AppResult<T>` | `Result<T, BoxError>` | 🟡 **不沿用别名**：用 Rust 标准 `Result<T, E>` | ✅ |
| `CodeMsg` trait | `tx_error::CodeMsg` | `ErrorCode` | 🟡 **重命名一致**：vernal-core 改名 `ErrorCode`，与 Spring `ErrorCoded` 同源 | ✅ |
| `BoxError` (Rust std) | `Box<dyn std::error::Error + Send + Sync>` | `vernal_core::BoxError` | ✅ **沿用类型别名** | ✅ |
| `SharedError` (vernal 自创) | — | `vernal_core::SharedError` | 🆕 **新增** | ✅ |
| `R<u64>` / `RIE<T>` | `tx_di::RIE<T>` | （不引入） | 🚫 | — |
| `BoxFuture<T>` | `tx_di::BoxFuture<T>` | （不引入；归 `vernal-context::lifecycle_future`） | 🟢 **跨 crate 对齐** | ✅ |
| `Scope` enum | `tx_di::Scope` | （不引入；归 `vernal-beans::component_scope`） | 🟢 **跨 crate 对齐** | ✅ |
| `Component` trait | `tx_di_core::Component` | （不引入；归 `vernal-beans::component_contract`） | 🟢 **跨 crate 对齐** | ✅ |
| `tx_common::id::*` | tx_common 各种 ID 类型 | `vernal_core::id::ObjectId` + 4 个 feature-gated 后端 | 🆕 **强化接管** | S7 |
| `tx_error::log_err` | `tx_error::log_err(e, err)` | （不引入；归 `vernal-log`） | 🟢 **跨 crate 对齐** | ✅ |
| `DiErr` enum | `tx_di_core::error::DiErr` | （不引入；归 `vernal-beans::definition_error`） | 🟢 **跨 crate 对齐** | ✅ |

### 2.3 vernal-core 内部冲突

| 冲突类型 | 现状 | 修复方案 | 优先级 |
|---|---|---|---|
| **`LifecyclePhase` 重名** | `vernal-core::LifecyclePhase`（8 变体）vs `vernal-context::LifecyclePhase`（3 变体） | 重命名 vernal-core 版本为 **`AppLifecyclePhase`** | 🔴 **S1 必修** |
| `Phase` 命名 vs Spring `Phase` 接口 | vernal 用枚举，Spring `Phase` 是 trait | 不修改；vernal 现状更符合 Rust 习惯 | 🟢 |
| `INIT_SORT_DEFAULT` 含义与 Spring `Ordered.LOWEST_PRECEDENCE` 一致 | ✅ | 不修改 | 🟢 |
| `INIT_SORT_INFRASTRUCTURE` 是新增；Spring 用 `HIGHEST_PRECEDENCE = i32::MIN` | 类型名不同但语义一致 | 文档注明；S4 增加 Spring 风格别名 | 🟡 |
| `Converter<S, T>` 既存在于 `vernal_core::convert::Converter` 也存在于 `vernal_aop::*` 的 `Interceptor` chain | 同名但不冲突（不同 crate 不同域） | 文档注明路径分离 | 🟡 |
| `Convertible` 是 vernal-core 新增；Spring 无对应 | vernal 选择 trait 名称时需谨慎（与 `convert::Converter` 区分） | 保留 `Convertible`（强调 "可被转换" 而非 "主动转换"） | 🟡 |
| `INIT_SORT_BUSINESS` 命名风格 vs Spring 用纯数字 | vernal 用语义名（`BUSINESS`），Spring 用前后端（`MIN/MAX`） | 保留 vernal 现状，但 S4 增加 Spring 别名 | 🟡 |
| `vernal-core` 与 `vernal-macros` 的 `ErrorCode` trait vs `thiserror::Error` | 都用于错误派生 | 明确边界：vernal-core 提供 trait，vernal-macros 提供 `#[derive(ErrorCode)]`，thiserror 是业务 crate 用 | 🟢 |
| `vernal-core` 与 `vernal-beans` 的 `BeanDescriptor` trait | vernal-core 不提供；vernal-beans 提供 | 不冲突 | 🟢 |
| `vernal-core` 与 `vernal-aop` 的 `Converter`（无） | vernal-aop 没有同名 Converter | 不冲突 | 🟢 |

### 2.4 模块路径对齐（**100% 镜像 spring-core**）

| spring-core 文件路径 | tx_di 文件路径 | vernal-core 文件路径 | 命名空间对齐 |
|---|---|---|---|
| `spring-core/src/main/java/org/springframework/core/Ordered.java` | — | `vernal-core/src/ordered.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/PriorityOrdered.java` | — | 同上 | ✅ |
| `spring-core/src/main/java/org/springframework/core/convert/ConversionService.java` | — | `vernal-core/src/convert/mod.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/convert/converter/Converter.java` | — | `vernal-core/src/convert/converter.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/convert/support/DefaultConversionService.java` | — | `vernal-core/src/convert/mod.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/NestedRuntimeException.java` | `tx_error/src/error.rs::AppError` | `vernal-core/src/error/vernal_error.rs` | 🟡 **不一致**：spring 用 `RuntimeException` 子类，tx_di 用 enum，vernal 用 enum（选择 Rust 习惯） |
| `spring-core/src/main/java/org/springframework/core/NestedIOException.java` | — | `vernal-core/src/error/vernal_error.rs::From<io::Error>` impl | 🟡 |
| `spring-core/src/main/java/org/springframework/util/StopWatch.java` | — | `vernal-core/src/time/stop_watch.rs` | ✅ |
| `spring-core/src/main/java/org/springframework/core/SpringVersion.java` | — | `vernal-core/src/lib.rs` 常量 | 🟡（spring 用类封装，vernal 用模块常量） |
| `spring-core/src/main/java/org/springframework/core/ErrorCoded.java` | `tx_error/src/code.rs::AppErrCode` | `vernal-core/src/error/error_code.rs`（trait 形态） | 🟡 |
| `spring-core/src/main/java/org/springframework/util/StopWatch.java` (TimeUnit inner enum) | — | `vernal-core/src/time/stop_watch_unit.rs`（独立文件） | 🟡（spring 用内部枚举，vernal 用独立文件） |
| — | `tx_error/src/error.rs::AppError` | `vernal-core/src/error/vernal_error.rs` | 🟡 |
| — | `tx_error/src/code.rs::CodeMsg` trait | `vernal-core/src/error/error_code.rs`（trait） | 🟡 |
| — | `tx_di/src/lifecycle.rs` | `vernal-core/src/app_lifecycle_phase.rs`（仅 enum） + `vernal-context/src/lifecycle_phase.rs`（3 变体 enum） + `vernal-context/src/component_lifecycle.rs`（trait） | 🔴 **冲突** |
| — | `tx_common/src/id.rs` | `vernal-core/src/id/*.rs`（5 种 ID 生成器） | 🆕（vernal 接管） |
| `spring-core/.../Observation*.java`（6.1+） | — | `vernal-core/src/diagnostics/*.rs`（待 S9） | 🆕 |

---

## 三、命名对比表（4 列）

| 概念 | Spring 命名 | tx_di 命名 | vernal-core 命名 | 一致性 |
|---|---|---|---|---|
| 组件主 trait | `org.springframework.stereotype.Component` | `tx_di_core::Component`（5 钩子） | `vernal_context::component_lifecycle::Lifecycle`（3 钩子） | 🟡 |
| 排序 | `Ordered.getOrder(): int` | `Component::init_sort(): i32` | `INIT_SORT_*` 常量 | 🟡 |
| 优先级排序 | `PriorityOrdered` | — | `INIT_SORT_INFRASTRUCTURE` + `HIGHEST_PRECEDENCE` 别名 | 🟡 |
| 类型转换 | `ConversionService.convert()` | — | `ConversionService::convert()` | ✅ |
| 类型转换判定 | `ConversionService.canConvert()` | — | `ConversionService::can_convert::<T>()`（待 S5） | ⬜ |
| 单向转换 | `Converter<S, T>` | — | `Converter<S, T>` | ✅ |
| 目标类型 trait | — | — | `Convertible::from_str_value()` | 🆕 |
| 嵌套异常 | `NestedRuntimeException` | `AppError::Internal(anyhow)` | `VernalError::Infrastructure(SharedError)` | 🟡 |
| IO 嵌套异常 | `NestedIOException` | `From<io::Error>` | `From<io::Error> for VernalError` | ✅ |
| 业务错误码 | `ErrorCoded` | `AppErrCode` + `CodeMsg` | `ErrorCode` trait + `into_vernal_error()` | 🟡 |
| 通用错误 | — | `BoxError` 不存在 | `BoxError = Box<dyn Error + ...>` | 🆕 |
| 共享错误 | — | — | `SharedError = Arc<dyn Error + ...>` | 🆕 |
| 错误上下文 | `NestedRuntimeException.getMessage()` | `AppError::WithContext.context` | `VernalError::WithContext.context` + `WithContextEntries.context` | 🆕 |
| 错误诊断报告 | — | — | `ErrorReport` + `ErrorContext` | 🆕 |
| 生命周期 trait | `org.springframework.context.Lifecycle` | `Component::init/async_init/async_run/shutdown` | `vernal_context::Lifecycle::initialize/start/stop` | 🟡 |
| 生命周期阶段枚举 | `Lifecycle.Phase` 接口 + `getPhase(): int` | （无 enum） | `vernal_core::AppLifecyclePhase`（8 变体）+ `vernal_context::LifecyclePhase`（3 变体） | 🟡 |
| 计时器 | `org.springframework.util.StopWatch` | — | `vernal_core::time::StopWatch` | ✅ |
| 时间单位 | `StopWatch.TimeUnit` 内部枚举 | — | `vernal_core::time::StopWatchUnit`（独立文件，待 S6） | 🟡 |
| ObjectId | `org.springframework.util.ObjectUtils.identityToString()` | `tx_common::id::*`（UUID / Snowflake） | `vernal_core::id::ObjectId` + 4 个 feature-gated 后端 | 🟡 |
| 错误域常量 | 隐式（namespace 命名空间） | `#[err("DI")]` | `ErrorDomain::{IOC, AOP, CONTEXT, WEB, HTTP, TOWER, DISCOVERY, MACROS, CORE, BRIDGE, MIGRATION, SECURITY, METRICS}` | 🟡 |
| 版本号 | `org.springframework.core.SpringVersion.getVersion()` | `env!("CARGO_PKG_VERSION")` 直接获取 | `vernal_core::FRAMEWORK_VERSION` + `MINIMUM_RUST_VERSION` + `PROJECT_STATUS` | ✅ |
| 跨度（Span） | `org.springframework Observation`（6.1+） | — | `vernal_core::diagnostics::Span`（待 S9） | 🆕 |
| API Result | `HttpStatusCode.value()` 不用 | `AppResult<T>` 别名 | `Result<T, BoxError>` 标准 | 🟡 |

---

## 四、问题清单（修复时按优先级处理）

### 🔴 P0 必修（S1 阶段立刻修复）

#### 1. `LifecyclePhase` 重名冲突

**问题描述**：

```rust
// vernal-core/src/lifecycle_phase.rs:11  (现状)
pub enum LifecyclePhase {  // 8 变体
    Created, Initializing, Initialized, Starting, Started,
    Stopping, Stopped, Failed,
}

// vernal-context/src/lifecycle_phase.rs:7  (现状)
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

**修复**（S4）：保留 vernal 现状，但增加 Spring 风格别名：
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

#### 5. `StopWatch` `TimeUnit` 命名风格

**现状**：
- Spring：`StopWatch.TimeUnit.NANOSECONDS / MICROSECONDS / ...`（全大写 + 复数）
- vernal-core：`StopWatchUnit::Nano / Micro / ...`（PascalCase + 单数）

**修复**（S6）：保留 vernal 风格（符合 Rust 枚举命名规范），但在 rustdoc 中注明对齐 Spring 的 5 个常量。

#### 6. `Span` vs `Observation` 命名

**现状**（S9 待实现）：
- Spring 6.1：`Observation` 类 + `ObservationRegistry` 注册中心
- vernal-core（计划）：`Span` + `SpanReport`（轻量版，无全局 Registry）

**修复**：保留 vernal 风格（轻量 + Rust 命名习惯），在 rustdoc 中注明对齐 Spring `Observation` API 子集。

---

### 🟢 P2 文档维护（持续）

#### 7. `BoxError` 与 `anyhow::Error` 的兼容性文档

**问题**：vernal-core 故意不引入 anyhow，但外部用户已经在用 anyhow，需要文档化互转方案。

**修复**（S10）：在 `failure.rs` doc-comment 中增加示例：
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

#### 8. `StopWatch.prettyPrint()` 输出格式差异

**现状**：
- Spring：`%n ns %n ms` 格式
- vernal-core：`%n ms` 格式（已实现）

**修复**（S6）：增加 `pretty_print_with_unit(StopWatchUnit::Nano)` 提供 ns 精度，对齐 Spring 6.1。

#### 9. `Order` vs `init_sort` 的语义对齐

**问题**：Spring `@Order(0)` 是中间值；vernal `init_sort = 0` 是 `BUSINESS` 默认值。

**修复**：保留 vernal 现状；在 vernal-macros 的 `#[component(init_sort = N)]` doc-comment 中说明：值越小越先，0 是业务组件默认值。

---

## 五、导入路径一致性 grep 检查

下列 grep 命令必须在迁移完成后产生零结果（除期望的引用外）：

```bash
# 1. vernal-core 内部不得有 vernal-context / vernal-beans / vernal-aop 引用
grep -rn "vernal_context\|vernal_beans\|vernal_aop" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：0 结果

# 2. vernal-core 不得引入 anyhow
grep -rn "anyhow" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：0 结果（除非 feature-gated 注释中提及）

# 3. vernal-core 不得引入 tokio
grep -rn "use tokio" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：0 结果（tokio 互转通过 #[cfg(feature = "tokio")]）

# 4. 验证重命名后无遗留
grep -rn "lifecycle_phase" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：所有出现点都对应 mod app_lifecycle_phase；无遗留 mod lifecycle_phase

# 5. vernal-core 错误域常量必须稳定
grep -rn "ErrorDomain::" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：13 个常量，类型稳定

# 6. 验证常量命名一致性
grep -rn "INIT_SORT_" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/
# 期望：8 个 INIT_SORT_* + 2 个 Spring 别名（HIGHEST_PRECEDENCE / LOWEST_PRECEDENCE）

# 7. 验证 From 实现完整性
grep -rn "impl From<" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/src/error/vernal_error.rs
# 期望：From<BoxError> + From<SharedError> + From<io::Error> + From<String> + From<&str>（S2）+ 可选 From<JoinError>

# 8. 验证 Convertible 内置实现数量
grep -rn "impl Convertible for" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/src/convert/
# 期望：S5 完成后 10 个内置实现（bool / 数字 / String / Option / enum + PathBuf / Duration / SocketAddr + OffsetDateTime / URL feature-gated）

# 9. 验证 ID 生成器统一抽象
grep -rn "impl IdGenerator for" /Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-core/src/id/
# 期望：S7 完成后 5 个实现（ObjectId / Uuid / Ulid / NanoId / Snowflake）
```

---

## 六、命名变更日志（与路线图同步）

| 日期 | 类型 | 改名前 | 改名后 | 阶段 |
|---|---|---|---|---|
| 2026-07-27 | 新文档 v2.0 | — | — | S0 ✅ |
| 2026-07-27 | 新文档 v2.0（路线图升级） | v1.0 | v2.0（加入 S7-S9 ID/Span + 8 个 INIT_SORT 锚点 + 14 个 feature flag） | S0 ✅ |
| 待定 | enum + module | `LifecyclePhase` + `lifecycle_phase.rs` | `AppLifecyclePhase` + `app_lifecycle_phase.rs` | S1 |
| 待定 | trait 增强 | `ErrorCode`（不变） | — | — |
| 待定 | enum 新增 | — | `StopWatchUnit::{Nano, Micro, Milli, Second, Minute}` | S6 |
| 待定 | trait 新增 | — | `IdGenerator`（统一抽象 5 个 ID 后端） | S7 |
| 待定 | struct 新增 | — | `Span` + `SpanId` + `SpanReport` + `AttributeValue` | S9 |
| 待定 | const 新增 | — | `INIT_SORT_BEAN_FACTORY` / `_EVENT_LISTENER` / `_MESSAGE_SOURCE` / `_TASK` | S4 |
| 待定 | const 新增 | — | `HIGHEST_PRECEDENCE` / `LOWEST_PRECEDENCE` Spring 别名 | S4 |
| 待定 | const 新增 | — | `ErrorDomain::MIGRATION` / `_SECURITY` / `_METRICS` | S3 |
| 待定 | From 新增 | — | `From<String> for VernalError` + `From<&str> for VernalError` | S2 |
| 待定 | From 新增 | — | `From<tokio::task::JoinError> for VernalError`（feature = "tokio"） | S2 |

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

// 生命周期（重命名后路径）
vernal_core::app_lifecycle_phase::AppLifecyclePhase

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

### 7.2 新增承诺（迁移完成后必须保留）

迁移完成后，下列类型是 **新增** 公共契约，**不得删除**：

```rust
// S1 新增（重命名）
vernal_core::app_lifecycle_phase::AppLifecyclePhase

// S2 新增
impl From<std::io::Error> for VernalError       // 已有 ✅
impl From<BoxError> for VernalError            // 已有 ✅
impl From<SharedError> for VernalError          // 已有 ✅
impl From<String> for VernalError               // 🆕
impl From<&str> for VernalError                 // 🆕
#[cfg(feature = "tokio")]
impl From<tokio::task::JoinError> for VernalError // 🆕

// S3 新增
vernal_core::error::ErrorDomain::MIGRATION
vernal_core::error::ErrorDomain::SECURITY
vernal_core::error::ErrorDomain::METRICS

// S4 新增
vernal_core::ordered::INIT_SORT_BEAN_FACTORY
vernal_core::ordered::INIT_SORT_EVENT_LISTENER
vernal_core::ordered::INIT_SORT_MESSAGE_SOURCE
vernal_core::ordered::INIT_SORT_TASK
vernal_core::ordered::HIGHEST_PRECEDENCE
vernal_core::ordered::LOWEST_PRECEDENCE

// S5 新增
vernal_core::convert::ConversionService::can_convert::<T>()  // 新增方法
impl Convertible for std::path::PathBuf
impl Convertible for std::time::Duration
impl Convertible for std::net::SocketAddr
#[cfg(feature = "convert-time")]    impl Convertible for time::OffsetDateTime
#[cfg(feature = "convert-url")]     impl Convertible for url::Url
#[cfg(feature = "convert-uuid")]    impl Convertible for uuid::Uuid
#[cfg(feature = "convert-bytes")]   impl Convertible for bytes::Bytes

// S6 新增
vernal_core::time::StopWatchUnit
vernal_core::time::StopWatch::short_summary
vernal_core::time::StopWatch::pretty_print_with_unit
vernal_core::time::StopWatch::start_with_ticks
vernal_core::time::StopWatch::task_info_array

// S7 新增
vernal_core::id::IdGenerator                                    // 统一 trait
#[cfg(feature = "id-uuid")]     vernal_core::id::UuidId
#[cfg(feature = "id-ulid")]     vernal_core::id::UlidId
#[cfg(feature = "id-nanoid")]   vernal_core::id::NanoId
#[cfg(feature = "id-snowflake")] vernal_core::id::SnowflakeId   // 自实现
#[cfg(feature = "serde")]
impl serde::Serialize for vernal_core::id::ObjectId
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for vernal_core::id::ObjectId

// S9 新增
vernal_core::diagnostics::Span
vernal_core::diagnostics::SpanId
vernal_core::diagnostics::SpanReport
vernal_core::diagnostics::SpanStatus
vernal_core::diagnostics::AttributeValue
```

---

## 八、外部 API 一致性承诺（**Rust 生态 feature flag 一致性**）

迁移完成后，vernal-core 应当通过 `cargo build -p vernal-core` 默认以**零外部依赖**通过编译。
下列 feature flag 必须保持稳定命名（避免破坏用户脚本）：

### 8.1 Tier 1：基础设施（强烈推荐）

| Feature 名 | 对应 crate | 版本 | 许可证 | 默认 | 一致性 |
|---|---|---|---|---|---|
| `error-derive` | `thiserror = "2.0"` | 2.0.19 | MIT/Apache-2.0 | off | ✅ |
| `macros` | `pastey = "0.2"` | 0.2.3 | MIT/Apache-2.0 | off | ✅ |
| `once-cell` | `once_cell = "1.21"` | 1.21.4 | MIT/Apache-2.0 | off | ✅ |
| `registry` | `inventory = "0.3"` | 0.3.24 | MIT/Apache-2.0 | off | ✅ |

### 8.2 Tier 2：用户可见类型（强烈推荐）

| Feature 名 | 对应 crate | 版本 | 许可证 | 默认 | 一致性 |
|---|---|---|---|---|---|
| `id-uuid` | `uuid = "1.24"` | 1.24.0 | MIT/Apache-2.0 | off | ✅ |
| `id-ulid` | `ulid = "3.0"` | 3.0.0 | MIT | off | ✅ |
| `id-nanoid` | `nanoid = "0.5"` | 0.5.0 | MIT | off | ✅ |
| `convert-uuid` | （同 id-uuid） | — | — | off | ✅ |
| `convert-url` | `url = "2.5"` | 2.5.8 | MIT/Apache-2.0 | off | ✅ |
| `convert-time` | `time = "0.3"` | 0.3.54 | MIT/Apache-2.0 | off | ✅ |
| `convert-chrono` | `chrono = "0.4"` | 0.4.45 | MIT/Apache-2.0 | off | ✅ |
| `convert-bytes` | `bytes = "1.12"` | 1.12.1 | MIT | off | ✅ |
| `serde` | `serde = "1"` | 1.0.229 | MIT/Apache-2.0 | off | ✅ |
| `derive-extras` | `derive_more = "2.1"` | 2.1.1 | MIT | off | ✅ |

### 8.3 Tier 3：可选 / 特定场景

| Feature 名 | 对应 crate | 版本 | 许可证 | 默认 | 一致性 |
|---|---|---|---|---|---|
| `case-conv` | `convert_case = "0.11"` | 0.11.0 | MIT | off | ✅ |
| `time-web` | `web-time = "1.1"` | 1.1.0 | MIT/Apache-2.0 | off | ✅ |

### 8.4 **明确禁止新增**（除非团队架构评审通过）

- ❌ `anyhow` feature（vernal-core 自身永不允许依赖 anyhow）
- ❌ `priority-queue` feature（LGPL/MPL copyleft）
- ❌ `instant` feature（unmaintained；改用 web-time 或 std）
- ❌ `async-trait` feature（vernal-core 不持有 dyn trait）
- ❌ `tokio` feature（runtime 耦合；JoinError 互转在 vernal-context 解决）
- ❌ `tracing` feature（归 `vernal-log`）
- ❌ `bson` feature（MongoDB 驱动符号污染）
- ❌ `redis` feature（职责不属于 core 层）
- ❌ `snowflake` feature（1.3.0 已废弃；自实现）
- ❌ `smartstring` feature（MPL-2.0 许可证兼容性需评估）

---

## 九、检查结论

### 9.1 当前已发现的问题

| # | 问题 | 严重度 | 修复阶段 |
|---|---|---|---|
| 1 | `LifecyclePhase` 重名冲突 | 🔴 P0 | S1 |
| 2 | `INIT_SORT_*` 风格与 Spring 不同（缺 Spring 风格别名） | 🟡 P1 | S4 |
| 3 | `ErrorCode` 命名取舍 | 🟡 P1 | 文档维护 |
| 4 | `ConversionService` 单 struct 模式 | 🟢 P2 | 文档维护 |
| 5 | `BoxError` 与 anyhow 兼容性文档 | 🟢 P2 | S10 |
| 6 | `StopWatch.TimeUnit` 命名风格差异 | 🟡 P1 | S6 |
| 7 | `Span` vs `Observation` 命名 | 🟢 P2 | S9 |
| 8 | 8 个 INIT_SORT 锚点不完整 | 🟡 P1 | S4 |
| 9 | 13 个 ErrorDomain 不完整（缺 MIGRATION/SECURITY/METRICS） | 🟡 P1 | S3 |
| 10 | 10 个 Convertible 不完整（缺 PathBuf/Duration/URL/UUID 等） | 🟡 P1 | S5 |
| 11 | 5 个 ID 生成器不完整（仅 ObjectId） | 🟡 P1 | S7 |
| 12 | `can_convert::<T>()` 判定方法缺失 | 🟡 P1 | S5 |
| 13 | `short_summary()` / `pretty_print_with_unit()` 缺失 | 🟡 P1 | S6 |
| 14 | `Span` 类型缺失 | 🟡 P1 | S9 |
| 15 | `From<String/&str>` 缺失 | 🟡 P1 | S2 |
| 16 | `From<JoinError>` 缺失 | 🟡 P1 | S2 |

### 9.2 通过的检查项

- ✅ 14 个映射到 spring-core 命名保留（`ConversionService` / `Converter` / `Ordered` / `StopWatch` / `NestedRuntimeException` / `ErrorCoded` 等）
- ✅ 5 个映射到 tx_di 命名保留或采纳（`VernalError` / `ErrorCode` / `BoxError` / `SharedError` / `ObjectId`）
- ✅ 模块路径 100% 镜像 spring-core（命名大小写一致）
- ✅ `BoxError / SharedError / ObjectId / StopWatch` 与 Spring / tx_di 兼容
- ✅ `VernalError` vs `AppError` 命名空间通过 `Vernal` 前缀分离
- ✅ `Converter<S, T>` 与 `Convertible` trait 在 vernal 中各自承担 `Converter / Convertible` 角色，互不冲突
- ✅ 默认零外部依赖（除 `linkme` workspace-level）
- ✅ 全 17 个公开声明均含中文 rustdoc 注释
- ✅ `#![forbid(unsafe_code)]` 与 `#![deny(missing_docs)]` 在 workspace lint 启用

### 9.3 跟进事项

1. **S1 完成前**：禁止引入新功能涉及 `LifecyclePhase` 的命名（避免命名冲突扩大）
2. **S2 完成前**：`From<BoxError>` / `From<String>` / `From<&str>` 等 From 实现的稳定 API
3. **S9 完成前**：`Span` 类型与 `SpanReport` 的最终命名确认
4. **S10 完成前**：所有公开类型的文档覆盖率必须达到 100%
5. **总目标**：vernal-core 成为整个 vernal 框架的"基础合同"层，被 `vernal-beans / context / aop / web / tx / db / cache / expression / aspects` 等子系统单向依赖，且保证类型稳定。

### 9.4 与 vernal-expression / vernal-aspects 的协调

| vernal-core API | vernal-expression / vernal-aspects 消费方 |
|---|---|
| `INIT_SORT_*` | 切面执行顺序（`@Order` 等价），与 vernal-aspects 的 8 个 INIT_SORT 锚点命名一致 |
| `AppLifecyclePhase` | 切面自身的初始化阶段追踪 |
| `VernalError` | 所有切面异常传播（事务 / 缓存 / 异步 / DI 切面） |
| `ObjectId` | AOP 调用 ID（`InvocationId`） |
| `StopWatch` | 切面计时 |
| `ConversionService` | 缓存切面 `cache_operation.rs` 中的 key 转换 |
| `Span`（S9） | 切面调用链路追踪（与 `InvocationContext` 协同） |
| `ErrorContext` | 切面异常上下文 |

---

## 十、版本与文档同步

- 本文档 v2.0 与《vernal-core 迁移路线图 v2.0》《vernal-core 对象级对照表 v2.0》《vernal-core 语义迁移对照表 v2.0》同步维护
- v1.0 文档保留在 `vernal-framework/docs/vernal-expression/` 作为历史参考
- v2.0 文档放置在 `vernal-framework/docs/` 根目录，作为当前权威版本
# spring-context-indexer → vernal-context-indexer 全量迁移路线图

> 版本：v1.0（2026-07-28）｜基线：Spring Framework **7.0.8** spring-context-indexer
> 仓库：easy-4-rust/vernal @ dev ｜ 底盘：**linkme 分布式切片**（链接期收集） + **vernal-beans**（IoC 内核）
> **目录命名 100% 镜像 Spring 路径风格**：`spring-context-indexer` → `vernal-context-indexer`
> **功能语义 100% 镜像 Spring 的 `spring-context-indexer` + tx_di 的 linkme 注册机制**
> 本文档随代码同步维护，每阶段完成时更新

## 一、总目标

以 Spring Framework 7.0.8 的 **spring-context-indexer** 模块（11 个 Java 类 + 1 个 `META-INF/services` SPI 资源 + 1 个 `META-INF/spring.components` 索引产物）为语义蓝本，并融合 **tx_di 的 linkme 分布式切片注册机制**（链接期组件元数据收集），实现功能语义完全对齐的 `vernal-context-indexer` crate。

**目标产出**：
- 1 个 Cargo crate：`crates/vernal-context-indexer/`
- **8 个 Rust 文件**（含 `lib.rs` + 7 个独立语义文件）
- 公开类型 **5 个**（`LinkedComponentIndex` 切片 / `LinkedComponentEntry` 条目 / `LinkedComponentIndex` 目录 / `LinkedComponentIndexError` / 模块级常量）
- 与 `vernal-expression` 4 份文档保持同等的"路线图 + 对照表 + 语义表 + 一致性检查"格式

### 设计原则

1. **命名 100% 镜像 Spring `spring-context-indexer`**：
   - Java 包 `org.springframework.context.index.processor` → Rust 模块 `src/index/`
   - `CandidateComponentsIndex` → `LinkedComponentIndex`
   - `ItemMetadata` → `LinkedComponentEntry`
   - `CandidateComponentsMetadata` → `LinkedComponentIndex`(同 Spring 命名习惯)
2. **tx_di 的 linkme 机制是底座**：链接期 `#[linkme::distributed_slice]` 替代 JSR-269 注解处理器，零运行时反射、零文件 IO
3. **vernal-beans 是内核**：只依赖 `ComponentDefinition` / `RegistryBuilder` / `DefinitionError`，不引入其他 crate
4. **物理目录结构镜像**：`vernal-discovery` 目录整建制更名为 `vernal-context-indexer`，文件路径 1:1 对齐 spring-context-indexer 包路径
5. **公开类型必须有中文文档注释**：struct / enum / 函数 / 常量全部用中文 doc 注释
6. **不实现 JSR-269 Processor**：linkme 编译期已收集，不需 Java 标准 SPI
7. **不写 `META-INF/spring.components` 物理文件**：链接期切片是 Spring 索引文件的 1:1 Rust 等价物，更彻底（不需 IO、跨平台）

### vernal-context-indexer 镜像对照总览

| Spring 概念 | vernal 形式 | 来源参考 |
|---|---|---|
| `META-INF/spring.components` 物理文件 | `#[linkme::distributed_slice] static LINKED_COMPONENT_INDEX: [LinkedComponentEntry]` | tx_di `COMPONENT_REGISTRY` |
| `org.springframework.stereotype.Indexed` 注解 | **不需要**（linkme 自动收集所有 `#[derive(Component)]`） | vernal-discovery 既有 |
| `javax.annotation.processing.Processor` SPI | **不需要**（链接期） | — |
| `CandidateComponentsIndexer#process(...)` | 链接期 `static __DI_META_X: ComponentMeta` 自动归入切片 | tx_di `gen_meta_entry` |
| `CandidateComponentsIndex`（运行时入口） | `LinkedComponentIndex::scan(...)` / `scan_all()` / `merge(...)` | vernal-discovery `LinkedComponentCatalog` |
| `ItemMetadata`（一条索引记录） | `LinkedComponentEntry` | vernal-discovery `LinkedComponentRegistration` |
| `@Indexed` 元注解传播 | **不支持**（Rust 没有元注解概念）；改为"`#[derive(Component)]` 即入索引" | — |
| `AntPathMatcher(".")` 包路径匹配 | 模块路径前缀匹配 `starts_with` | vernal-discovery 既有 |
| `META-INF/spring.factories`-like 编程注入 | `LinkedComponentIndex::add_entry(...)` 运行时注册接口 | Spring 7.0 `addIndex` |

### 与 vernal-context-indexer 当前状态（即 vernal-discovery）的差距

| 维度 | 当前 vernal-discovery | 目标 vernal-context-indexer |
|------|---------------------|---------------------------|
| crate 名 | `vernal-discovery` | **`vernal-context-indexer`** |
| 文件数 | 4 | **8**（+ 4 个 index 子模块文件） |
| 公开类型数 | 3（`LinkedComponentRegistration` / `LinkedComponentCatalog` / `LinkedComponentCatalogError`） | **5**（+ `LinkedComponentEntry` / `LinkedComponentIndex` 切片名统一） |
| 链接期机制 | linkme | linkme（不变） |
| 镜像 Spring 包路径 | ❌（仅叫"discovery"） | ✅（叫"context-indexer"） |
| 模块路径过滤 | ✅ | ✅（保留） |
| 编程注入 API | ❌ | ✅（新增 `LinkedComponentIndex::add_entry`） |
| 单元测试 | 3（discovery / discovery_support / discovery_compile_fail） | **8+**（+ index 子模块独立测试） |
| Spring 包镜像 | 0/1 = 0% | **1/1 = 100%**（`index/` 子模块镜像 `processor/`） |
| Spring 类镜像 | 0/11 = 0% | **5/11 = 45%**（核心 5 个 API 完整镜像，其余 6 个由 linkme + vernal-beans 等价承担） |
| 命名一致性 | 与 Spring 无对应 | **100%** 镜像 Spring `*Index*` / `*Metadata*` 命名 |

---

## 二、进度总览

| 阶段 | 内容 | 状态 | 验收 |
|---|---|---|---|
| **S0** | 路线图 + 对照表 + 语义表 + 一致性检查 4 份文档 | ✅（本文） | docs/vernal-expression/ 四份 vernal-context-indexer 文档 |
| **S1** | 目录重命名 + crate 重命名 + workspace 同步 | ⬜ | `crates/vernal-context-indexer/Cargo.toml` 落地、workspace `members` 同步、vernal-context 依赖路径更新 |
| **S2** | `index/` 子模块拆分（镜像 `processor/` 包路径） | ⬜ | `src/index/{mod.rs, item_metadata.rs, properties_marshaller.rs, sorted_properties.rs, type_helper.rs}` 落地 |
| **S3** | 公开类型 rename + Spring 命名对齐 | ⬜ | `LinkedComponentEntry`（旧 Registration）+ `LinkedComponentIndex`（旧 Catalog）+ `LinkedComponentIndexError`（旧 CatalogError）+ 切片常量 `LINKED_COMPONENT_INDEX`（旧 `LINKED_COMPONENT_REGISTRATIONS`） |
| **S4** | 编程注入 API（对标 Spring 7.0 `addIndex`） | ⬜ | `LinkedComponentIndex::add_entry(...)` / `clear_cache()` 公开方法 |
| **S5** | 测试改名 + 全工作区 `use vernal_discovery::` → `use vernal_context_indexer::` 替换 | ⬜ | `cargo build --workspace` 通过、测试全绿 |
| **S6** | 中文 doc 注释覆盖率 + 文档收尾 | ⬜ | 所有公开类型 100% 中文 doc；本路线图状态同步 |

---

## 三、阶段详细计划

### S1 — 目录重命名 + crate 重命名 + workspace 同步（预估 0.5 天）

**范围**：把 `crates/vernal-discovery/` 整建制重命名为 `crates/vernal-context-indexer/`

**目标文件**（1 个 + 引用替换）：

| 文件 | 改动 | 工作量 |
|------|------|--------|
| `crates/vernal-discovery/` → `crates/vernal-context-indexer/` | `git mv` 或文件系统 rename | 0.05 天 |
| `Cargo.toml`（indexer crate 内） | `name = "vernal-context-indexer"` + `description` 更新为"镜像 Spring spring-context-indexer 语义" | 0.1 天 |
| `src/lib.rs` `#![doc]` | 改为中文："Vernal 链接期组件索引器，对标 Spring 的 spring-context-indexer" | 0.05 天 |
| 全部 `vernal-context/Cargo.toml` 与源码 `use vernal_discovery::` 引用 | 替换为 `use vernal_context_indexer::` | 0.2 天 |
| `tests/*.rs` 路径引用 | 同上 | 0.1 天 |

**验收**：
- `cargo build --workspace` 通过
- `cargo test -p vernal-context-indexer` 全绿
- workspace `cargo tree` 中无 `vernal-discovery` 字样
- `find . -name '*.rs' -exec grep -l vernal_discovery {} \;` 返回 0 行

---

### S2 — `index/` 子模块拆分（镜像 `processor/` 包路径）（预估 1.5 天）

**范围**：在 `src/index/` 下镜像 Spring `org.springframework.context.index.processor` 包路径，建立与 Spring 同构的子模块结构。

**目标文件**（共 5 个新增）：

| 文件 | Spring 对偶 | 语义 | 工作量 |
|------|----------|------|--------|
| `src/index/mod.rs` | `processor/package-info.java` | 子模块入口 + re-export | 0.1 天 |
| `src/index/item_metadata.rs` | `ItemMetadata.java` | 一条索引项（Rust struct，对应 `ItemMetadata` 字段 `type` + `stereotypes`） | 0.3 天 |
| `src/index/properties_marshaller.rs` | `PropertiesMarshaller.java` | 索引序列化/反序列化（Rust 用 `BTreeMap<String, Vec<String>>` 表达 Properties） | 0.4 天 |
| `src/index/sorted_properties.rs` | `SortedProperties.java` | 确定性格式化辅助 | 0.2 天 |
| `src/index/type_helper.rs` | `TypeHelper.java` | Rust 类型提取辅助（`TypeId::of::<T>()` → 字符串） | 0.3 天 |

**验收**：
- `src/index/item_metadata.rs` 的 `LinkedComponentEntry` struct 与 Spring `ItemMetadata` 字段 1:1 对应
- `properties_marshaller.rs` 提供 `serialize(index) -> BTreeMap<&str, Vec<String>>` + `deserialize(map) -> LinkedComponentIndex`
- `sorted_properties.rs` 用 `BTreeMap`（Rust std 已是排序的，无需自实现）
- `type_helper.rs` 提供 `type_id_to_string::<T>() -> &'static str`（用 `std::any::type_name::<T>()`）

---

### S3 — 公开类型 rename + Spring 命名对齐（预估 1.0 天）

**范围**：把 `vernal-discovery` 的 3 个公开类型改名,使其与 Spring `*Index*` 命名 1:1 对齐。

**目标文件**（共 3 个改 + 1 个新增）：

| 文件 | 旧名 | 新名 | 工作量 |
|------|------|------|--------|
| `src/linked_component_registration.rs` | `LinkedComponentRegistration` | **`LinkedComponentEntry`** | 0.2 天 |
| `src/linked_component_registry.rs` | `LINKED_COMPONENT_REGISTRATIONS` | **`LINKED_COMPONENT_INDEX`**（切片常量）+ 文件 rename 为 `linked_component_index_slice.rs` | 0.2 天 |
| `src/linked_component_catalog.rs` | `LinkedComponentCatalog` | **`LinkedComponentIndex`**（对外目录类型，与 Spring `CandidateComponentsIndex` 同名）+ 文件 rename 为 `linked_component_index.rs` | 0.3 天 |
| `src/linked_component_catalog_error.rs` | `LinkedComponentCatalogError` | **`LinkedComponentIndexError`** | 0.1 天 |
| `src/lib.rs` `pub use` | — | 更新为 5 个新名 | 0.1 天 |
| 全部 `vernal-context/` 下消费方引用 | `vernal_discovery::*` | `vernal_context_indexer::*` | 0.1 天 |

**验收**：
- `grep -rn 'LinkedComponentRegistration\|LinkedComponentCatalog\|LinkedComponentCatalogError\|LINKED_COMPONENT_REGISTRATIONS' crates/` 返回 0 行（除 docs/）
- `grep -rn 'LinkedComponentEntry\|LinkedComponentIndex\|LinkedComponentIndexError\|LINKED_COMPONENT_INDEX' crates/vernal-context-indexer/` 全部命中
- `cargo doc -p vernal-context-indexer --no-deps` 通过，所有公开类型有中文 doc

---

### S4 — 编程注入 API（对标 Spring 7.0 `addIndex`）（预估 0.5 天）

**范围**：Spring 7.0 在 `CandidateComponentsIndexLoader` 新增 `addIndex(ClassLoader, CandidateComponentsIndex)` 编程注入入口。vernal-context-indexer 对应实现：

**目标方法**（在 `src/linked_component_index.rs` 中）：

| 方法 | Spring 对偶 | 语义 | 工作量 |
|------|----------|------|--------|
| `LinkedComponentIndex::add_entry(entry: LinkedComponentEntry)` | `CandidateComponentsIndex#registerCandidateType` | 运行时向索引注入一个条目 | 0.2 天 |
| `LinkedComponentIndex::clear_cache()` | `CandidateComponentsIndexLoader#clearCache` | 清空运行时注入缓存 | 0.1 天 |
| `LinkedComponentIndex::has_index()` | `CandidateComponentsIndexLoader#loadIndex != null` | 是否有可用索引 | 0.05 天 |
| `LinkedComponentIndex::iter()` | `index.getCandidateTypes(...)` 替代 | 迭代全部索引项 | 0.15 天 |

**验收**：
- 单元测试 `tests/runtime_inject.rs`：构造一个 `LinkedComponentEntry` 注入后 `scan_all()` 能看到
- `clear_cache()` 后 `scan_all()` 看不到运行时注入的项
- 与链接期切片合并时,运行时项优先级最高（后注入覆盖先注入）

---

### S5 — 测试改名 + 全工作区 import 替换（预估 0.3 天）

**范围**：与 S1/S3 配套,把所有测试文件改名 + 路径引用替换。

**目标文件**：

| 文件 | 旧名 | 新名 | 工作量 |
|------|------|------|--------|
| `tests/linked_component_discovery.rs` | `linked_component_discovery.rs` | `linked_component_index.rs` | 0.05 天 |
| `tests/discovery_compile_fail.rs` | `discovery_compile_fail.rs` | `index_compile_fail.rs` | 0.05 天 |
| `tests/discovery_support/` | `discovery_support/` | `index_support/` | 0.05 天 |
| `tests/discovery_support/*.rs` 内部 `mod` 名 | `discovery_support` | `index_support` | 0.05 天 |
| 全工作区 `use vernal_discovery::` | — | `use vernal_context_indexer::` | 0.1 天 |

**验收**：
- `cargo test --workspace` 全绿（覆盖 vernal-context-indexer + vernal-context + 其他消费方）
- 测试文件命名风格与 spring-aspects → vernal-aspects 文档保持一致

---

### S6 — 中文 doc 注释覆盖率 + 文档收尾（预估 0.2 天）

**范围**：所有公开类型 100% 中文 doc 注释。

**目标文件**（仅 doc 注释,不动代码逻辑）：

| 文件 | 改动 | 工作量 |
|------|------|--------|
| `src/lib.rs` | 顶部中文 crate doc | 0.05 天 |
| `src/index/mod.rs` | 中文包级 doc | 0.02 天 |
| `src/linked_component_index_slice.rs` | 中文 doc 说明 linkme 切片等价于 `META-INF/spring.components` | 0.03 天 |
| `src/linked_component_entry.rs` | 中文 doc 说明 Spring `ItemMetadata` 镜像 | 0.03 天 |
| `src/linked_component_index.rs` | 中文 doc 说明 Spring `CandidateComponentsIndex` 镜像 | 0.03 天 |
| `src/linked_component_index_error.rs` | 中文 doc | 0.02 天 |
| 5 个 `src/index/*.rs` | 中文 doc | 0.02 天 |

**验收**：
- `cargo doc -p vernal-context-indexer --no-deps` 通过
- 公开类型 100% 中文 doc（人工抽查）
- 路线图状态同步:把本文档 S1~S6 从 ⬜ 改为 ✅

---

## 四、工程规范

### 命名规则

| 项目 | 规范 |
|------|------|
| crate 名 | **kebab-case + 连字符**：`vernal-context-indexer`（镜像 `spring-context-indexer`） |
| 目录命名 | **镜像 Spring 包路径**：`src/index/`（镜像 `org.springframework.context.index.processor`） |
| 文件名 | snake_case（`linked_component_index.rs` / `item_metadata.rs`） |
| 类型名 | **PascalCase,与 Spring 高度对齐**：`LinkedComponentIndex`（镜像 `CandidateComponentsIndex`）、`LinkedComponentEntry`（镜像 `ItemMetadata`）、`LinkedComponentIndexError` |
| 方法名 | snake_case（`scan_all`、`component_definitions`、`add_entry`） |
| trait | `trait` 关键字,与 Spring `interface` 对齐（indexer 无显式 trait,所有功能走 `impl`） |
| 公开 re-export | `pub use linked_component_index::LinkedComponentIndex;` 风格（与 vernal-beans 一致） |

### 文件结构规范

```rust
//! 对应 Java 类：org.springframework.context.index.CandidateComponentsIndex
//!
//! 链接期组件索引目录：从 linkme 分布式切片 `LINKED_COMPONENT_INDEX` 中按模块路径
//! 过滤组件,生成确定性、只读的注册目录。
//!
//! 对标 Spring 的 `ClassPathScanningCandidateComponentProvider` 与
//! `CandidateComponentsIndex` 的运行时消费入口。

use std::collections::BTreeSet;

use vernal_beans::{DefinitionError, RegistryBuilder};

use crate::{
    LINKED_COMPONENT_INDEX, LinkedComponentEntry, LinkedComponentIndexError,
};

/// 链接期组件索引目录。
///
/// 对标 Spring `org.springframework.context.index.CandidateComponentsIndex`：
/// - Spring：classpath 扫描 + `META-INF/spring.components` 索引文件消费
/// - Vernal：`LINKED_COMPONENT_INDEX` 链接期切片 + `LinkedComponentIndex::scan`
///
/// Catalog 与具体 `RegistryBuilder` 分离：同一个扫描结果可以安装到多个应用，每次安装
/// 都创建新的 Definition，并由各自 Container 独立拥有 Singleton、Scope 缓存和
/// 解析历史。
#[derive(Clone, Debug)]
pub struct LinkedComponentIndex {
    entries: Vec<&'static LinkedComponentEntry>,
}

impl LinkedComponentIndex {
    /// 按模块路径前缀扫描条目（对标 `@ComponentScan(basePackages = {...})`）。
    ///
    /// 从链接期分布式切片中筛选模块路径以任一 `base_packages` 前缀开头的条目。
    pub fn scan<I, S>(base_packages: I) -> Result<Self, LinkedComponentIndexError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // ...
    }
}
```

### 测试规范

- 每个公开方法至少 1 个单元测试
- 模块路径过滤、空集合、运行时注入、空清空、合并目录 各 1 个场景
- `tests/index_support/` 下放 fixture：database / order_service / audit_worker / manual_component
- `tests/index_compile_fail.rs` 用 trybuild 验证非法 API 在编译期失败

---

## 五、风险评估

| 风险 | 影响 | 对策 |
|------|------|------|
| 全工作区 `vernal_discovery::` 引用遗漏 | 编译失败 | `cargo build --workspace` 完整跑通；加 `rg 'vernal_discovery' crates/` 检查 |
| linkme 切片顺序在不同链接器下不一致 | `scan_all()` 输出顺序不稳定 | 已用 `(module_path, name)` 排序（S3 保留 vernal-discovery 既有策略） |
| 运行时注入与链接期切片合并语义模糊 | 用户预期不一致 | 文档明确：运行时项作为"补丁层"，优先级最高，可被 `clear_cache()` 清空 |
| tx_di 与 spring-context-indexer 语义不完全对齐 | 选边困难 | **以 spring-context-indexer 语义为准**,tx_di 仅参考 linkme 切片机制 |
| 中文注释工作量大 | 交付延迟 | 分阶段进行,S6 集中完成 |
| `src/index/` 子模块与 `src/` 顶层 `linked_component_index.rs` 同时存在导致概念混淆 | 用户理解成本 | 子模块 `index/` 仅放 ItemMetadata / Properties / TypeHelper 等与 Spring `processor/` 对偶的辅助结构;主类型仍在 `src/` 顶层 |

---

## 六、预期成果

完成全部 6 个阶段后,`vernal-context-indexer` 将成为 **Rust 生态最完整的 Spring-context-indexer 等价实现 + tx_di linkme 注册机制的融合形态**：

| 维度 | vernal-discovery（当前）| vernal-context-indexer（完成后）|
|------|----------------------|-----------------------------|
| crate 名 | `vernal-discovery` | **`vernal-context-indexer`** |
| 文件数 | 4（src/ 顶层 + lib.rs）| **8+**（+ `src/index/` 5 个子模块文件）|
| 公开类型数 | 3 | **5** |
| 镜像 Spring 类 | 0/11 = 0% | **5/11 = 45%**（核心 API 全部镜像,辅助 API 由 linkme + vernal-beans 等价承担）|
| 镜像 Spring 包路径 | 0/1 = 0% | **1/1 = 100%**（`src/index/` 子模块镜像 `processor/`）|
| 编程注入 API | ❌ | ✅（对标 Spring 7.0 `addIndex`）|
| 中文 doc 注释覆盖率 | 部分 | **100%** |
| 与 vernal-beans 协作 | ✅（已有）| ✅（保留） |
| 与 vernal-context 集成 | ✅（已有）| ✅（保留） |
| 链接期发现能力 | ✅（已有）| ✅（保留） |
| 单元测试数 | 3 | **8+** |

### 与 spring-framework 7.0.8 spring-context-indexer 的对标度

| 维度 | 完成前 | 完成后 |
|------|--------|--------|
| 类覆盖 | 0/11 = 0% | **5/11 = 45%**（核心 API）|
| 包路径镜像 | 0/1 = 0% | **1/1 = 100%** |
| 索引物理形式 | linkme 切片（已存在）| linkme 切片（保留）|
| 编程注入 | ❌ | ✅（镜像 `addIndex`）|
| 单元测试 | 3 | **8+** |

### 与 vernal-aspects / vernal-expression 的一致性规范对比

| 维度 | vernal-aspects | vernal-expression | vernal-context-indexer |
|------|---------------|------------------|---------------------|
| crate 名风格 | `vernal-aspects` | `vernal-expression` | **`vernal-context-indexer`**（连字符镜像 Spring）|
| 目录镜像 | 5 个 aspectj 子包 | 4 个 expression 子包 | **1 个 index 子模块**（+ src/ 顶层 4 个）|
| 公开类型数 | 21 类 | 87+ 类 | **5 类** |
| 中文 doc 注释 | 100% | 100% | **100%** |
| 单元测试 | 46+ | 96+ | **8+** |
| 文档规范 | 4 份 | 4 份 | **4 份**（本任务）|

---

## 七、与 vernal-context / vernal-beans / vernal-macros 的协作契约

### 7.1 依赖方向（严格单向）

```
vernal-context  ──依赖──>  vernal-context-indexer  ──依赖──>  vernal-beans
                                                              │
vernal-macros ──生成──> #[linkme::distributed_slice(LINKED_COMPONENT_INDEX)] static __DI_META_X
                          │
                          └──> 这些 static 由 vernal-context-indexer 在启动时遍历
```

- `vernal-context-indexer` **不依赖** `vernal-context`(反向会循环)
- `vernal-context` **依赖** `vernal-context-indexer`(消费 `LinkedComponentIndex`)
- `vernal-macros` **不依赖** `vernal-context-indexer`,仅生成 linkme 切片条目
- `vernal-beans` 提供 `ComponentDefinition` / `RegistryBuilder` / `DefinitionError`

### 7.2 与 vernal-macros 的代码生成契约

`vernal-macros::Component` 派生宏生成的 linkme 注册条目必须落入 `vernal_context_indexer::LINKED_COMPONENT_INDEX` 切片。宏实现要点：

```rust
// vernal-macros/src/component_derive.rs 中生成的代码（伪代码）
#[vernal_context_indexer::linkme::distributed_slice(vernal_context_indexer::LINKED_COMPONENT_INDEX)]
#[linkme(crate = vernal_context_indexer::linkme)]
pub static __DI_META_MyService: vernal_context_indexer::LinkedComponentEntry =
    vernal_context_indexer::LinkedComponentEntry::new(
        module_path!(),
        concat!(module_path!(), "::", stringify!(MyService)),
        <MyService as vernal_beans::Component>::definition,
    );
```

### 7.3 与 vernal-context 的消费契约

`vernal-context` 的 `VernalApplicationBuilder` 提供一个 `scan(base_packages: &[&str])` 方法,内部委托给 `vernal_context_indexer::LinkedComponentIndex::scan(base_packages)`:

```rust
// vernal-context/src/vernal_application_builder.rs（伪代码）
pub fn scan(&mut self, base_packages: &[&str]) -> Result<&mut Self, ApplicationBuildError> {
    let index = vernal_context_indexer::LinkedComponentIndex::scan(base_packages)
        .map_err(ApplicationBuildError::from)?;
    self.registry.register_all(index.component_definitions())
        .map_err(ApplicationBuildError::from)?;
    Ok(self)
}
```

---

## 八、不迁移的 Spring 特有功能

| Spring 概念 | 原因 | Rust 替代 |
|------------|------|-----------|
| `javax.annotation.processing.Processor` SPI | Java 标准,需要 javac | linkme 链接期已收集,无需 SPI |
| JSR-269 多 round 编译 | Java 注解处理器模型 | linkme 一次性切片,无 round 概念 |
| `@Indexed` 注解元注解传播 | Java 注解可以标注在注解上 | Rust 没有元注解概念,改为"`#[derive(Component)]` 即入索引" |
| `META-INF/spring.components` 物理文件 | JVM 类路径资源 | linkme 切片是物理文件的 1:1 Rust 等价物 |
| `jakarta.*` 注解扫描（`StandardStereotypesProvider`）| JSR 标准 | Rust 无对应概念（无命名空间注解）|
| `package-info` stereotype | Java 特有文件 | Rust 模块路径本身就是 stereotype |
| `MetadataCollector` 三 round 增量合并 | javac 多轮编译 | linkme 一次性收集,无需增量 |
| `SortedProperties` 确定性输出 | Properties 文件对比 | Rust `BTreeMap` 天然有序 |

---

## 九、Rust 侧新增、Spring 无对偶（🆕）

| Rust 模块 | 承载的语义 | 对应 Java 概念 |
|----------|----------|---------------|
| `linkme::distributed_slice` | 链接期组件元数据收集 | JSR-269 注解处理器 |
| `linkme` 自定义 link section | 编译期全收 | `META-INF/spring.components` 物理文件 |
| `LinkedComponentEntry::module_path` | 模块路径作为 stereotype | Spring package name |
| `LinkedComponentIndex::add_entry` | 运行时注入 | Spring 7.0 `addIndex` |
| `LinkedComponentIndex::clear_cache` | 清空运行时注入 | Spring 7.0 `clearCache` |
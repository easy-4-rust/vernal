# spring-context-indexer → vernal-context-indexer 对象名称一致性检查

> 检查时间：2026-07-28
> 基线：Spring Framework 7.0.8 spring-context-indexer（11 个 Java 类 + 2 个 spring-context 运行期消费类 = 13 类）
> vernal-discovery：4 文件 / 3 公开类型 / ~600 行 / 3 测试
> 底盘：linkme（链接期分布式切片） + vernal-beans（IoC 内核）
> **crate 名 100% 镜像 Spring `spring-context-indexer`（连字符 + kebab-case）**
> **目录 100% 镜像 Spring `processor/` 包路径**

## 命名对齐原则（强制要求）

1. **crate 名 100% 镜像 Spring**：保留连字符 `spring-context-indexer` → `vernal-context-indexer`
2. **类型名 100% 与 Spring 一致**：保留 Spring `*Index*` / `*Metadata*` 命名（如 `LinkedComponentIndex` 镜像 `CandidateComponentsIndex`、`LinkedComponentEntry` 镜像 `ItemMetadata`）
3. **目录命名 100% 镜像 Spring 包路径**：Spring `org.springframework.context.index.processor` → `src/index/`
4. **不引入 `Config` 简化后缀**：保留 Spring 全名（如 `CandidateComponentsIndex` 不简化为 `CandidateComponentsIndexConfig`）
5. **链接期 linkme 切片名 100% 镜像 Spring 物理文件**：`META-INF/spring.components` → `LINKED_COMPONENT_INDEX`（静态切片常量）

## 统计汇总

| 维度 | Spring | vernal | 说明 |
|------|--------|--------|------|
| **crate 名完全匹配** | `spring-context-indexer` | `vernal-discovery` → `vernal-context-indexer`（待改） | **❌ 当前错用 `vernal-discovery`，目标改 `vernal-context-indexer`** |
| Java 类总数 | 11（编译期）+ 2（运行期消费）= **13** | — | 排除 `package-info` + SPI 资源 |
| Rust 文件总数 | — | 4（当前）→ 目标 8 | 排除 `mod.rs` / `lib.rs` |
| **类型名完全匹配** | 5 | **0**（当前）+ **5（目标）** | 当前 0 个匹配；目标全部对齐 Spring |
| **目录镜像** | — | **0/1** | 当前无 `index/` 子模块；目标建 `src/index/` 镜像 `processor/` |
| Spring 有 vernal 没有 | 11+2 = **13** | — | 见下文分类 |
| vernal 有 Spring 没有 | — | 1 | `index/TypeHelper`（Rust 类型辅助,Spring TypeHelper 不迁移但子模块仍镜像）|
| Rust 新增、Spring 无对偶 | — | 3 | `add_entry` / `clear_cache` / `runtime_entries` 运行时注入层 |

---

## 一、crate 名一致性（强制 100% 镜像）

### 1.1 crate 名强制映射

| Spring crate | vernal crate（当前）| vernal crate（目标）| 一致性 |
|---|---|---|---|
| `spring-context-indexer` | `vernal-discovery` | **`vernal-context-indexer`** | **❌ 当前错,目标镜像** |
| `spring-context`（运行期消费者）| `vernal-context` | `vernal-context` | ✅ 已对齐 |

### 1.2 crate 名修改清单

| 序号 | 当前（错误）| 应改为（正确）|
|---|---|---|
| 1 | `crates/vernal-discovery/Cargo.toml` `name = "vernal-discovery"` | `crates/vernal-context-indexer/Cargo.toml` `name = "vernal-context-indexer"` |
| 2 | `crates/vernal-discovery/` 目录 | `crates/vernal-context-indexer/` 目录 |
| 3 | `crates/vernal-discovery/src/lib.rs` `#![doc]` 中文 crate doc | `#![doc = "Vernal 链接期组件索引器，镜像 Spring 的 spring-context-indexer"]` |
| 4 | 全部 `vernal-context/` 下 `use vernal_discovery::` | `use vernal_context_indexer::` |
| 5 | `crates/vernal-context/Cargo.toml` `vernal-discovery = { path = ... }` | `vernal-context-indexer = { path = ... }` |
| 6 | 全部 `tests/*.rs` 中 `vernal_discovery::` 路径 | `vernal_context_indexer::` |

---

## 二、目录路径镜像（强制规则）

### 2.1 Spring 包路径 → vernal 子模块路径

| Spring 包 | vernal 子模块 | 现状 | 计划 |
|---|---|---|---|
| `org.springframework.context.index.processor`（编译期 8 类）| `src/index/` | ⬜ 当前**无 `src/index/` 子模块** | ⬜ 待创建 |
| `org.springframework.context.index`（运行期消费 2 类）| `src/linked_component_index.rs`（公开类型）| 🔶 当前 `src/linked_component_catalog.rs` 类型名 `LinkedComponentCatalog` | ⬜ 待改名 `LinkedComponentIndex` |

### 2.2 子模块 `src/index/` 镜像 `processor/`（8 个 Java 类）

| Spring 文件 | vernal 文件 | 一致性 |
|---|---|---|
| `processor/CandidateComponentsIndexer.java`（SPI,弃用）| **不迁移**（linkme 替代）| 🚫 |
| `processor/package-info.java` | `src/index/mod.rs`（中文 doc 注释）| 🔶 形态不同 |
| `processor/ItemMetadata.java` | `src/index/item_metadata.rs` + `src/linked_component_entry.rs`（高层 + 子模块双镜像）| ⬜ 待迁移 |
| `processor/CandidateComponentsMetadata.java` | `src/index/` 子模块作为容器 + `LinkedComponentIndex` 集成 | ⬜ 待迁移 |
| `processor/StereotypesProvider.java`（interface）| **不迁移**（Rust 无 stereotype provider 接口）| 🚫 |
| `processor/StandardStereotypesProvider.java` | **不迁移**（jakarta 命名空间注解是 Java 特有）| 🚫 |
| `processor/IndexedStereotypesProvider.java` | **不迁移**（@Indexed 元注解传播是 Java 特有）| 🚫 |
| `processor/PackageInfoStereotypesProvider.java` | **不迁移**（package-info 是 Java 特有）| 🚫 |
| `processor/MetadataStore.java` | **不迁移**（物理文件 → linkme 切片）| 🚫 |
| `processor/PropertiesMarshaller.java` | `src/index/properties_marshaller.rs` | ⬜ 待迁移 |
| `processor/SortedProperties.java` | `src/index/sorted_properties.rs` | ⬜ 待迁移 |
| `processor/TypeHelper.java` | `src/index/type_helper.rs` | ⬜ 待迁移 |
| `processor/MetadataCollector.java` | **不迁移**（多 round 是 javac 特有）| 🚫 |

---

## 三、类型名称一致性（强制 100% 对齐 Spring `*Index*` 命名）

### 3.1 当前错用清单（必须修正）

| 序号 | 当前错误（vernal-discovery 现状）| 应改为（vernal-context-indexer 目标）| Spring 对偶 |
|---|---|---|---|
| 1 | `src/linked_component_registration.rs` 中的 `pub struct LinkedComponentRegistration` | `src/linked_component_entry.rs` 中的 `pub struct LinkedComponentEntry` | `ItemMetadata` |
| 2 | `src/linked_component_registry.rs` 中的 `pub static LINKED_COMPONENT_REGISTRATIONS` | `src/linked_component_index_slice.rs` 中的 `pub static LINKED_COMPONENT_INDEX` | `META-INF/spring.components`（物理文件 → 链接期切片）|
| 3 | `src/linked_component_catalog.rs` 中的 `pub struct LinkedComponentCatalog` | `src/linked_component_index.rs` 中的 `pub struct LinkedComponentIndex` | `CandidateComponentsIndex`（Spring 运行期消费入口）|
| 4 | `src/linked_component_catalog_error.rs` 中的 `pub enum LinkedComponentCatalogError` | `src/linked_component_index_error.rs` 中的 `pub enum LinkedComponentIndexError` | （无 Spring 对偶,Rust 特有错误类型）|

### 3.2 公开类型名最终清单（5 个 + 1 个子模块辅助）

| vernal 公开类型 | Spring 对偶 | 类型 | 状态 |
|---|---|---|---|
| `vernal_context_indexer::LINKED_COMPONENT_INDEX` | `META-INF/spring.components`（物理文件）| `&'static [LinkedComponentEntry]`（链接期切片）| ⬜ 待改名 |
| `vernal_context_indexer::LinkedComponentEntry` | `ItemMetadata` | `pub struct` | ⬜ 待改名 |
| `vernal_context_indexer::LinkedComponentIndex` | `CandidateComponentsIndex` | `pub struct`（目录）| ⬜ 待改名 |
| `vernal_context_indexer::LinkedComponentIndexError` | （无对偶,Rust 特有）| `pub enum` | ⬜ 待改名 |
| `vernal_context_indexer::index::ItemMetadata`（子模块）| `ItemMetadata`（镜像 Spring `processor/ItemMetadata`）| `pub struct`（子模块镜像版）| ⬜ 待创建 |
| `vernal_context_indexer::linkme` re-export | （无对偶,Rust 特有）| `pub use linkme;` | ✅ 保留 |

### 3.3 配置类命名（spring-context-indexer 无 `@Configuration` 类）

spring-context-indexer 没有 Configuration 类（无 `*Configuration` 后缀类型）。本模块无需检查此条。

---

## 四、文件名一致性（snake_case,镜像 Spring）

| Spring 文件 | vernal 文件（目标）| 规则 |
|---|---|---|
| `CandidateComponentsIndexer.java` | **不迁移**（JSR-269 SPI 不替代）| — |
| `ItemMetadata.java` | `item_metadata.rs`（子模块） + `linked_component_entry.rs`（高层镜像）| snake_case |
| `CandidateComponentsMetadata.java` | `linked_component_index.rs`（集成到 `LinkedComponentIndex`）| snake_case |
| `MetadataStore.java` | **不迁移**（物理文件 → linkme 切片）| — |
| `PropertiesMarshaller.java` | `properties_marshaller.rs`（子模块）| snake_case |
| `SortedProperties.java` | `sorted_properties.rs`（子模块）| snake_case |
| `TypeHelper.java` | `type_helper.rs`（子模块）| snake_case |
| `MetadataCollector.java` | **不迁移**（多 round 是 javac 特有）| — |
| `package-info.java` | `mod.rs`（子模块入口）| snake_case |
| `CandidateComponentsIndex.java`（spring-context）| `linked_component_index.rs` | snake_case |
| `CandidateComponentsIndexLoader.java`（spring-context）| `linked_component_index_slice.rs`（同 linkme 切片文件）| snake_case |

---

## 五、方法名一致性（snake_case）

| Spring 方法 | Rust 方法 | 规则 |
|---|---|---|
| `getSupportedOptions()` | **不迁移** | JSR-269 特有 |
| `getSupportedAnnotationTypes()` | **不迁移** | JSR-269 特有 |
| `getSupportedSourceVersion()` | **不迁移** | JSR-269 特有 |
| `process(Set, RoundEnvironment)` | **不迁移** | JSR-269 特有 |
| `getStereotypes(Element)` | **不迁移** | stereotype provider 概念不替代 |
| `collectStereotypesOnAnnotations(Element)` | **不迁移** | @Indexed 传播不替代 |
| `collectStereotypesOnTypes(Element)` | **不迁移** | 类层级回溯不替代 |
| `readMetadata()` | **不迁移** | 物理文件 IO 不替代 |
| `writeMetadata(metadata)` | **不迁移** | 物理文件 IO 不替代 |
| `PropertiesMarshaller#write(metadata, outputStream)` | `properties_marshaller::serialize(index) -> BTreeMap` | camelCase → snake_case |
| `PropertiesMarshaller#read(inputStream) : metadata` | `properties_marshaller::deserialize(map) -> LinkedComponentIndex` | 同上 |
| `TypeHelper#getType(Element)` | `type_helper::type_id_to_string::<T>()` | camelCase → snake_case |
| `TypeHelper#getSuperClass(Element)` | **不迁移** | indexer 不做类型层级 |
| `TypeHelper#getDirectInterfaces(Element)` | **不迁移** | 同上 |
| `TypeHelper#getAllAnnotationMirrors(Element)` | **不迁移** | 注解概念不替代 |
| `CandidateComponentsIndex#registerScan(String...)` | `LinkedComponentIndex::scan(base_packages)` | camelCase → snake_case |
| `CandidateComponentsIndex#hasScannedPackage(String)` | `LinkedComponentIndex::has(base_package)` | 同上 |
| `CandidateComponentsIndex#registerCandidateType(String, String...)` | `LinkedComponentIndex::add_entry(entry)` | 同上 |
| `CandidateComponentsIndex#getRegisteredScans()` | `LinkedComponentIndex::base_packages()` | 同上 |
| `CandidateComponentsIndex#getRegisteredStereotypes()` | `LinkedComponentIndex::stereotypes()` | 同上 |
| `CandidateComponentsIndex#getCandidateTypes(String, String)` | `LinkedComponentIndex::get(base_package, stereotype)` | 同上 |
| `CandidateComponentsIndexLoader#loadIndex(ClassLoader)` | `LinkedComponentIndex::scan_all()`（在 `VernalApplicationBuilder` 调用）| 同上 |
| `CandidateComponentsIndexLoader#addIndex(ClassLoader, CandidateComponentsIndex)` | `LinkedComponentIndex::add_entry(...)` | 同上 |
| `CandidateComponentsIndexLoader#clearCache()` | `LinkedComponentIndex::clear_cache()` | 同上 |
| `Entry#match(String basePackage)` | `LinkedComponentEntry::matches(base_package)` | camelCase → snake_case |
| `Entry#type / packageName` 字段 | `LinkedComponentEntry::name / module_path` 字段 | camelCase → snake_case |

---

## 六、字段名一致性（snake_case）

| Java 字段 | Rust 字段 |
|---|---|
| `ItemMetadata.type` | `LinkedComponentEntry::name`（Rust 改用 `name` 含模块路径）|
| `ItemMetadata.stereotypes` | `LinkedComponentEntry::stereotypes`（已一致）|
| `CandidateComponentsMetadata.items` | `LinkedComponentIndex::entries`（已一致）|
| `CandidateComponentsIndex.registeredScans` | `LinkedComponentIndex::base_packages`（语义调整）|
| `CandidateComponentsIndex.index` (MultiValueMap) | `LinkedComponentIndex::runtime_entries`（运行时注入层）|
| `CandidateComponentsIndex.complete` | **不迁移**（用两个 Vec 表达:静态 + 运行时）|
| `Entry.type` | `LinkedComponentEntry::name` |
| `Entry.packageName` | `LinkedComponentEntry::module_path` |
| `MetadataStore.environment` | **不迁移**（物理文件 IO 不替代）|

---

## 七、目录结构一致性（强制 100% 镜像）

### 7.1 包路径 → 子模块路径（最终强制映射）

| Spring 包 | vernal 子模块 / 文件 | 中间层保留 |
|---|---|---|
| `org.springframework.context.index.processor`（编译期 8 类）| `src/index/`（5 个子模块文件）| 0 层 |
| `org.springframework.context.index`（运行期 2 类）| `src/linked_component_index.rs` + `src/linked_component_index_slice.rs`（顶层 2 文件）| 0 层 |
| `META-INF/spring.components`（物理索引文件）| `src/linked_component_index_slice.rs::LINKED_COMPONENT_INDEX`（链接期切片）| vernal 特有 |
| `META-INF/services`（SPI 注册）| **不迁移** | vernal 用 linkme,无需 SPI |

**关键约束**：
- 2 个 spring-context-indexer 子模块目录（`processor/` + `index/`）全部镜像,不抽取业务词根
- `src/index/` 子模块的 5 个文件(`item_metadata.rs` / `properties_marshaller.rs` / `sorted_properties.rs` / `type_helper.rs` / `mod.rs`)严格对应 Spring `processor/` 的 5 个保留类(其余 3 个不迁移)
- 接受 `src/index/` + `src/linked_component_index.rs` 双文件形态(子模块镜像 + 顶层公开类型)

### 7.2 最终目录布局

```
crates/vernal-context-indexer/
├── Cargo.toml
├── src/
│   ├── lib.rs                        # crate 入口 + 5 个公开类型 re-export
│   ├── index/                        # 镜像 Spring processor/ 包
│   │   ├── mod.rs                    # 镜像 package-info.java
│   │   ├── item_metadata.rs          # 镜像 ItemMetadata.java（子模块版）
│   │   ├── properties_marshaller.rs  # 镜像 PropertiesMarshaller.java
│   │   ├── sorted_properties.rs      # 镜像 SortedProperties.java
│   │   └── type_helper.rs            # 镜像 TypeHelper.java
│   ├── linked_component_entry.rs     # 镜像 ItemMetadata.java（高层版）
│   ├── linked_component_index.rs     # 镜像 CandidateComponentsIndex.java
│   ├── linked_component_index_error.rs  # Rust 特有错误类型
│   └── linked_component_index_slice.rs  # 镜像 META-INF/spring.components
└── tests/
    ├── linked_component_index.rs     # 端到端测试
    ├── index_compile_fail.rs         # trybuild 编译失败测试
    └── index_support/                # fixture 模块
        ├── mod.rs
        ├── database.rs
        ├── order_service.rs
        ├── audit_worker.rs
        ├── manual_component.rs
        └── duplicate_entries.rs
```

---

## 八、业务逻辑一致性检查

### ✅ 已对齐（仅类型名层面）
- `LinkedComponentCatalog::scan(base_packages)` 行为对齐 Spring `CandidateComponentsIndex#registerScan`
- `LinkedComponentCatalog::install(&mut RegistryBuilder)` 行为对齐 Spring `ClassPathBeanDefinitionScanner#doScan`
- 链接期 linkme 切片自动收集所有 `#[derive(Component)]` 结构体
- `starts_with` 模块路径前缀匹配(对齐 Spring `AntPathMatcher(".")` 精确匹配分支)

### 🔶 形态不同（待迁移时统一）
- Spring `CandidateComponentsIndex` 是 class(具体类),vernal 用 struct
- Spring `ItemMetadata` 是 class,vernal 高层用 struct + 子模块用 struct（双镜像）
- Spring `META-INF/spring.components` 是物理 Properties 文件,vernal 是链接期 linkme 切片
- Spring `PropertiesMarshaller` 读写 `Properties`,vernal 读写 `BTreeMap<String, Vec<String>>`
- Spring `SortedProperties` 自实现排序,vernal 用 `BTreeMap`(天然有序)
- Spring `Entry.match(basePackage)` 用 `AntPathMatcher(".")` 支持 `*` / `**`,vernal 用 `starts_with` 前缀匹配(MVP 不支持 Ant 通配)
- Spring `complete` 字段(true=文件加载,false=编程注册),vernal 用两个 Vec 表达(静态切片 + 运行时注入)

### ⬜ 未实现的语义
- `LinkedComponentIndex::add_entry(...)` 运行时注入(对标 Spring 7.0 `addIndex`)
- `LinkedComponentIndex::clear_cache()` 清空运行时注入(对标 Spring 7.0 `clearCache`)
- `LinkedComponentIndex::has(base_package)` 路径包含判定
- `LinkedComponentIndex::get(base_package, stereotype)` 候选类型查询
- `LinkedComponentIndex::stereotypes()` 已注册 stereotype 集合
- `LinkedComponentIndex::base_packages()` 已注册基包集合
- `properties_marshaller::serialize / deserialize` 序列化往返

### 🚫 不实现（Java 特有）
- JSR-269 `javax.annotation.processing.Processor` SPI
- 多 round 编译（`RoundEnvironment`）
- `@Indexed` 元注解传播
- `jakarta.*` 命名空间注解扫描
- `package-info.java` stereotype
- `MetadataCollector` 多 round 增量合并
- `MetadataStore` 物理文件 IO
- `ClassLoader` 类加载器级缓存
- `AntPathMatcher` `*` / `**` 通配符（MVP 不支持）

---

## 九、与 vernal-beans / vernal-context 的一致性检查

### 9.1 与 vernal-beans 的契约

| vernal-beans 公开 API | vernal-context-indexer 用法 | 一致性 |
|---|---|---|
| `ComponentDefinition` | `LinkedComponentEntry::component_definition()` 返回 | ✅ 已用 |
| `RegistryBuilder` | `LinkedComponentIndex::install(&mut RegistryBuilder)` 接收 | ✅ 已用 |
| `DefinitionError` | `LinkedComponentIndex::install` 返回错误 | ✅ 已用 |
| `ComponentKey` | `LinkedComponentEntry::key()` 内部使用 | ✅ 已用 |
| `BuildPlan` | 不使用(indexer 不做拓扑排序)| ✅ 不需要 |
| `Container` | 不使用(indexer 不做实例解析)| ✅ 不需要 |

### 9.2 与 vernal-context 的契约

| vernal-context 公开 API | vernal-context-indexer 用法 | 一致性 |
|---|---|---|
| `VernalApplicationBuilder::scan(base_packages)` | 委托 `vernal_context_indexer::LinkedComponentIndex::scan(base_packages)` | ✅ 已用 |
| `Lifecycle` trait | 不使用(indexer 不涉及生命周期)| ✅ 不需要 |
| `ApplicationEnvironment` | 不使用(indexer 不涉及属性)| ✅ 不需要 |

### 9.3 与 vernal-macros 的契约

| vernal-macros 派生宏 | vernal-context-indexer 用法 | 一致性 |
|---|---|---|
| `#[derive(Component)]` | 宏生成 `static __DI_META_X: LinkedComponentEntry` 自动归入 `LINKED_COMPONENT_INDEX` | ✅ 已用 |
| `#[derive(ConfigurationProperties)]` | 不使用(indexer 不涉及属性)| ✅ 不需要 |

---

## 十、当前命名错误清单（必须修正）

| 序号 | 当前错误 | 应改为 |
|---|---|---|
| 1 | `crates/vernal-discovery/` | `crates/vernal-context-indexer/` |
| 2 | `crates/vernal-discovery/Cargo.toml` `name = "vernal-discovery"` | `name = "vernal-context-indexer"` |
| 3 | `src/linked_component_registration.rs` 中的 `pub struct LinkedComponentRegistration` | `src/linked_component_entry.rs` 中的 `pub struct LinkedComponentEntry` |
| 4 | `src/linked_component_registry.rs` 中的 `pub static LINKED_COMPONENT_REGISTRATIONS` | `src/linked_component_index_slice.rs` 中的 `pub static LINKED_COMPONENT_INDEX` |
| 5 | `src/linked_component_catalog.rs` 中的 `pub struct LinkedComponentCatalog` | `src/linked_component_index.rs` 中的 `pub struct LinkedComponentIndex` |
| 6 | `src/linked_component_catalog_error.rs` 中的 `pub enum LinkedComponentCatalogError` | `src/linked_component_index_error.rs` 中的 `pub enum LinkedComponentIndexError` |
| 7 | `src/lib.rs` `pub use linked_component_catalog::LinkedComponentCatalog;` | `pub use linked_component_index::LinkedComponentIndex;` |
| 8 | `src/lib.rs` `pub use linked_component_registration::LinkedComponentRegistration;` | `pub use linked_component_entry::LinkedComponentEntry;` |
| 9 | `src/lib.rs` `pub use linked_component_catalog_error::LinkedComponentCatalogError;` | `pub use linked_component_index_error::LinkedComponentIndexError;` |
| 10 | `src/lib.rs` `pub use linked_component_registry::LINKED_COMPONENT_REGISTRATIONS;` | `pub use linked_component_index_slice::LINKED_COMPONENT_INDEX;` |
| 11 | 全部 `vernal-context/Cargo.toml` 与源码 `use vernal_discovery::` | `use vernal_context_indexer::` |
| 12 | `tests/linked_component_discovery.rs` | `tests/linked_component_index.rs` |
| 13 | `tests/discovery_compile_fail.rs` | `tests/index_compile_fail.rs` |
| 14 | `tests/discovery_support/` | `tests/index_support/` |
| 15 | `src/lib.rs` `#![doc]` 顶部 crate doc | `#![doc = "Vernal 链接期组件索引器，对标 Spring 的 spring-context-indexer"]` |
| 16 | `src/linked_component_catalog.rs` `LinkedComponentCatalog::scan_all` 返回类型 | `LinkedComponentIndex::scan_all` 返回 `Self`（类型名替换）|

---

## 十一、与 vernal-expression 的一致性检查的对比

| 维度 | vernal-expression | vernal-context-indexer |
|---|---|---|
| 命名规范来源 | vernal-expression 4 份文档 | **vernal-context-indexer 4 份文档（本任务产出）** |
| crate 名镜像 | `spring-expression` → `vernal-expression` | **`spring-context-indexer` → `vernal-context-indexer`** |
| 目录镜像 | 4 个 expression 子包 | **1 个 index 子模块（processor 镜像）+ 4 个顶层文件** |
| 公开类型数 | 87+ 类 | **5 类** |
| 业务逻辑一致性 | 核心 AST 节点 100% | **核心 5 类镜像 + 8 类不迁移** |
| 中文 doc 注释覆盖率 | 100% | **目标 100%** |

---

## 十二、结论

| 维度 | 完成前 | 完成后 |
|------|--------|--------|
| **crate 名一致性** | ❌ `vernal-discovery`（错） | ✅ `vernal-context-indexer`（100% 镜像 Spring）|
| 类型名称一致性 | 0/13 = 0%（全部错用）| 5/13 = 38%（核心 5 类对齐 Spring）|
| 文件名一致性 | 0/11 = 0%（路径全错）| 8/11 = 73%（镜像 + 不迁移）|
| **目录镜像** | 0/1 = 0%（无 `src/index/`）| 1/1 = 100%（`src/index/` 镜像 `processor/`）|
| 业务逻辑一致性 | 60%（仅基础 catalog 行为）| **95%+**（5 公开类型镜像 Spring + 4 不迁移 + 3 🆕 运行时注入）|

### 完成 P0 + P1 任务后：

- **crate 名一致性**：`vernal-discovery` → `vernal-context-indexer`（**100%**）
- **类型名称一致性**：5/13 = **38%**（核心 5 类对齐 Spring，其余 8 类由 linkme + vernal-beans 等价承担）
- **目录路径镜像**：1/1 = **100%**（`src/index/` 镜像 `processor/`）
- **业务逻辑一致性**：从 60% 提升到 **95%+**（保留 vernal-discovery 既有能力 + 补充 Spring 7.0 编程注入 API）

### 下一步行动

1. **P0（紧急,路线图 S1）**：crate 重命名 `vernal-discovery` → `vernal-context-indexer`,全工作区 `use` 路径替换
2. **P0（路线图 S3）**：公开类型 rename 5 个（`LinkedComponentRegistration` → `LinkedComponentEntry` 等）
3. **P0（路线图 S2）**：创建 `src/index/` 子模块,镜像 Spring `processor/` 包路径
4. **P0（路线图 S4）**：实现 Spring 7.0 编程注入 API（`add_entry` / `clear_cache`）
5. **P0（路线图 S5）**：测试改名 + 全工作区 import 替换
6. **P1（路线图 S6）**：中文 doc 注释覆盖率 100% + 文档收尾
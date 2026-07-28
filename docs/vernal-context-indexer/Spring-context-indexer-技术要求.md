# vernal-context-indexer 技术交接文档

> **对标**: Spring Framework `spring-context-indexer`（链接期组件索引器）
> **crate**: `vernal-context-indexer`
> **现状**: 11 文件 / ~1371 行 / 5 子模块
> **edition**: 2024 / rustc 1.88.0 / MSRV 1.88.0
> **状态**: experimental
> **最后更新**: 2026-07-28

---

## 一、定位

vernal-context-indexer 是 Vernal 框架的**链接期组件索引器**，对标 Spring Framework 的 `spring-context-indexer` 模块。
它提供 vernal 框架的链接期组件索引机制：通过 `linkme` 分布式切片收集所有 `#[derive(Component)]` 的结构体，
替代 Spring 的 APT（Annotation Processing Tool）编译期索引文件生成。

**设计原则**:

- 零 `unsafe`（`#![forbid(unsafe_code)]`）
- 所有公开类型必须满足 `Send + Sync + 'static`
- 链接期事实只读：`LINKED_COMPONENT_INDEX` 是 `&'static` 切片，不可修改
- 运行时注入与链接期分离：`runtime_entries` 优先级高于 `static_entries`
- 确定性排序：按 `(module_path, name)` 排序，消除链接器和目标平台对顺序的影响

**与 Spring Context Indexer 的对应关系**:

| Spring Context Indexer | vernal-context-indexer | 说明 |
|---|---|---|
| `META-INF/spring.components` 物理文件 | `LINKED_COMPONENT_INDEX` 链接期切片 | 物理文件→分布式切片 |
| `CandidateComponentsIndex` | `LinkedComponentIndex` | 运行期入口 |
| `ClassPathScanningCandidateComponentProvider` | `LinkedComponentIndex::scan(base_packages)` | 包路径过滤 |
| `ItemMetadata` | `LinkedComponentEntry` + `index::ItemMetadata` | 索引记录 |
| `CandidateComponentsMetadata` | `index::CandidateComponentsMetadata` | 元数据容器 |
| `PropertiesMarshaller` | `index::PropertiesMarshaller` | 序列化/反序列化 |
| `SortedProperties` | `index::SortedProperties` | 确定性格式化 |
| `TypeHelper` | `index::TypeHelper` | 类型辅助 |
| `@Indexed` 元注解传播 | linkme 分布式切片 | 自动收集 |
| JSR-269 APT 处理器 | `vernal-macros generate_discovery_registration` | 过程宏生成 |

---

## 二、组件清单详细

### 2.1 链接期组件索引切片

**对标**: `META-INF/spring.components` + `CandidateComponentsIndexLoader`

#### Spring 机制

```java
// Spring 通过 JSR-269 APT 在编译期生成 META-INF/spring.components 文件
// 文件格式：com.example.UserService=org.springframework.stereotype.Component
// 运行时通过 CandidateComponentsIndexLoader 加载

public class CandidateComponentsIndexLoader {
    public static final String COMPONENTS_RESOURCE_LOCATION = "META-INF/spring.components";
    // 从 classpath 加载所有 spring.components 文件
}
```

#### Rust 实现

```rust
// crates/vernal-context-indexer/src/linked_component_index_slice.rs

/// 链接期组件索引切片常量。
///
/// 对标 Spring `META-INF/spring.components` 物理文件：
/// - Spring：classpath 上的物理 Properties 文件
/// - Vernal：链接期 linkme 分布式切片（`&'static`）
///
/// 每个 `#[derive(Component)]` 的结构体由 `vernal-macros` 自动生成一个
/// `#[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]` 的静态条目。
#[linkme::distributed_slice]
pub static LINKED_COMPONENT_INDEX: [LinkedComponentEntry] = [..];
```

#### linkme 分布式切片机制

```text
编译期：
  #[derive(Component)]
  struct UserService { ... }

  ↓ vernal-macros generate_discovery_registration 展开为：

  #[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]
  static __VERNAL_LINKED_COMPONENT_ENTRY_UserService: LinkedComponentEntry =
      LinkedComponentEntry::new(
          module_path!(),                                    // "my_app::service"
          concat!(module_path!(), "::", "UserService"),      // "my_app::service::UserService"
          &["Component"],                                    // stereotypes
          || ComponentDefinition::new::<UserService>(),      // 定义工厂
      );

链接期：
  链接器将所有 `__VERNAL_LINKED_COMPONENT_ENTRY_*` 静态项拼成连续切片
  `LINKED_COMPONENT_INDEX` 启动时即可用
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| linkme 0.3.37 | 分布式切片依赖 | 满足 |
| `&'static` 只读 | 链接期事实不可修改 | 满足 |
| 确定性顺序 | 按 `(module_path, name)` 排序 | 满足（`scan()` 中排序） |
| 跨平台 | linkme 支持 Linux/macOS/Windows | 满足 |

---

### 2.2 链接期组件条目

**对标**: `org.springframework.context.index.processor.ItemMetadata`

#### Spring API

```java
public class ItemMetadata {
    private final String type;
    private final Set<String> stereotypes;

    public ItemMetadata(String type, Set<String> stereotypes);
    public String getType();
    public Set<String> getStereotypes();
}
```

#### Rust 实现

```rust
// crates/vernal-context-indexer/src/linked_component_entry.rs

/// 一个由过程宏提交到链接期分布式切片的不可变组件定义条目。
///
/// 对应 Spring `ItemMetadata`：
/// - `module_path` + `name` 对应 `ItemMetadata.type`
/// - `stereotypes` 对应 `ItemMetadata.stereotypes`
///
/// stereotype 字段语义：
/// - Spring：`@Indexed` 元注解传播 + `jakarta.*` 命名空间注解 + `package-info`
/// - Vernal：`#[component(stereotype = "...")]` 属性显式指定
/// - 默认：`&["Component"]`（对标 Spring `@Component` 自带 `@Indexed`）
#[derive(Clone, Debug)]
pub struct LinkedComponentEntry {
    module_path: &'static str,                    // "my_app::service"
    name: &'static str,                           // "my_app::service::UserService"
    stereotypes: &'static [&'static str],         // &["Component"]
    definition: fn() -> ComponentDefinition,      // 定义工厂函数
}

impl LinkedComponentEntry {
    pub const fn new(
        module_path: &'static str,
        name: &'static str,
        stereotypes: &'static [&'static str],
        definition: fn() -> ComponentDefinition,
    ) -> Self;

    pub const fn new_default(
        module_path: &'static str,
        name: &'static str,
        definition: fn() -> ComponentDefinition,
    ) -> Self;  // stereotypes = &["Component"]

    pub const fn module_path(&self) -> &'static str;
    pub const fn name(&self) -> &'static str;
    pub const fn stereotypes(&self) -> &[&'static str];
    pub fn has_stereotype(&self, stereotype: &str) -> bool;
    pub fn stereotypes_iter(&self) -> impl Iterator<Item = &'static str> + '_;
    pub fn component_definition(&self) -> ComponentDefinition;
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| `const fn` 构造器 | 可在 `static` 上下文中调用 | 满足 |
| `&'static` 字段 | 链接期不可变数据 | 满足 |
| Send + Sync | `LinkedComponentEntry` 满足 | 满足 |
| Clone | 支持克隆 | 满足（`#[derive(Clone)]`） |

---

### 2.3 链接期组件索引

**对标**: `org.springframework.context.index.CandidateComponentsIndex`

#### Spring API

```java
public class CandidateComponentsIndex {
    public CandidateComponentsIndex(Map<String, List<String>> index);
    public Set<String> getCandidateTypes(String basePackage, String stereotype);
    public boolean hasScannedPackage(String packageName);
    public Set<String> getRegisteredStereotypes();
}
```

#### Rust 实现

```rust
// crates/vernal-context-indexer/src/linked_component_index.rs

/// 一次模块路径扫描得到的确定性、只读组件注册索引。
///
/// 内部存储：
/// - `static_entries`：链接期 `LINKED_COMPONENT_INDEX` 切片的引用副本
/// - `runtime_entries`：运行时通过 `add_entry()` 注入的额外条目
/// - `base_packages`：注册时的基包路径集合
/// - `complete`：true = 链接期完整加载；false = 纯运行时注册
#[derive(Clone, Debug)]
pub struct LinkedComponentIndex {
    static_entries: Vec<&'static LinkedComponentEntry>,
    runtime_entries: Vec<LinkedComponentEntry>,
    base_packages: BTreeSet<String>,
    complete: bool,
}

impl LinkedComponentIndex {
    // 扫描方法
    pub fn scan<I, S>(base_packages: I) -> Result<Self, LinkedComponentIndexError>
    where I: IntoIterator<Item = S>, S: AsRef<str>;
    pub fn scan_all() -> Result<Self, LinkedComponentIndexError>;
    pub fn empty() -> Self;
    pub fn merge(indices: &[&Self]) -> Result<Self, LinkedComponentIndexError>;

    // 运行时注入
    pub fn add_entry(&mut self, entry: LinkedComponentEntry);
    pub fn clear_cache(&mut self);

    // 查询方法
    pub fn get(&self, base_package: &str, stereotype: &str) -> BTreeSet<&'static str>;
    pub fn has(&self, package_name: &str) -> bool;
    pub fn base_packages(&self) -> &BTreeSet<String>;
    pub fn stereotypes(&self) -> BTreeSet<&'static str>;
    pub fn is_complete(&self) -> bool;

    // 安装方法
    pub fn install<'registry>(
        &self, registry: &'registry mut RegistryBuilder
    ) -> Result<&'registry mut RegistryBuilder, DefinitionError>;
    pub fn component_definitions(&self)
        -> impl Iterator<Item = ComponentDefinition> + '_;

    // 迭代方法
    pub fn iter_entries(&self) -> impl Iterator<Item = EntryRef<'_>>;
    pub fn static_entries(&self) -> &[&'static LinkedComponentEntry];
    pub fn runtime_entries(&self) -> &[LinkedComponentEntry];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    // 注册方法
    pub fn register_scan<I, S>(&mut self, base_packages: I)
        -> Result<(), LinkedComponentIndexError>
    where I: IntoIterator<Item = S>, S: AsRef<str>;
}

/// 索引条目引用枚举。
#[derive(Clone, Debug)]
pub enum EntryRef<'a> {
    Static(&'static LinkedComponentEntry),
    Runtime(&'a LinkedComponentEntry),
}

impl<'a> EntryRef<'a> {
    pub fn module_path(&self) -> &'static str;
    pub fn name(&self) -> &'static str;
    pub fn stereotypes(&self) -> &[&'static str];
    pub fn has_stereotype(&self, stereotype: &str) -> bool;
}
```

#### 扫描语义

```text
Spring: @ComponentScan(basePackages = {"com.example.web"})
  ↓ 对标
Vernal: LinkedComponentIndex::scan(["my_app::web"])
```

| Spring 行为 | vernal 行为 |
|---|---|
| classpath 扫描 `@Component` 类 | linkme 切片过滤 `module_path` 前缀 |
| `AntPathMatcher(".")` 路径匹配 | `module_path.starts_with(base_package)` |
| 按 `type` 排序 | 按 `(module_path, name)` 排序 |
| 重复检测 | `validate_no_duplicates()` |

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | 无 trait（纯结构体） | N/A |
| Send + Sync | `LinkedComponentIndex` 满足 | 满足 |
| Clone | 支持克隆 | 满足 |
| 确定性排序 | 按 `(module_path, name)` 排序 | 满足（测试覆盖） |
| 重复检测 | 同名条目返回 `DuplicateEntry` 错误 | 满足（测试覆盖） |
| 空选择检测 | 空 `base_packages` 返回 `EmptySelection` 错误 | 满足（测试覆盖） |

#### 待补齐

- [ ] `LinkedComponentIndexBuilder`（构建器模式）
- [ ] 并行扫描（`rayon` 并行过滤）

---

### 2.4 链接期组件索引错误

**对标**: Spring `processor/` 子模块中各种 `IllegalStateException` / `IllegalArgumentException`

#### Rust 实现

```rust
// crates/vernal-context-indexer/src/linked_component_index_error.rs

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkedComponentIndexError {
    /// 调用方没有选择任何分组。
    EmptySelection,
    /// 分组名为空或包含空白、控制字符。
    InvalidGroup { group: Arc<str> },
    /// 请求的分组没有任何链接期条目。
    MissingGroup { group: Arc<str> },
    /// 手工提交的稳定声明名为空或包含空白、控制字符。
    InvalidEntry { group: Arc<str>, name: Arc<str> },
    /// 同一分组出现两个相同稳定声明名。
    DuplicateEntry { group: Arc<str>, name: Arc<str> },
}

impl fmt::Display for LinkedComponentIndexError { /* 稳定诊断输出 */ }
impl Error for LinkedComponentIndexError {}
```

#### 错误语义

| 错误变体 | 触发条件 | Spring 等价 |
|---|---|---|
| `EmptySelection` | `scan([])` 空迭代器 | `IllegalArgumentException` |
| `InvalidGroup` | 包名含空白/控制字符 | `IllegalArgumentException` |
| `MissingGroup` | 请求的包无条目 | `IllegalStateException` |
| `InvalidEntry` | 条目名含空白/控制字符 | `IllegalArgumentException` |
| `DuplicateEntry` | 同名条目冲突 | `IllegalStateException` |

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| `#[non_exhaustive]` | 未来可扩展 | 满足 |
| Display 脱敏 | 不暴露组件字段值 | 满足 |
| Send + Sync | `LinkedComponentIndexError` 满足 | 满足 |
| Clone | 支持克隆 | 满足 |

---

### 2.5 index 子模块（Spring processor/ 镜像）

**对标**: `org.springframework.context.index.processor`

#### Spring 类映射

| Spring Java 类 | Vernal Rust 文件 | 迁移状态 |
|---|---|---|
| `CandidateComponentsIndexer` | 不迁移（linkme 替代 JSR-269） | N/A |
| `CandidateComponentsMetadata` | `candidate_components_metadata.rs` | 完成 |
| `ItemMetadata` | `item_metadata.rs` | 完成 |
| `MetadataCollector` | 不迁移（多 round 是 javac 特有） | N/A |
| `MetadataStore` | 不迁移（物理文件→linkme 切片） | N/A |
| `PropertiesMarshaller` | `properties_marshaller.rs` | 完成 |
| `SortedProperties` | `sorted_properties.rs` | 完成 |
| `StereotypesProvider` | 不迁移（Rust 无 stereotype provider 接口） | N/A |
| `IndexedStereotypesProvider` | 不迁移（@Indexed 元注解传播是 Java 特有） | N/A |
| `StandardStereotypesProvider` | 不迁移（jakarta.* 命名空间注解是 Java 特有） | N/A |
| `PackageInfoStereotypesProvider` | 不迁移（package-info.java 是 Java 特有） | N/A |
| `TypeHelper` | `type_helper.rs` | 完成 |

#### 2.5.1 ItemMetadata

```rust
// crates/vernal-context-indexer/src/index/item_metadata.rs

/// 索引中的一条记录。
///
/// 对应 Spring `ItemMetadata`：
/// - `type` 定义候选目标的标识（通常为完全限定类名）
/// - `stereotypes` 是可用于检索候选的"标记"
///
/// 与 `LinkedComponentEntry` 的关系：
/// - `LinkedComponentEntry` 是 vernal 链接期 + 运行时通用的条目
/// - `ItemMetadata` 是 Spring `processor/` 子模块的高层镜像
/// - 两者字段完全对应（type + stereotypes）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemMetadata {
    r#type: String,
    stereotypes: BTreeSet<String>,
}

impl ItemMetadata {
    pub fn new(r#type: impl Into<String>, stereotypes: BTreeSet<String>) -> Self;
    pub fn get_type(&self) -> &str;
    pub fn get_stereotypes(&self) -> &BTreeSet<String>;
}
```

#### 2.5.2 CandidateComponentsMetadata

```rust
// crates/vernal-context-indexer/src/index/candidate_components_metadata.rs

/// 候选组件元数据容器。
///
/// 对应 Spring `CandidateComponentsMetadata`：
/// - 内部维护 `Vec<ItemMetadata>`，按插入顺序保存
/// - `add()` 追加条目
/// - `get_items()` 返回只读视图
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandidateComponentsMetadata {
    items: Vec<ItemMetadata>,
}

impl CandidateComponentsMetadata {
    pub fn new() -> Self;
    pub fn add(&mut self, item: ItemMetadata);
    pub fn get_items(&self) -> &[ItemMetadata];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn types(&self) -> BTreeSet<&str>;       // 所有 candidate types（去重）
    pub fn stereotypes(&self) -> BTreeSet<&str>; // 所有 stereotypes（去重）
}

impl Display for CandidateComponentsMetadata { /* "CandidateComponentsMetadata{items=[...]}" */ }
```

#### 2.5.3 PropertiesMarshaller

```rust
// crates/vernal-context-indexer/src/index/properties_marshaller.rs

/// 索引元数据序列化器。
///
/// 对应 Spring `PropertiesMarshaller`：
/// - `write(metadata, writer)`：序列化为 `type=stereotype1,stereotype2,...`
/// - `read(reader)`：反序列化回 `CandidateComponentsMetadata`
///
/// 格式：每行 `type=stereotype1,stereotype2,...`（stereotype 按字典序，逗号分隔）
pub struct PropertiesMarshaller;

impl PropertiesMarshaller {
    pub fn write(metadata: &CandidateComponentsMetadata, writer: &mut impl Write)
        -> io::Result<()>;
    pub fn read(reader: &mut impl Read) -> io::Result<CandidateComponentsMetadata>;
}
```

#### 2.5.4 SortedProperties

```rust
// crates/vernal-context-indexer/src/index/sorted_properties.rs

/// 按 key 字母序排序的 Properties。
///
/// 对应 Spring `SortedProperties`：
/// - Spring：继承 `java.util.Properties`，重写 `store` / `keySet`
/// - Vernal：直接用 `BTreeMap<String, String>`（天然有序）
pub const EOL: &str = "\n";  // 固定换行符，跨平台确定性

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SortedProperties {
    inner: BTreeMap<String, String>,
    omit_comments: bool,
}

impl SortedProperties {
    pub fn new(omit_comments: bool) -> Self;
    pub fn from_map(map: BTreeMap<String, String>, omit_comments: bool) -> Self;
    pub fn omit_comments(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>);
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)>;
    pub fn store(&self, writer: &mut impl Write) -> io::Result<()>;
    pub fn load(&mut self, reader: &mut impl Read) -> io::Result<()>;
}
```

#### 2.5.5 TypeHelper

```rust
// crates/vernal-context-indexer/src/index/type_helper.rs

/// 类型工具。
///
/// 对应 Spring `TypeHelper`：
/// - `getType(Element)` → `get_type::<T>()`（使用 `std::any::type_name`）
/// - `getSuperClass(Element)` → `super_class_str(parent)`（用户传入描述）
/// - `getDirectInterfaces(Element)` → `direct_interfaces_str(...)`
/// - `getAllAnnotationMirrors(Element)` → `safe_annotations(input)`
/// - `isJakartaAnnotation(type)` → `is_jakarta_annotation(type)`
pub struct TypeHelper;

impl TypeHelper {
    pub const fn new() -> Self;
    pub fn get_type<T: ?Sized>() -> String;                    // type_name::<T>()
    pub fn to_nested_format(type_name_str: &str) -> String;    // Outer::Inner → Outer$Inner
    pub fn to_package_format(type_name_str: &str) -> String;   // :: → .
    pub fn is_jakarta_annotation(annotation_type: &str) -> bool;
    pub fn is_indexed_annotation(annotation_type: &str) -> bool;
    pub fn collect_stereotypes(
        element_annotations: &[&str],
        meta_annotations: &[&str],
        is_package_info: bool,
        type_annotations: &[&str],
    ) -> Vec<String>;
    pub fn package_of(qualified_name: &str) -> &str;           // "a::b::C" → "a::b"
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| BTreeSet 排序 | `ItemMetadata.stereotypes` 使用 `BTreeSet` | 满足 |
| 确定性输出 | `SortedProperties` 固定 `\n` 换行 | 满足 |
| serde-free | 不依赖 serde，纯手工序列化 | 满足 |
| Send + Sync | 所有 index 子模块类型 | 满足 |

---

## 三、关键技术要求

### 3.1 选型基线

| 用途 | Rust crate | 版本 | Spring 等价 |
|---|---|---|---|
| 链接期注入 | `linkme` | 0.3.37 | JSR-269 APT |
| 组件定义 | `vernal-beans` | path | BeanDefinition |
| 核心类型 | `vernal-core` | path | spring-core |
| 过程宏 | `vernal-macros` | path（dev-dep） | APT 处理器 |

### 3.2 linkme vs inventory 选型对比

> **决策依据**: 参见《Spring 组件替换约定》第 8.7 节。

| 维度 | `linkme` | `inventory` |
|---|---|---|
| 机制 | 链接器段注入（`#[link_section]`） | 类型擦除 + 构造函数注册 |
| 数据类型 | `&'static [T]` 连续切片 | 类型化迭代器 |
| 零开销 | 链接期完成，运行时零分配 | 运行时构造函数调用 |
| 跨 crate | 支持（同一链接器段） | 支持（同一类型注册表） |
| `const fn` | 支持（条目是 `static` 常量） | 不支持（构造函数非 const） |
| 平台兼容 | Linux/macOS/Windows/FreeBSD | 全平台 |
| vernal 选择 | **采用**（vernal-context-indexer） | 不采用 |

**选型理由**:
1. `linkme` 的 `&'static` 切片与 Spring `META-INF/spring.components` 物理文件语义一致
2. `const fn` 构造器可在 `static` 上下文中调用，避免运行时开销
3. 连续切片布局对 CPU 缓存友好，遍历性能优于迭代器

### 3.3 vernal-macros generate_discovery_registration

`vernal-macros` 的 `#[derive(Component)]` 宏自动生成以下代码：

```rust
// 由 #[derive(Component)] 自动生成
#[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]
static __VERNAL_LINKED_COMPONENT_ENTRY_<Type>: vernal_context_indexer::LinkedComponentEntry =
    vernal_context_indexer::LinkedComponentEntry::new(
        module_path!(),
        concat!(module_path!(), "::", stringify!(<Type>)),
        &["Component"],  // 或用户指定的 stereotypes
        || vernal_beans::ComponentDefinition::new::<Type>(),
    );
```

#### 生成规则

| 属性 | 生成内容 |
|---|---|
| `#[derive(Component)]` | 默认 `stereotypes = &["Component"]` |
| `#[component(stereotype = "Service")]` | `stereotypes = &["Service"]` |
| `#[component(stereotype = "Service,Component")]` | `stereotypes = &["Component", "Service"]`（排序） |
| `module_path!()` | Rust 内置宏，编译期展开为模块路径 |
| `stringify!(Type)` | Rust 内置宏，编译期展开为类型名字符串 |

### 3.4 安全约束

```text
零 unsafe 策略:
  - lib.rs 声明 #![forbid(unsafe_code)]
  - linkme 内部使用 unsafe（链接器段注入），但对外暴露 safe API
  - 所有 vernal 代码零 unsafe

Send + Sync + 'static:
  - LinkedComponentEntry 的字段全部是 &'static 或 fn()
  - LinkedComponentIndex 通过 Vec<BTreeSet> 管理
  - 所有公开类型满足 Send + Sync + 'static

确定性保证:
  - scan() 输出按 (module_path, name) 排序
  - SortedProperties 使用 BTreeMap（天然有序）
  - EOL 固定为 "\n"（跨平台）
```

### 3.5 Feature Flag 策略

```text
vernal-context-indexer 当前无 feature flag：
  - linkme 是核心依赖，始终启用
  - vernal-beans 是核心依赖，始终启用
  - vernal-core 是核心依赖，始终启用

可能的未来 feature：
  - parallel-scan → rayon 并行扫描
  - serde → CandidateComponentsMetadata 的 serde 支持
```

---

## 四、架构

### 4.1 模块依赖图

```text
vernal-context-indexer (lib.rs)
  +-- LinkedComponentEntry          (链接期组件条目)
  +-- LinkedComponentIndex          (链接期组件索引)
  |   +-- scan() / scan_all()       (模块路径过滤)
  |   +-- get() / has()             (查询方法)
  |   +-- install()                 (安装到 RegistryBuilder)
  |   +-- component_definitions()   (生成组件定义)
  +-- LinkedComponentIndexError     (索引错误聚合)
  +-- LINKED_COMPONENT_INDEX        (linkme 分布式切片)
  +-- index/                        (Spring processor/ 镜像)
      +-- ItemMetadata              (索引记录镜像)
      +-- CandidateComponentsMetadata (元数据容器镜像)
      +-- PropertiesMarshaller      (序列化/反序列化镜像)
      +-- SortedProperties          (确定性格式化镜像)
      +-- TypeHelper                (类型辅助镜像)
```

### 4.2 crate 依赖关系

```text
vernal-context-indexer
  ├── linkme 0.3.37 (workspace)   ← 链接期分布式切片
  ├── vernal-beans (path)          ← ComponentDefinition
  └── vernal-core (path)           ← 核心类型

dev-dependencies:
  ├── linkme (workspace)           ← 测试中的分布式切片
  ├── tokio (workspace)            ← 异步测试
  ├── trybuild (workspace)         ← 编译期失败测试
  ├── vernal-context (path)        ← 集成测试
  └── vernal-macros (path)         ← 过程宏测试
```

### 4.3 数据流

```text
编译期：
  #[derive(Component)]
  struct UserService { ... }
       │
       ▼
  vernal-macros generate_discovery_registration
       │
       ▼
  #[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]
  static __VERNAL_LINKED_COMPONENT_ENTRY_UserService: LinkedComponentEntry = ...;
       │
       ▼
  链接器拼接所有 __VERNAL_LINKED_COMPONENT_ENTRY_* 到 LINKED_COMPONENT_INDEX

运行时：
  LinkedComponentIndex::scan(["my_app::web"])
       │
       ▼
  过滤 LINKED_COMPONENT_INDEX 中 module_path 以 "my_app::web" 开头的条目
       │
       ▼
  按 (module_path, name) 排序
       │
       ▼
  校验无重复名称
       │
       ▼
  LinkedComponentIndex { static_entries, runtime_entries, ... }
       │
       ▼
  index.install(&mut registry)  →  批量注册到 IoC 容器
```

---

## 五、验收标准

### 5.1 编译验收

```bash
# 1. 标准编译
cargo build -p vernal-context-indexer

# 2. 测试编译（含 dev-dependencies）
cargo build -p vernal-context-indexer --tests
```

### 5.2 测试验收

```bash
# 全量测试
cargo test -p vernal-context-indexer

# 单模块测试
cargo test -p vernal-context-indexer --lib linked_component_index
cargo test -p vernal-context-indexer --lib linked_component_entry
cargo test -p vernal-context-indexer --lib linked_component_index_error

# 集成测试
cargo test -p vernal-context-indexer --test linked_component_index
cargo test -p vernal-context-indexer --test linked_component_index_runtime
cargo test -p vernal-context-indexer --test coverage_completion

# 编译期失败测试
cargo test -p vernal-context-indexer --test index_compile_fail
```

### 5.3 质量门禁

| 检查项 | 命令 | 通过标准 |
|---|---|---|
| 零 unsafe | `grep -r "unsafe" crates/vernal-context-indexer/src/` | 无 unsafe 块 |
| Clippy | `cargo clippy -p vernal-context-indexer -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt -p vernal-context-indexer --check` | 无 diff |
| 文档 | `cargo doc -p vernal-context-indexer --no-deps` | 零警告 |
| MSRV | `cargo build -p vernal-context-indexer`（rustc 1.88） | 编译通过 |

### 5.4 dyn-compatible / Send + Sync 验收

```rust
// 编译期验证：所有公开类型满足 Send + Sync + 'static
fn _assert_send_sync<T: Send + Sync + 'static>() {}
fn _check_bounds() {
    _assert_send_sync::<LinkedComponentEntry>();
    _assert_send_sync::<LinkedComponentIndex>();
    _assert_send_sync::<LinkedComponentIndexError>();
    _assert_send_sync::<ItemMetadata>();
    _assert_send_sync::<CandidateComponentsMetadata>();
    _assert_send_sync::<PropertiesMarshaller>();
    _assert_send_sync::<SortedProperties>();
    _assert_send_sync::<TypeHelper>();
}

// 编译期验证：LinkedComponentEntry 可在 static 上下文中使用
#[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]
static _TEST_ENTRY: LinkedComponentEntry = LinkedComponentEntry::new_default(
    "test::module",
    "test::module::TestType",
    || ComponentDefinition::new::<()>(),
);
```

---

## 六、相关文档

| 文档 | 路径 | 说明 |
|---|---|---|
| 对象名称一致性检查 | `docs/vernal-context-indexer/spring-context-indexer到vernal-context-indexer对象名称一致性检查.md` | Spring→vernal 命名映射 |
| 对象级对照表 | `docs/vernal-context-indexer/spring-context-indexer到vernal-context-indexer对象级对照表.md` | 逐对象映射关系 |
| 语义迁移对照表 | `docs/vernal-context-indexer/spring-context-indexer到vernal-context-indexer语义迁移对照表.md` | 语义级迁移指南 |
| 迁移路线图 | `docs/vernal-context-indexer/spring-context-indexer到vernal-context-indexer迁移路线图.md` | 实施计划 |
| Spring 组件替换约定 | `docs/Spring-组件替换约定.md` | 选型权威字典（8.7 节 linkme） |
| Cargo.toml | `crates/vernal-context-indexer/Cargo.toml` | 依赖定义 |

---

> **文档版本**: v1
> **维护者**: Vernal Framework Team
> **下次审查**: 每个 milestone 结束时更新

<!-- migration-doc: authority=historical canonical=语义迁移对照表.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [语义迁移对照表.md](语义迁移对照表.md) 为准。

# spring-context-indexer → vernal-context-indexer 功能语义迁移对照表

> 基线：Spring Framework **7.0.8** spring-context-indexer 模块（11 个 Java 类 + 2 个运行期消费类）。
> 现状基线：vernal-discovery 4 个文件 / 3 个公开类型（`LinkedComponentRegistration` / `LinkedComponentCatalog` / `LinkedComponentCatalogError`）。
> 实现底盘：**linkme 分布式切片**（链接期收集）+ **vernal-beans**（IoC 内核）。
> 目录命名：100% 镜像 Spring `processor/` 包路径 + Spring `spring-context-indexer` crate 名。

**目录结构镜像表**：

| Spring 包 / crate | vernal 模块 |
|---|---|
| `org.springframework.context.index.processor`（编译期）| `src/index/`（子模块镜像）|
| `org.springframework.context.index`（运行期消费）| `src/linked_component_index.rs`（公开类型）|
| `META-INF/spring.components`（物理索引文件）| `src/linked_component_index_slice.rs`（链接期 linkme 切片）|
| `spring-context-indexer`（crate 名）| `vernal-context-indexer`（crate 名）|

迁移原则：**功能语义对齐,实现方式 Rust 化 + linkme 链接期**。JSR-269 注解处理器 → linkme `#[distributed_slice]`；classpath 扫描 → 模块路径前缀匹配；`@Indexed` 元注解传播 → "`#[derive(Component)]` 即入索引"；`META-INF/spring.components` 物理文件 → 链接期 linkme 静态切片。

状态图例：✅ 已迁移并有测试 / 🔶 语义等价但形态不同 / ⬜ 未迁移（路线图） / 🚫 不迁移 / 🆕 Rust 侧新增

---

## 一、编译期索引生成（Spring JSR-269 vs vernal linkme）

| Java 概念 | 语义 | vernal-context-indexer 实现 | 状态 |
|---|---|---|---|
| JSR-269 `javax.annotation.processing.Processor` SPI | 编译期自动加载并执行注解处理器 | **不需要**——linkme 链接期已收集 | 🚫 |
| `META-INF/services/javax.annotation.processing.Processor` | SPI 注册文件，列出全部 `Processor` 实现 | **不迁移**——linkme 替代 | 🚫 |
| `CandidateComponentsIndexer#init(ProcessingEnvironment)` | 初始化 4 个字段（store / collector / helper / providers）| `linkme::distributed_slice` 静态常量无需初始化 | 🚫 |
| `CandidateComponentsIndexer#process(Set, RoundEnvironment)` | 遍历 root elements → 收集 stereotypes → 写 metadata | `#[derive(Component)]` 宏生成 `static __DI_META_X: LinkedComponentEntry` 自动归入切片 | ✅ |
| `getSupportedAnnotationTypes() = ["*"]` | 监听所有注解 | `#[linkme::distributed_slice]` 监听所有 `#[derive(Component)]` | ✅ |
| `getSupportedSourceVersion() = latest()` | 支持最新 Java 版本 | `linkme` 跟随 rustc 版本 | ✅ |
| `processElement(Element)` 递归处理静态内部类 | 遍历嵌套类 | **不需要**——Rust 没有内部类概念 | 🚫 |
| `staticTypesIn(Iterable<Element>)` 过滤 STAITC 嵌套类 | 仅静态嵌套类入索引 | **不需要**——Rust 没有嵌套类 | 🚫 |
| `addMetadataFor(Element)` 汇总三提供者的 stereotypes | union | `LinkedComponentEntry` 直接持有 stereotypes Vec | 🆕 |
| `writeMetaData()` 最后一 round 写出 | `MetadataStore.writeMetadata()` | **不需要**——linkme 切片在链接期已确定 | 🚫 |

---

## 二、运行时索引读取（Spring ClassLoader vs vernal 启动时遍历）

| Java 概念 | 语义 | vernal-context-indexer 实现 | 状态 |
|---|---|---|---|
| `CandidateComponentsIndexLoader#loadIndex(ClassLoader)` | 加载 classloader 内全部 `META-INF/spring.components`，聚合为 `CandidateComponentsIndex` | `LinkedComponentIndex::scan_all()` / `scan(base_packages)` 直接遍历 `LINKED_COMPONENT_INDEX` 静态切片 | 🆕 |
| `ConcurrentReferenceHashMap<ClassLoader, CandidateComponentsIndex>` 类加载器级缓存 | 避免重复加载 | **不需要**——`LINKED_COMPONENT_INDEX` 是 `&'static` 切片,本身就是常量 | 🚫 |
| `ClassLoader.getResources("META-INF/spring.components")` | 枚举全部 jar 的索引文件 | **不需要**——linkme 把所有 `__DI_META_X` 拼成连续切片 | 🚫 |
| `PropertiesLoaderUtils.loadProperties(UrlResource)` 解析 Properties | 读取 `key=value` 列表 | `slice.iter()` 遍历 `&'static LinkedComponentEntry` | ✅ |
| `CandidateComponentsIndex#complete` 字段（true=文件加载,false=编程注册）| 区分索引来源 | `LinkedComponentIndex` 用 `Vec<&'static LinkedComponentEntry>` + `Vec<LinkedComponentEntry>`(运行时注入) 两个内部 Vec 表达 | 🆕 |
| `Entry { type, packageName, match(basePackage) }` 嵌套类 | AntPathMatcher(".") 模式匹配 | `LinkedComponentEntry::module_path` + `starts_with` 前缀匹配 | ✅ |
| `AntPathMatcher(".")` 支持 `*` / `**` | Spring 风格路径模式 | **`starts_with` 前缀匹配**——MVP 不支持 Ant `*` 通配,如需可加 | 🔶 |
| `CandidateComponentsIndexLoader#COMPONENTS_RESOURCE_LOCATION = "META-INF/spring.components"` | 索引文件路径常量 | `LINKED_COMPONENT_INDEX` 静态切片常量 | ✅ |
| `CandidateComponentsIndexLoader#IGNORE_INDEX = "spring.index.ignore"` | 系统属性开关 | **不迁移**——JVM 系统属性是 Java 特有 | 🚫 |

---

## 三、API 镜像对照（`CandidateComponentsIndex` → `LinkedComponentIndex`）

| Spring 公开 API | 语义 | vernal 公开 API | 状态 |
|---|---|---|---|
| `CandidateComponentsIndex(List<Properties> content)` 包级私有构造 | 从 Properties 文件构造 | `LinkedComponentIndex::from_slice(...)` 内部 | ⬜ |
| `CandidateComponentsIndex()` 公开空构造（`@since 7.0`）| 编程注册路径起点 | `LinkedComponentIndex::new()` | ⬜ |
| `void registerScan(String... basePackages)` | 注册基包或基包模式 | `LinkedComponentIndex::scan(base_packages: &[&str])`（替代物,在 slice 上过滤）| ✅ |
| `Set<String> getRegisteredScans()` | 返回已注册基包 | `LinkedComponentIndex::base_packages()` | ⬜ |
| `boolean hasScannedPackage(String packageName)` | 判断某包是否已扫描；`complete=true` 时总返回 true | `LinkedComponentIndex::has(base_package: &str)` | ⬜ |
| `void registerCandidateType(String type, String... stereotypes)` | 手动注册候选组件及其 stereotype（`@since 7.0`）| `LinkedComponentIndex::add_entry(entry: LinkedComponentEntry)` | ⬜ |
| `Set<String> getRegisteredStereotypes()` | 返回所有已注册 stereotype | `LinkedComponentIndex::stereotypes()` | ⬜ |
| `Set<String> getCandidateTypes(String basePackage, String stereotype)` | 给定包和 stereotype 返回候选类型 FQN 集合 | `LinkedComponentIndex::get(base_package, stereotype)` | ⬜ |
| `CandidateComponentsIndexLoader#addIndex(ClassLoader, CandidateComponentsIndex)` | 编程注入索引（`@since 7.0`）| `LinkedComponentIndex::add_entry(...)`（注入层 API）| ⬜ |
| `CandidateComponentsIndexLoader#clearCache()` | 清空缓存（`@since 7.0`）| `LinkedComponentIndex::clear_cache()` | ⬜ |
| `ClassPathScanningCandidateComponentProvider#setResourceLoader(ResourceLoader)` | 加载索引入口 | `LinkedComponentIndex::scan_all()` 在 `VernalApplicationBuilder` 调用 | ✅ |

---

## 四、Stereotype 提供者镜像（Spring 3 个 vs vernal 1 个直接持有）

| Spring 类型 | 语义 | vernal 实现 | 状态 |
|---|---|---|---|
| `StereotypesProvider`（interface）| 提供者接口：`Set<String> getStereotypes(Element)` | **不迁移**——Rust 用 `LinkedComponentEntry::stereotypes: Vec<String>` 直接持有 | 🚫 |
| `IndexedStereotypesProvider` | 处理 `@Indexed` 元注解 + 沿类层级传播 | **不迁移**——`#[derive(Component)]` 即入索引,无元注解传播 | 🚫 |
| `StandardStereotypesProvider` | 提取 `jakarta.*` 注解为 stereotype | **不迁移**——Rust 无命名空间注解 | 🚫 |
| `PackageInfoStereotypesProvider` | `package-info` → `"package-info"` stereotype | **不迁移**——Rust 模块路径本身就是 stereotype | 🚫 |
| —（无 Spring 对偶）| — | `LinkedComponentEntry::stereotypes: Vec<String>` | 🆕 |

---

## 五、元数据模型镜像（Spring 3 个类 vs vernal 1 个 struct）

| Spring 类型 | 语义 | vernal 类型 | 状态 |
|---|---|---|---|
| `ItemMetadata` | 一条索引记录（`type` + `stereotypes` Set）| `LinkedComponentEntry`（高层） + `index::ItemMetadata`（子模块镜像）| ⬜ |
| `CandidateComponentsMetadata` | 元数据容器（`List<ItemMetadata>`）| `LinkedComponentIndex` 内部 `Vec<&'static LinkedComponentEntry>` | ⬜ |
| `MetadataCollector`（多 round 合并）| javac 多轮编译累积元数据 | **不迁移**——linkme 一次性收集 | 🚫 |
| `shouldBeMerged(itemMetadata)` 增量合并逻辑 | 删除/重新编译的文件不丢旧条目 | **不需要**——linkme 切片是 final binary 的常量 | 🚫 |
| `previousMetadata` 字段 | 上一构建结果 | **不需要** | 🚫 |

---

## 六、存储与序列化镜像

| Spring 类型 | 语义 | vernal 类型 | 状态 |
|---|---|---|---|
| `MetadataStore` | 物理读写 `META-INF/spring.components` | **不迁移**——linkme 切片替代 | 🚫 |
| `MetadataStore#METADATA_PATH = "META-INF/spring.components"` | 索引文件路径 | `LINKED_COMPONENT_INDEX` 静态切片名 | ✅ |
| `PropertiesMarshaller#write(metadata, outputStream)` | 序列化为 Properties | `index::properties_marshaller::serialize(index) -> BTreeMap<&str, Vec<String>>` | ⬜ |
| `PropertiesMarshaller#read(inputStream) : metadata` | 反序列化 | `index::properties_marshaller::deserialize(map) -> LinkedComponentIndex` | ⬜ |
| `SortedProperties` | key 按字母序排序 + omitComments | `BTreeMap<String, Vec<String>>`（Rust std 天然有序）| ✅ |
| `SortedProperties#keySet() / entrySet()` 用 `TreeSet(keyComparator)` | 排序迭代 | `BTreeMap::iter()` | ✅ |
| `omitComments = true` | 写时丢弃注释行 | **不需要**——Rust slice 无注释概念 | 🚫 |

---

## 七、类型辅助（Spring TypeHelper vs vernal TypeHelper）

| Spring `TypeHelper` 方法 | 语义 | vernal `index::TypeHelper` 方法 | 状态 |
|---|---|---|---|
| `getType(Element) : String` | `Outer$Inner` 嵌套类名 | `type_id_to_string::<T>() -> &'static str`（`std::any::type_name::<T>()`）| ⬜ |
| `getSuperClass(Element)` | 直接父类 | **不迁移**——indexer 不做类型层级回溯（无 `@Indexed` 传播）| 🚫 |
| `getDirectInterfaces(Element)` | 直接接口列表 | **不迁移**——同上 | 🚫 |
| `getAllAnnotationMirrors(Element)` | 包装 `ElementUtils.getAllAnnotationMirrors`,捕获异常返回空列表 | **不迁移**——Rust 无注解概念 | 🚫 |
| `DeclaredType` 模式匹配 | Java 17 instanceof 模式 | **不迁移**——Rust 无对应 | 🚫 |

---

## 八、运行时消费流程（与 vernal-context / vernal-beans 集成）

### 8.1 Spring 运行时消费流程

```
ApplicationContext.refresh
  → ClassPathBeanDefinitionScanner.scan(basePackages)
    → ClassPathScanningCandidateComponentProvider
       ├── setResourceLoader() 时调用 CandidateComponentsIndexLoader.loadIndex(cl)
       │     ├── 读 META-INF/spring.components（所有 jar 中）
       │     ├── PropertiesLoaderUtils.loadProperties → List<Properties>
       │     └── new CandidateComponentsIndex(List<Properties>)   // complete=true
       └── findCandidateComponents(basePackage):
            ├── if 索引可用 && 所有 includeFilter 都是 AnnotationTypeFilter(Indexed/jakarta.*) 或 AssignableTypeFilter(Indexed target):
            │     └── addCandidateComponentsFromIndex(...)
            │           对每个 filter 抽 stereotype → index.getCandidateTypes(basePackage, stereotype)
            │           对每个 type → MetadataReaderFactory.getMetadataReader(type) (按名加载 ASM)
            │           → ScannedGenericBeanDefinition
            └── else 回退到 scanCandidateComponents(...)        // ASM 扫描 classpath
```

### 8.2 vernal 运行时消费流程

```
VernalApplicationBuilder::scan(base_packages)
  └─> vernal_context_indexer::LinkedComponentIndex::scan(base_packages)
        ├─> 遍历 LINKED_COMPONENT_INDEX 静态切片
        ├─> 过滤 module_path 以任一 base_package 前缀开头
        ├─> 按 (module_path, name) 排序
        └─> validate_no_duplicates → 返回 LinkedComponentIndex
  └─> index.install(&mut RegistryBuilder)
        └─> registry.register_all(component_definitions_iter)
              └─> 对每个 entry 调用 entry.component_definition() → ComponentDefinition
  └─> context.build() → 依赖图冻结 → Container

运行时注入分支（Spring 7.0 addIndex 对应）：
LinkedComponentIndex::add_entry(entry)
  └─> index.runtime_entries.push(entry)
LinkedComponentIndex::clear_cache()
  └─> index.runtime_entries.clear()
```

---

## 九、不迁移的 Java 特有功能

| Java 概念 | 原因 | Rust 替代 |
|------------|------|-----------|
| JSR-269 注解处理器（`javax.annotation.processing.Processor`）| javac 编译期 SPI | linkme 链接期切片 |
| 多 round 编译（`RoundEnvironment`）| javac 多轮编译模型 | linkme 一次性切片 |
| `@Indexed` 元注解传播 | Java 注解可标注在注解上 | `#[derive(Component)]` 即入索引 |
| `jakarta.*` 命名空间注解扫描 | JSR 标准 | Rust 无命名空间注解 |
| `package-info.java` 文件 | Java 特有 | Rust 模块路径本身就是 stereotype |
| `ClassLoader` 类加载器级缓存 | JVM 类加载器模型 | `&'static` 切片是常量 |
| `AntPathMatcher(".")` `*` / `**` 通配 | Spring 风格路径模式 | `starts_with` 前缀匹配（MVP）|
| `META-INF/spring.components` 物理文件 | JVM 类路径资源 | linkme 切片是物理文件的 1:1 Rust 等价物 |
| `META-INF/services` SPI 注册 | Java 标准 | linkme 链接期已注册 |

---

## 十、Rust 侧新增功能

| Rust 模块 | 说明 | 对应 Java 语义 |
|----------|------|---------------|
| `linkme::distributed_slice` | 链接期组件元数据收集 | JSR-269 注解处理器 |
| `#[linkme::distributed_slice] static LINKED_COMPONENT_INDEX` | 编译期全收,无运行时 IO | `META-INF/spring.components` 物理文件 |
| `LinkedComponentEntry::module_path` | Rust 模块路径作为 stereotype | Spring package name |
| `LinkedComponentEntry::name` | `concat!(module_path, "::", stringify!(Type))` | `CandidateComponentsIndex.Entry#type` |
| `LinkedComponentEntry::component_definition()` | 返回 `vernal_beans::ComponentDefinition` | Spring `ScannedGenericBeanDefinition` |
| `LinkedComponentIndex::add_entry(...)` | 运行时注入 | Spring 7.0 `addIndex(ClassLoader, CandidateComponentsIndex)` |
| `LinkedComponentIndex::clear_cache()` | 清空运行时注入 | Spring 7.0 `clearCache` |
| `LinkedComponentIndex::has(base_package)` | 路径是否已扫描 | `CandidateComponentsIndex#hasScannedPackage` |
| `LinkedComponentIndex::get(base_package, stereotype)` | 候选类型查询 | `CandidateComponentsIndex#getCandidateTypes` |

---

## 十一、tx_di 中借鉴的对象

| tx_di 类型 | vernal-context-indexer 用法 | 状态 |
|---|---|---|
| `tx_di_core::COMPONENT_REGISTRY`（`#[linkme::distributed_slice]`）| `LINKED_COMPONENT_INDEX`（`#[linkme::distributed_slice]`）| ✅ 借鉴 |
| `tx_di_core::ComponentMeta`（元数据 struct）| `LinkedComponentEntry`（字段简化,只保留 module_path / name / definition）| 🔶 简化 |
| `tx_di_macros::codegen::meta_entry` | 借鉴代码生成模式,由 `vernal-macros` 在 `#[derive(Component)]` 时生成 `static __DI_META_X` | ✅ 借鉴 |
| `tx_di_core::topology::topo_sort` | **不引入**——indexer 不做拓扑排序,只做路径过滤 | 🚫 |
| `tx_di_core::Store#inject` | **不引入**——indexer 不做实例解析 | 🚫 |

---

## 测试基线

| 模块 | 测试数目标 | 关键场景 |
|------|----------|---------|
| `linked_component_index_slice.rs` | 1 | linkme 切片自动收集 `#[derive(Component)]` 的所有条目 |
| `linked_component_entry.rs` | 1 | entry 字段访问器（`module_path` / `name` / `component_definition`）|
| `linked_component_index.rs` | 4 | `scan(base_packages)` / `scan_all()` / `merge()` / `install()` / `add_entry()` / `clear_cache()` |
| `linked_component_index_error.rs` | 1 | `EmptySelection` / `InvalidGroup` / `MissingGroup` / `InvalidEntry` / `DuplicateEntry` |
| `index/item_metadata.rs` | 1 | 高层 `LinkedComponentEntry` 与子模块 `index::ItemMetadata` 字段一致性 |
| `index/properties_marshaller.rs` | 2 | `serialize` / `deserialize` 往返 |
| `index/sorted_properties.rs` | 1 | 确定性格式化（Rust `BTreeMap` 天然有序）|
| `index/type_helper.rs` | 1 | `type_id_to_string::<T>()` 提取 |
| **合计** | **8+** | 与 `vernal-expression` 的 96 测试、`vernal-aspects` 的 46+ 测试形成统一量级 |

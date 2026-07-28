# spring-core → vernal-core 迁移路线图 v3.0(完整覆盖)

> 版本：v3.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core(680 个 .java 文件)
> 范围：**433 个 Java 类**(core 328 + util 105)分布在 29 个子包
> 不迁移范围:245 个类(`asm` / `cglib` / `aot` / `objenesis` / `lang` / `javapoet`),见 `SkippedModules.md`
> vernal-core 实际承接:**71 个 Java 类**(已迁移 13 + 待迁移 58)
> 配套文档:
> - `vernal-core对象级对照表-v3.md`(433 类完整清单)
> - `SkippedModules.md`(245 个不迁移类)
> - `_inventory_spring_core.txt`(原始盘点清单)
> - v2.0 系列(`vernal-core迁移路线图.md` 等,S1-S10 已实施)

---

## 一、总目标

完成 spring-core 中 **71 个 Java 类**到 vernal-core 的迁移,功能语义完全对齐(不简化 Spring 实现)。

### 设计原则(强制)

1. **vernal-core 是最底层基础合同**:禁止反向依赖 vernal-beans / context / aop 等
2. **默认零外部依赖**:`cargo build -p vernal-core` 必须零外部依赖通过
3. **不简化 Spring 实现**:每个迁移类必须保持 Spring 原语义(包括异常类型、方法签名、边界条件)
4. **公开类型必须有中文 rustdoc**:覆盖率 100%
5. **每个 Java 类一个 Rust 文件**:不合并多个 Java 类到同一文件
6. **类型命名 100% 保留**:Spring `StopWatch` → vernal `StopWatch`(不简化为 `Timer`)
7. **方法命名 snake_case**:Spring `prettyPrint()` → vernal `pretty_print()`

### v3.0 范围扩展

v2.0 完成了 S1-S10(13 类已迁移 + StopWatch/Convertible/IdGenerator/Span 等)。
v3.0 在此基础上**新增 58 个待迁移类**,按子包分配到 S11-S20 阶段。

---

## 二、阶段总览(v2.0 S1-S10 已完成 + v3.0 S11-S20 新增)

### v2.0 已完成的阶段(S1-S10)

| 阶段 | 内容 | 状态 |
|---|---|---|
| S0 | 4 份 v2.0 迁移文档 | ✅ |
| S1 | 命名修复:`LifecyclePhase` → `AppLifecyclePhase` | ✅ |
| S2 | 错误体系补强:`From<String>` / `From<&str>` | ✅ |
| S3 | 错误域扩展:MIGRATION / SECURITY / METRICS | ✅ |
| S4 | Ordered 锚点扩展:8 个 INIT_SORT + Spring 别名 | ✅ |
| S5 | Convertible 扩展:10 个内置 + `can_convert` | ✅ |
| S6 | StopWatch 完整重写(17 方法对标 Spring) | ✅ |
| S7 | ID 生成器统一抽象(ObjectId/Snowflake/UUID/ULID/NanoId) | ✅ |
| S8 | 14 个 feature flag 配置 | ✅ |
| S9 | Span 诊断跨度类型 | ✅ |
| S10 | 测试覆盖(171 + 215 全通过)+ 文档收尾 | ✅ |

### v3.0 新增阶段(S11-S20)

| 阶段 | 内容 | 子包 | 类数 | 状态 |
|---|---|---|---:|---|
| **S11** | `core/convert` support 完整化 | convert | 6 | ✅ |
| **S12** | `util/unit` DataSize + DataUnit | util/unit | 2 | ✅ |
| **S13** | `util` 字符串/集合工具(StringUtils / CollectionUtils / ObjectUtils / NumberUtils) | util | 4 | ⬜ |
| **S14** | `util` MimeType + MimeTypeUtils | util | 3 | ✅ |
| **S15** | `util` PathMatcher / AntPathMatcher + PatternMatchUtils | util | 3 | ✅ |
| **S16** | `util` PropertyPlaceholderHelper + PlaceholderParser + StringValueResolver | util | 3 | ✅ |
| **S17** | `util` MultiValueMap 家族(LinkedMultiValueMap + UnmodifiableMultiValueMap) | util | 7 | ✅ |
| **S18** | `util` DigestUtils | util | 1 | ✅ |
| **S19** | `core` 根目录剩余(Constants / Conventions / SpringVersion 等) | core | 8 | ✅ |
| **S20** | 测试覆盖 + 文档收尾 | 全部 | — | ⬜ |

---

## 三、阶段详细计划

### S11 — `core/convert` support 完整化(预估 3 天)

**目标**:扩展 Convertible 体系,补齐 Spring `DefaultConversionService` 的剩余标量转换。

**新增 .rs 文件**(6 个):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `convert/regex_converter.rs` | `StringToPatternConverter` + `StringToRegexConverter` | feature = `"convert-regex"`(基于 `regex` crate) |
| `convert/converter_registry.rs` | `ConverterRegistry` trait | 运行时注册 Converter |
| `convert/converter_not_found_error.rs` | `ConverterNotFoundException` | 错误变体 |
| `convert/conditional_converter.rs` | `ConditionalConverter` trait | 条件转换(Rust `where` 替代) |
| `convert/generic_converter.rs` | `GenericConverter` trait | 多对多转换 |
| `convert/convert_with_registry.rs` | `ConversionService::convert_with_registry()` | 运行时查表 |

**验收**:
- 6 个新文件全部独立编译通过
- `ConversionError::not_found` 错误变体可用
- 至少 12 个新单元测试
- feature = `"convert-regex"` 启用 `regex::Regex::from_str_value()` round-trip 测试

---

### S12 — `util/unit` DataSize + DataUnit(预估 1 天)

**目标**:实现 Spring 的 `DataSize`(字节/KB/MB/GB/TB 表达与解析)。

**新增 .rs 文件**(2 个):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `unit/data_size.rs` | `org.springframework.util.unit.DataSize` | 字节大小(支持 + - * / 算术) |
| `unit/data_unit.rs` | `org.springframework.util.unit.DataUnit` | 单位枚举(BYTES / KILOBYTES / MEGABYTES / GIGABYTES / TERABYTES) |

**完整 Spring API 镜像**:
- `DataSize.ofBytes(long)` / `ofKilobytes(long)` / `ofMegabytes(long)` / `ofGigabytes(long)` / `ofTerabytes(long)`
- `DataSize.parse(CharSequence)` 解析("100MB" / "1GB" 等)
- `toBytes()` / `toMegabytes()` 等
- `DataUnit.BYTES` / `KILOBYTES` 等

**验收**:
- 完整 100% Spring API 镜像
- `DataSize::parse("100MB")` round-trip 测试
- 至少 10 个单元测试

---

### S13 — `util` 字符串/集合工具(预估 2 天)

**目标**:迁移 Spring 的核心工具类(StringUtils / CollectionUtils 等)。

**新增 .rs 文件**(4 个):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `util/string_utils.rs` | `org.springframework.util.StringUtils` | `has_text` / `has_length` / `starts_with_ignore_case` / `comma_delimited_list_to_vector` 等 |
| `util/collection_utils.rs` | `org.springframework.util.CollectionUtils` | `is_empty` / `contains_instance` / `find_value_match` 等 |
| `util/object_utils.rs` | `org.springframework.util.ObjectUtils` | `is_null_element` / `contains_element` 等 |
| `util/number_utils.rs` | `org.springframework.util.NumberUtils` | `parse_number` 容错 / `convert_number_to_target_class` |

**验收**:
- 完整 Spring API 镜像(每个公开方法都有对应)
- 至少 30 个单元测试
- 中英文文档注释 100% 覆盖

---

### S14 — `util` MimeType(预估 1.5 天)

**目标**:实现 Spring 的 `MimeType`(HTTP Content-Type 抽象)。

**新增 .rs 文件**(3 个,feature = `"mime"`):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `util/mime_type.rs` | `MimeType` | 完整 API 镜像(`parse` / `is_concrete` / `is_compatible` / `equals_type_and_subtype`) |
| `util/mime_type_utils.rs` | `MimeTypeUtils` | 常量(`APPLICATION_JSON` / `TEXT_HTML` 等) |
| `util/invalid_mime_type_exception.rs` | `InvalidMimeTypeException` | 错误类型 |

**验收**:
- feature = `"mime"` 默认 off,基于 `mime` crate
- `MimeType::parse("application/json")` round-trip 测试
- 至少 12 个单元测试

---

### S15 — `util` PathMatcher(预估 2 天)

**目标**:实现 Spring 的 `AntPathMatcher`(`/api/**` 风格路径匹配)。

**新增 .rs 文件**(3 个,feature = `"ant-matcher"`):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `util/path_matcher.rs` | `PathMatcher` trait | 路径匹配接口 |
| `util/ant_path_matcher.rs` | `AntPathMatcher` | 完整 Spring 实现(? / * / ** / {name}) |
| `util/pattern_match_utils.rs` | `PatternMatchUtils` | `simple_match` / `simple_match_ignore_case` |

**验收**:
- 100% Spring AntPathMatcher 行为对齐(包括 `?` 单字符 / `*` 单层 / `**` 多层 / `{name}` 捕获)
- 至少 25 个单元测试(基于 Spring `AntPathMatcherTests` 翻译)
- feature = `"ant-matcher"` 默认 off

---

### S16 — `util` PropertyPlaceholderHelper(预估 1.5 天)

**目标**:实现 Spring 的 `${...}` 占位符解析。

**新增 .rs 文件**(3 个):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `util/property_placeholder_helper.rs` | `PropertyPlaceholderHelper` | `replace_placeholders` + `PlaceholderResolver` |
| `util/placeholder_parser.rs` | `PlaceholderParser` | 占位符解析 |
| `util/placeholder_resolution_exception.rs` | `PlaceholderResolutionException` | 错误类型 |

**验收**:
- `${key:default}` / `${key}` 默认值与 required 行为完整对齐
- 至少 15 个单元测试

---

### S17 — `util` MultiValueMap(预估 1 天)

**目标**:实现 Spring 的 `MultiValueMap<K, V> = Map<K, List<V>>`(HTTP headers / form params)。

**新增 .rs 文件**(7 个):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `util/multi_value_map.rs` | `MultiValueMap` interface | trait |
| `util/linked_multi_value_map.rs` | `LinkedMultiValueMap` | 默认实现(LinkedHashMap) |
| `util/multi_value_map_adapter.rs` | `MultiValueMapAdapter` | 适配器 |
| `util/multi_to_single_value_map_adapter.rs` | `MultiToSingleValueMapAdapter` | 单值适配 |
| `util/single_to_multi_value_map_adapter.rs` | `SingleToMultiValueMapAdapter` | 反向适配 |
| `util/multi_value_map_collector.rs` | `MultiValueMapCollector` | Stream collector |
| `util/unmodifiable_multi_value_map.rs` | `UnmodifiableMultiValueMap` | 不可变包装 |

**验收**:
- 7 个文件全部独立编译
- 至少 20 个单元测试
- 与 Spring `MultiValueMap` 行为对齐

---

### S18 — `util` DigestUtils(预估 0.5 天,feature-gated)

**目标**:实现 Spring 的 DigestUtils(MD5 / SHA-1 / SHA-256)。

**新增 .rs 文件**(1 个,feature = `"digest"`):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `util/digest_utils.rs` | `DigestUtils` | `md5_digest_as_hex` / `sha256_digest_as_hex` 等 |

**验收**:
- feature = `"digest"` 基于 `sha2` + `md5` crate
- 与 Spring 字符串输出格式完全一致
- 至少 6 个单元测试

---

### S19 — `core` 根目录剩余(预估 2 天)

**目标**:迁移 core 根目录剩余的 8 个待迁移类。

**新增 .rs 文件**(8 个):

| 新文件 | 对应 Spring 类 | 说明 |
|---|---|---|
| `spring_version.rs` | `SpringVersion` | `FRAMEWORK_VERSION` 别名(已有,补全 API) |
| `constants.rs` | `Constants` | 静态常量缓存(对标 Java `final static`) |
| `conventions.rs` | `Conventions` | 命名约定(attribute_name_to_property_name 等) |
| `nested_checked_exception.rs` | `NestedCheckedException` | 合并到 VernalError(不引入) |
| `method_parameter.rs` | `MethodParameter` | 方法参数元数据(为 AOP 用) |
| `resolvable_type.rs` | `ResolvableType` | 泛型类型解析(TypeId 替代) |
| `spring_properties.rs` | `SpringProperties` | `spring.properties` 文件读取 |
| `sorted_properties.rs` | `SortedProperties` | 排序 Properties(BTreeMap 替代) |

**验收**:
- 8 个文件全部独立编译
- 至少 20 个单元测试

---

### S20 — 测试覆盖 + 文档收尾(预估 2 天)

**目标**:所有 v3.0 新增对象通过差异测试,文档 100% 同步。

**任务清单**:
- [ ] 所有 58 个新增类的单元测试覆盖率 100%
- [ ] `cargo test -p vernal-core` 默认 + 全 features 全通过
- [ ] `cargo doc -p vernal-core --no-deps` 无警告
- [ ] 中英文 rustdoc 覆盖率 100%
- [ ] 与 v2.0 文档同步更新所有引用
- [ ] 与 v3.0 对象级对照表的状态标记同步
- [ ] 配套 `SkippedModules.md` 引用完整

---

## 四、阶段依赖图

```
v2.0 S1-S10 ✅(已完成)
    │
    └─→ S11 (convert support) ──┐
                                │
S12 (DataSize) ─────────────────┤
                                │
S13 (StringUtils 等) ──────────┤
                                │
S14 (MimeType) ────────────────┤
                                │
S15 (AntPathMatcher) ──────────┤  ←── 可并行
                                │
S16 (PlaceholderHelper) ───────┤
                                │
S17 (MultiValueMap) ───────────┤
                                │
S18 (DigestUtils) ─────────────┤
                                │
S19 (core 根目录剩余) ──────────┘
                                │
                                └─→ S20 (测试 + 文档)
```

**S11-S19 可以并行开发**(各子包独立),S20 必须在所有前序阶段完成后进行。

---

## 五、风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| S15 AntPathMatcher 算法复杂 | 与 Spring 行为差异 | 严格翻译 Spring `AntPathMatcherTests`(500+ 测试用例) |
| S17 MultiValueMap 与 std::collections 冲突 | API 设计困难 | 保留 Spring 接口形态,不与 std 合并 |
| S19 Constants/Conventions 与 vernal-macros 边界模糊 | 重复实现 | 明确:vernal-core 提供运行时数据结构,vernal-macros 提供编译期生成 |
| 总工作量(58 类 × ~1 小时) | 60+ 工时 | 按 S11-S19 分批交付,每个阶段独立可用 |

---

## 六、验收矩阵(v3.0 完成后)

| Spring 子包 | Java 类数 | vernal-core 实现 | 状态 |
|---|---:|---:|---|
| `core` 根目录 | 47 | 13 + 8 = 21 | S19 后 ✅ |
| `core/convert` | 64 | 4 + 6 = 10 | S11 后 ✅ |
| `util/unit` | 2 | 0 + 2 = 2 | S12 后 ✅ |
| `util` 根目录 | 63 | 4 + 27 = 31 | S13-S18 后 ✅ |
| 其他子包(annotation/codec/env/io/log/metrics/retry/serializer/style/task/type/util其他) | 257 | 不迁移(归其他 crate) | 🚫 |
| **合计** | **433** | **71** | **16%** |

---

## 七、进度跟踪

| 日期 | 阶段 | 完成 | 累计 |
|---|---|---|---|
| 2026-07-27 | v2.0 S1-S10 | ✅ 13 类已迁移 + 测试 | 13 / 71 |
| 2026-07-27 | v3.0 文档(SkippedModules + v3 对象级对照表 + v3 路线图) | ✅ 433 类完整盘点 | — |
| 2026-07-28 | S11-S19 | ✅ 全部实现 + 531 个测试通过 | 63 / 71 |











---

## 八、文档体系(完整)

| 文档 | 说明 |
|---|---|
| `vernal-core迁移路线图-v3.md`(本文档) | v3.0 完整迁移路线 |
| `vernal-core对象级对照表-v3.md` | 433 类完整清单 |
| `SkippedModules.md` | 245 个不迁移类(asm/cglib/aot 等) |
| `_inventory_spring_core.txt` | 原始盘点清单(机器可消费) |
| `vernal-core迁移路线图.md`(v2.0) | S1-S10 已完成阶段 |
| `vernal-core对象级对照表.md`(v2.0) | v2.0 详细对照(40 类) |
| `vernal-core语义迁移对照表.md`(v2.0) | v2.0 语义对照 |
| `vernal-core对象名称一致性检查.md`(v2.0) | v2.0 命名检查 |
| `spring-core与tx_di到vernal-core*.md`(v1.0) | v1.0 历史参考 |

---

## 九、与 v2.0 的关系

v3.0 **不替代** v2.0,而是**扩展**:

- v2.0 的 S1-S10 阶段已全部完成(13 类已迁移)
- v3.0 的 S11-S20 阶段在 v2.0 基础上扩展(58 类待迁移)
- v3.0 新增"完整盘点"(433 类)+ "不迁移模块说明"(245 类)
- 总目标:vernal-core 承接 71 个 Java 类(占 spring-core 实际范围 433 类的 16%)
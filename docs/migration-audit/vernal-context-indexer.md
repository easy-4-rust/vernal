<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-context-indexer 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 12 个 class/interface/enum/record；`package-info.java` 不计入 |
| 目录算法 | 去掉组织和模块根包，保留末 2 层包目录 |
| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |
| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |
| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |
| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |
| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |

本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。
<!-- current-migration-contract-end -->

## 汇总

| 指标 | 数量 |
|---|---:|
| Java 业务对象 | 12 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 5 |
| `MISSING` | 7 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `linked_component_index.rs`：`LinkedComponentIndex`、`EntryRef`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.context.index.processor.CandidateComponentsIndexer` | `processor/CandidateComponentsIndexer.java` | `processor/candidate_components_indexer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.CandidateComponentsMetadata` | `processor/CandidateComponentsMetadata.java` | `processor/candidate_components_metadata.rs` | `index/candidate_components_metadata.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.index.processor.IndexedStereotypesProvider` | `processor/IndexedStereotypesProvider.java` | `processor/indexed_stereotypes_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.ItemMetadata` | `processor/ItemMetadata.java` | `processor/item_metadata.rs` | `index/item_metadata.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.index.processor.MetadataCollector` | `processor/MetadataCollector.java` | `processor/metadata_collector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.MetadataStore` | `processor/MetadataStore.java` | `processor/metadata_store.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.PackageInfoStereotypesProvider` | `processor/PackageInfoStereotypesProvider.java` | `processor/package_info_stereotypes_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.PropertiesMarshaller` | `processor/PropertiesMarshaller.java` | `processor/properties_marshaller.rs` | `index/properties_marshaller.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.index.processor.SortedProperties` | `processor/SortedProperties.java` | `processor/sorted_properties.rs` | `index/sorted_properties.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.index.processor.StandardStereotypesProvider` | `processor/StandardStereotypesProvider.java` | `processor/standard_stereotypes_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.StereotypesProvider` | `processor/StereotypesProvider.java` | `processor/stereotypes_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.processor.TypeHelper` | `processor/TypeHelper.java` | `processor/type_helper.rs` | `index/type_helper.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |

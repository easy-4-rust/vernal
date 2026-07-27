//! 对应 Java 包：`org.springframework.context.index.processor`
//!
//! Spring 编译期组件索引处理器子模块镜像。
//! 本子模块是 `vernal-context-indexer` 的高层类型模块，完整镜像 Spring
//! `processor/` 包下的所有类。
//!
//! ## 与 Spring 的对应关系
//!
//! | Spring Java 类 | Vernal Rust 文件 |
//! |----------------|------------------|
//! | `CandidateComponentsIndexer` | 不迁移（linkme 链接期已收集，无需 JSR-269） |
//! | `CandidateComponentsMetadata` | [`candidate_components_metadata`] |
//! | `ItemMetadata` | [`item_metadata`] |
//! | `MetadataCollector` | 不迁移（多 round 是 javac 特有概念） |
//! | `MetadataStore` | 不迁移（物理文件 → linkme 切片） |
//! | `PropertiesMarshaller` | [`properties_marshaller`] |
//! | `SortedProperties` | [`sorted_properties`] |
//! | `StereotypesProvider` (interface) | 不迁移（Rust 无 stereotype provider 接口） |
//! | `IndexedStereotypesProvider` | 不迁移（@Indexed 元注解传播是 Java 特有） |
//! | `StandardStereotypesProvider` | 不迁移（jakarta.* 命名空间注解是 Java 特有） |
//! | `PackageInfoStereotypesProvider` | 不迁移（package-info.java 是 Java 特有） |
//! | `TypeHelper` | [`type_helper`] |
//! | `package-info.java` | 本 `mod.rs`（中文包级 doc） |

mod candidate_components_metadata;
mod item_metadata;
mod properties_marshaller;
mod sorted_properties;
mod type_helper;

pub use candidate_components_metadata::CandidateComponentsMetadata;
pub use item_metadata::ItemMetadata;
pub use properties_marshaller::PropertiesMarshaller;
pub use sorted_properties::{SortedProperties, EOL as SORTED_PROPERTIES_EOL};
pub use type_helper::TypeHelper;
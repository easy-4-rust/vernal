#![forbid(unsafe_code)]
#![doc = "Vernal 链接期组件索引器，对标 Spring 的 spring-context-indexer。"]

//! 对应 Spring Framework 7.0.8 `spring-context-indexer` 模块。
//!
//! `vernal-context-indexer` 提供 vernal 框架的链接期组件索引机制：
//! - `LinkedComponentEntry`：一条索引记录（对标 Spring `ItemMetadata`）
//! - `LinkedComponentIndex`：模块路径过滤 + stereotype 查询（对标 Spring `CandidateComponentsIndex`）
//! - `LINKED_COMPONENT_INDEX`：链接期 linkme 分布式切片（对标 `META-INF/spring.components` 物理文件）
//! - `LinkedComponentIndexError`：索引错误聚合
//! - `index::ItemMetadata`：高层镜像 Spring `ItemMetadata`
//! - `index::CandidateComponentsMetadata`：高层镜像 Spring `CandidateComponentsMetadata`
//! - `index::PropertiesMarshaller`：序列化/反序列化（对标 Spring `PropertiesMarshaller`）
//! - `index::SortedProperties`：确定性格式化（对标 Spring `SortedProperties`）
//! - `index::TypeHelper`：类型辅助（对标 Spring `TypeHelper`）
//!
//! ## 与 Spring 的对应关系
//!
//! | Spring crate | vernal crate |
//! |--------------|--------------|
//! | `spring-context-indexer` | `vernal-context-indexer` |
//! | `META-INF/spring.components` 物理文件 | `LINKED_COMPONENT_INDEX` 链接期切片 |
//! | `CandidateComponentsIndex` (运行期入口) | `LinkedComponentIndex` (扫描 + 查询入口) |
//! | `ClassPathScanningCandidateComponentProvider` | `LinkedComponentIndex::scan(base_packages)` |
//! | `ItemMetadata` | `LinkedComponentEntry` + `index::ItemMetadata` |
//! | `PropertiesMarshaller#write/read` | `index::PropertiesMarshaller::write/read` |
//! | `SortedProperties` | `index::SortedProperties`（`BTreeMap` 天然有序）|
//! | `TypeHelper#getType` | `index::TypeHelper::get_type` |
//!
//! ## 与 vernal 的协作
//!
//! - `vernal-macros` 的 `#[derive(Component)]` 自动生成
//!   `#[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]` 静态条目
//! - 应用通过 `LinkedComponentIndex::scan(base_packages)` 过滤
//! - `vernal-context` 通过 `LinkedComponentIndex::install(&mut RegistryBuilder)`
//!   把发现的组件定义批量注册到 IoC 容器

mod index;
mod linked_component_entry;
mod linked_component_index;
mod linked_component_index_error;
mod linked_component_index_slice;

pub use index::{
    CandidateComponentsMetadata, ItemMetadata, PropertiesMarshaller, SORTED_PROPERTIES_EOL,
    SortedProperties, TypeHelper,
};
pub use linked_component_entry::LinkedComponentEntry;
pub use linked_component_index::{EntryRef, LinkedComponentIndex};
pub use linked_component_index_error::LinkedComponentIndexError;
pub use linked_component_index_slice::LINKED_COMPONENT_INDEX;
pub use linkme;

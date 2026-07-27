//! 对应 Java 类：`org.springframework.context.index.processor.PropertiesMarshaller`
//!
//! 把 [`CandidateComponentsMetadata`] 序列化为 properties 格式；反向解析回 metadata。
//!
//! @author Stephane Nicoll
//! @author Vedran Pavic
//! @since 5.0

use std::collections::BTreeSet;
use std::io::{self, Read, Write};

use crate::index::{
    candidate_components_metadata::CandidateComponentsMetadata, item_metadata::ItemMetadata,
    sorted_properties::SortedProperties,
};

/// stereotype 列表分隔符。
///
/// 对应 Spring `PropertiesMarshaller#write` 内的 `String.join(",", m.getStereotypes())`。
const STEREOTYPE_SEPARATOR: char = ',';

/// 索引元数据序列化器。
///
/// 对应 Spring `org.springframework.context.index.processor.PropertiesMarshaller`：
/// - `write(metadata, OutputStream)`：把 `CandidateComponentsMetadata` 序列化为
///   `Properties`，每行 `type=stereotype1,stereotype2,...`
/// - `read(InputStream)`：从 `Properties` 反序列化回 `CandidateComponentsMetadata`，
///   按逗号 `,` 拆分 stereotype 列表
///
/// 使用 [`SortedProperties`] 保证确定性输出（按 key 升序，无注释行）。
///
/// 这是 Spring `META-INF/spring.components` 物理文件格式的 1:1 镜像。
/// vernal 不写物理文件（链接期 linkme 切片替代），但序列化/反序列化 API 完整保留。
pub struct PropertiesMarshaller;

impl PropertiesMarshaller {
    /// 把 [`CandidateComponentsMetadata`] 序列化为 properties 格式，写入 writer。
    ///
    /// 对应 Spring `PropertiesMarshaller#write(CandidateComponentsMetadata, OutputStream)`。
    /// 使用 [`SortedProperties::new(true)`]（omit_comments=true）保证输出无注释行。
    ///
    /// # Errors
    ///
    /// 返回底层 `Write` 操作的 IO 错误。
    pub fn write(metadata: &CandidateComponentsMetadata, writer: &mut impl Write) -> io::Result<()> {
        let mut props = SortedProperties::new(true);
        for item in metadata.get_items() {
            let stereotypes_joined = {
                let mut sorted: BTreeSet<&str> = BTreeSet::new();
                for s in item.get_stereotypes() {
                    sorted.insert(s.as_str());
                }
                sorted
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(&STEREOTYPE_SEPARATOR.to_string())
            };
            props.set(item.get_type(), stereotypes_joined);
        }
        props.store(writer)
    }

    /// 从 reader 反序列化 properties 格式，还原 [`CandidateComponentsMetadata`]。
    ///
    /// 对应 Spring `PropertiesMarshaller#read(InputStream)`：用普通 `Properties.load`
    /// 解析；每条 `type=stereotype1,stereotype2` 按 `,` 拆 stereotype 串。
    ///
    /// # Errors
    ///
    /// 返回底层 `Read` 操作的 IO 错误。
    pub fn read(reader: &mut impl Read) -> io::Result<CandidateComponentsMetadata> {
        let mut props = SortedProperties::new(true);
        props.load(reader)?;

        let mut result = CandidateComponentsMetadata::new();
        for (type_name, value) in props.iter() {
            let stereotypes: BTreeSet<String> = value
                .split(STEREOTYPE_SEPARATOR)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect();
            result.add(ItemMetadata::new(type_name.to_owned(), stereotypes));
        }
        Ok(result)
    }
}
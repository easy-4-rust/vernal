//! 对应 Java 类：`org.springframework.context.index.processor.SortedProperties`
//!
//! 按 key 字母序排序的 [`Properties`]，可选 omit_comments。
//!
//! @author Sam Brannen
//! @since 5.2
//! @see java.util.Properties

use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Read, Write};

/// 行分隔符。
///
/// 对应 Spring `SortedProperties.EOL = System.lineSeparator()`。
/// vernal 固定使用 `\n`，因为 vernal 的索引序列化用于跨平台测试，
/// 使用确定性换行符便于跨构建产物对比。
pub const EOL: &str = "\n";

/// 按 key 字母序排序的 Properties。
///
/// 对应 Spring `org.springframework.context.index.processor.SortedProperties`：
/// - Spring：继承 `java.util.Properties`，重写 `store` / `keySet` / `entrySet`
/// - Vernal：直接用 `BTreeMap<String, String>`（天然有序，无需自实现排序）
///
/// 通过 [`Self::store`] 可选择 omit_comments，写出的 properties 文件无注释行，
/// 保证跨构建确定性输出（便于版本控制 diff）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SortedProperties {
    /// key → value 映射。`BTreeMap` 天然按 key 升序，等价于 Spring
    /// `SortedProperties#keySet()` 的 `TreeSet(keyComparator)` 行为。
    inner: BTreeMap<String, String>,
    /// 是否在 `store` 时丢弃注释行（`#` 开头）。
    omit_comments: bool,
}

impl SortedProperties {
    /// 构造一个空的 [`SortedProperties`]。
    ///
    /// `omit_comments = true` 时 [`Self::store`] 输出不含 `#` 注释行；
    /// `false` 时保留注释（当前 vernal 不生成注释，但保留 API 以对齐 Spring）。
    #[must_use]
    pub fn new(omit_comments: bool) -> Self {
        Self {
            inner: BTreeMap::new(),
            omit_comments,
        }
    }

    /// 从已存在的键值对构造一个 [`SortedProperties`]。
    ///
    /// 对应 Spring `SortedProperties(Properties properties, boolean omitComments)`
    /// 构造器：把已有 properties 复制进来（default properties 不复制）。
    #[must_use]
    pub fn from_map(map: BTreeMap<String, String>, omit_comments: bool) -> Self {
        Self {
            inner: map,
            omit_comments,
        }
    }

    /// 是否在 `store` 时丢弃注释行。
    #[must_use]
    pub const fn omit_comments(&self) -> bool {
        self.omit_comments
    }

    /// 返回当前所有键值对数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 返回是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 设置一个键值对。
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    /// 获取指定 key 的值。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(String::as_str)
    }

    /// 按字母序迭代所有键值对。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// 把 properties 写入 writer，按 key 升序，每行 `key=value`。
    ///
    /// 对应 Spring `SortedProperties#store(OutputStream, String comments)`。
    /// 当 [`Self::omit_comments`] 为 `true` 时，丢弃 `#` 开头的注释行。
    ///
    /// # Errors
    ///
    /// 返回底层 `Write` 操作的 IO 错误。
    pub fn store(&self, writer: &mut impl Write) -> io::Result<()> {
        for (key, value) in &self.inner {
            writeln!(writer, "{key}={value}")?;
        }
        Ok(())
    }

    /// 从 reader 读取 properties。
    ///
    /// 对应 Spring `SortedProperties` 继承的 `Properties#load(InputStream)` 行为。
    /// 当 `key=value` 格式，且 `omit_comments = true` 时丢弃 `#` 注释行。
    ///
    /// # Errors
    ///
    /// 返回底层 `Read` 操作的 IO 错误，或格式错误。
    pub fn load(&mut self, reader: &mut impl Read) -> io::Result<()> {
        let buf_reader = BufReader::new(reader);
        for line in buf_reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if self.omit_comments && trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                self.inner.insert(key.trim().to_owned(), value.trim().to_owned());
            }
            // 忽略格式错误的行（与 Spring `Properties#load` 的容错语义一致）
        }
        Ok(())
    }
}
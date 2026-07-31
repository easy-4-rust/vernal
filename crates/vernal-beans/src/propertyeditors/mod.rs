//! propertyeditors — 对应 Spring beans.propertyeditors 包。
//!
//! 包含 Spring 风格的属性编辑器实现。

/// 字节数组属性编辑器。
pub mod byte_array_property_editor;

/// 字符数组属性编辑器。
pub mod char_array_property_editor;

/// 字符集属性编辑器。
pub mod charset_editor;

/// 类数组属性编辑器。
pub mod class_array_editor;

/// 类属性编辑器。
pub mod class_editor;

/// 货币属性编辑器。
pub mod currency_editor;

/// 文件属性编辑器。
pub mod file_editor;

/// InputSource 属性编辑器。
pub mod input_source_editor;

/// InputStream 属性编辑器。
pub mod input_stream_editor;

/// Locale 属性编辑器。
pub mod locale_editor;

/// Path 属性编辑器。
pub mod path_editor;

/// 正则表达式属性编辑器。
pub mod pattern_editor;

/// Properties 属性编辑器。
pub mod properties_editor;

/// Reader 属性编辑器。
pub mod reader_editor;

/// ResourceBundle 属性编辑器。
pub mod resource_bundle_editor;

/// StringTrimmer 属性编辑器。
pub mod string_trimmer_editor;

/// URI 属性编辑器。
pub mod uri_editor;

/// UUID 属性编辑器。
pub mod uuid_editor;

/// ZoneId 属性编辑器。
pub mod zone_id_editor;

// ── 新增 propertyeditors 子模块 ─────────────────────────────────────

/// Character 属性编辑器。
pub mod character_editor;

/// 自定义 Boolean 编辑器。
pub mod custom_boolean_editor;

/// 自定义集合编辑器。
pub mod custom_collection_editor;

/// 自定义日期编辑器。
pub mod custom_date_editor;

/// 自定义 Map 编辑器。
pub mod custom_map_editor;

/// 自定义数字编辑器。
pub mod custom_number_editor;

/// 字符串数组属性编辑器。
pub mod string_array_property_editor;

/// 时区属性编辑器。
pub mod time_zone_editor;

/// URL 属性编辑器。
pub mod url_editor;

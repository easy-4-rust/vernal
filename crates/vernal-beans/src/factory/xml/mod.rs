//! XML 解析子模块 — 对应 Spring beans.factory.xml 包。
//!
//! 提供 XML Bean 定义解析的接口契约。
//! 实际解析逻辑由 `quick-xml` 实现（见组件替换约定 §7.3）。

pub mod bean_definition_parser_delegate;
pub mod default_bean_definition_document_reader;
pub mod default_document_loader;
pub mod document_loader;
pub mod entity_resolver;
pub mod namespace_handler;
pub mod namespace_handler_resolver;
pub mod pluggable_schema_resolver;
pub mod xml_bean_definition_reader;
pub mod xml_bean_definition_store_exception;

// ── 新增 xml 子模块 ────────────────────────────────────────────────

/// 抽象 Bean 定义解析器。
pub mod abstract_bean_definition_parser;

/// 简单 Bean 定义解析器。
pub mod abstract_simple_bean_definition_parser;

/// 单 Bean 定义解析器。
pub mod abstract_single_bean_definition_parser;

/// Bean 定义装饰器接口。
pub mod bean_definition_decorator;

/// Bean 定义文档读取器。
pub mod bean_definition_document_reader;

/// Bean 定义解析器接口。
pub mod bean_definition_parser;

/// Beans DTD 解析器。
pub mod beans_dtd_resolver;

/// 默认命名空间处理器解析器。
pub mod default_namespace_handler_resolver;

/// 委托实体解析器。
pub mod delegating_entity_resolver;

/// 文档默认值定义。
pub mod document_defaults_definition;

/// 命名空间处理器支持基类。
pub mod namespace_handler_support;

/// 解析器上下文。
pub mod parser_context;

/// 资源实体解析器。
pub mod resource_entity_resolver;

/// 简单构造器命名空间处理器。
pub mod simple_constructor_namespace_handler;

/// 简单属性命名空间处理器。
pub mod simple_property_namespace_handler;

/// util 命名空间处理器。
pub mod util_namespace_handler;

/// XML 读取器上下文。
pub mod xml_reader_context;

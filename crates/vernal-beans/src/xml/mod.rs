//! XML 解析子模块 — 对应 Spring beans.factory.xml 包。
//!
//! 提供 XML Bean 定义解析的接口契约。
//! 实际解析逻辑由 `quick-xml` 实现（见组件替换约定 §7.3）。

pub mod bean_definition_parser_delegate;
pub mod default_bean_definition_document_reader;
pub mod document_loader;
pub mod entity_resolver;
pub mod namespace_handler;
pub mod namespace_handler_resolver;
pub mod xml_bean_definition_reader;

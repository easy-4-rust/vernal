//! FreeMarker 模板引擎适配 — 对标 `org.springframework.ui.freemarker`。
//!
//! 使用 `tera` crate 作为 Rust 端的模板引擎实现。

mod free_marker_configuration_factory;
mod free_marker_template_utils;
mod spring_template_loader;

pub use free_marker_configuration_factory::FreeMarkerConfigurationFactory;
pub use free_marker_template_utils::FreeMarkerTemplateUtils;
pub use spring_template_loader::SpringTemplateLoader;

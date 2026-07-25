#![forbid(unsafe_code)]
#![doc = "Vernal 可选的链接期组件发现前端。"]

mod linked_component_catalog;
mod linked_component_catalog_error;
mod linked_component_registration;
mod linked_component_registry;

pub use linked_component_catalog::LinkedComponentCatalog;
pub use linked_component_catalog_error::LinkedComponentCatalogError;
pub use linked_component_registration::LinkedComponentRegistration;
pub use linked_component_registry::LINKED_COMPONENT_REGISTRATIONS;
pub use linkme;

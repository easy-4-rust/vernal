#![forbid(unsafe_code)]
#![doc = "Vernal 的 Actix Web 4 原生集成。"]

mod actix_body_error;
mod actix_rejection;
mod actix_scoped_body;
mod vernal_actix_component;
mod vernal_actix_context;
mod vernal_actix_middleware;
mod vernal_actix_request_scope;
mod vernal_actix_service;

pub use actix_body_error::ActixBodyError;
pub use actix_rejection::ActixRejection;
pub use actix_scoped_body::ActixScopedBody;
pub use vernal_actix_component::VernalActixComponent;
pub use vernal_actix_context::VernalActixContext;
pub use vernal_actix_middleware::VernalActixMiddleware;
pub use vernal_actix_request_scope::VernalActixRequestScope;
pub use vernal_actix_service::VernalActixService;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Actix Web 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-actix-web",
    "actix-web",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

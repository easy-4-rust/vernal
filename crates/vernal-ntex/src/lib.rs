#![forbid(unsafe_code)]
#![doc = "Vernal 的 Ntex 2 原生集成。"]

mod ntex_aop_error;
mod ntex_body_error;
mod ntex_borrowed_target;
mod ntex_rejection;
mod ntex_request_snapshot;
mod ntex_response_envelope;
mod ntex_scoped_body;
mod ntex_service_error;
mod vernal_ntex_component;
mod vernal_ntex_context;
mod vernal_ntex_middleware;
mod vernal_ntex_request_context;
mod vernal_ntex_request_scope;
mod vernal_ntex_service;

pub use ntex_aop_error::NtexAopError;
pub use ntex_body_error::NtexBodyError;
pub use ntex_rejection::NtexRejection;
pub use ntex_scoped_body::NtexScopedBody;
pub use vernal_ntex_component::VernalNtexComponent;
pub use vernal_ntex_context::VernalNtexContext;
pub use vernal_ntex_middleware::VernalNtexMiddleware;
pub use vernal_ntex_request_context::VernalNtexRequestContext;
pub use vernal_ntex_request_scope::VernalNtexRequestScope;
pub use vernal_ntex_service::VernalNtexService;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Ntex 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-ntex",
    "ntex",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

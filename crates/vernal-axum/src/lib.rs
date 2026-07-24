#![forbid(unsafe_code)]
#![doc = "Vernal 的 Axum 0.8 原生集成。"]

mod axum_aop_error;
mod axum_aop_error_mapper;
mod axum_rejection;
mod axum_request_scope;
mod axum_route_resolver;
mod vernal_component;
mod vernal_context;
mod vernal_request_context;
mod vernal_request_scope;
mod vernal_router_ext;

pub use axum_aop_error::AxumAopError;
pub use axum_rejection::AxumRejection;
pub use axum_request_scope::AxumRequestScope;
pub use axum_route_resolver::AxumRouteResolver;
pub use vernal_component::VernalComponent;
pub use vernal_context::VernalContext;
pub use vernal_request_context::VernalRequestContext;
pub use vernal_request_scope::VernalRequestScope;
pub use vernal_router_ext::VernalRouterExt;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Axum 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-axum",
    "axum",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

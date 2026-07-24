#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tower Layer 与 Service 集成底座。"]

mod aop_layer;
mod aop_service;
mod aop_service_error;
mod extension_route_resolver;
mod missing_plan_policy;
mod request_scope_layer;
mod request_scope_service;
mod scoped_body;
mod tower_body_error;
mod tower_error;
mod tower_response;
mod tower_route_resolver;
mod tower_upstream_error;
mod vernal_layer;
mod vernal_service;

pub use aop_layer::AopLayer;
pub use aop_service::AopService;
pub use aop_service_error::AopServiceError;
pub use extension_route_resolver::ExtensionRouteResolver;
pub use missing_plan_policy::MissingPlanPolicy;
pub use request_scope_layer::RequestScopeLayer;
pub use request_scope_service::RequestScopeService;
pub use scoped_body::ScopedBody;
pub use tower_body_error::TowerBodyError;
pub use tower_error::TowerError;
pub use tower_response::TowerResponse;
pub use tower_route_resolver::TowerRouteResolver;
pub use vernal_layer::VernalLayer;
pub use vernal_service::VernalService;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Tower 基础集成的静态描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-tower",
    "tower",
    IntegrationRole::Foundation,
    TransportKind::FrameworkNeutral,
    "phase-4-foundation",
);

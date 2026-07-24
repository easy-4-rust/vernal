#![forbid(unsafe_code)]
#![doc = "Vernal 的 Warp 0.4 原生 Filter 与 Tower Service 集成。"]

mod vernal_warp_aop_layer;
mod vernal_warp_aop_service;
mod vernal_warp_component;
mod vernal_warp_context;
mod vernal_warp_layer;
mod vernal_warp_request_context;
mod vernal_warp_request_scope;
mod warp_aop_error;
mod warp_rejection;
mod warp_route_resolver;

pub use vernal_warp_aop_layer::VernalWarpAopLayer;
pub use vernal_warp_component::VernalWarpComponent;
pub use vernal_warp_context::VernalWarpContext;
pub use vernal_warp_layer::VernalWarpLayer;
pub use vernal_warp_request_context::VernalWarpRequestContext;
pub use vernal_warp_request_scope::VernalWarpRequestScope;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};
pub use warp_aop_error::WarpAopError;
pub use warp_rejection::WarpRejection;

/// Warp 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-warp",
    "warp",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

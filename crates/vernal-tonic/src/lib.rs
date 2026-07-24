#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tonic 0.12 gRPC 原生集成。"]

mod tonic_aop_error_mapper;
mod tonic_aop_layer;
mod tonic_aop_service;
mod tonic_context_interceptor;
mod tonic_request_error;
mod tonic_request_ext;
mod tonic_route_resolver;
mod tonic_status_mapper;

pub use tonic_aop_error_mapper::TonicAopErrorMapper;
pub use tonic_aop_layer::TonicAopLayer;
pub use tonic_aop_service::TonicAopService;
pub use tonic_context_interceptor::TonicContextInterceptor;
pub use tonic_request_error::TonicRequestError;
pub use tonic_request_ext::TonicRequestExt;
pub use tonic_route_resolver::TonicRouteResolver;
pub use tonic_status_mapper::TonicStatusMapper;
pub use vernal_tower::{RequestScopeLayer, VernalLayer};
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Tonic 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-tonic",
    "tonic",
    IntegrationRole::RpcFramework,
    TransportKind::Rpc,
    "phase-5-adapter",
);

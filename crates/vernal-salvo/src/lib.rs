#![forbid(unsafe_code)]
#![doc = "Vernal 的 Salvo 0.85 原生集成。"]

mod salvo_aop_error;
mod salvo_borrowed_target;
mod salvo_rejection;
mod salvo_request_snapshot;
mod salvo_response;
mod salvo_route_metadata;
mod salvo_scoped_body;
mod vernal_salvo_depot_ext;
mod vernal_salvo_hoop;

pub use salvo_aop_error::SalvoAopError;
pub use salvo_rejection::SalvoRejection;
pub use salvo_response::SalvoResponse;
pub use salvo_scoped_body::SalvoScopedBody;
pub use vernal_salvo_depot_ext::VernalSalvoDepotExt;
pub use vernal_salvo_hoop::VernalSalvoHoop;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Salvo 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-salvo",
    "salvo",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

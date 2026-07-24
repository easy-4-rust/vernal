#![forbid(unsafe_code)]
#![doc = "Vernal 的 Salvo 0.85 原生集成。"]

mod salvo_rejection;
mod salvo_scoped_body;
mod vernal_salvo_depot_ext;
mod vernal_salvo_hoop;

pub use salvo_rejection::SalvoRejection;
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

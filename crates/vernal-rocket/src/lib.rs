#![forbid(unsafe_code)]
#![doc = "Vernal 的 Rocket 0.5 原生集成。"]

mod rocket_aop_error;
mod rocket_borrowed_target;
mod rocket_outcome_marker;
mod rocket_rejection;
mod rocket_request_snapshot;
mod rocket_request_state;
mod rocket_scoped_reader;
mod vernal_rocket_component;
mod vernal_rocket_context;
mod vernal_rocket_fairing;
mod vernal_rocket_handler;
mod vernal_rocket_request_context;
mod vernal_rocket_request_scope;
mod vernal_rocket_routes_ext;

pub use rocket_aop_error::RocketAopError;
pub use rocket_rejection::RocketRejection;
pub use rocket_scoped_reader::RocketScopedReader;
pub use vernal_rocket_component::VernalRocketComponent;
pub use vernal_rocket_context::VernalRocketContext;
pub use vernal_rocket_fairing::VernalRocketFairing;
pub use vernal_rocket_request_context::VernalRocketRequestContext;
pub use vernal_rocket_request_scope::VernalRocketRequestScope;
pub use vernal_rocket_routes_ext::VernalRocketRoutesExt;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Rocket 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-rocket",
    "rocket",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

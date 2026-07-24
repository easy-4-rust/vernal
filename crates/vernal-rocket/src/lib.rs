#![forbid(unsafe_code)]
#![doc = "Vernal 的 Rocket 0.5 原生集成。"]

mod rocket_rejection;
mod rocket_request_state;
mod rocket_scoped_reader;
mod vernal_rocket_component;
mod vernal_rocket_context;
mod vernal_rocket_fairing;
mod vernal_rocket_request_scope;

pub use rocket_rejection::RocketRejection;
pub use rocket_scoped_reader::RocketScopedReader;
pub use vernal_rocket_component::VernalRocketComponent;
pub use vernal_rocket_context::VernalRocketContext;
pub use vernal_rocket_fairing::VernalRocketFairing;
pub use vernal_rocket_request_scope::VernalRocketRequestScope;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Rocket 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-rocket",
    "rocket",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

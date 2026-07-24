#![forbid(unsafe_code)]
#![doc = "Vernal 的 Gotham 0.8 原生集成。"]

mod gotham_body_error;
mod gotham_rejection;
mod gotham_scoped_body;
mod vernal_gotham_context;
mod vernal_gotham_middleware;
mod vernal_gotham_request_scope;
mod vernal_gotham_state_ext;

pub use gotham_body_error::GothamBodyError;
pub use gotham_rejection::GothamRejection;
pub use gotham_scoped_body::GothamScopedBody;
pub use vernal_gotham_context::VernalGothamContext;
pub use vernal_gotham_middleware::VernalGothamMiddleware;
pub use vernal_gotham_request_scope::VernalGothamRequestScope;
pub use vernal_gotham_state_ext::VernalGothamStateExt;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Gotham 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-gotham",
    "gotham",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

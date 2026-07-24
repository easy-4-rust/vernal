#![forbid(unsafe_code)]
#![doc = "Vernal 的 Poem 3 原生集成。"]

mod poem_aop_error;
mod poem_rejection;
mod poem_response;
mod poem_scoped_stream;
mod vernal_poem_component;
mod vernal_poem_context;
mod vernal_poem_endpoint;
mod vernal_poem_middleware;
mod vernal_poem_request_context;
mod vernal_poem_request_scope;

pub use poem_aop_error::PoemAopError;
pub use poem_rejection::PoemRejection;
pub use poem_response::PoemResponse;
pub use poem_scoped_stream::PoemScopedStream;
pub use vernal_poem_component::VernalPoemComponent;
pub use vernal_poem_context::VernalPoemContext;
pub use vernal_poem_endpoint::VernalPoemEndpoint;
pub use vernal_poem_middleware::VernalPoemMiddleware;
pub use vernal_poem_request_context::VernalPoemRequestContext;
pub use vernal_poem_request_scope::VernalPoemRequestScope;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Poem 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-poem",
    "poem",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

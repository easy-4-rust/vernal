#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tide 0.17 beta 原生集成。"]

mod tide_body_error;
mod tide_rejection;
mod tide_scoped_reader;
mod vernal_tide_middleware;
mod vernal_tide_request_ext;

pub use tide_body_error::TideBodyError;
pub use tide_rejection::TideRejection;
pub use tide_scoped_reader::TideScopedReader;
pub use vernal_tide_middleware::VernalTideMiddleware;
pub use vernal_tide_request_ext::VernalTideRequestExt;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Tide 适配器的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-tide",
    "tide",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "phase-5-adapter",
);

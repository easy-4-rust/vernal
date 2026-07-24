#![forbid(unsafe_code)]
#![doc = "Vernal 的框架中立 HTTP 1.x 协议合同。"]

mod collected_body;
mod http_body;
mod http_body_error;
mod http_request;
mod http_request_snapshot;
mod http_response;

pub use bytes::Bytes;
pub use collected_body::CollectedBody;
pub use http::{
    HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri, Version, request, response,
};
pub use http_body::Frame;
pub use http_body::HttpBody;
pub use http_body_error::HttpBodyError;
pub use http_request::HttpRequest;
pub use http_request_snapshot::HttpRequestSnapshot;
pub use http_response::HttpResponse;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// HTTP 协议合同的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-http",
    "http/http-body",
    IntegrationRole::ProtocolContract,
    TransportKind::Http,
    "phase-4-contract",
);

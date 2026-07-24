#![forbid(unsafe_code)]
#![doc = "Vernal 的框架中立 Web 应用层合同。"]

mod context_carrier;
mod handler_invocation;
mod integration_descriptor;
mod integration_role;
mod problem_details;
mod problem_kind;
mod request_context;
mod request_id;
mod route_metadata;
mod scope_error;
mod scope_future;
mod scope_state;
mod security_principal;
mod transport_kind;
mod web_failure;
mod web_request_scope;

pub use context_carrier::ContextCarrier;
pub use handler_invocation::HandlerInvocation;
pub use integration_descriptor::IntegrationDescriptor;
pub use integration_role::IntegrationRole;
pub use problem_details::ProblemDetails;
pub use problem_kind::ProblemKind;
pub use request_context::RequestContext;
pub use request_id::RequestId;
pub use route_metadata::RouteMetadata;
pub use scope_error::ScopeError;
pub use scope_future::ScopeFuture;
pub use scope_state::ScopeState;
pub use security_principal::SecurityPrincipal;
pub use transport_kind::TransportKind;
pub use web_failure::WebFailure;
pub use web_request_scope::WebRequestScope;

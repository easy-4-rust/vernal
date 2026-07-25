#![forbid(unsafe_code)]
#![doc = "Vernal Web Adapter 的共享合同测试工具。"]

mod failing_byte_stream;
mod failing_futures_reader;
mod failing_http_body;
mod failing_tokio_reader;
mod scope_cleanup_timeout_fixture;
mod scope_close_probe;
mod scope_rejecting_interceptor;
mod web_adapter_contract;

pub use failing_byte_stream::FailingByteStream;
pub use failing_futures_reader::FailingFuturesReader;
pub use failing_http_body::FailingHttpBody;
pub use failing_tokio_reader::FailingTokioReader;
pub use scope_cleanup_timeout_fixture::ScopeCleanupTimeoutFixture;
pub use scope_close_probe::ScopeCloseProbe;
pub use scope_rejecting_interceptor::ScopeRejectingInterceptor;
pub use web_adapter_contract::WebAdapterContract;

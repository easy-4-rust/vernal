#![forbid(unsafe_code)]
#![doc = "Vernal Web Adapter 的共享合同测试工具。"]

mod scope_close_probe;
mod scope_rejecting_interceptor;
mod web_adapter_contract;

pub use scope_close_probe::ScopeCloseProbe;
pub use scope_rejecting_interceptor::ScopeRejectingInterceptor;
pub use web_adapter_contract::WebAdapterContract;

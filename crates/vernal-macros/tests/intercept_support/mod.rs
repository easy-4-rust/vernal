//! `intercept` 宏测试支持对象。

mod metadata_probe_interceptor;
mod stateful_service;

pub(crate) use metadata_probe_interceptor::MetadataProbeInterceptor;
pub(crate) use stateful_service::StatefulService;

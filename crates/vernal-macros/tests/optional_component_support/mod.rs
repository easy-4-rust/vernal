//! Option 构造注入宏测试支持对象。

mod optional_derived_port;
mod optional_derived_service;
mod optional_native_client;

pub(crate) use optional_derived_port::OptionalDerivedPort;
pub(crate) use optional_derived_service::OptionalDerivedService;
pub(crate) use optional_native_client::OptionalNativeClient;

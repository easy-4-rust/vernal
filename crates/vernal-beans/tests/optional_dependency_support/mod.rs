//! 立即可选依赖测试支持对象。

mod optional_consumer;
mod optional_port;
mod optional_service;

pub(crate) use optional_consumer::OptionalConsumer;
pub(crate) use optional_port::OptionalPort;
pub(crate) use optional_service::OptionalService;

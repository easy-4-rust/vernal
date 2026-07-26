#![forbid(unsafe_code)]
#![doc = "Vernal 日志门面（对标 spring-jcl）。"]
//!
//! 底层使用 `tracing`，提供框架级日志初始化和工厂。

mod log_factory;

pub use log_factory::LogFactory;

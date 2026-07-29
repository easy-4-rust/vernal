//! Resource trait — Spring 风格的资源接口。
//! 对应 Java 类：`org.springframework.core.io.Resource`。
use std::any::Any;
use std::fmt;

/// Spring 风格的资源接口。
pub trait Resource: Send + Sync + Any + fmt::Debug {
    fn exists(&self) -> bool { true }
    fn is_readable(&self) -> bool { true }
    fn description(&self) -> &str;
    fn read_to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }
}

#![forbid(unsafe_code)]
#![doc = "Vernal 事务抽象（对标 spring-tx）。"]

mod manager;
mod definition;
mod status;

pub use manager::PlatformTransactionManager;
pub use definition::{TransactionDefinition, Propagation, Isolation};
pub use status::TransactionStatus;

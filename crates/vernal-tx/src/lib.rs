#![forbid(unsafe_code)]
#![doc = "Vernal 事务抽象（对标 spring-tx）。"]

mod definition;
mod manager;
mod status;

pub use definition::{Isolation, Propagation, TransactionDefinition};
pub use manager::PlatformTransactionManager;
pub use status::TransactionStatus;

#![forbid(unsafe_code)]
#![doc = "Vernal 异步执行抽象（对标 spring-async）。"]

mod executor;

pub use executor::AsyncTaskExecutor;

#![forbid(unsafe_code)]
#![doc = "Vernal 数据库抽象（对标 spring-jdbc）。"]

mod datasource;

pub use datasource::DataSource;

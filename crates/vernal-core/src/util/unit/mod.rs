//! 数据单位模块。
//!
//! 对标 Spring `org.springframework.util.unit`。

mod data_size;
mod data_unit;

pub use data_size::{
    BYTES_PER_GB, BYTES_PER_KB, BYTES_PER_MB, BYTES_PER_TB, DataSize, DataSizeParseError,
};
pub use data_unit::{DataUnit, UnknownDataUnitSuffix};

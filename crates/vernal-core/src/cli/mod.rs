//! CLI 模块。
//!
//! 对标 Spring Boot CLI 解析层。

mod clap_parser;
mod cli_error;

#[cfg(feature = "cli")]
pub use clap_parser::ClapParser;
pub use cli_error::CliError;

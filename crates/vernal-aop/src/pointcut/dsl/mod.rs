//! 对应 aspect-rs：aspect-core/src/pointcut/mod.rs
//! 语义参照 spring-aop：AspectJ 表达式解析器（Weaver）
//!
//! Pointcut DSL 模块（Rust 原生切点表达式）。
//! 移植自 aspect-rs 的 pointcut 模块，扩展 Tag/Qualifier 变体。

mod ast;
mod matcher;
mod parser;
mod pattern;

pub use ast::PointcutExpr;
pub use matcher::{FunctionDescriptor, PointcutMatcher};
pub use parser::{parse_pointcut_expr, PointcutParseError};
pub use pattern::{ExecutionPattern, ModulePattern, NamePattern, Visibility};

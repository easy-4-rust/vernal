#![forbid(unsafe_code)]
#![doc = "Vernal 简化版表达式语言（对标 spring-expression SpEL 子集）。"]

mod parser;
mod expression;
mod context;

pub use parser::ExpressionParser;
pub use expression::Expression;
pub use context::EvaluationContext;

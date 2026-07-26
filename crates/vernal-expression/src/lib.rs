#![forbid(unsafe_code)]
#![doc = r#"Vernal 表达式语言（对标 Spring Framework 7.0 的 spring-expression）。

提供 SpEL 子集的完整实现，包括：
- 核心接口：Expression / ExpressionParser / EvaluationContext
- AST 节点：42 个节点（字面量、运算符、表达式）
- 解析器：递归下降解析器
- 求值上下文：StandardEvaluationContext / SimpleEvaluationContext
- 属性访问器：PropertyAccessor / IndexAccessor
- 类型系统：TypeConverter / TypeLocator / TypeComparator
- 错误体系：ExpressionException / EvaluationException / ParseException
"#]

// ─── 核心接口层 ───
mod access_exception;
mod bean_resolver;
mod constructor_executor;
mod constructor_resolver;
mod evaluation_context;
mod evaluation_exception;
mod expression;
mod expression_exception;
mod method_executor;
mod method_resolver;
mod operation;
mod operator_overloader;
mod parse_exception;
mod parser;
mod parser_context;
mod property_accessor;
mod type_comparator;
mod type_converter;
mod type_locator;
mod typed_value;

// ─── SpEL 实现层 ───
pub mod spel;

// ─── 通用工具层 ───
pub mod common;

// ─── 公开导出 ───
pub use access_exception::AccessException;
pub use bean_resolver::BeanResolver;
pub use constructor_executor::ConstructorExecutor;
pub use constructor_resolver::ConstructorResolver;
pub use evaluation_context::EvaluationContext;
pub use evaluation_exception::EvaluationException;
pub use expression::Expression;
pub use expression_exception::ExpressionException;
pub use method_executor::MethodExecutor;
pub use method_resolver::MethodResolver;
pub use operation::Operation;
pub use operator_overloader::OperatorOverloader;
pub use parse_exception::ParseException;
pub use parser::ExpressionParser;
pub use parser_context::{ParserContext, TemplateParserContext};
pub use property_accessor::{IndexAccessor, PropertyAccessor};
pub use type_comparator::TypeComparator;
pub use type_converter::TypeConverter;
pub use type_locator::TypeLocator;
pub use typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

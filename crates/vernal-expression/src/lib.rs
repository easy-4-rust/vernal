#![forbid(unsafe_code)]
#![doc = r#"Vernal 表达式语言（对标 Spring Framework 7.0 的 spring-expression，100% 语义迁移）。

提供 SpEL 完整实现：

# 核心接口层
- [`expression::Expression`]：`Expression` trait（求值/赋值/类型查询）
- [`parser::ExpressionParser`]：`ExpressionParser` trait（解析字符串为 `Expression`）
- [`evaluation_context::EvaluationContext`]：`EvaluationContext` trait
- [`typed_value::TypedValue`] + [`expression_value::ExpressionValue`]：值与类型化值
- [`type_descriptor::TypeDescriptor`] + [`type_descriptor::PrimitiveKind`]：富类型描述符（对标 Spring `TypeDescriptor`）
- 21 个访问器/解析器/转换器/比较器/重载器/异常 trait

# SpEL 实现层
- [`spel`]:解析器（递归下降）、AST 节点（54 个）、求值上下文（`ExpressionState`）
- [`spel::spel_message::SpelMessage`]：86 项错误码（与 Spring 一一对应）
- [`spel::spel_parse_exception::SpelParseException`] / [`spel::spel_evaluation_exception::SpelEvaluationException`]：结构化异常
- [`spel::tokenizer::Tokenizer`]：手写词法分析器（46 种 token）
- [`spel::support`]：`StandardEvaluationContext` / `SimpleEvaluationContext` + 反射/DataBinding/Vernal Accessor/Resolver

# 通用工具层
- [`common`]:模板解析器（`#{...}`）、`LiteralExpression`、`CompositeStringExpression`

# 操作
- [`operation::Operation`]:21 项运算符统一入口，`ExpressionState.operate` 集中调度。
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
mod expression_invocation_target_exception;
mod expression_value;
mod method_executor;
mod method_filter;
mod method_resolver;
mod operation;
mod operator_overloader;
mod parse_exception;
mod parser;
mod parser_context;
mod property_accessor;
mod type_comparator;
mod type_converter;
mod type_descriptor;
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
pub use expression_invocation_target_exception::ExpressionInvocationTargetException;
pub use expression_value::ExpressionValue;
pub use method_executor::MethodExecutor;
pub use method_filter::MethodFilter;
pub use method_resolver::MethodResolver;
pub use operation::Operation;
pub use operator_overloader::OperatorOverloader;
pub use parse_exception::ParseException;
pub use parser::ExpressionParser;
pub use parser_context::{ParserContext, TemplateParserContext};
pub use property_accessor::{IndexAccessor, PropertyAccessor};
pub use type_comparator::TypeComparator;
pub use type_converter::TypeConverter;
pub use type_descriptor::{PrimitiveKind, TypeDescriptor};
pub use type_locator::TypeLocator;
pub use typed_value::TypedValue;

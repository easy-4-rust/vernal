use vernal_expression::common::literal_expression::LiteralExpression;
use vernal_expression::common::composite_string_expression::CompositeStringExpression;
use vernal_expression::common::template_aware_expression_parser::TemplateAwareExpressionParser;
use vernal_expression::common::template_parser_context::TemplateParserContextImpl;
use vernal_expression::common::expression_utils;
use vernal_expression::*;

// ═══════════════════════════════════════════════════════════════════
//  Mock types
// ═══════════════════════════════════════════════════════════════════

/// Minimal EvaluationContext that returns null root and empty resolvers.
struct MockEvaluationContext {
    root: TypedValue,
}

impl MockEvaluationContext {
    fn new() -> Self {
        Self {
            root: TypedValue::null(),
        }
    }
}

impl EvaluationContext for MockEvaluationContext {
    fn root_object(&self) -> &TypedValue {
        &self.root
    }
    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor> {
        vec![]
    }
    fn bean_resolver(&self) -> Option<&dyn BeanResolver> {
        None
    }
    fn type_converter(&self) -> Option<&dyn TypeConverter> {
        None
    }
    fn type_locator(&self) -> Option<&dyn TypeLocator> {
        None
    }
    fn type_comparator(&self) -> Option<&dyn TypeComparator> {
        None
    }
    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader> {
        None
    }
    fn method_resolvers(&self) -> Vec<&dyn MethodResolver> {
        vec![]
    }
    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver> {
        vec![]
    }
    fn set_variable(&mut self, _name: &str, _value: TypedValue) {}
    fn lookup_variable(&self, _name: &str) -> Option<&TypedValue> {
        None
    }
}

/// ExpressionParser that wraps every input string in a LiteralExpression.
struct MockExpressionParser;

impl ExpressionParser for MockExpressionParser {
    fn parse_expression(
        &self,
        expression_string: &str,
    ) -> Result<Box<dyn Expression>, ParseException> {
        Ok(Box::new(LiteralExpression::new(expression_string.to_string())))
    }

    fn parse_expression_with_context(
        &self,
        expression_string: &str,
        _context: &dyn ParserContext,
    ) -> Result<Box<dyn Expression>, ParseException> {
        self.parse_expression(expression_string)
    }
}

/// Expression that always returns an Int (non-string) -- used to test error paths.
struct MockNonStringExpression;

impl Expression for MockNonStringExpression {
    fn expression_string(&self) -> &str {
        "42"
    }
    fn get_value(&self) -> Result<TypedValue, EvaluationException> {
        Ok(TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT))
    }
    fn get_value_with_context(
        &self,
        _ctx: &dyn EvaluationContext,
    ) -> Result<TypedValue, EvaluationException> {
        self.get_value()
    }
    fn get_value_with_root(
        &self,
        _ctx: &dyn EvaluationContext,
        _root: &TypedValue,
    ) -> Result<TypedValue, EvaluationException> {
        self.get_value()
    }
}

/// ParserContext that reports is_template = false.
struct NonTemplateContext;

impl ParserContext for NonTemplateContext {
    fn is_template(&self) -> bool {
        false
    }
    fn expression_prefix(&self) -> &str {
        ""
    }
    fn expression_suffix(&self) -> &str {
        ""
    }
}

/// TypeConverter that converts Int -> Int(42), Boolean -> Boolean(true), Float -> Float(3.14).
struct MockTypeConverter;

impl TypeConverter for MockTypeConverter {
    fn can_convert(&self, _source: &TypeDescriptor, _target: &TypeDescriptor) -> bool {
        true
    }
    fn convert_value(
        &self,
        _value: &TypedValue,
        target_type: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException> {
        match target_type {
            TypeDescriptor::Primitive(PrimitiveKind::Int) => Ok(TypedValue::new(
                ExpressionValue::Int(42),
                TypeDescriptor::INT,
            )),
            TypeDescriptor::Primitive(PrimitiveKind::Boolean) => Ok(TypedValue::new(
                ExpressionValue::Boolean(true),
                TypeDescriptor::BOOLEAN,
            )),
            TypeDescriptor::Primitive(PrimitiveKind::Float) => Ok(TypedValue::new(
                ExpressionValue::Float(3.14),
                TypeDescriptor::FLOAT,
            )),
            _ => Err(EvaluationException::new("", None, "unsupported conversion")),
        }
    }
}

/// TypeConverter that always fails.
struct FailingTypeConverter;

impl TypeConverter for FailingTypeConverter {
    fn can_convert(&self, _: &TypeDescriptor, _: &TypeDescriptor) -> bool {
        false
    }
    fn convert_value(
        &self,
        _: &TypedValue,
        _: &TypeDescriptor,
    ) -> Result<TypedValue, EvaluationException> {
        Err(EvaluationException::new("", None, "conversion failed"))
    }
}

// ═══════════════════════════════════════════════════════════════════
//  LiteralExpression tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn literal_expression_new() {
    let expr = LiteralExpression::new("hello".to_string());
    assert_eq!(expr.expression_string(), "hello");
}

#[test]
fn literal_expression_expression_string() {
    let expr = LiteralExpression::new("test value".to_string());
    assert_eq!(expr.expression_string(), "test value");
}

#[test]
fn literal_expression_get_value_returns_string() {
    let expr = LiteralExpression::new("hello".to_string());
    let result = expr.get_value().unwrap();
    assert!(matches!(result.value(), ExpressionValue::String(s) if s == "hello"));
}

#[test]
fn literal_expression_get_value_with_context() {
    let expr = LiteralExpression::new("hello".to_string());
    let ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert!(matches!(result.value(), ExpressionValue::String(s) if s == "hello"));
}

#[test]
fn literal_expression_get_value_with_root() {
    let expr = LiteralExpression::new("hello".to_string());
    let ctx = MockEvaluationContext::new();
    let root = TypedValue::null();
    let result = expr.get_value_with_root(&ctx, &root).unwrap();
    assert!(matches!(result.value(), ExpressionValue::String(s) if s == "hello"));
}

#[test]
fn literal_expression_empty_string() {
    let expr = LiteralExpression::new("".to_string());
    let result = expr.get_value().unwrap();
    assert!(matches!(result.value(), ExpressionValue::String(s) if s.is_empty()));
}

#[test]
fn literal_expression_returns_string_type_descriptor() {
    let expr = LiteralExpression::new("hello".to_string());
    let result = expr.get_value().unwrap();
    assert!(matches!(
        result.type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::String)
    ));
}

#[test]
fn literal_expression_is_not_null() {
    let expr = LiteralExpression::new("hello".to_string());
    let result = expr.get_value().unwrap();
    assert!(!result.is_null());
}

// ═══════════════════════════════════════════════════════════════════
//  CompositeStringExpression tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn composite_new_and_expressions_accessor() {
    let sub_exprs: Vec<Box<dyn Expression>> = vec![
        Box::new(LiteralExpression::new("a".to_string())),
        Box::new(LiteralExpression::new("b".to_string())),
    ];
    let composite = CompositeStringExpression::new("ab".to_string(), sub_exprs);
    assert_eq!(composite.expressions().len(), 2);
}

#[test]
fn composite_expression_string() {
    let composite = CompositeStringExpression::new(
        "hello #{name}".to_string(),
        vec![Box::new(LiteralExpression::new("hello".to_string()))],
    );
    assert_eq!(composite.expression_string(), "hello #{name}");
}

#[test]
fn composite_get_value_without_context_errors() {
    let composite = CompositeStringExpression::new(
        "test".to_string(),
        vec![Box::new(LiteralExpression::new("hello".to_string()))],
    );
    let result = composite.get_value();
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.simple_message().contains("上下文"));
    }
}

#[test]
fn composite_get_value_with_context_concatenates() {
    let composite = CompositeStringExpression::new(
        "hello world".to_string(),
        vec![
            Box::new(LiteralExpression::new("hello".to_string())),
            Box::new(LiteralExpression::new(" ".to_string())),
            Box::new(LiteralExpression::new("world".to_string())),
        ],
    );
    let ctx = MockEvaluationContext::new();
    let result = composite.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello world".to_string())
    );
}

#[test]
fn composite_get_value_with_root_delegates() {
    let composite = CompositeStringExpression::new(
        "hello".to_string(),
        vec![Box::new(LiteralExpression::new("hello".to_string()))],
    );
    let ctx = MockEvaluationContext::new();
    let root = TypedValue::null();
    let result = composite.get_value_with_root(&ctx, &root).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello".to_string())
    );
}

#[test]
fn composite_single_literal_only() {
    let composite = CompositeStringExpression::new(
        "hello".to_string(),
        vec![Box::new(LiteralExpression::new("hello".to_string()))],
    );
    let ctx = MockEvaluationContext::new();
    let result = composite.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello".to_string())
    );
}

#[test]
fn composite_multiple_template_parts() {
    let composite = CompositeStringExpression::new(
        "a and b".to_string(),
        vec![
            Box::new(LiteralExpression::new("a".to_string())),
            Box::new(LiteralExpression::new(" and ".to_string())),
            Box::new(LiteralExpression::new("b".to_string())),
        ],
    );
    let ctx = MockEvaluationContext::new();
    let result = composite.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("a and b".to_string())
    );
}

#[test]
fn composite_non_string_sub_expression_errors() {
    let composite = CompositeStringExpression::new(
        "test".to_string(),
        vec![
            Box::new(LiteralExpression::new("hello".to_string())),
            Box::new(MockNonStringExpression),
        ],
    );
    let ctx = MockEvaluationContext::new();
    let result = composite.get_value_with_context(&ctx);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.simple_message().contains("字符串"));
    }
}

#[test]
fn composite_empty_expressions_returns_empty_string() {
    let composite = CompositeStringExpression::new("test".to_string(), vec![]);
    let ctx = MockEvaluationContext::new();
    let result = composite.get_value_with_context(&ctx).unwrap();
    assert_eq!(*result.value(), ExpressionValue::String("".to_string()));
}

#[test]
fn composite_returns_string_type_descriptor() {
    let composite = CompositeStringExpression::new(
        "test".to_string(),
        vec![Box::new(LiteralExpression::new("hello".to_string()))],
    );
    let ctx = MockEvaluationContext::new();
    let result = composite.get_value_with_context(&ctx).unwrap();
    assert!(matches!(
        result.type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::String)
    ));
}

// ═══════════════════════════════════════════════════════════════════
//  TemplateAwareExpressionParser tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn template_parser_new() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let expr = parser.parse_expression("test").unwrap();
    assert_eq!(expr.expression_string(), "test");
}

#[test]
fn template_parser_default_parse_creates_composite() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let expr = parser.parse_expression("hello #{world} foo").unwrap();
    let ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello world foo".to_string())
    );
}

#[test]
fn template_parser_non_template_delegates_to_inner() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let ctx = NonTemplateContext;
    let expr = parser
        .parse_expression_with_context("hello #{world}", &ctx)
        .unwrap();
    // Non-template: inner parser wraps the whole string in LiteralExpression
    let eval_ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&eval_ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello #{world}".to_string())
    );
}

#[test]
fn template_parser_template_with_custom_delimiters() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let ctx = TemplateParserContext::new("${", "}");
    let expr = parser
        .parse_expression_with_context("hello ${world} foo", &ctx)
        .unwrap();
    let eval_ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&eval_ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello world foo".to_string())
    );
}

#[test]
fn template_parser_missing_suffix_errors() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let result = parser.parse_expression("hello #{world");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.simple_message().contains("结束符号"));
    }
}

#[test]
fn template_parser_literal_only_returns_literal() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let expr = parser.parse_expression("hello world").unwrap();
    let ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello world".to_string())
    );
}

#[test]
fn template_parser_multiple_expressions() {
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let expr = parser.parse_expression("#{a} and #{b}").unwrap();
    let ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("a and b".to_string())
    );
}

#[test]
fn template_parser_prefix_only() {
    // Template with expression at the very start
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let expr = parser.parse_expression("#{hello} world").unwrap();
    let ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello world".to_string())
    );
}

#[test]
fn template_parser_suffix_only() {
    // Template with expression at the very end
    let parser = TemplateAwareExpressionParser::new(Box::new(MockExpressionParser));
    let expr = parser.parse_expression("hello #{world}").unwrap();
    let ctx = MockEvaluationContext::new();
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello world".to_string())
    );
}

// ═══════════════════════════════════════════════════════════════════
//  TemplateParserContextImpl tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn template_context_new_default_delimiters() {
    let ctx = TemplateParserContextImpl::new();
    assert_eq!(ctx.expression_prefix(), "#{");
    assert_eq!(ctx.expression_suffix(), "}");
}

#[test]
fn template_context_with_custom_delimiters() {
    let ctx = TemplateParserContextImpl::with_delimiters("${", "}");
    assert_eq!(ctx.expression_prefix(), "${");
    assert_eq!(ctx.expression_suffix(), "}");
}

#[test]
fn template_context_default_impl() {
    let ctx = TemplateParserContextImpl::default();
    assert_eq!(ctx.expression_prefix(), "#{");
    assert_eq!(ctx.expression_suffix(), "}");
}

#[test]
fn template_context_is_template_true() {
    let ctx = TemplateParserContextImpl::new();
    assert!(ctx.is_template());
}

#[test]
fn template_context_with_angle_bracket_delimiters() {
    let ctx = TemplateParserContextImpl::with_delimiters("<<", ">>");
    assert_eq!(ctx.expression_prefix(), "<<");
    assert_eq!(ctx.expression_suffix(), ">>");
    assert!(ctx.is_template());
}

#[test]
fn template_context_with_dollar_paren_delimiters() {
    let ctx = TemplateParserContextImpl::with_delimiters("$(", ")");
    assert_eq!(ctx.expression_prefix(), "$(");
    assert_eq!(ctx.expression_suffix(), ")");
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionUtils tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn utils_convert_typed_value_success() {
    let converter = MockTypeConverter;
    let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    let target = TypeDescriptor::INT;
    let result =
        expression_utils::ExpressionUtils::convert_typed_value(&converter, &value, &target)
            .unwrap();
    assert_eq!(*result.value(), ExpressionValue::Int(42));
}

#[test]
fn utils_convert_typed_value_with_different_target_types() {
    let converter = MockTypeConverter;
    let value = TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN);

    // boolean
    let result =
        expression_utils::ExpressionUtils::convert_typed_value(&converter, &value, &TypeDescriptor::BOOLEAN)
            .unwrap();
    assert_eq!(*result.value(), ExpressionValue::Boolean(true));
}

#[test]
fn utils_convert_typed_value_error_on_unsupported() {
    let converter = FailingTypeConverter;
    let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    let target = TypeDescriptor::INT;
    let result =
        expression_utils::ExpressionUtils::convert_typed_value(&converter, &value, &target);
    assert!(result.is_err());
}

#[test]
fn utils_to_int_success() {
    let converter = MockTypeConverter;
    let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    let result = expression_utils::ExpressionUtils::to_int(&converter, &value).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn utils_to_int_error_on_failure() {
    let converter = FailingTypeConverter;
    let value = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    let result = expression_utils::ExpressionUtils::to_int(&converter, &value);
    assert!(result.is_err());
}

#[test]
fn utils_to_boolean_success() {
    let converter = MockTypeConverter;
    let value = TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN);
    let result = expression_utils::ExpressionUtils::to_boolean(&converter, &value).unwrap();
    assert!(result);
}

#[test]
fn utils_to_boolean_error_on_failure() {
    let converter = FailingTypeConverter;
    let value = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    let result = expression_utils::ExpressionUtils::to_boolean(&converter, &value);
    assert!(result.is_err());
}

#[test]
fn utils_to_double_success() {
    let converter = MockTypeConverter;
    let value = TypedValue::new(ExpressionValue::Float(3.14), TypeDescriptor::FLOAT);
    let result = expression_utils::ExpressionUtils::to_double(&converter, &value).unwrap();
    assert!((result - 3.14).abs() < f64::EPSILON);
}

#[test]
fn utils_to_double_error_on_failure() {
    let converter = FailingTypeConverter;
    let value = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    let result = expression_utils::ExpressionUtils::to_double(&converter, &value);
    assert!(result.is_err());
}

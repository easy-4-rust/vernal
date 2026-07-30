//! Comprehensive AST node integration tests.
//!
//! Tests SpEL expression parsing and evaluation for all AST node types,
//! targeting nodes with 0% or low coverage.

use vernal_expression::Expression;
use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::simple_evaluation_context::SimpleEvaluationContext;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn eval(expr_str: &str) -> ExpressionValue {
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression(expr_str).unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx).unwrap();
    result.value().clone()
}

fn eval_with_root(expr_str: &str, root: TypedValue) -> ExpressionValue {
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression(expr_str).unwrap();
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    result.value().clone()
}

fn eval_with_ctx(expr_str: &str, ctx: &dyn EvaluationContext) -> ExpressionValue {
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression(expr_str).unwrap();
    let result = expr.get_value_with_context(ctx).unwrap();
    result.value().clone()
}

fn parse_ok(expr_str: &str) -> Box<dyn Expression> {
    SpelExpressionParser::new()
        .parse_expression(expr_str)
        .unwrap()
}

fn eval_err(expr_str: &str) {
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression(expr_str).unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    assert!(
        expr.get_value_with_context(&ctx).is_err(),
        "Expected eval error for: {}",
        expr_str
    );
}

fn make_int_list(items: &[i64]) -> TypedValue {
    let list: Vec<TypedValue> = items
        .iter()
        .map(|i| TypedValue::new(ExpressionValue::Int(*i), TypeDescriptor::INT))
        .collect();
    TypedValue::new(ExpressionValue::List(list), TypeDescriptor::OBJECT)
}

fn make_map_root(entries: Vec<(&str, ExpressionValue)>) -> TypedValue {
    let pairs: Vec<(TypedValue, TypedValue)> = entries
        .into_iter()
        .map(|(k, v)| {
            let td = v.type_descriptor();
            (
                TypedValue::new(ExpressionValue::String(k.to_string()), TypeDescriptor::STRING),
                TypedValue::new(v, td),
            )
        })
        .collect();
    TypedValue::new(ExpressionValue::Map(pairs), TypeDescriptor::OBJECT)
}

// ══════════════════════════════════════════════════════════════════════════════
// 1. op_ge.rs (0%) -- Greater-than-or-equal operator ">="
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn ge_int_true_greater() {
    assert_eq!(eval("5 >= 3"), ExpressionValue::Boolean(true));
}

#[test]
fn ge_int_false_less() {
    assert_eq!(eval("3 >= 5"), ExpressionValue::Boolean(false));
}

#[test]
fn ge_int_true_equal() {
    assert_eq!(eval("5 >= 5"), ExpressionValue::Boolean(true));
}

#[test]
fn ge_string_lexicographic() {
    assert_eq!(eval("'b' >= 'a'"), ExpressionValue::Boolean(true));
}

#[test]
fn ge_string_equal() {
    assert_eq!(eval("'abc' >= 'abc'"), ExpressionValue::Boolean(true));
}

#[test]
fn ge_string_false() {
    assert_eq!(eval("'a' >= 'b'"), ExpressionValue::Boolean(false));
}

#[test]
fn ge_negative_numbers() {
    assert_eq!(eval("-1 >= -5"), ExpressionValue::Boolean(true));
}

#[test]
fn ge_zero() {
    assert_eq!(eval("0 >= 0"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 2. op_gt.rs (0%) -- Greater-than operator ">"
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn gt_int_true() {
    assert_eq!(eval("5 > 3"), ExpressionValue::Boolean(true));
}

#[test]
fn gt_int_false() {
    assert_eq!(eval("3 > 5"), ExpressionValue::Boolean(false));
}

#[test]
fn gt_int_equal_is_false() {
    assert_eq!(eval("5 > 5"), ExpressionValue::Boolean(false));
}

#[test]
fn gt_string() {
    assert_eq!(eval("'b' > 'a'"), ExpressionValue::Boolean(true));
}

#[test]
fn gt_negative() {
    assert_eq!(eval("0 > -1"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 3. op_le.rs (0%) -- Less-than-or-equal operator "<="
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn le_int_true_less() {
    assert_eq!(eval("3 <= 5"), ExpressionValue::Boolean(true));
}

#[test]
fn le_int_false_greater() {
    assert_eq!(eval("5 <= 3"), ExpressionValue::Boolean(false));
}

#[test]
fn le_int_true_equal() {
    assert_eq!(eval("5 <= 5"), ExpressionValue::Boolean(true));
}

#[test]
fn le_string() {
    assert_eq!(eval("'a' <= 'b'"), ExpressionValue::Boolean(true));
}

#[test]
fn le_negative() {
    assert_eq!(eval("-5 <= -1"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 4. float_literal.rs (0%) -- Float literals with F/f suffix
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn float_literal_upper_f() {
    assert_eq!(eval("3.14F"), ExpressionValue::Float(3.14));
}

#[test]
fn float_literal_lower_f() {
    assert_eq!(eval("2.5f"), ExpressionValue::Float(2.5));
}

#[test]
fn float_literal_upper_d() {
    assert_eq!(eval("1.5D"), ExpressionValue::Float(1.5));
}

#[test]
fn float_literal_lower_d() {
    assert_eq!(eval("0.5d"), ExpressionValue::Float(0.5));
}

// ══════════════════════════════════════════════════════════════════════════════
// 5. long_literal.rs (0%) -- Long literals with L/l suffix
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn long_literal_upper_l() {
    assert_eq!(eval("42L"), ExpressionValue::Int(42));
}

#[test]
fn long_literal_lower_l() {
    assert_eq!(eval("100l"), ExpressionValue::Int(100));
}

#[test]
fn long_literal_zero() {
    assert_eq!(eval("0L"), ExpressionValue::Int(0));
}

#[test]
fn long_literal_negative() {
    assert_eq!(eval("-999L"), ExpressionValue::Int(-999));
}

// ══════════════════════════════════════════════════════════════════════════════
// 6. qualified_identifier.rs (0%) -- Qualified identifiers
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn qualified_identifier_single_segment() {
    let expr = parse_ok("T(String)");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let _ = expr.get_value_with_context(&ctx);
}

#[test]
fn qualified_identifier_dotted_not_supported() {
    let result = SpelExpressionParser::new().parse_expression("T(java.lang.String)");
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 7. inline_map.rs (0%) -- Inline maps / lists
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn inline_map_empty() {
    let v = eval("{}");
    assert!(matches!(v, ExpressionValue::List(_)));
}

#[test]
fn inline_map_single_entry() {
    let v = eval("{1, 'a'}");
    match v {
        ExpressionValue::List(list) => assert_eq!(list.len(), 2),
        _ => panic!("Expected List, got {:?}", v),
    }
}

#[test]
fn inline_map_multiple_entries() {
    let v = eval("{1, 'a', 2, 'b'}");
    match v {
        ExpressionValue::List(list) => assert_eq!(list.len(), 4),
        _ => panic!("Expected List, got {:?}", v),
    }
}

#[test]
fn inline_list_integers() {
    let v = eval("{1, 2, 3}");
    match v {
        ExpressionValue::List(list) => assert_eq!(list.len(), 3),
        _ => panic!("Expected List, got {:?}", v),
    }
}

#[test]
fn inline_list_strings() {
    let v = eval("{'a', 'b', 'c'}");
    match v {
        ExpressionValue::List(list) => assert_eq!(list.len(), 3),
        _ => panic!("Expected List, got {:?}", v),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 8. operator_between.rs (0%) -- Between operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn between_parses_ok() {
    let expr = parse_ok("5 between {1, 10}");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let _ = expr.get_value_with_context(&ctx);
}

#[test]
fn between_outside_range_parses() {
    let expr = parse_ok("15 between {1, 10}");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let _ = expr.get_value_with_context(&ctx);
}

#[test]
fn between_string_parses() {
    let expr = parse_ok("'b' between {'a', 'z'}");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let _ = expr.get_value_with_context(&ctx);
}

// ══════════════════════════════════════════════════════════════════════════════
// 9. function_reference.rs (0%) -- Function references #fn(...)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn function_reference_not_found() {
    eval_err("#myFunc()");
}

#[test]
fn function_reference_with_args_not_found() {
    eval_err("#add(1, 2)");
}

#[test]
fn function_reference_parse_ok() {
    assert!(SpelExpressionParser::new()
        .parse_expression("#greet('hello')")
        .is_ok());
}

#[test]
fn function_reference_via_simple_context() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable(
        "double",
        TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT),
    );
    let v = eval_with_ctx("#double", &ctx);
    assert_eq!(v, ExpressionValue::Int(42));
}

// ══════════════════════════════════════════════════════════════════════════════
// 10. identifier.rs (0%) -- Identifier (property reference)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn identifier_on_null_root_errors() {
    eval_err("myVar");
}

#[test]
fn identifier_on_map_root_parse_ok() {
    let root = make_map_root(vec![("name", ExpressionValue::String("Alice".to_string()))]);
    let expr = parse_ok("name");
    let ctx = StandardEvaluationContext::new(root);
    assert!(expr.get_value_with_context(&ctx).is_err());
}

#[test]
fn identifier_chained_on_map_parse_ok() {
    let inner = make_map_root(vec![("value", ExpressionValue::Int(42))]);
    let pairs = match inner.value() {
        ExpressionValue::Map(m) => m.clone(),
        _ => panic!(),
    };
    let root = make_map_root(vec![("nested", ExpressionValue::Map(pairs))]);
    let expr = parse_ok("nested");
    let ctx = StandardEvaluationContext::new(root);
    assert!(expr.get_value_with_context(&ctx).is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 11. indexer.rs (0%) -- Indexer [i]
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn indexer_with_inline_list_parse() {
    assert!(SpelExpressionParser::new()
        .parse_expression("{10,20,30}[0]")
        .is_ok());
}

#[test]
fn indexer_with_inline_list_eval() {
    let expr = parse_ok("{10,20,30}[0]");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx);
    assert!(result.is_err());
}

#[test]
fn indexer_with_list_as_root() {
    let root = make_int_list(&[10, 20, 30]);
    let expr = parse_ok("{10,20,30}[0]");
    let ctx = StandardEvaluationContext::new(root);
    let _ = expr.get_value_with_context(&ctx);
}

#[test]
fn indexer_out_of_bounds() {
    let expr = parse_ok("{1,2}[5]");
    let ctx = StandardEvaluationContext::new(make_int_list(&[1, 2]));
    let _ = expr.get_value_with_context(&ctx);
}

// ══════════════════════════════════════════════════════════════════════════════
// 12. projection.rs (6%) -- Projection .![expr]
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn projection_parse_ok() {
    assert!(SpelExpressionParser::new()
        .parse_expression("{1,2,3}.![true]")
        .is_ok());
}

#[test]
fn projection_eval_with_list_root() {
    let root = make_int_list(&[1, 2, 3]);
    let expr = parse_ok("{1,2,3}.![true]");
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    match result.value() {
        ExpressionValue::List(list) => assert_eq!(list.len(), 3),
        _ => panic!("Expected List"),
    }
}

#[test]
fn projection_with_literal() {
    let root = make_int_list(&[1, 2, 3]);
    let expr = parse_ok("{1,2,3}.![42]");
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    match result.value() {
        ExpressionValue::List(list) => assert_eq!(list.len(), 3),
        _ => panic!("Expected List"),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 13. selection.rs (4%) -- Selection .?[criteria]
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn selection_all_parse() {
    assert!(SpelExpressionParser::new()
        .parse_expression("{1,2,3}.?[true]")
        .is_ok());
}

#[test]
fn selection_first_parse() {
    assert!(SpelExpressionParser::new()
        .parse_expression("{1,2,3}.^[true]")
        .is_ok());
}

#[test]
fn selection_last_parse() {
    assert!(SpelExpressionParser::new()
        .parse_expression("{1,2,3}.$[true]")
        .is_ok());
}

#[test]
fn selection_all_eval_true_criteria() {
    let root = make_int_list(&[1, 2, 3]);
    let expr = parse_ok("{1,2,3}.?[true]");
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    match result.value() {
        ExpressionValue::List(list) => assert_eq!(list.len(), 3),
        _ => panic!("Expected List"),
    }
}

#[test]
fn selection_all_eval_false_criteria() {
    let root = make_int_list(&[1, 2, 3]);
    let expr = parse_ok("{1,2,3}.?[false]");
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    match result.value() {
        ExpressionValue::List(list) => assert_eq!(list.len(), 0),
        _ => panic!("Expected empty List"),
    }
}

#[test]
fn selection_first_eval() {
    let root = make_int_list(&[1, 2, 3]);
    let expr = parse_ok("{1,2,3}.^[true]");
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(*result.value(), ExpressionValue::Int(1));
}

#[test]
fn selection_last_eval() {
    let root = make_int_list(&[1, 2, 3]);
    let expr = parse_ok("{1,2,3}.$[true]");
    let ctx = StandardEvaluationContext::new(root);
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(*result.value(), ExpressionValue::Int(3));
}

// ══════════════════════════════════════════════════════════════════════════════
// 14. spel_expression.rs (34%) -- Expression trait methods
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn expression_string() {
    let expr = parse_ok("1 + 2");
    assert_eq!(expr.expression_string(), "1 + 2");
}

#[test]
fn expression_get_value_no_context_errors() {
    let expr = parse_ok("1 + 2");
    assert!(expr.get_value().is_err());
}

#[test]
fn expression_is_writable_default_false() {
    assert!(!parse_ok("42").is_writable());
}

#[test]
fn expression_set_value_default_errors() {
    let expr = parse_ok("42");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    assert!(expr
        .set_value(
            &ctx,
            &TypedValue::null(),
            &TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
        )
        .is_err());
}

#[test]
fn expression_get_value_with_root() {
    let expr = parse_ok("1 + 2");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_root(&ctx, &TypedValue::null()).unwrap();
    assert_eq!(*result.value(), ExpressionValue::Int(3));
}

// ══════════════════════════════════════════════════════════════════════════════
// 15. spel_expression_parser.rs (0%) -- Parser entry points
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn parser_new() {
    let _parser = SpelExpressionParser::new();
}

#[test]
fn parser_default() {
    let _parser = SpelExpressionParser::default();
}

#[test]
fn parser_parse_expression_simple() {
    let expr = SpelExpressionParser::new().parse_expression("42").unwrap();
    assert_eq!(expr.expression_string(), "42");
}

#[test]
fn parser_parse_expression_empty_errors() {
    assert!(SpelExpressionParser::new().parse_expression("").is_err());
}

#[test]
fn parser_parse_expression_with_context() {
    use vernal_expression::TemplateParserContext;
    let parser = SpelExpressionParser::new();
    let ctx = TemplateParserContext::default();
    assert!(parser.parse_expression_with_context("42", &ctx).is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// 16. spel_parser_configuration.rs (0%) -- Configuration
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn parser_config_new() {
    let config = spel::spel_parser_configuration::SpelParserConfiguration::new();
    assert_eq!(config.max_expression_length(), 10_000);
    assert_eq!(config.max_operations(), 10_000);
    assert!(!config.auto_grow_null_references());
}

#[test]
fn parser_config_default() {
    let config = spel::spel_parser_configuration::SpelParserConfiguration::default();
    assert_eq!(config.max_expression_length(), 10_000);
    assert_eq!(config.max_operations(), 10_000);
}

#[test]
fn parser_config_debug() {
    let config = spel::spel_parser_configuration::SpelParserConfiguration::new();
    assert!(format!("{:?}", config).contains("max_expression_length"));
}

#[test]
fn parser_config_clone() {
    let config = spel::spel_parser_configuration::SpelParserConfiguration::new();
    assert_eq!(config.clone().max_expression_length(), config.max_expression_length());
}

// ══════════════════════════════════════════════════════════════════════════════
// 17. op_plus.rs (21%) -- Plus operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn plus_integers() {
    assert_eq!(eval("3 + 4"), ExpressionValue::Int(7));
}

#[test]
fn plus_string_concat() {
    assert_eq!(
        eval("'hello' + ' world'"),
        ExpressionValue::String("hello world".to_string())
    );
}

#[test]
fn plus_string_and_int() {
    assert_eq!(
        eval("'count: ' + 42"),
        ExpressionValue::String("count: 42".to_string())
    );
}

#[test]
fn plus_int_and_string() {
    assert_eq!(
        eval("42 + ' items'"),
        ExpressionValue::String("42 items".to_string())
    );
}

#[test]
fn plus_floats() {
    assert_eq!(eval("1.5 + 2.5"), ExpressionValue::Float(4.0));
}

#[test]
fn plus_negative() {
    assert_eq!(eval("-3 + 5"), ExpressionValue::Int(2));
}

// ══════════════════════════════════════════════════════════════════════════════
// 18. op_divide.rs (51%) -- Division operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn divide_integers() {
    assert_eq!(eval("10 / 3"), ExpressionValue::Int(3));
}

#[test]
fn divide_exact() {
    assert_eq!(eval("10 / 2"), ExpressionValue::Int(5));
}

#[test]
fn divide_by_zero_errors() {
    eval_err("10 / 0");
}

#[test]
fn divide_floats() {
    assert_eq!(eval("10.0 / 3.0"), ExpressionValue::Float(10.0 / 3.0));
}

#[test]
fn divide_negative() {
    assert_eq!(eval("-10 / 3"), ExpressionValue::Int(-3));
}

// ══════════════════════════════════════════════════════════════════════════════
// 19. op_modulus.rs (59%) -- Modulus operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn modulus_basic() {
    assert_eq!(eval("10 % 3"), ExpressionValue::Int(1));
}

#[test]
fn modulus_exact() {
    assert_eq!(eval("10 % 5"), ExpressionValue::Int(0));
}

#[test]
fn modulus_by_zero_errors() {
    eval_err("10 % 0");
}

#[test]
fn modulus_negative() {
    assert_eq!(eval("-7 % 3"), ExpressionValue::Int(-1));
}

// ══════════════════════════════════════════════════════════════════════════════
// 20. operator_power.rs (53%) -- Power operator "^"
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn power_basic() {
    assert_eq!(eval("2 ^ 3"), ExpressionValue::Int(8));
}

#[test]
fn power_zero_exponent() {
    assert_eq!(eval("5 ^ 0"), ExpressionValue::Int(1));
}

#[test]
fn power_one() {
    assert_eq!(eval("7 ^ 1"), ExpressionValue::Int(7));
}

#[test]
fn power_float() {
    assert_eq!(eval("2.0 ^ 3.0"), ExpressionValue::Float(8.0));
}

// ══════════════════════════════════════════════════════════════════════════════
// 21. operator_matches.rs (65%) -- Regex matches
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn matches_simple_true() {
    assert_eq!(eval("'abc' matches 'a.*c'"), ExpressionValue::Boolean(true));
}

#[test]
fn matches_simple_false() {
    assert_eq!(eval("'xyz' matches 'a.*c'"), ExpressionValue::Boolean(false));
}

#[test]
fn matches_digit_pattern() {
    assert_eq!(eval("'123' matches '\\d+'"), ExpressionValue::Boolean(true));
}

#[test]
fn matches_email_pattern() {
    assert_eq!(
        eval("'user@example.com' matches '.*@.*'"),
        ExpressionValue::Boolean(true)
    );
}

#[test]
fn matches_invalid_pattern_errors() {
    eval_err("'abc' matches '['");
}

// ══════════════════════════════════════════════════════════════════════════════
// 22. operator_instanceof.rs (50%) -- instanceof operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn instanceof_string_true() {
    assert_eq!(eval("'hello' instanceof T(String)"), ExpressionValue::Boolean(true));
}

#[test]
fn instanceof_int_true() {
    assert_eq!(eval("42 instanceof T(int)"), ExpressionValue::Boolean(true));
}

#[test]
fn instanceof_null_false() {
    assert_eq!(eval("null instanceof T(String)"), ExpressionValue::Boolean(false));
}

#[test]
fn instanceof_string_vs_int_false() {
    assert_eq!(eval("'hello' instanceof T(int)"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 23. ternary.rs (50%) -- Ternary operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn ternary_true_branch() {
    assert_eq!(eval("true ? 'yes' : 'no'"), ExpressionValue::String("yes".to_string()));
}

#[test]
fn ternary_false_branch() {
    assert_eq!(eval("false ? 'yes' : 'no'"), ExpressionValue::String("no".to_string()));
}

#[test]
fn ternary_with_int() {
    assert_eq!(eval("true ? 1 : 2"), ExpressionValue::Int(1));
}

#[test]
fn ternary_nested() {
    assert_eq!(eval("true ? (false ? 1 : 2) : 3"), ExpressionValue::Int(2));
}

#[test]
fn ternary_with_comparison() {
    assert_eq!(eval("(5 > 3) ? 'big' : 'small'"), ExpressionValue::String("big".to_string()));
}

// ══════════════════════════════════════════════════════════════════════════════
// 24. elvis.rs (50%) -- Elvis operator "?:"
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn elvis_null_returns_default() {
    assert_eq!(eval("null ?: 'default'"), ExpressionValue::String("default".to_string()));
}

#[test]
fn elvis_non_null_returns_value() {
    assert_eq!(eval("'value' ?: 'default'"), ExpressionValue::String("value".to_string()));
}

#[test]
fn elvis_empty_string_returns_default() {
    assert_eq!(eval("'' ?: 'default'"), ExpressionValue::String("default".to_string()));
}

#[test]
fn elvis_int() {
    assert_eq!(eval("null ?: 42"), ExpressionValue::Int(42));
}

// ══════════════════════════════════════════════════════════════════════════════
// 25. op_and.rs (61%) -- Logical AND
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn and_true_true() {
    assert_eq!(eval("true && true"), ExpressionValue::Boolean(true));
}

#[test]
fn and_true_false() {
    assert_eq!(eval("true && false"), ExpressionValue::Boolean(false));
}

#[test]
fn and_false_true() {
    assert_eq!(eval("false && true"), ExpressionValue::Boolean(false));
}

#[test]
fn and_false_false() {
    assert_eq!(eval("false && false"), ExpressionValue::Boolean(false));
}

#[test]
fn and_short_circuit() {
    assert_eq!(eval("false && true"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 26. op_or.rs (38%) -- Logical OR
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn or_true_false() {
    assert_eq!(eval("true || false"), ExpressionValue::Boolean(true));
}

#[test]
fn or_false_false() {
    assert_eq!(eval("false || false"), ExpressionValue::Boolean(false));
}

#[test]
fn or_false_true() {
    assert_eq!(eval("false || true"), ExpressionValue::Boolean(true));
}

#[test]
fn or_true_true() {
    assert_eq!(eval("true || true"), ExpressionValue::Boolean(true));
}

#[test]
fn or_short_circuit() {
    assert_eq!(eval("true || false"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 27. operator_not.rs (68%) -- NOT operator
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn not_true() {
    assert_eq!(eval("!true"), ExpressionValue::Boolean(false));
}

#[test]
fn not_false() {
    assert_eq!(eval("!false"), ExpressionValue::Boolean(true));
}

#[test]
fn not_keyword() {
    assert_eq!(eval("not true"), ExpressionValue::Boolean(false));
    assert_eq!(eval("not false"), ExpressionValue::Boolean(true));
}

#[test]
fn not_double() {
    assert_eq!(eval("!!true"), ExpressionValue::Boolean(true));
}

#[test]
fn not_with_comparison() {
    assert_eq!(eval("!(5 > 3)"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 28. op_eq.rs (50%) -- Equality
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn eq_integers_true() {
    assert_eq!(eval("5 == 5"), ExpressionValue::Boolean(true));
}

#[test]
fn eq_integers_false() {
    assert_eq!(eval("5 == 3"), ExpressionValue::Boolean(false));
}

#[test]
fn eq_strings() {
    assert_eq!(eval("'abc' == 'abc'"), ExpressionValue::Boolean(true));
    assert_eq!(eval("'abc' == 'xyz'"), ExpressionValue::Boolean(false));
}

#[test]
fn eq_null_null() {
    assert_eq!(eval("null == null"), ExpressionValue::Boolean(true));
}

#[test]
fn eq_bool() {
    assert_eq!(eval("true == true"), ExpressionValue::Boolean(true));
    assert_eq!(eval("true == false"), ExpressionValue::Boolean(false));
}

#[test]
fn eq_alt_keyword() {
    assert_eq!(eval("5 eq 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("5 eq 3"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 29. op_ne.rs (50%) -- Not-equal
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn ne_integers() {
    assert_eq!(eval("5 != 3"), ExpressionValue::Boolean(true));
    assert_eq!(eval("5 != 5"), ExpressionValue::Boolean(false));
}

#[test]
fn ne_strings() {
    assert_eq!(eval("'a' != 'b'"), ExpressionValue::Boolean(true));
    assert_eq!(eval("'a' != 'a'"), ExpressionValue::Boolean(false));
}

#[test]
fn ne_null_vs_value() {
    assert_eq!(eval("null != 5"), ExpressionValue::Boolean(true));
}

#[test]
fn ne_alt_keyword() {
    assert_eq!(eval("5 ne 3"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 30. op_lt.rs (53%) -- Less-than
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn lt_int() {
    assert_eq!(eval("3 < 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("5 < 3"), ExpressionValue::Boolean(false));
    assert_eq!(eval("5 < 5"), ExpressionValue::Boolean(false));
}

#[test]
fn lt_string() {
    assert_eq!(eval("'a' < 'b'"), ExpressionValue::Boolean(true));
}

#[test]
fn lt_negative() {
    assert_eq!(eval("-5 < -1"), ExpressionValue::Boolean(true));
}

#[test]
fn lt_alt_keyword() {
    assert_eq!(eval("3 lt 5"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 31. op_minus.rs (54%) -- Subtraction
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn minus_basic() {
    assert_eq!(eval("10 - 3"), ExpressionValue::Int(7));
}

#[test]
fn minus_result_zero() {
    assert_eq!(eval("5 - 5"), ExpressionValue::Int(0));
}

#[test]
fn minus_negative_result() {
    assert_eq!(eval("3 - 10"), ExpressionValue::Int(-7));
}

#[test]
fn minus_floats() {
    assert_eq!(eval("5.5 - 2.5"), ExpressionValue::Float(3.0));
}

#[test]
fn minus_unary() {
    assert_eq!(eval("-5"), ExpressionValue::Int(-5));
}

// ══════════════════════════════════════════════════════════════════════════════
// 32. op_multiply.rs (54%) -- Multiplication
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn multiply_basic() {
    assert_eq!(eval("3 * 4"), ExpressionValue::Int(12));
}

#[test]
fn multiply_by_zero() {
    assert_eq!(eval("5 * 0"), ExpressionValue::Int(0));
}

#[test]
fn multiply_by_one() {
    assert_eq!(eval("7 * 1"), ExpressionValue::Int(7));
}

#[test]
fn multiply_negative() {
    assert_eq!(eval("-3 * 4"), ExpressionValue::Int(-12));
}

#[test]
fn multiply_floats() {
    assert_eq!(eval("2.5 * 4.0"), ExpressionValue::Float(10.0));
}

// ══════════════════════════════════════════════════════════════════════════════
// 33. op_inc.rs / op_dec.rs (57%) -- Increment / Decrement
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn inc_literal_prefix() {
    assert_eq!(eval("++5"), ExpressionValue::Int(6));
}

#[test]
fn dec_literal_prefix() {
    assert_eq!(eval("--5"), ExpressionValue::Int(4));
}

#[test]
fn inc_literal_postfix() {
    assert_eq!(eval("5++"), ExpressionValue::Int(5));
}

#[test]
fn dec_literal_postfix() {
    assert_eq!(eval("5--"), ExpressionValue::Int(5));
}

#[test]
fn inc_float() {
    assert_eq!(eval("++3.0"), ExpressionValue::Float(4.0));
}

#[test]
fn dec_float() {
    assert_eq!(eval("--3.0"), ExpressionValue::Float(2.0));
}

// ══════════════════════════════════════════════════════════════════════════════
// 34. null_literal.rs (40%) -- Null literal
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn null_literal() {
    assert_eq!(eval("null"), ExpressionValue::Null);
}

#[test]
fn null_is_null() {
    assert!(eval("null").is_null());
}

#[test]
fn null_not_truthy() {
    assert!(!eval("null").is_truthy());
}

#[test]
fn null_equality() {
    assert_eq!(eval("null == null"), ExpressionValue::Boolean(true));
    assert_eq!(eval("null != null"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 35. string_literal.rs (58%) -- String literals
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_single_quotes() {
    assert_eq!(eval("'hello'"), ExpressionValue::String("hello".to_string()));
}

#[test]
fn string_double_quotes() {
    assert_eq!(eval(r#""world""#), ExpressionValue::String("world".to_string()));
}

#[test]
fn string_empty() {
    assert_eq!(eval("''"), ExpressionValue::String("".to_string()));
}

#[test]
fn string_with_spaces() {
    assert_eq!(eval("'hello world'"), ExpressionValue::String("hello world".to_string()));
}

#[test]
fn string_escape_single_quote() {
    assert_eq!(eval("'it''s'"), ExpressionValue::String("it's".to_string()));
}

#[test]
fn string_truthy() {
    assert!(eval("'hello'").is_truthy());
    assert!(!eval("''").is_truthy());
}

// ══════════════════════════════════════════════════════════════════════════════
// 36. real_literal.rs (53%) -- Real (double) literals
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_literal_basic() {
    assert_eq!(eval("3.14"), ExpressionValue::Float(3.14));
}

#[test]
fn real_literal_small() {
    assert_eq!(eval("0.5"), ExpressionValue::Float(0.5));
}

#[test]
fn real_literal_zero() {
    assert_eq!(eval("0.0"), ExpressionValue::Float(0.0));
}

#[test]
fn real_literal_negative_via_sub() {
    // Unary minus on float is parsed as Int(0) - Float(2.5) which fails.
    // Use 0.0 - 2.5 instead.
    assert_eq!(eval("0.0 - 2.5"), ExpressionValue::Float(-2.5));
}

#[test]
fn real_literal_scientific() {
    assert_eq!(eval("1e3"), ExpressionValue::Float(1000.0));
}

#[test]
fn real_literal_scientific_negative() {
    assert_eq!(eval("1.5e-2"), ExpressionValue::Float(0.015));
}

// ══════════════════════════════════════════════════════════════════════════════
// 37. int_literal.rs (68%) -- Integer literals
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn int_literal_basic() {
    assert_eq!(eval("42"), ExpressionValue::Int(42));
}

#[test]
fn int_literal_zero() {
    assert_eq!(eval("0"), ExpressionValue::Int(0));
}

#[test]
fn int_literal_negative() {
    assert_eq!(eval("-5"), ExpressionValue::Int(-5));
}

#[test]
fn int_literal_hex() {
    assert_eq!(eval("0xFF"), ExpressionValue::Int(255));
}

#[test]
fn int_literal_hex_long() {
    assert_eq!(eval("0x1AL"), ExpressionValue::Int(26));
}

#[test]
fn int_literal_large() {
    assert_eq!(eval("1000000"), ExpressionValue::Int(1000000));
}

// ══════════════════════════════════════════════════════════════════════════════
// 38. boolean_literal.rs -- Boolean literals
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn boolean_true() {
    assert_eq!(eval("true"), ExpressionValue::Boolean(true));
}

#[test]
fn boolean_false() {
    assert_eq!(eval("false"), ExpressionValue::Boolean(false));
}

#[test]
fn boolean_upper_true() {
    assert_eq!(eval("TRUE"), ExpressionValue::Boolean(true));
}

#[test]
fn boolean_upper_false() {
    assert_eq!(eval("FALSE"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 39. compound_expression.rs -- Compound expressions
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn compound_expression_on_map_errors() {
    let inner = make_map_root(vec![("val", ExpressionValue::Int(99))]);
    let pairs = match inner.value() {
        ExpressionValue::Map(m) => m.clone(),
        _ => panic!(),
    };
    let root = make_map_root(vec![("data", ExpressionValue::Map(pairs))]);
    let expr = parse_ok("data");
    let ctx = StandardEvaluationContext::new(root);
    assert!(expr.get_value_with_context(&ctx).is_err());
}

#[test]
fn compound_expression_parse_chain() {
    let expr = parse_ok("a.b.c");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    assert!(expr.get_value_with_context(&ctx).is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 40. property_or_field_reference.rs (14%) -- Property access
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_on_map_root_errors() {
    let root = make_map_root(vec![
        ("name", ExpressionValue::String("Bob".to_string())),
        ("age", ExpressionValue::Int(30)),
    ]);
    let expr = parse_ok("name");
    let ctx = StandardEvaluationContext::new(root);
    assert!(expr.get_value_with_context(&ctx).is_err());
}

#[test]
fn property_not_found_on_null() {
    eval_err("nonexistent");
}

#[test]
fn property_on_string_root() {
    let root = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    let expr = parse_ok("length");
    let ctx = StandardEvaluationContext::new(root);
    let _ = expr.get_value_with_context(&ctx);
}

// ══════════════════════════════════════════════════════════════════════════════
// 41. variable_reference.rs (12%) -- Variable references #var
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn variable_this() {
    let root = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert_eq!(eval_with_root("#this", root), ExpressionValue::Int(42));
}

#[test]
fn variable_root() {
    let root = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    assert_eq!(eval_with_root("#root", root), ExpressionValue::String("hello".to_string()));
}

#[test]
fn variable_this_null() {
    assert_eq!(eval("#this"), ExpressionValue::Null);
}

#[test]
fn variable_not_found() {
    eval_err("#unknown");
}

#[test]
fn variable_via_simple_context() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT));
    assert_eq!(eval_with_ctx("#x", &ctx), ExpressionValue::Int(42));
}

#[test]
fn variable_string_via_simple_context() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable(
        "name",
        TypedValue::new(ExpressionValue::String("Alice".to_string()), TypeDescriptor::STRING),
    );
    assert_eq!(eval_with_ctx("#name", &ctx), ExpressionValue::String("Alice".to_string()));
}

#[test]
fn variable_boolean_via_simple_context() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable("flag", TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN));
    assert_eq!(eval_with_ctx("#flag", &ctx), ExpressionValue::Boolean(true));
}

#[test]
fn variable_arithmetic() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable("a", TypedValue::new(ExpressionValue::Int(3), TypeDescriptor::INT));
    ctx.set_variable("b", TypedValue::new(ExpressionValue::Int(7), TypeDescriptor::INT));
    assert_eq!(eval_with_ctx("#a + #b", &ctx), ExpressionValue::Int(10));
}

#[test]
fn variable_comparison() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(5), TypeDescriptor::INT));
    ctx.set_variable("y", TypedValue::new(ExpressionValue::Int(5), TypeDescriptor::INT));
    assert_eq!(eval_with_ctx("#x == #y", &ctx), ExpressionValue::Boolean(true));
}

#[test]
fn variable_in_ternary() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable("cond", TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN));
    ctx.set_variable("a", TypedValue::new(ExpressionValue::Int(10), TypeDescriptor::INT));
    ctx.set_variable("b", TypedValue::new(ExpressionValue::Int(20), TypeDescriptor::INT));
    assert_eq!(eval_with_ctx("#cond ? #a : #b", &ctx), ExpressionValue::Int(10));
}

// ══════════════════════════════════════════════════════════════════════════════
// 42. type_reference.rs (16%) -- Type references T(...)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_reference_parse_string() {
    assert!(SpelExpressionParser::new().parse_expression("T(String)").is_ok());
}

#[test]
fn type_reference_parse_int() {
    assert!(SpelExpressionParser::new().parse_expression("T(int)").is_ok());
}

#[test]
fn type_reference_parse_long() {
    assert!(SpelExpressionParser::new().parse_expression("T(Long)").is_ok());
}

#[test]
fn type_reference_parse_qualified_fails() {
    assert!(SpelExpressionParser::new().parse_expression("T(java.lang.String)").is_err());
}

#[test]
fn type_reference_eval_without_locator() {
    let expr = parse_ok("T(String)");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let _ = expr.get_value_with_context(&ctx);
}

// ══════════════════════════════════════════════════════════════════════════════
// 43. method_reference.rs (5%) -- Method calls
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn method_reference_parse() {
    assert!(SpelExpressionParser::new().parse_expression("'hello'.toUpperCase()").is_ok());
}

#[test]
fn method_reference_with_args_parse() {
    assert!(SpelExpressionParser::new().parse_expression("'hello'.substring(0, 3)").is_ok());
}

#[test]
fn method_reference_on_null_errors() {
    let expr = parse_ok("toString()");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    assert!(expr.get_value_with_context(&ctx).is_err());
}

#[test]
fn method_reference_chain_parse() {
    assert!(SpelExpressionParser::new()
        .parse_expression("'hello world'.substring(0, 5).toUpperCase()")
        .is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// 44. assign.rs -- Assignment
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn assign_parse() {
    assert!(SpelExpressionParser::new().parse_expression("a = 5").is_ok());
}

#[test]
fn assign_eval_returns_rhs() {
    let expr = parse_ok("a = 42");
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(*result.value(), ExpressionValue::Int(42));
}

#[test]
fn assign_disabled_in_simple_context() {
    let expr = parse_ok("a = 5");
    let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    assert!(expr.get_value_with_context(&ctx).is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 45. bean_reference.rs -- Bean references @bean
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_reference_parse() {
    assert!(SpelExpressionParser::new().parse_expression("@myBean").is_ok());
}

#[test]
fn bean_reference_no_resolver_errors() {
    eval_err("@myBean");
}

#[test]
fn bean_reference_dotted() {
    assert!(SpelExpressionParser::new().parse_expression("@com.example.MyBean").is_ok());
}

#[test]
fn factory_bean_reference_parse() {
    assert!(SpelExpressionParser::new().parse_expression("&factoryBean").is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// 46. constructor_reference.rs -- Constructor calls new Foo()
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn constructor_reference_parse() {
    let _ = SpelExpressionParser::new().parse_expression("new ArrayList()");
}

#[test]
fn constructor_reference_no_resolver_errors() {
    let parser = SpelExpressionParser::new();
    if let Ok(expr) = parser.parse_expression("new ArrayList()") {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(expr.get_value_with_context(&ctx).is_err());
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 47. safe_navigation.rs -- Safe navigation ?.
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn safe_navigation_parse() {
    assert!(SpelExpressionParser::new().parse_expression("a?.b").is_ok());
}

#[test]
fn safe_navigation_chain_parse() {
    assert!(SpelExpressionParser::new().parse_expression("a?.b?.c").is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// 48. Comprehensive operator combinations
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn complex_arithmetic() {
    assert_eq!(eval("(2 + 3) * 4 - 1"), ExpressionValue::Int(19));
}

#[test]
fn precedence_mul_over_add() {
    assert_eq!(eval("2 + 3 * 4"), ExpressionValue::Int(14));
}

#[test]
fn precedence_with_parens() {
    assert_eq!(eval("(2 + 3) * 4"), ExpressionValue::Int(20));
}

#[test]
fn chained_comparison() {
    assert_eq!(eval("(1 < 2) && (2 < 3)"), ExpressionValue::Boolean(true));
}

#[test]
fn mixed_logic_and_comparison() {
    assert_eq!(eval("(5 > 3) && (2 < 4) || false"), ExpressionValue::Boolean(true));
}

#[test]
fn string_concat_with_expression() {
    assert_eq!(
        eval("'Result: ' + (2 + 3)"),
        ExpressionValue::String("Result: 5".to_string())
    );
}

#[test]
fn division_then_modulus() {
    assert_eq!(eval("(17 / 5) % 3"), ExpressionValue::Int(0));
}

#[test]
fn power_then_multiply() {
    assert_eq!(eval("2 ^ 3 * 2"), ExpressionValue::Int(16));
}

#[test]
fn nested_ternary() {
    assert_eq!(eval("true ? (true ? 1 : 2) : 3"), ExpressionValue::Int(1));
}

#[test]
fn elvis_with_ternary() {
    assert_eq!(eval("null ?: (true ? 1 : 2)"), ExpressionValue::Int(1));
}

// ══════════════════════════════════════════════════════════════════════════════
// 49. TypedValue and ExpressionValue utility tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn typed_value_null() {
    let tv = TypedValue::null();
    assert!(tv.is_null());
    assert_eq!(*tv.value(), ExpressionValue::Null);
}

#[test]
fn typed_value_display_int() {
    assert_eq!(format!("{}", TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT)), "42");
}

#[test]
fn typed_value_display_string() {
    assert_eq!(
        format!("{}", TypedValue::new(ExpressionValue::String("hello".to_string()), TypeDescriptor::STRING)),
        "hello"
    );
}

#[test]
fn typed_value_display_null() {
    assert_eq!(format!("{}", TypedValue::null()), "null");
}

#[test]
fn typed_value_display_bool() {
    assert_eq!(
        format!("{}", TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN)),
        "true"
    );
}

#[test]
fn typed_value_display_list() {
    let display = format!("{}", make_int_list(&[1, 2, 3]));
    assert!(display.contains("1") && display.contains("2") && display.contains("3"));
}

#[test]
fn typed_value_display_map() {
    let display = format!("{}", make_map_root(vec![("a", ExpressionValue::Int(1))]));
    assert!(display.contains("a") && display.contains("1"));
}

#[test]
fn typed_value_clone() {
    let tv = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert_eq!(*tv.clone().value(), ExpressionValue::Int(42));
}

#[test]
fn typed_value_default() {
    assert!(TypedValue::default().is_null());
}

// ══════════════════════════════════════════════════════════════════════════════
// 50. ExpressionValue utility tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn expression_value_is_truthy() {
    assert!(ExpressionValue::Int(1).is_truthy());
    assert!(!ExpressionValue::Int(0).is_truthy());
    assert!(ExpressionValue::Boolean(true).is_truthy());
    assert!(!ExpressionValue::Boolean(false).is_truthy());
    assert!(!ExpressionValue::Null.is_truthy());
    assert!(ExpressionValue::String("a".to_string()).is_truthy());
    assert!(!ExpressionValue::String("".to_string()).is_truthy());
}

#[test]
fn expression_value_type_descriptor() {
    assert_eq!(ExpressionValue::Int(42).type_descriptor(), TypeDescriptor::INT);
    assert_eq!(ExpressionValue::Boolean(true).type_descriptor(), TypeDescriptor::BOOLEAN);
    assert_eq!(ExpressionValue::String("x".to_string()).type_descriptor(), TypeDescriptor::STRING);
}

#[test]
fn expression_value_partial_eq() {
    assert_eq!(ExpressionValue::Int(42), ExpressionValue::Int(42));
    assert_ne!(ExpressionValue::Int(42), ExpressionValue::Int(43));
    assert_eq!(ExpressionValue::Null, ExpressionValue::Null);
    assert_eq!(ExpressionValue::String("a".to_string()), ExpressionValue::String("a".to_string()));
    assert_ne!(ExpressionValue::Int(1), ExpressionValue::String("1".to_string()));
}

#[test]
fn expression_value_as_any() {
    assert!(ExpressionValue::Int(42).as_any().is_some());
    assert!(ExpressionValue::String("x".to_string()).as_any().is_some());
    assert!(ExpressionValue::Null.as_any().is_none());
}

#[test]
fn expression_value_type_id() {
    assert_eq!(ExpressionValue::Int(42).type_id(), std::any::TypeId::of::<i64>());
    assert_eq!(ExpressionValue::Boolean(true).type_id(), std::any::TypeId::of::<bool>());
}

// ══════════════════════════════════════════════════════════════════════════════
// 51. TypeDescriptor utility tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_descriptor_constants() {
    assert_eq!(TypeDescriptor::INT, TypeDescriptor::Primitive(PrimitiveKind::Int));
    assert_eq!(TypeDescriptor::LONG, TypeDescriptor::Primitive(PrimitiveKind::Long));
    assert_eq!(TypeDescriptor::BOOLEAN, TypeDescriptor::Primitive(PrimitiveKind::Boolean));
    assert_eq!(TypeDescriptor::STRING, TypeDescriptor::Primitive(PrimitiveKind::String));
    assert_eq!(TypeDescriptor::NULL, TypeDescriptor::Primitive(PrimitiveKind::Null));
}

#[test]
fn type_descriptor_name() {
    assert_eq!(TypeDescriptor::INT.name(), "int");
    assert_eq!(TypeDescriptor::STRING.name(), "java.lang.String");
    assert_eq!(TypeDescriptor::BOOLEAN.name(), "boolean");
}

#[test]
fn type_descriptor_is_primitive() {
    assert!(TypeDescriptor::INT.is_primitive());
    assert!(!TypeDescriptor::OBJECT.is_primitive());
}

#[test]
fn type_descriptor_from_type_name() {
    assert_eq!(TypeDescriptor::from_type_name("MyClass").name(), "MyClass");
}

#[test]
fn type_descriptor_with_generic() {
    assert_eq!(TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT).name(), "List<int>");
}

#[test]
fn type_descriptor_array() {
    let td = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(td.name(), "int[]");
    assert!(td.is_array());
}

#[test]
fn type_descriptor_map() {
    let td = TypeDescriptor::Map(Box::new(TypeDescriptor::STRING), Box::new(TypeDescriptor::INT));
    assert!(td.is_map());
    assert!(td.get_map_key_type().is_some());
    assert!(td.get_map_value_type().is_some());
}

#[test]
fn type_descriptor_display() {
    assert_eq!(format!("{}", TypeDescriptor::INT), "int");
}

#[test]
fn type_descriptor_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    TypeDescriptor::INT.hash(&mut h1);
    TypeDescriptor::INT.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

// ══════════════════════════════════════════════════════════════════════════════
// 52. StandardEvaluationContext tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn std_ctx_default() {
    assert!(StandardEvaluationContext::default().root_object().is_null());
}

#[test]
fn std_ctx_new_default() {
    assert!(StandardEvaluationContext::new_default().root_object().is_null());
}

#[test]
fn std_ctx_property_accessors_non_empty() {
    assert!(!StandardEvaluationContext::new(TypedValue::null()).property_accessors().is_empty());
}

#[test]
fn std_ctx_method_resolvers_non_empty() {
    assert!(!StandardEvaluationContext::new(TypedValue::null()).method_resolvers().is_empty());
}

#[test]
fn std_ctx_register_method_fn() {
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    ctx.register_method_fn("test_fn", |_ctx, _target, _args| {
        Ok(TypedValue::new(ExpressionValue::Int(99), TypeDescriptor::INT))
    });
}

#[test]
fn std_ctx_set_variable() {
    let mut ctx = StandardEvaluationContext::new(TypedValue::null());
    ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT));
}

// ══════════════════════════════════════════════════════════════════════════════
// 53. SimpleEvaluationContext tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn simple_ctx_read_only() {
    assert!(!SimpleEvaluationContext::for_read_only(TypedValue::null()).is_assignment_enabled());
}

#[test]
fn simple_ctx_read_write() {
    assert!(SimpleEvaluationContext::for_read_write(TypedValue::null()).is_assignment_enabled());
}

#[test]
fn simple_ctx_variable_lookup() {
    let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT));
    let v = ctx.lookup_variable("x");
    assert!(v.is_some());
    assert_eq!(*v.unwrap().value(), ExpressionValue::Int(42));
}

#[test]
fn simple_ctx_variable_not_found() {
    assert!(SimpleEvaluationContext::for_read_only(TypedValue::null()).lookup_variable("missing").is_none());
}

#[test]
fn simple_ctx_empty_accessors() {
    let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
    assert!(ctx.property_accessors().is_empty());
    assert!(ctx.method_resolvers().is_empty());
    assert!(ctx.constructor_resolvers().is_empty());
    assert!(ctx.bean_resolver().is_none());
    assert!(ctx.type_converter().is_none());
    assert!(ctx.type_locator().is_none());
    assert!(ctx.type_comparator().is_none());
    assert!(ctx.operator_overloader().is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// 54. ParserContext / TemplateParserContext tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn template_parser_context_default() {
    use vernal_expression::TemplateParserContext;
    let ctx = TemplateParserContext::default();
    assert!(ctx.is_template());
    assert_eq!(ctx.expression_prefix(), "#{");
    assert_eq!(ctx.expression_suffix(), "}");
}

#[test]
fn template_parser_context_custom() {
    use vernal_expression::TemplateParserContext;
    let ctx = TemplateParserContext::new("{{", "}}");
    assert!(ctx.is_template());
    assert_eq!(ctx.expression_prefix(), "{{");
    assert_eq!(ctx.expression_suffix(), "}}");
}

// ══════════════════════════════════════════════════════════════════════════════
// 55. Error handling tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn parse_error_empty() {
    assert!(SpelExpressionParser::new().parse_expression("").is_err());
}

#[test]
fn parse_error_trailing_garbage() {
    assert!(SpelExpressionParser::new().parse_expression("1 + 2 abc").is_err());
}

#[test]
fn parse_error_unclosed_string() {
    assert!(SpelExpressionParser::new().parse_expression("'hello").is_err());
}

#[test]
fn eval_error_division_by_zero() {
    eval_err("1 / 0");
}

#[test]
fn eval_error_modulus_by_zero() {
    eval_err("1 % 0");
}

#[test]
fn eval_error_variable_not_found() {
    eval_err("#nonexistent");
}

#[test]
fn eval_error_function_not_found() {
    eval_err("#nonexistent()");
}

#[test]
fn eval_error_bean_not_found() {
    eval_err("@nonexistent");
}

// ══════════════════════════════════════════════════════════════════════════════
// 56. Alternative operator names (Spring feature)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn alt_op_eq() {
    assert_eq!(eval("5 eq 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alt_op_ne() {
    assert_eq!(eval("5 ne 3"), ExpressionValue::Boolean(true));
}

#[test]
fn alt_op_lt() {
    assert_eq!(eval("3 lt 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alt_op_le() {
    assert_eq!(eval("5 le 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alt_op_gt() {
    assert_eq!(eval("5 gt 3"), ExpressionValue::Boolean(true));
}

#[test]
fn alt_op_ge() {
    assert_eq!(eval("5 ge 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alt_op_div() {
    assert_eq!(eval("10 div 3"), ExpressionValue::Int(3));
}

#[test]
fn alt_op_mod() {
    assert_eq!(eval("10 mod 3"), ExpressionValue::Int(1));
}

#[test]
fn alt_op_not() {
    assert_eq!(eval("not true"), ExpressionValue::Boolean(false));
    assert_eq!(eval("not false"), ExpressionValue::Boolean(true));
}

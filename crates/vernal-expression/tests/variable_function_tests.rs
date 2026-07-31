//! 变量引用与函数引用测试。

use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::spel::support::simple_evaluation_context::SimpleEvaluationContext;
use vernal_expression::*;
fn ae(e: &str, ctx: &dyn EvaluationContext) { assert!(SpelExpressionParser::new().parse_expression(e).unwrap().get_value_with_context(ctx).is_err()); }
fn aec(e: &str, ctx: &dyn EvaluationContext, s: &str) { let err = SpelExpressionParser::new().parse_expression(e).unwrap().get_value_with_context(ctx).unwrap_err().to_string(); assert!(err.contains(s), "'{}' not in '{}'", s, err); }
#[test] fn var_parse() { assert!(SpelExpressionParser::new().parse_expression("#myVar").is_ok()); }
#[test] fn var_this_parse() { assert!(SpelExpressionParser::new().parse_expression("#this").is_ok()); }
#[test] fn var_root_parse() { assert!(SpelExpressionParser::new().parse_expression("#root").is_ok()); }
#[test] fn var_underscore() { assert!(SpelExpressionParser::new().parse_expression("#_private_var").is_ok()); }
#[test] fn var_long() { assert!(SpelExpressionParser::new().parse_expression("#thisIsAVeryLongVariableName").is_ok()); }
#[test] fn var_this_root() { let root = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT); let e = SpelExpressionParser::new().parse_expression("#this").unwrap(); assert_eq!(e.get_value_with_context(&StandardEvaluationContext::new(root.clone())).unwrap().value(), root.value()); }
#[test] fn var_root_root() { let root = TypedValue::new(ExpressionValue::String("hello".to_string()), TypeDescriptor::STRING); let e = SpelExpressionParser::new().parse_expression("#root").unwrap(); assert_eq!(e.get_value_with_context(&StandardEvaluationContext::new(root.clone())).unwrap().value(), root.value()); }
#[test] fn var_this_null() { let e = SpelExpressionParser::new().parse_expression("#this").unwrap(); assert_eq!(e.get_value_with_context(&StandardEvaluationContext::new(TypedValue::null())).unwrap().value(), &ExpressionValue::Null); }
#[test] fn var_root_list() { let items = vec![TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT), TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT)]; let root = TypedValue::new(ExpressionValue::List(items), TypeDescriptor::OBJECT); let e = SpelExpressionParser::new().parse_expression("#root").unwrap(); assert!(matches!(e.get_value_with_context(&StandardEvaluationContext::new(root)).unwrap().value(), ExpressionValue::List(_))); }
#[test] fn var_not_found() { ae("#myVar", &StandardEvaluationContext::new(TypedValue::null())); }
#[test] fn var_not_found_msg() { aec("#myVar", &StandardEvaluationContext::new(TypedValue::null()), "myVar"); }
#[test] fn var_not_found_hash() { aec("#unknown", &StandardEvaluationContext::new(TypedValue::null()), "#unknown"); }
#[test] fn var_set_str() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("greeting", TypedValue::new(ExpressionValue::String("hello".to_string()), TypeDescriptor::STRING)); let e = SpelExpressionParser::new().parse_expression("#greeting").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::String("hello".to_string())); }
#[test] fn var_set_int() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("count", TypedValue::new(ExpressionValue::Int(10), TypeDescriptor::INT)); let e = SpelExpressionParser::new().parse_expression("#count").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Int(10)); }
#[test] fn var_set_bool() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("flag", TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN)); let e = SpelExpressionParser::new().parse_expression("#flag").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Boolean(true)); }
#[test] fn var_set_null() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("empty", TypedValue::null()); let e = SpelExpressionParser::new().parse_expression("#empty").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Null); }
#[test] fn var_not_found_simple() { let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ae("#missing", &ctx); }
#[test] fn var_overwrite() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT)); ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT)); let e = SpelExpressionParser::new().parse_expression("#x").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Int(2)); }
#[test] fn var_arithmetic() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("a", TypedValue::new(ExpressionValue::Int(3), TypeDescriptor::INT)); ctx.set_variable("b", TypedValue::new(ExpressionValue::Int(7), TypeDescriptor::INT)); let e = SpelExpressionParser::new().parse_expression("#a + #b").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Int(10)); }
#[test] fn var_comparison() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("x", TypedValue::new(ExpressionValue::Int(5), TypeDescriptor::INT)); ctx.set_variable("y", TypedValue::new(ExpressionValue::Int(5), TypeDescriptor::INT)); let e = SpelExpressionParser::new().parse_expression("#x == #y").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Boolean(true)); }
#[test] fn var_ternary() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("cond", TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN)); ctx.set_variable("a", TypedValue::new(ExpressionValue::Int(10), TypeDescriptor::INT)); ctx.set_variable("b", TypedValue::new(ExpressionValue::Int(20), TypeDescriptor::INT)); let e = SpelExpressionParser::new().parse_expression("#cond ? #a : #b").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::Int(10)); }
#[test] fn var_concat() { let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null()); ctx.set_variable("name", TypedValue::new(ExpressionValue::String("world".to_string()), TypeDescriptor::STRING)); let e = SpelExpressionParser::new().parse_expression("'hello ' + #name").unwrap(); assert_eq!(e.get_value_with_context(&ctx).unwrap().value(), &ExpressionValue::String("hello world".to_string())); }
#[test] fn fn_no_args() { assert!(SpelExpressionParser::new().parse_expression("#fn()").is_ok()); }
#[test] fn fn_args() { assert!(SpelExpressionParser::new().parse_expression("#add(1, 2)").is_ok()); }
#[test] fn fn_string() { assert!(SpelExpressionParser::new().parse_expression("#greet('hello')").is_ok()); }
#[test] fn fn_mixed() { assert!(SpelExpressionParser::new().parse_expression("#process(1, 'two', true)").is_ok()); }
#[test] fn fn_not_found() { ae("#myFunc(1, 2)", &StandardEvaluationContext::new(TypedValue::null())); }
#[test] fn fn_not_found_msg() { aec("#myFunc()", &StandardEvaluationContext::new(TypedValue::null()), "myFunc"); }
#[test] fn bean_parse() { assert!(SpelExpressionParser::new().parse_expression("@myBean").is_ok()); }
#[test] fn bean_dotted() { assert!(SpelExpressionParser::new().parse_expression("@com.example.MyBean").is_ok()); }
#[test] fn bean_underscore() { assert!(SpelExpressionParser::new().parse_expression("@_internal_bean").is_ok()); }
#[test] fn bean_no_resolver() { ae("@myBean", &StandardEvaluationContext::new(TypedValue::null())); }
#[test] fn bean_err_msg() { aec("@myBean", &StandardEvaluationContext::new(TypedValue::null()), "BeanResolver"); }
#[test] fn factory_bean() { assert!(SpelExpressionParser::new().parse_expression("&factoryBean").is_ok()); }
#[test] fn factory_bean_err() { ae("&factoryBean", &StandardEvaluationContext::new(TypedValue::null())); }
#[test] fn var_expr_str() { let e = SpelExpressionParser::new().parse_expression("#myVar").unwrap(); assert_eq!(e.expression_string(), "#myVar"); }
#[test] fn bean_expr_str() { let e = SpelExpressionParser::new().parse_expression("@myBean").unwrap(); assert_eq!(e.expression_string(), "@myBean"); }
#[test] fn fn_expr_str() { let e = SpelExpressionParser::new().parse_expression("#fn(1, 2)").unwrap(); let es = e.expression_string(); assert!(es.contains("#fn") && es.contains("1") && es.contains("2")); }

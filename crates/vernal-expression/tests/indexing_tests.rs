use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;
fn ml(i: i64) -> TypedValue { TypedValue::new(ExpressionValue::Int(i), TypeDescriptor::INT) }
fn ml_list(items: &[i64]) -> TypedValue { TypedValue::new(ExpressionValue::List(items.iter().map(|i| ml(*i)).collect()), TypeDescriptor::OBJECT) }
fn er(e: &str, r: TypedValue) { assert!(SpelExpressionParser::new().parse_expression(e).unwrap().get_value_with_context(&StandardEvaluationContext::new(r)).is_err()); }
fn ev(e: &str, r: TypedValue) -> ExpressionValue { SpelExpressionParser::new().parse_expression(e).unwrap().get_value_with_context(&StandardEvaluationContext::new(r)).unwrap().value().clone() }
#[test] fn list_first() { assert_eq!(ev("{10,20,30}[0]", ml_list(&[10,20,30])), ExpressionValue::Int(10)); }
#[test] fn list_middle() { assert_eq!(ev("{10,20,30}[1]", ml_list(&[10,20,30])), ExpressionValue::Int(20)); }
#[test] fn list_last() { assert_eq!(ev("{10,20,30}[2]", ml_list(&[10,20,30])), ExpressionValue::Int(30)); }
#[test] fn list_single() { assert_eq!(ev("{42}[0]", ml_list(&[42])), ExpressionValue::Int(42)); }
#[test] fn list_ten() { let items: Vec<i64> = (1..=10).map(|i| i*10).collect(); assert_eq!(ev("{10,20,30,40,50,60,70,80,90,100}[9]", ml_list(&items)), ExpressionValue::Int(100)); }
#[test] fn oob() { er("{1,2}[5]", ml_list(&[1,2])); }
#[test] fn negative() { er("{1,2}[-1]", ml_list(&[1,2])); }
#[test] fn single_oob() { er("{42}[1]", ml_list(&[42])); }
#[test] fn add_expr() { assert_eq!(ev("{10,20,30}[0+1]", ml_list(&[10,20,30])), ExpressionValue::Int(20)); }
#[test] fn mul_expr() { assert_eq!(ev("{10,20,30}[1*2]", ml_list(&[10,20,30])), ExpressionValue::Int(30)); }
#[test] fn ternary() { assert_eq!(ev("{10,20,30}[true?1:0]", ml_list(&[10,20,30])), ExpressionValue::Int(20)); }
#[test] fn sub_expr() { assert_eq!(ev("{10,20,30,40,50}[4-2]", ml_list(&[10,20,30,40,50])), ExpressionValue::Int(30)); }
#[test] fn complex() { assert_eq!(ev("{100,200,300}[(1+1)*1]", ml_list(&[100,200,300])), ExpressionValue::Int(300)); }
#[test] fn idx_eq() { assert_eq!(ev("{10,20,30}[0] == 10", ml_list(&[10,20,30])), ExpressionValue::Boolean(true)); }
#[test] fn idx_ne() { assert_eq!(ev("{10,20,30}[0] != 20", ml_list(&[10,20,30])), ExpressionValue::Boolean(true)); }
#[test] fn idx_lt() { assert_eq!(ev("{10,20,30}[0] < {10,20,30}[2]", ml_list(&[10,20,30])), ExpressionValue::Boolean(true)); }
#[test] fn idx_mul() { assert_eq!(ev("{2,3,4}[0] * {2,3,4}[2]", ml_list(&[2,3,4])), ExpressionValue::Int(8)); }
#[test] fn idx_sub() { assert_eq!(ev("{10,20,30}[2] - {10,20,30}[0]", ml_list(&[10,20,30])), ExpressionValue::Int(20)); }
#[test] fn str_idx_err() { er("'hello'[0]", TypedValue::null()); }
#[test] fn int_idx_err() { er("5[0]", TypedValue::null()); }
#[test] fn bool_idx_err() { er("true[0]", TypedValue::null()); }
#[test] fn parse_ok() { let p = SpelExpressionParser::new(); assert!(p.parse_expression("{1,2,3}[0]").is_ok()); assert!(p.parse_expression("{{1,2},{3,4}}[0][1]").is_ok()); assert!(p.parse_expression("{1,2,3}[0+1]").is_ok()); }
#[test] fn compound_parse() { let p = SpelExpressionParser::new(); assert!(p.parse_expression("{1,2,3}[0] + {4,5,6}[1]").is_ok()); assert!(p.parse_expression("{1,2,3}[0] == 1").is_ok()); }
#[test] fn literal_parse() { let p = SpelExpressionParser::new(); assert!(p.parse_expression("5[0]").is_ok()); assert!(p.parse_expression("'hello'[0]").is_ok()); assert!(p.parse_expression("true[0]").is_ok()); }
#[test] fn empty_parse() { assert!(SpelExpressionParser::new().parse_expression("{}").is_ok()); }
#[test] fn nested_parse() { let p = SpelExpressionParser::new(); assert!(p.parse_expression("{{1,2},{3,4}}").is_ok()); assert!(p.parse_expression("{{{1}}}").is_ok()); }
#[test] fn null_parse() { assert!(SpelExpressionParser::new().parse_expression("null[0]").is_ok()); }

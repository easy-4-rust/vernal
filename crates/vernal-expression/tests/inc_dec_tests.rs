//! 自增/自减运算符测试（`++` / `--`）。

use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;
fn ev(e: &str) -> ExpressionValue {
    SpelExpressionParser::new()
        .parse_expression(e)
        .unwrap()
        .get_value_with_context(&StandardEvaluationContext::new(TypedValue::null()))
        .unwrap()
        .value()
        .clone()
}
fn ae(e: &str) {
    assert!(
        SpelExpressionParser::new()
            .parse_expression(e)
            .unwrap()
            .get_value_with_context(&StandardEvaluationContext::new(TypedValue::null()))
            .is_err()
    );
}
#[test]
fn pfx_inc_int() {
    assert_eq!(ev("++5"), ExpressionValue::Int(6));
}
#[test]
fn pfx_inc_zero() {
    assert_eq!(ev("++0"), ExpressionValue::Int(1));
}
#[test]
fn pfx_inc_neg() {
    assert_eq!(ev("++(-5)"), ExpressionValue::Int(-4));
}
#[test]
fn pfx_inc_float() {
    assert_eq!(ev("++3.5"), ExpressionValue::Float(4.5));
}
#[test]
fn pfx_inc_large() {
    assert_eq!(ev("++999"), ExpressionValue::Int(1000));
}
#[test]
fn pfx_dec_int() {
    assert_eq!(ev("--5"), ExpressionValue::Int(4));
}
#[test]
fn pfx_dec_zero() {
    assert_eq!(ev("--0"), ExpressionValue::Int(-1));
}
#[test]
fn pfx_dec_neg() {
    assert_eq!(ev("--(-3)"), ExpressionValue::Int(-4));
}
#[test]
fn pfx_dec_float() {
    assert_eq!(ev("--3.5"), ExpressionValue::Float(2.5));
}
#[test]
fn pfx_dec_large() {
    assert_eq!(ev("--1000"), ExpressionValue::Int(999));
}
#[test]
fn sfx_inc_int() {
    assert_eq!(ev("5++"), ExpressionValue::Int(5));
}
#[test]
fn sfx_inc_zero() {
    assert_eq!(ev("0++"), ExpressionValue::Int(0));
}
#[test]
fn sfx_inc_neg() {
    assert_eq!(ev("(-5)++"), ExpressionValue::Int(-5));
}
#[test]
fn sfx_inc_float() {
    assert_eq!(ev("3.5++"), ExpressionValue::Float(3.5));
}
#[test]
fn sfx_inc_large() {
    assert_eq!(ev("999++"), ExpressionValue::Int(999));
}
#[test]
fn sfx_dec_int() {
    assert_eq!(ev("5--"), ExpressionValue::Int(5));
}
#[test]
fn sfx_dec_zero() {
    assert_eq!(ev("0--"), ExpressionValue::Int(0));
}
#[test]
fn sfx_dec_neg() {
    assert_eq!(ev("(-3)--"), ExpressionValue::Int(-3));
}
#[test]
fn sfx_dec_float() {
    assert_eq!(ev("3.5--"), ExpressionValue::Float(3.5));
}
#[test]
fn sfx_dec_large() {
    assert_eq!(ev("1000--"), ExpressionValue::Int(1000));
}
#[test]
fn pfx_inc_str() {
    ae("++'hello'");
}
#[test]
fn pfx_inc_bool() {
    ae("++true");
}
#[test]
fn pfx_inc_null() {
    ae("++null");
}
#[test]
fn pfx_dec_str() {
    ae("--'hello'");
}
#[test]
fn pfx_dec_bool() {
    ae("--true");
}
#[test]
fn sfx_inc_str() {
    ae("'hello'++");
}
#[test]
fn sfx_dec_bool() {
    ae("true--");
}
#[test]
fn dbl_pfx_inc() {
    assert_eq!(ev("++ ++5"), ExpressionValue::Int(7));
}
#[test]
fn dbl_pfx_dec() {
    assert_eq!(ev("-- --5"), ExpressionValue::Int(3));
}
#[test]
fn pfx_then_sfx() {
    assert_eq!(ev("(++5)++"), ExpressionValue::Int(6));
}
#[test]
fn sfx_then_pfx() {
    assert_eq!(ev("++(5++)"), ExpressionValue::Int(6));
}
#[test]
fn pfx_add() {
    assert_eq!(ev("++5 + 3"), ExpressionValue::Int(9));
}
#[test]
fn pfx_sub() {
    assert_eq!(ev("--10 - 2"), ExpressionValue::Int(7));
}
#[test]
fn sfx_mul() {
    assert_eq!(ev("5++ * 2"), ExpressionValue::Int(10));
}
#[test]
fn sfx_div() {
    assert_eq!(ev("10-- / 2"), ExpressionValue::Int(5));
}
#[test]
fn pfx_float_add() {
    match ev("++2.5 + 1.5") {
        ExpressionValue::Float(f) => assert!((f - 5.0).abs() < f64::EPSILON),
        other => panic!("expected Float, got {:?}", other),
    }
}
#[test]
fn parse_forms() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("++a").is_ok());
    assert!(p.parse_expression("a++").is_ok());
    assert!(p.parse_expression("--a").is_ok());
    assert!(p.parse_expression("a--").is_ok());
}
#[test]
fn parse_exprs() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("++(1+2)").is_ok());
    assert!(p.parse_expression("(1+2)++").is_ok());
    assert!(p.parse_expression("--(3*4)").is_ok());
    assert!(p.parse_expression("(3*4)--").is_ok());
}
#[test]
fn parse_ternary() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("true ? ++1 : --1")
            .is_ok()
    );
}
#[test]
fn parse_compare() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("++1 == 2").is_ok());
    assert!(p.parse_expression("5-- > 4").is_ok());
}
#[test]
fn pfx_inc_str_out() {
    let e = SpelExpressionParser::new().parse_expression("++5").unwrap();
    assert!(e.expression_string().contains("++"));
}
#[test]
fn pfx_dec_str_out() {
    let e = SpelExpressionParser::new().parse_expression("--5").unwrap();
    assert!(e.expression_string().contains("--"));
}
#[test]
fn sfx_inc_str_out() {
    let e = SpelExpressionParser::new().parse_expression("5++").unwrap();
    assert!(e.expression_string().contains("++"));
}
#[test]
fn sfx_dec_str_out() {
    let e = SpelExpressionParser::new().parse_expression("5--").unwrap();
    assert!(e.expression_string().contains("--"));
}

//! 构造器引用表达式测试（`new Foo(...)`）。
//!
//! 覆盖构造器表达式的解析与求值路径。

use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;
fn ae(e: &str) {
    assert!(
        SpelExpressionParser::new()
            .parse_expression(e)
            .unwrap()
            .get_value_with_context(&StandardEvaluationContext::new(TypedValue::null()))
            .is_err()
    );
}
fn aec(e: &str, s: &str) {
    let err = SpelExpressionParser::new()
        .parse_expression(e)
        .unwrap()
        .get_value_with_context(&StandardEvaluationContext::new(TypedValue::null()))
        .unwrap_err()
        .to_string();
    assert!(err.contains(s), "'{}' not in '{}'", s, err);
}
#[test]
fn no_args() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new ArrayList()")
            .is_ok()
    );
}
#[test]
fn one_int() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo(1)")
            .is_ok()
    );
}
#[test]
fn two_args() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo(1, 2)")
            .is_ok()
    );
}
#[test]
fn string_arg() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo('hello')")
            .is_ok()
    );
}
#[test]
fn mixed() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo(1, 'two', true)")
            .is_ok()
    );
}
#[test]
fn expr_args() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo(1 + 2, 3 * 4)")
            .is_ok()
    );
}
#[test]
fn nested() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Outer(new Inner())")
            .is_ok()
    );
}
#[test]
fn simple_class() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new MyClass()")
            .is_ok()
    );
}
#[test]
fn no_resolver() {
    ae("new ArrayList()");
}
#[test]
fn with_args_err() {
    ae("new Foo(1, 2)");
}
#[test]
fn err_type() {
    aec("new ArrayList()", "ArrayList");
}
#[test]
fn err_msg() {
    aec("new Foo()", "构造器");
}
#[test]
fn expr_str_no_args() {
    let e = SpelExpressionParser::new()
        .parse_expression("new Foo()")
        .unwrap();
    assert!(e.expression_string().contains("new") && e.expression_string().contains("Foo"));
}
#[test]
fn expr_str_args() {
    let e = SpelExpressionParser::new()
        .parse_expression("new Foo(1, 2)")
        .unwrap();
    let es = e.expression_string();
    assert!(es.contains("new") && es.contains("Foo") && es.contains("1") && es.contains("2"));
}
#[test]
fn expr_str_str() {
    let e = SpelExpressionParser::new()
        .parse_expression("new Foo('hello')")
        .unwrap();
    let es = e.expression_string();
    assert!(es.contains("Foo") && es.contains("hello"));
}
#[test]
fn in_ternary() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("true ? new Foo() : new Bar()")
            .is_ok()
    );
}
#[test]
fn in_elvis() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("null ?: new Foo()")
            .is_ok()
    );
}
#[test]
fn in_compare() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo() == new Foo()")
            .is_ok()
    );
}
#[test]
fn in_logical() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Foo() != null && true")
            .is_ok()
    );
}
#[test]
fn in_assign() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("x = new Foo(1)")
            .is_ok()
    );
}
#[test]
fn string_type() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new String()")
            .is_ok()
    );
}
#[test]
fn integer_type() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new Integer(42)")
            .is_ok()
    );
}
#[test]
fn list_type() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new ArrayList(10)")
            .is_ok()
    );
}
#[test]
fn hashmap_type() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("new HashMap()")
            .is_ok()
    );
}

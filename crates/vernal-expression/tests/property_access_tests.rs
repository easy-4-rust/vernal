//! 属性访问表达式测试。

use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::map_accessor::MapAccessor;
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
fn mm(p: Vec<(&str, ExpressionValue)>) -> TypedValue {
    let m: Vec<(TypedValue, TypedValue)> = p
        .into_iter()
        .map(|(k, v)| {
            (
                TypedValue::new(
                    ExpressionValue::String(k.to_string()),
                    TypeDescriptor::STRING,
                ),
                TypedValue::new(v, TypeDescriptor::OBJECT),
            )
        })
        .collect();
    TypedValue::new(ExpressionValue::Map(m), TypeDescriptor::OBJECT)
}
#[test]
fn parse_simple() {
    assert!(SpelExpressionParser::new().parse_expression("name").is_ok());
}
#[test]
fn parse_dotted() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("person.name")
            .is_ok()
    );
}
#[test]
fn parse_deep() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("a.b.c.d.e")
            .is_ok()
    );
}
#[test]
fn parse_very_deep() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("a.b.c.d.e.f.g.h")
            .is_ok()
    );
}
#[test]
fn parse_method_chain() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("obj.method().property")
            .is_ok()
    );
}
#[test]
fn parse_index_chain() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("list[0].name")
            .is_ok()
    );
}
#[test]
fn not_found() {
    ae("name");
}
#[test]
fn not_found_msg() {
    aec("name", "name");
}
#[test]
fn dotted_not_found() {
    ae("person.name");
}
#[test]
fn dotted_msg() {
    aec("person.name", "person");
}
#[test]
fn deep_not_found() {
    ae("a.b.c.d");
}
#[test]
fn on_int() {
    ae("5.name");
}
#[test]
fn on_string() {
    ae("'hello'.name");
}
#[test]
fn safe_single() {
    assert!(SpelExpressionParser::new().parse_expression("a?.b").is_ok());
}
#[test]
fn safe_chain() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("a?.b?.c")
            .is_ok()
    );
}
#[test]
fn safe_index() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("list?[0]")
            .is_ok()
    );
}
#[test]
fn safe_method() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("obj?.method()")
            .is_ok()
    );
}
#[test]
fn safe_deep() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("a?.b?.c?.d?.e")
            .is_ok()
    );
}
#[test]
fn safe_eval_err() {
    ae("a?.b");
}
#[test]
fn safe_chain_eval_err() {
    ae("a?.b?.c");
}
#[test]
fn safe_mixed() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("a?.b.c").is_ok());
    assert!(p.parse_expression("a.b?.c").is_ok());
}
#[test]
fn expr_string() {
    let e = SpelExpressionParser::new()
        .parse_expression("name")
        .unwrap();
    assert_eq!(e.expression_string(), "name");
}
#[test]
fn chain_expr_string() {
    let e = SpelExpressionParser::new()
        .parse_expression("person.name")
        .unwrap();
    let es = e.expression_string();
    assert!(es.contains("person") && es.contains("name"));
}
#[test]
fn safe_expr_string() {
    let e = SpelExpressionParser::new()
        .parse_expression("a?.b")
        .unwrap();
    assert!(e.expression_string().contains("?."));
}
#[test]
fn ternary() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("true ? a : b")
            .is_ok()
    );
}
#[test]
fn elvis() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("a ?: b")
            .is_ok()
    );
}
#[test]
fn compare() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("a == b").is_ok());
    assert!(p.parse_expression("a != b").is_ok());
    assert!(p.parse_expression("a < b").is_ok());
    assert!(p.parse_expression("a > b").is_ok());
}
#[test]
fn logical() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("a && b").is_ok());
    assert!(p.parse_expression("a || b").is_ok());
}
#[test]
fn arithmetic() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("a + b").is_ok());
    assert!(p.parse_expression("a - b").is_ok());
    assert!(p.parse_expression("a * b").is_ok());
}
#[test]
fn assign() {
    let p = SpelExpressionParser::new();
    assert!(p.parse_expression("x = 5").is_ok());
    assert!(p.parse_expression("a.b = 5").is_ok());
}
#[test]
fn map_found() {
    let root = mm(vec![("name", ExpressionValue::String("alice".to_string()))]);
    let e = SpelExpressionParser::new()
        .parse_expression("name")
        .unwrap();
    let ctx = StandardEvaluationContext::new(root);
    ctx.set_property_accessors(vec![Box::new(MapAccessor)]);
    assert_eq!(
        e.get_value_with_context(&ctx).unwrap().value(),
        &ExpressionValue::String("alice".to_string())
    );
}
#[test]
fn map_int() {
    let root = mm(vec![("count", ExpressionValue::Int(42))]);
    let e = SpelExpressionParser::new()
        .parse_expression("count")
        .unwrap();
    let ctx = StandardEvaluationContext::new(root);
    ctx.set_property_accessors(vec![Box::new(MapAccessor)]);
    assert_eq!(
        e.get_value_with_context(&ctx).unwrap().value(),
        &ExpressionValue::Int(42)
    );
}
#[test]
fn map_bool() {
    let root = mm(vec![("active", ExpressionValue::Boolean(true))]);
    let e = SpelExpressionParser::new()
        .parse_expression("active")
        .unwrap();
    let ctx = StandardEvaluationContext::new(root);
    ctx.set_property_accessors(vec![Box::new(MapAccessor)]);
    assert_eq!(
        e.get_value_with_context(&ctx).unwrap().value(),
        &ExpressionValue::Boolean(true)
    );
}
#[test]
fn map_missing() {
    let root = mm(vec![("exists", ExpressionValue::Int(1))]);
    let e = SpelExpressionParser::new()
        .parse_expression("missing")
        .unwrap();
    let ctx = StandardEvaluationContext::new(root);
    ctx.set_property_accessors(vec![Box::new(MapAccessor)]);
    assert_eq!(
        e.get_value_with_context(&ctx).unwrap().value(),
        &ExpressionValue::Null
    );
}
#[test]
fn map_multi() {
    let root = mm(vec![
        ("a", ExpressionValue::Int(1)),
        ("b", ExpressionValue::Int(2)),
        ("c", ExpressionValue::Int(3)),
    ]);
    let ctx = StandardEvaluationContext::new(root);
    ctx.set_property_accessors(vec![Box::new(MapAccessor)]);
    let p = SpelExpressionParser::new();
    assert_eq!(
        p.parse_expression("a")
            .unwrap()
            .get_value_with_context(&ctx)
            .unwrap()
            .value(),
        &ExpressionValue::Int(1)
    );
    assert_eq!(
        p.parse_expression("b")
            .unwrap()
            .get_value_with_context(&ctx)
            .unwrap()
            .value(),
        &ExpressionValue::Int(2)
    );
    assert_eq!(
        p.parse_expression("c")
            .unwrap()
            .get_value_with_context(&ctx)
            .unwrap()
            .value(),
        &ExpressionValue::Int(3)
    );
}
#[test]
fn map_not_map() {
    let e = SpelExpressionParser::new()
        .parse_expression("name")
        .unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::new(
        ExpressionValue::String("x".to_string()),
        TypeDescriptor::STRING,
    ));
    ctx.set_property_accessors(vec![Box::new(MapAccessor)]);
    assert!(e.get_value_with_context(&ctx).is_err());
}
#[test]
fn method_parse() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("obj.method()")
            .is_ok()
    );
}
#[test]
fn method_args_parse() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("obj.add(1, 2)")
            .is_ok()
    );
}
#[test]
fn method_eval_err() {
    assert!(
        SpelExpressionParser::new()
            .parse_expression("obj.method()")
            .unwrap()
            .get_value_with_context(&StandardEvaluationContext::new(TypedValue::null()))
            .is_err()
    );
}
#[test]
fn method_eval_err_msg() {
    let e = SpelExpressionParser::new()
        .parse_expression("obj.doSomething(1)")
        .unwrap();
    let err = e
        .get_value_with_context(&StandardEvaluationContext::new(TypedValue::null()))
        .unwrap_err()
        .to_string();
    assert!(err.contains("obj"), "error: {}", err);
}

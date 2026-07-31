//! internal_spel_expression_parser.rs 内部逻辑测试。
//!
//! 目标：覆盖递归下降解析器的所有主要代码路径。
//! 当前覆盖率：0%（1015 行未覆盖）。

use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;

fn eval(expr: &str) -> ExpressionValue {
    let parser = SpelExpressionParser::new();
    let parsed = parser.parse_expression(expr).unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = parsed.get_value_with_context(&ctx).unwrap();
    result.value().clone()
}

#[allow(dead_code)] // 测试辅助函数：供后续带根对象求值用例复用。
fn eval_with_root(expr: &str, root: TypedValue) -> ExpressionValue {
    let parser = SpelExpressionParser::new();
    let parsed = parser.parse_expression(expr).unwrap();
    let ctx = StandardEvaluationContext::new(root);
    let result = parsed.get_value_with_context(&ctx).unwrap();
    result.value().clone()
}

fn parse_ok(expr: &str) {
    let parser = SpelExpressionParser::new();
    parser.parse_expression(expr).unwrap_or_else(|e| panic!("parse failed for {expr:?}: {e}"));
}

fn parse_err(expr: &str) {
    let parser = SpelExpressionParser::new();
    assert!(parser.parse_expression(expr).is_err(), "expected error for: {expr:?}");
}

// ══════════════════════════════════════════════════════════════════════════════
// 一、字面量解析
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn literal_int() { assert_eq!(eval("42"), ExpressionValue::Int(42)); }

#[test]
fn literal_int_zero() { assert_eq!(eval("0"), ExpressionValue::Int(0)); }

#[test]
fn literal_int_negative() { assert_eq!(eval("-5"), ExpressionValue::Int(-5)); }

#[test]
fn literal_long() { assert_eq!(eval("42L"), ExpressionValue::Int(42)); }

#[test]
fn literal_long_lowercase() { assert_eq!(eval("42l"), ExpressionValue::Int(42)); }

#[test]
fn literal_real() { assert_eq!(eval("3.14"), ExpressionValue::Float(3.14)); }

#[test]
fn literal_real_float_suffix() { assert_eq!(eval("3.14F"), ExpressionValue::Float(3.14)); }

#[test]
fn literal_real_double_suffix() { assert_eq!(eval("3.14D"), ExpressionValue::Float(3.14)); }

#[test]
fn literal_hex_int() { assert_eq!(eval("0xFF"), ExpressionValue::Int(255)); }

#[test]
fn literal_hex_long() { assert_eq!(eval("0x1AL"), ExpressionValue::Int(26)); }

#[test]
fn literal_scientific() { assert_eq!(eval("1e3"), ExpressionValue::Float(1000.0)); }

#[test]
fn literal_scientific_negative() { assert_eq!(eval("1.5e-2"), ExpressionValue::Float(0.015)); }

#[test]
fn literal_string_single() { assert_eq!(eval("'hello'"), ExpressionValue::String("hello".to_string())); }

#[test]
fn literal_string_double() { assert_eq!(eval(r#""world""#), ExpressionValue::String("world".to_string())); }

#[test]
fn literal_string_escape() { assert_eq!(eval("'it''s'"), ExpressionValue::String("it's".to_string())); }

#[test]
fn literal_true() { assert_eq!(eval("true"), ExpressionValue::Boolean(true)); }

#[test]
fn literal_false() { assert_eq!(eval("false"), ExpressionValue::Boolean(false)); }

#[test]
fn literal_null() { assert_eq!(eval("null"), ExpressionValue::Null); }

// ══════════════════════════════════════════════════════════════════════════════
// 二、算术运算
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn add_integers() { assert_eq!(eval("2 + 3"), ExpressionValue::Int(5)); }

#[test]
fn subtract_integers() { assert_eq!(eval("10 - 4"), ExpressionValue::Int(6)); }

#[test]
fn multiply_integers() { assert_eq!(eval("3 * 5"), ExpressionValue::Int(15)); }

#[test]
fn divide_integers() { assert_eq!(eval("10 / 2"), ExpressionValue::Int(5)); }

#[test]
fn modulus_integers() { assert_eq!(eval("10 % 3"), ExpressionValue::Int(1)); }

#[test]
fn power_integers() { assert_eq!(eval("2 ^ 3"), ExpressionValue::Int(8)); }

#[test]
fn string_concat() { assert_eq!(eval("'hello' + ' world'"), ExpressionValue::String("hello world".to_string())); }

#[test]
fn string_int_concat() { assert_eq!(eval("'value: ' + 42"), ExpressionValue::String("value: 42".to_string())); }

#[test]
fn int_string_concat() { assert_eq!(eval("42 + ' is the answer'"), ExpressionValue::String("42 is the answer".to_string())); }

// ══════════════════════════════════════════════════════════════════════════════
// 三、比较运算
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn equal_true() { assert_eq!(eval("5 == 5"), ExpressionValue::Boolean(true)); }

#[test]
fn equal_false() { assert_eq!(eval("5 == 3"), ExpressionValue::Boolean(false)); }

#[test]
fn not_equal() { assert_eq!(eval("5 != 3"), ExpressionValue::Boolean(true)); }

#[test]
fn less_than() { assert_eq!(eval("3 < 5"), ExpressionValue::Boolean(true)); }

#[test]
fn less_equal() { assert_eq!(eval("5 <= 5"), ExpressionValue::Boolean(true)); }

#[test]
fn greater_than() { assert_eq!(eval("5 > 3"), ExpressionValue::Boolean(true)); }

#[test]
fn greater_equal() { assert_eq!(eval("5 >= 5"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_eq() { assert_eq!(eval("5 eq 5"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_ne() { assert_eq!(eval("5 ne 3"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_lt() { assert_eq!(eval("3 lt 5"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_le() { assert_eq!(eval("5 le 5"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_gt() { assert_eq!(eval("5 gt 3"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_ge() { assert_eq!(eval("5 ge 5"), ExpressionValue::Boolean(true)); }

#[test]
fn alt_div() { assert_eq!(eval("10 div 3"), ExpressionValue::Int(3)); }

#[test]
fn alt_mod() { assert_eq!(eval("10 mod 3"), ExpressionValue::Int(1)); }

// ══════════════════════════════════════════════════════════════════════════════
// 四、逻辑运算
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn and_true() { assert_eq!(eval("true && true"), ExpressionValue::Boolean(true)); }

#[test]
fn and_false() { assert_eq!(eval("true && false"), ExpressionValue::Boolean(false)); }

#[test]
fn or_true() { assert_eq!(eval("true || false"), ExpressionValue::Boolean(true)); }

#[test]
fn or_false() { assert_eq!(eval("false || false"), ExpressionValue::Boolean(false)); }

#[test]
fn not_true() { assert_eq!(eval("!true"), ExpressionValue::Boolean(false)); }

#[test]
fn not_false() { assert_eq!(eval("!false"), ExpressionValue::Boolean(true)); }

// ══════════════════════════════════════════════════════════════════════════════
// 五、三元 / Elvis
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn ternary_true() { assert_eq!(eval("true ? 1 : 2"), ExpressionValue::Int(1)); }

#[test]
fn ternary_false() { assert_eq!(eval("false ? 1 : 2"), ExpressionValue::Int(2)); }

#[test]
fn elvis_non_null() { assert_eq!(eval("'value' ?: 'default'"), ExpressionValue::String("value".to_string())); }

// ══════════════════════════════════════════════════════════════════════════════
// 六、matches
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn matches_true() { assert_eq!(eval("'hello' matches 'h.*'"), ExpressionValue::Boolean(true)); }

#[test]
fn matches_false() { assert_eq!(eval("'hello' matches 'x.*'"), ExpressionValue::Boolean(false)); }

// ══════════════════════════════════════════════════════════════════════════════
// 七、between
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn between_parse() { parse_ok("5 between {1, 10}"); }

// ══════════════════════════════════════════════════════════════════════════════
// 八、instanceof
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn instanceof_parse() { parse_ok("'hello' instanceof T(String)"); }

// ══════════════════════════════════════════════════════════════════════════════
// 九、Selection / Projection
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn selection_parse() { parse_ok("{1,2,3}.?[true]"); }

#[test]
fn projection_parse() { parse_ok("{1,2,3}.![true]"); }

// ══════════════════════════════════════════════════════════════════════════════
// 十、Assign / Inc / Dec
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn assign_parse() { parse_ok("a = 5"); }

#[test]
fn inc_prefix() { assert_eq!(eval("++5"), ExpressionValue::Int(6)); }

#[test]
fn dec_prefix() { assert_eq!(eval("--5"), ExpressionValue::Int(4)); }

#[test]
fn inc_postfix() { assert_eq!(eval("5++"), ExpressionValue::Int(5)); }

#[test]
fn dec_postfix() { assert_eq!(eval("5--"), ExpressionValue::Int(5)); }

// ══════════════════════════════════════════════════════════════════════════════
// 十一、Compound expression / dot chain
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dot_chain_parse() { parse_ok("a.b.c"); }

#[test]
fn safe_nav_parse() { parse_ok("a?.b?.c"); }

#[test]
fn indexer_parse() { parse_ok("{1,2,3}[0]"); }

// ══════════════════════════════════════════════════════════════════════════════
// 十二、Method / Constructor / Variable / Bean
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn method_call_parse() { parse_ok("'hello'.toUpperCase()"); }

#[test]
fn constructor_parse() { parse_ok("new ArrayList()"); }

#[test]
fn variable_parse() { parse_ok("#myVar"); }

#[test]
fn bean_ref_parse() { parse_ok("@myBean"); }

#[test]
fn type_ref_parse() { parse_ok("T(String)"); }

// ══════════════════════════════════════════════════════════════════════════════
// 十三、错误路径
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn empty_expression_errors() { parse_err(""); }

#[test]
fn unclosed_string_errors() { parse_err("'hello"); }

#[test]
fn trailing_garbage_errors() { parse_err("1 + 2 abc"); }

// ══════════════════════════════════════════════════════════════════════════════
// 十四、复杂混合表达式
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn complex_arithmetic() { assert_eq!(eval("(2 + 3) * 4 - 1"), ExpressionValue::Int(19)); }

#[test]
fn string_concat_expression() { assert_eq!(eval("'Result: ' + (2 + 3)"), ExpressionValue::String("Result: 5".to_string())); }

#[test]
fn comparison_chain() { parse_ok("1 < 2 && 3 > 2"); }

#[test]
fn ternary_with_comparison() { assert_eq!(eval("5 > 3 ? 'yes' : 'no'"), ExpressionValue::String("yes".to_string())); }

//! 内部解析器测试（对标 Spring LiteralTests / OperatorTests / ParsingTests）。
//!
//! 这些测试直接调用 `SpelExpressionParser::parse_expression` 然后
//! `get_value_with_context` 求值，验证解析器 + AST 求值的正确性。

use vernal_expression::spel::spel_expression_parser::SpelExpressionParser;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;

/// 辅助：解析 + 默认上下文求值
fn eval(expr_str: &str) -> ExpressionValue {
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression(expr_str).unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx).unwrap();
    result.value().clone()
}

// ══════════════════════════════════════════════════════════════════════════════
// 一、字面量（对标 Spring LiteralTests）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn literal_integer() {
    assert_eq!(eval("42"), ExpressionValue::Int(42));
}

#[test]
fn literal_negative_integer() {
    let v = eval("-5");
    assert_eq!(v, ExpressionValue::Int(-5));
}

#[test]
fn literal_zero() {
    assert_eq!(eval("0"), ExpressionValue::Int(0));
}

#[test]
fn literal_long() {
    assert_eq!(eval("42L"), ExpressionValue::Int(42));
}

#[test]
fn literal_real() {
    assert_eq!(eval("3.14"), ExpressionValue::Float(3.14));
}

#[test]
fn literal_string_single_quotes() {
    assert_eq!(
        eval("'hello'"),
        ExpressionValue::String("hello".to_string())
    );
}

#[test]
fn literal_string_double_quotes() {
    assert_eq!(
        eval(r#""world""#),
        ExpressionValue::String("world".to_string())
    );
}

#[test]
fn literal_string_with_escape() {
    // '' is single-quote escape in SpEL
    assert_eq!(eval("'it''s'"), ExpressionValue::String("it's".to_string()));
}

#[test]
fn literal_null() {
    assert_eq!(eval("null"), ExpressionValue::Null);
}

#[test]
fn literal_true() {
    assert_eq!(eval("true"), ExpressionValue::Boolean(true));
}

#[test]
fn literal_false() {
    assert_eq!(eval("false"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 二、算术运算符（对标 Spring OperatorTests）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn addition_integers() {
    assert_eq!(eval("2 + 3"), ExpressionValue::Int(5));
}

#[test]
fn subtraction_integers() {
    assert_eq!(eval("10 - 4"), ExpressionValue::Int(6));
}

#[test]
fn multiplication_integers() {
    assert_eq!(eval("3 * 5"), ExpressionValue::Int(15));
}

#[test]
fn division_integers() {
    assert_eq!(eval("10 / 2"), ExpressionValue::Int(5));
}

#[test]
fn modulus_integers() {
    assert_eq!(eval("10 % 3"), ExpressionValue::Int(1));
}

#[test]
fn unary_minus() {
    assert_eq!(eval("-5"), ExpressionValue::Int(-5));
}

#[test]
fn addition_reals() {
    assert_eq!(eval("3.0 + 2.5"), ExpressionValue::Float(5.5));
}

#[test]
fn string_concatenation() {
    assert_eq!(
        eval("'hello' + ' world'"),
        ExpressionValue::String("hello world".to_string())
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 三、比较运算符（对标 Spring OperatorTests::equal / lessThan 等）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn equal_integers_true() {
    assert_eq!(eval("5 == 5"), ExpressionValue::Boolean(true));
}

#[test]
fn equal_integers_false() {
    assert_eq!(eval("5 == 3"), ExpressionValue::Boolean(false));
}

#[test]
fn not_equal() {
    assert_eq!(eval("5 != 3"), ExpressionValue::Boolean(true));
    assert_eq!(eval("5 != 5"), ExpressionValue::Boolean(false));
}

#[test]
fn less_than() {
    assert_eq!(eval("3 < 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("5 < 3"), ExpressionValue::Boolean(false));
    assert_eq!(eval("5 < 5"), ExpressionValue::Boolean(false));
}

#[test]
fn less_equal() {
    assert_eq!(eval("5 <= 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("3 <= 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("6 <= 5"), ExpressionValue::Boolean(false));
}

#[test]
fn greater_than() {
    assert_eq!(eval("5 > 3"), ExpressionValue::Boolean(true));
    assert_eq!(eval("3 > 5"), ExpressionValue::Boolean(false));
    assert_eq!(eval("5 > 5"), ExpressionValue::Boolean(false));
}

#[test]
fn greater_equal() {
    assert_eq!(eval("5 >= 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("6 >= 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("3 >= 5"), ExpressionValue::Boolean(false));
}

// ══════════════════════════════════════════════════════════════════════════════
// 四、逻辑运算符（对标 Spring BooleanExpressionTests）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn logical_and() {
    assert_eq!(eval("true && true"), ExpressionValue::Boolean(true));
    assert_eq!(eval("true && false"), ExpressionValue::Boolean(false));
    assert_eq!(eval("false && true"), ExpressionValue::Boolean(false));
}

#[test]
fn logical_or() {
    assert_eq!(eval("true || false"), ExpressionValue::Boolean(true));
    assert_eq!(eval("false || false"), ExpressionValue::Boolean(false));
    assert_eq!(eval("false || true"), ExpressionValue::Boolean(true));
}

// ══════════════════════════════════════════════════════════════════════════════
// 五、三元 / Elvis（对标 Spring OperatorTests::ternary / elvis）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn ternary_basic() {
    // true ? 1 : 2 → Int(1)
    let v = eval("true ? 1 : 2");
    assert_eq!(v, ExpressionValue::Int(1));
}

#[test]
fn elvis_basic() {
    // null ?: 'default' → "default"
    let v = eval("null ?: 'default'");
    assert_eq!(v, ExpressionValue::String("default".to_string()));
}

#[test]
fn elvis_with_value() {
    // 'value' ?: 'default' → "value"
    let v = eval("'value' ?: 'default'");
    assert_eq!(v, ExpressionValue::String("value".to_string()));
}

// ══════════════════════════════════════════════════════════════════════════════
// 六、括号与嵌套（对标 Spring ParsingTests）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn parenthesized_expression() {
    assert_eq!(eval("(2 + 3)"), ExpressionValue::Int(5));
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
fn string_literal_with_spaces() {
    assert_eq!(
        eval("'hello world'"),
        ExpressionValue::String("hello world".to_string())
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 七、标识符与复合表达式（property access 占位）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn identifier_returns_string_name() {
    // PropertyOrFieldReference 通过 PropertyAccessor 查找
    // 默认上下文没有配置 PropertyAccessor，应返回错误
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression("myVar").unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx);
    assert!(result.is_err());
}

#[test]
fn dot_chain_returns_last() {
    // CompoundExpression 链式求值：a.b.c
    // 默认上下文没有配置 PropertyAccessor，应返回错误
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression("a.b.c").unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx);
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 八、变量引用 #var（占位）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hash_var_reference() {
    // VariableReference 通过 lookup_variable 查找
    // 默认上下文没有配置变量，应返回错误
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression("#myVar").unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx);
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 九、Bean 引用 @bean（占位）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_reference() {
    // BeanReference 通过 BeanResolver 查找
    // 默认上下文没有配置 BeanResolver，应返回错误
    let parser = SpelExpressionParser::new();
    let expr = parser.parse_expression("@myBean").unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let result = expr.get_value_with_context(&ctx);
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 十、错误处理（对标 Spring ParserErrorMessagesTests）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn empty_expression_fails() {
    let parser = SpelExpressionParser::new();
    let result = parser.parse_expression("");
    assert!(result.is_err());
}

#[test]
fn trailing_garbage_fails() {
    let parser = SpelExpressionParser::new();
    let result = parser.parse_expression("1 + 2 abc");
    assert!(result.is_err());
}

#[test]
fn unclosed_string_fails() {
    let parser = SpelExpressionParser::new();
    let result = parser.parse_expression("'hello");
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 十一、替代运算符名（Spring feature: eq/ne/lt/le/gt/ge/div/mod/not）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn alternative_operator_eq() {
    assert_eq!(eval("5 eq 5"), ExpressionValue::Boolean(true));
    assert_eq!(eval("5 eq 3"), ExpressionValue::Boolean(false));
}

#[test]
fn alternative_operator_ne() {
    assert_eq!(eval("5 ne 3"), ExpressionValue::Boolean(true));
}

#[test]
fn alternative_operator_lt() {
    assert_eq!(eval("3 lt 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alternative_operator_le() {
    assert_eq!(eval("5 le 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alternative_operator_gt() {
    assert_eq!(eval("5 gt 3"), ExpressionValue::Boolean(true));
}

#[test]
fn alternative_operator_ge() {
    assert_eq!(eval("5 ge 5"), ExpressionValue::Boolean(true));
}

#[test]
fn alternative_operator_div() {
    assert_eq!(eval("10 div 3"), ExpressionValue::Int(3));
}

#[test]
fn alternative_operator_mod() {
    assert_eq!(eval("10 mod 3"), ExpressionValue::Int(1));
}

// ══════════════════════════════════════════════════════════════════════════════
// 十二、十六进制字面量（Spring LiteralTests::hexIntegers）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hex_integer_literal() {
    assert_eq!(eval("0xFF"), ExpressionValue::Int(255));
}

#[test]
fn hex_long_literal() {
    assert_eq!(eval("0x1AL"), ExpressionValue::Int(26));
}

// ══════════════════════════════════════════════════════════════════════════════
// 十三、科学计数法（Spring LiteralTests::doublesUsingExponents）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn scientific_notation() {
    assert_eq!(eval("1e3"), ExpressionValue::Float(1000.0));
}

#[test]
fn scientific_notation_negative_exponent() {
    assert_eq!(eval("1.5e-2"), ExpressionValue::Float(0.015));
}

// ══════════════════════════════════════════════════════════════════════════════
// 十四、幂运算（Spring OperatorTests::power）
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn power_basic() {
    assert_eq!(eval("2 ^ 3"), ExpressionValue::Int(8));
}

// ══════════════════════════════════════════════════════════════════════════════
// 十五、复杂混合表达式
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn complex_arithmetic_expression() {
    assert_eq!(eval("(2 + 3) * 4 - 1"), ExpressionValue::Int(19));
}

#[test]
fn string_concat_with_expression() {
    assert_eq!(
        eval("'Result: ' + (2 + 3)"),
        ExpressionValue::String("Result: 5".to_string())
    );
}

#[test]
fn debug_inc_literal_v2() {
    // ++5 should be OpInc(IntLiteral(5))
    let parser = SpelExpressionParser::new();
    let e = parser.parse_expression("++5").unwrap();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let r = e.get_value_with_context(&ctx).unwrap();
    // Just print for debug
    eprintln!("++5 => {:?}", r.value());
}

#[test]
fn debug_inc_literal() {
    let parser = SpelExpressionParser::new();
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    for e in &["5", "++5", "--5"] {
        let parsed = parser.parse_expression(e).unwrap();
        let r = parsed.get_value_with_context(&ctx).unwrap();
        eprintln!("{e} => {:?}", r.value());
    }
}

#[test]
fn debug_five_ast() {
    let parser = SpelExpressionParser::new();
    let e = parser.parse_expression("5").unwrap();
    // The Expression trait has expression_string() and get_value methods
    // Let's check what AST we get
    let ctx = StandardEvaluationContext::new(TypedValue::null());
    let r = e.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *r.value(),
        ExpressionValue::Int(5),
        "5 should parse to Int(5), got {:?}",
        r.value()
    );
}

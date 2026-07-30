//! ValueBinding 测试 - 对标 Spring @Value 注解

use vernal_context::value_binding::ValueBinding;

#[test]
fn test_value_binding_new() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    assert_eq!(binding.expression, "#{bean.property}");
    assert_eq!(binding.field_name, "email");
    assert!(binding.required);
}

#[test]
fn test_value_binding_optional() {
    let binding = ValueBinding::new("${server.port:8080}", "port").optional();
    assert!(!binding.required);
}

#[test]
fn test_value_binding_is_placeholder() {
    let binding = ValueBinding::new("${server.port}", "port");
    assert!(binding.is_placeholder());
}

#[test]
fn test_value_binding_is_placeholder_with_default() {
    let binding = ValueBinding::new("${server.port:8080}", "port");
    assert!(binding.is_placeholder());
}

#[test]
fn test_value_binding_is_not_placeholder() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    assert!(!binding.is_placeholder());
}

#[test]
fn test_value_binding_is_spel_with_hash() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    assert!(binding.is_spel());
}

#[test]
fn test_value_binding_is_spel_bare_expression() {
    let binding = ValueBinding::new("1 + 2", "sum");
    assert!(binding.is_spel());
}

#[test]
fn test_value_binding_placeholder_key() {
    let binding = ValueBinding::new("${server.port}", "port");
    assert_eq!(binding.placeholder_key(), Some("server.port"));
}

#[test]
fn test_value_binding_placeholder_key_with_default() {
    let binding = ValueBinding::new("${server.port:8080}", "port");
    assert_eq!(binding.placeholder_key(), Some("server.port:8080"));
}

#[test]
fn test_value_binding_placeholder_key_none_for_spel() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    assert!(binding.placeholder_key().is_none());
}

#[test]
fn test_value_binding_spel_expression() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    assert_eq!(binding.spel_expression(), "bean.property");
}

#[test]
fn test_value_binding_spel_expression_bare() {
    let binding = ValueBinding::new("1 + 2", "sum");
    assert_eq!(binding.spel_expression(), "1 + 2");
}

#[test]
fn test_value_binding_debug() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    let debug = format!("{binding:?}");
    assert!(debug.contains("ValueBinding"));
    assert!(debug.contains("bean.property"));
}

#[test]
fn test_value_binding_clone() {
    let binding = ValueBinding::new("#{bean.property}", "email");
    let cloned = binding.clone();
    assert_eq!(binding.expression, cloned.expression);
    assert_eq!(binding.field_name, cloned.field_name);
    assert_eq!(binding.required, cloned.required);
}

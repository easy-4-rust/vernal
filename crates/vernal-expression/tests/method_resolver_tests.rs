//! MethodResolver 链路集成测试。
use vernal_expression::spel::ast::method_reference::MethodReference;
use vernal_expression::spel::ast::spel_node::SpelNode;
use vernal_expression::spel::support::reflective_method_resolver::{
    ArcReflectiveMethodExecutor, ReflectiveMethodResolver,
};
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::*;

#[test]
fn resolver_register_and_resolve_by_name() {
    let resolver = ReflectiveMethodResolver::new();
    let executor = ArcReflectiveMethodExecutor::new(|_ctx, _target, _args| {
        Ok(TypedValue::new(
            ExpressionValue::Int(42),
            TypeDescriptor::Primitive(PrimitiveKind::Int),
        ))
    });
    resolver.register("answer".to_string(), executor);
    let ctx = StandardEvaluationContext::new_default();
    let target = TypedValue::null();
    let result = resolver.resolve(&ctx, &target, "answer", &[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn resolver_returns_none_for_unregistered_method() {
    let resolver = ReflectiveMethodResolver::new();
    let ctx = StandardEvaluationContext::new_default();
    let target = TypedValue::null();
    let result = resolver.resolve(&ctx, &target, "nonexistent", &[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn resolver_has_method_and_method_names() {
    let resolver = ReflectiveMethodResolver::new();
    assert!(!resolver.has_method("foo"));
    resolver.register(
        "foo".to_string(),
        ArcReflectiveMethodExecutor::new(|_, _, _| Ok(TypedValue::null())),
    );
    resolver.register(
        "bar".to_string(),
        ArcReflectiveMethodExecutor::new(|_, _, _| Ok(TypedValue::null())),
    );
    assert!(resolver.has_method("foo"));
    assert!(resolver.has_method("bar"));
    assert!(!resolver.has_method("baz"));
    let mut names = resolver.method_names();
    names.sort();
    assert_eq!(names, vec!["bar", "foo"]);
}

#[test]
fn method_reference_calls_registered_method() {
    let ctx = StandardEvaluationContext::new_default();
    ctx.register_method_fn("greet", |_ctx, _target, _args| {
        Ok(TypedValue::new(
            ExpressionValue::String("hello".to_string()),
            TypeDescriptor::Primitive(PrimitiveKind::String),
        ))
    });
    let method_ref = MethodReference::new("greet".to_string(), vec![], false);
    let result = method_ref.get_value(&ctx);
    assert!(result.is_ok());
    assert_eq!(
        *result.unwrap().value(),
        ExpressionValue::String("hello".to_string())
    );
}

#[test]
fn method_reference_not_found_returns_error() {
    let ctx = StandardEvaluationContext::new_default();
    let method_ref = MethodReference::new("unknown".to_string(), vec![], false);
    let result = method_ref.get_value(&ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().simple_message().contains("unknown"));
}

#[test]
fn parsed_expression_calls_registered_method() {
    let parser = vernal_expression::spel::spel_expression_parser::SpelExpressionParser::new();
    let expr = parser.parse_expression("greet()").unwrap();
    let ctx = StandardEvaluationContext::new_default();
    ctx.register_method_fn("greet", |_ctx, _target, _args| {
        Ok(TypedValue::new(
            ExpressionValue::String("hello, world".to_string()),
            TypeDescriptor::Primitive(PrimitiveKind::String),
        ))
    });
    let result = expr.get_value_with_context(&ctx).unwrap();
    assert_eq!(
        *result.value(),
        ExpressionValue::String("hello, world".to_string())
    );
}

#[test]
fn parsed_expression_method_not_found() {
    let parser = vernal_expression::spel::spel_expression_parser::SpelExpressionParser::new();
    let expr = parser.parse_expression("noSuchMethod()").unwrap();
    let ctx = StandardEvaluationContext::new_default();
    let result = expr.get_value_with_context(&ctx);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .simple_message()
            .contains("noSuchMethod")
    );
}

#[test]
fn method_with_int_argument_via_ast() {
    let ctx = StandardEvaluationContext::new_default();
    ctx.register_method_fn("add", |_ctx, _target, args| {
        if args.len() >= 2 {
            if let (ExpressionValue::Int(a), ExpressionValue::Int(b)) =
                (args[0].value(), args[1].value())
            {
                return Ok(TypedValue::new(
                    ExpressionValue::Int(a + b),
                    TypeDescriptor::Primitive(PrimitiveKind::Int),
                ));
            }
        }
        Err(AccessException::new("需要两个 Int 参数"))
    });
    let arg1 = Box::new(vernal_expression::spel::ast::int_literal::IntLiteral::new(
        1,
        "1".to_string(),
    ));
    let arg2 = Box::new(vernal_expression::spel::ast::int_literal::IntLiteral::new(
        2,
        "2".to_string(),
    ));
    let method_ref = MethodReference::new("add".to_string(), vec![arg1, arg2], false);
    let result = method_ref.get_value(&ctx);
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().value(), ExpressionValue::Int(3));
}

#[test]
fn arc_executor_executes_and_clones() {
    let executor = ArcReflectiveMethodExecutor::new(|_ctx, _target, _args| {
        Ok(TypedValue::new(
            ExpressionValue::Boolean(true),
            TypeDescriptor::Primitive(PrimitiveKind::Boolean),
        ))
    });
    let ctx = StandardEvaluationContext::new_default();
    let target = TypedValue::null();
    let result = executor.execute(&ctx, &target, &[]).unwrap();
    assert_eq!(*result.value(), ExpressionValue::Boolean(true));
    let cloned = executor.clone();
    let result2 = cloned.execute(&ctx, &target, &[]).unwrap();
    assert_eq!(*result2.value(), ExpressionValue::Boolean(true));
}

#[test]
fn executor_with_args() {
    let executor = ArcReflectiveMethodExecutor::new(|_ctx, _target, args| {
        if let Some(arg) = args.first() {
            if let ExpressionValue::Int(n) = arg.value() {
                return Ok(TypedValue::new(
                    ExpressionValue::Int(n * n),
                    TypeDescriptor::Primitive(PrimitiveKind::Int),
                ));
            }
        }
        Err(AccessException::new("需要 Int 参数"))
    });
    let ctx = StandardEvaluationContext::new_default();
    let target = TypedValue::null();
    let args = vec![TypedValue::new(
        ExpressionValue::Int(5),
        TypeDescriptor::Primitive(PrimitiveKind::Int),
    )];
    let result = executor.execute(&ctx, &target, &args).unwrap();
    assert_eq!(*result.value(), ExpressionValue::Int(25));
}

#[test]
fn multiple_methods_registered() {
    let ctx = StandardEvaluationContext::new_default();
    ctx.register_method_fn("add", |_ctx, _target, args| {
        if args.len() >= 2 {
            if let (ExpressionValue::Int(a), ExpressionValue::Int(b)) =
                (args[0].value(), args[1].value())
            {
                return Ok(TypedValue::new(
                    ExpressionValue::Int(a + b),
                    TypeDescriptor::Primitive(PrimitiveKind::Int),
                ));
            }
        }
        Err(AccessException::new("需要两个 Int 参数"))
    });
    ctx.register_method_fn("multiply", |_ctx, _target, args| {
        if args.len() >= 2 {
            if let (ExpressionValue::Int(a), ExpressionValue::Int(b)) =
                (args[0].value(), args[1].value())
            {
                return Ok(TypedValue::new(
                    ExpressionValue::Int(a * b),
                    TypeDescriptor::Primitive(PrimitiveKind::Int),
                ));
            }
        }
        Err(AccessException::new("需要两个 Int 参数"))
    });
    let arg_a = Box::new(vernal_expression::spel::ast::int_literal::IntLiteral::new(
        3,
        "3".to_string(),
    ));
    let arg_b = Box::new(vernal_expression::spel::ast::int_literal::IntLiteral::new(
        4,
        "4".to_string(),
    ));
    let add_ref = MethodReference::new("add".to_string(), vec![arg_a, arg_b], false);
    assert_eq!(
        *add_ref.get_value(&ctx).unwrap().value(),
        ExpressionValue::Int(7)
    );
    let arg_c = Box::new(vernal_expression::spel::ast::int_literal::IntLiteral::new(
        3,
        "3".to_string(),
    ));
    let arg_d = Box::new(vernal_expression::spel::ast::int_literal::IntLiteral::new(
        4,
        "4".to_string(),
    ));
    let mul_ref = MethodReference::new("multiply".to_string(), vec![arg_c, arg_d], false);
    assert_eq!(
        *mul_ref.get_value(&ctx).unwrap().value(),
        ExpressionValue::Int(12)
    );
}

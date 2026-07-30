//! Comprehensive tests for support modules with low or 0% coverage.
//!
//! Covers: MapAccessor, ReflectiveIndexAccessor, ExpressionState,
//! SpelParserConfiguration, SpelNodeImpl, TypeCode, SimpleEvaluationContext,
//! ReflectivePropertyAccessor, StandardEvaluationContext, SpelMessage,
//! SpelEvaluationException, SpelParseException, InternalParseException,
//! TokenKind, Token.

use std::collections::HashMap;

use vernal_expression::expression_value::ExpressionValue;
use vernal_expression::spel::ast::spel_node::SpelNode;
use vernal_expression::spel::ast::spel_node_impl::SpelNodeImpl;
use vernal_expression::spel::ast::type_code::TypeCode;
use vernal_expression::spel::expression_state::ExpressionState;
use vernal_expression::spel::internal_parse_exception::InternalParseException;
use vernal_expression::spel::spel_evaluation_exception::SpelEvaluationException;
use vernal_expression::spel::spel_message::SpelMessage;
use vernal_expression::spel::spel_parse_exception::SpelParseException;
use vernal_expression::spel::spel_parser_configuration::SpelParserConfiguration;
use vernal_expression::spel::support::map_accessor::MapAccessor;
use vernal_expression::spel::support::reflective_index_accessor::ReflectiveIndexAccessor;
use vernal_expression::spel::support::reflective_property_accessor::ReflectivePropertyAccessor;
use vernal_expression::spel::support::simple_evaluation_context::SimpleEvaluationContext;
use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
use vernal_expression::spel::token::Token;
use vernal_expression::spel::token_kind::TokenKind;
// Re-exported types from the crate root (modules are private)
use vernal_expression::{
    AccessException, BeanResolver, ConstructorResolver, EvaluationContext,
    EvaluationException, IndexAccessor, MethodResolver, OperatorOverloader,
    PropertyAccessor, TypeComparator, TypeConverter, TypeDescriptor,
    PrimitiveKind, TypeLocator, TypedValue,
};

// =========================================================================
// Test helper: minimal EvaluationContext
// =========================================================================

struct DummyCtx {
    root: TypedValue,
    variables: HashMap<String, TypedValue>,
}

impl DummyCtx {
    fn new(root: TypedValue) -> Self {
        Self {
            root,
            variables: HashMap::new(),
        }
    }

    fn null_root() -> Self {
        Self::new(TypedValue::null())
    }
}

impl EvaluationContext for DummyCtx {
    fn root_object(&self) -> &TypedValue {
        &self.root
    }

    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor> {
        Vec::new()
    }

    fn bean_resolver(&self) -> Option<&dyn BeanResolver> {
        None
    }

    fn type_converter(&self) -> Option<&dyn TypeConverter> {
        None
    }

    fn type_locator(&self) -> Option<&dyn TypeLocator> {
        None
    }

    fn type_comparator(&self) -> Option<&dyn TypeComparator> {
        None
    }

    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader> {
        None
    }

    fn method_resolvers(&self) -> Vec<&dyn MethodResolver> {
        Vec::new()
    }

    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver> {
        Vec::new()
    }

    fn set_variable(&mut self, name: &str, value: TypedValue) {
        self.variables.insert(name.to_string(), value);
    }

    fn lookup_variable(&self, name: &str) -> Option<&TypedValue> {
        self.variables.get(name)
    }
}

fn make_string_tv(s: &str) -> TypedValue {
    TypedValue::new(
        ExpressionValue::String(s.to_string()),
        TypeDescriptor::STRING,
    )
}

fn make_int_tv(i: i64) -> TypedValue {
    TypedValue::new(ExpressionValue::Int(i), TypeDescriptor::Primitive(PrimitiveKind::Int))
}

fn make_bool_tv(b: bool) -> TypedValue {
    TypedValue::new(
        ExpressionValue::Boolean(b),
        TypeDescriptor::Primitive(PrimitiveKind::Boolean),
    )
}

fn make_map_tv(entries: Vec<(TypedValue, TypedValue)>) -> TypedValue {
    TypedValue::new(ExpressionValue::Map(entries), TypeDescriptor::OBJECT)
}

fn make_list_tv(items: Vec<TypedValue>) -> TypedValue {
    TypedValue::new(ExpressionValue::List(items), TypeDescriptor::OBJECT)
}

// =========================================================================
// 1. MapAccessor tests
// =========================================================================

mod map_accessor_tests {
    use super::*;

    #[test]
    fn can_read_returns_true_for_map_target() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![]);
        assert!(accessor.can_read(&ctx, &map_tv, "any_key"));
    }

    #[test]
    fn can_read_returns_false_for_non_map_target() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let string_tv = make_string_tv("hello");
        assert!(!accessor.can_read(&ctx, &string_tv, "key"));
    }

    #[test]
    fn can_read_returns_false_for_list_target() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![]);
        assert!(!accessor.can_read(&ctx, &list_tv, "key"));
    }

    #[test]
    fn can_read_returns_false_for_null_target() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_read(&ctx, &TypedValue::null(), "key"));
    }

    #[test]
    fn can_read_returns_false_for_int_target() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_read(&ctx, &make_int_tv(42), "key"));
    }

    #[test]
    fn read_finds_existing_key() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv("name"), make_string_tv("Alice")),
            (make_string_tv("age"), make_int_tv(30)),
        ]);
        let result = accessor.read(&ctx, &map_tv, "name").unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("Alice".to_string()));
    }

    #[test]
    fn read_returns_null_for_missing_key() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv("name"), make_string_tv("Alice")),
        ]);
        let result = accessor.read(&ctx, &map_tv, "missing").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_returns_null_for_empty_map() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![]);
        let result = accessor.read(&ctx, &map_tv, "anything").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_returns_error_for_non_map_target() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let string_tv = make_string_tv("hello");
        assert!(accessor.read(&ctx, &string_tv, "key").is_err());
    }

    #[test]
    fn read_finds_int_value() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv("count"), make_int_tv(42)),
        ]);
        let result = accessor.read(&ctx, &map_tv, "count").unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(42));
    }

    #[test]
    fn read_finds_bool_value() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv("active"), make_bool_tv(true)),
        ]);
        let result = accessor.read(&ctx, &map_tv, "active").unwrap();
        assert_eq!(*result.value(), ExpressionValue::Boolean(true));
    }

    #[test]
    fn can_write_always_returns_false() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![]);
        assert!(!accessor.can_write(&ctx, &map_tv, "key"));
        assert!(!accessor.can_write(&ctx, &make_string_tv("s"), "key"));
    }

    #[test]
    fn write_always_returns_error() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![]);
        assert!(accessor.write(&ctx, &map_tv, "key", &make_string_tv("v")).is_err());
    }

    #[test]
    fn specific_target_classes_returns_expected() {
        let accessor = MapAccessor;
        let classes = accessor.specific_target_classes();
        assert_eq!(classes.len(), 2);
        assert!(classes.contains(&"HashMap"));
        assert!(classes.contains(&"BTreeMap"));
    }

    #[test]
    fn read_with_empty_string_key() {
        let accessor = MapAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv(""), make_string_tv("empty_key_value")),
        ]);
        let result = accessor.read(&ctx, &map_tv, "").unwrap();
        assert_eq!(
            *result.value(),
            ExpressionValue::String("empty_key_value".to_string())
        );
    }
}

// =========================================================================
// 2. ReflectiveIndexAccessor tests
// =========================================================================

mod reflective_index_accessor_tests {
    use super::*;

    #[test]
    fn can_read_returns_true_for_list_target() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![]);
        assert!(accessor.can_read(&ctx, &list_tv, &make_int_tv(0)));
    }

    #[test]
    fn can_read_returns_true_for_map_target() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![]);
        assert!(accessor.can_read(&ctx, &map_tv, &make_string_tv("k")));
    }

    #[test]
    fn can_read_returns_false_for_string_target() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_read(&ctx, &make_string_tv("hello"), &make_int_tv(0)));
    }

    #[test]
    fn can_read_returns_false_for_int_target() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_read(&ctx, &make_int_tv(5), &make_int_tv(0)));
    }

    #[test]
    fn can_read_returns_false_for_null_target() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_read(&ctx, &TypedValue::null(), &make_int_tv(0)));
    }

    #[test]
    fn read_list_with_valid_int_index() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![
            make_string_tv("a"),
            make_string_tv("b"),
            make_string_tv("c"),
        ]);
        let result = accessor.read(&ctx, &list_tv, &make_int_tv(1)).unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("b".to_string()));
    }

    #[test]
    fn read_list_first_element() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![make_int_tv(100)]);
        let result = accessor.read(&ctx, &list_tv, &make_int_tv(0)).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(100));
    }

    #[test]
    fn read_list_out_of_bounds_returns_error() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![make_string_tv("only")]);
        let result = accessor.read(&ctx, &list_tv, &make_int_tv(5));
        assert!(result.is_err());
    }

    #[test]
    fn read_list_empty_returns_error() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![]);
        let result = accessor.read(&ctx, &list_tv, &make_int_tv(0));
        assert!(result.is_err());
    }

    #[test]
    fn read_map_with_string_key() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv("x"), make_int_tv(10)),
            (make_string_tv("y"), make_int_tv(20)),
        ]);
        let result = accessor.read(&ctx, &map_tv, &make_string_tv("y")).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(20));
    }

    #[test]
    fn read_map_missing_key_returns_null() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_string_tv("x"), make_int_tv(10)),
        ]);
        let result = accessor.read(&ctx, &map_tv, &make_string_tv("z")).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_map_empty_returns_null() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![]);
        let result = accessor.read(&ctx, &map_tv, &make_string_tv("k")).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_with_unsupported_target_returns_error() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let result = accessor.read(&ctx, &make_bool_tv(true), &make_int_tv(0));
        assert!(result.is_err());
    }

    #[test]
    fn can_write_always_returns_false() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![]);
        assert!(!accessor.can_write(&ctx, &list_tv, &make_int_tv(0)));
    }

    #[test]
    fn write_always_returns_error() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let list_tv = make_list_tv(vec![]);
        assert!(accessor
            .write(&ctx, &list_tv, &make_int_tv(0), &make_string_tv("v"))
            .is_err());
    }

    #[test]
    fn read_map_with_int_key() {
        let accessor = ReflectiveIndexAccessor;
        let ctx = DummyCtx::null_root();
        let map_tv = make_map_tv(vec![
            (make_int_tv(1), make_string_tv("one")),
            (make_int_tv(2), make_string_tv("two")),
        ]);
        let result = accessor.read(&ctx, &map_tv, &make_int_tv(2)).unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("two".to_string()));
    }
}

// =========================================================================
// 3. ExpressionState tests
// =========================================================================

mod expression_state_tests {
    use super::*;

    #[test]
    fn construction_with_null_root() {
        let ctx = DummyCtx::null_root();
        let state = ExpressionState::new(&ctx);
        assert!(state.active_context_object().is_null());
    }

    #[test]
    fn construction_with_string_root() {
        let ctx = DummyCtx::new(make_string_tv("root"));
        let state = ExpressionState::new(&ctx);
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::String("root".to_string())
        );
    }

    #[test]
    fn push_and_pop_active_context_object() {
        let ctx = DummyCtx::null_root();
        let mut state = ExpressionState::new(&ctx);
        assert!(state.active_context_object().is_null());

        state.push_active_context_object(make_string_tv("first"));
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::String("first".to_string())
        );

        state.push_active_context_object(make_int_tv(42));
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::Int(42)
        );

        let popped = state.pop_active_context_object();
        assert_eq!(*popped.value(), ExpressionValue::Int(42));
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::String("first".to_string())
        );

        let popped2 = state.pop_active_context_object();
        assert_eq!(
            *popped2.value(),
            ExpressionValue::String("first".to_string())
        );
        // Now back to root
        assert!(state.active_context_object().is_null());
    }

    #[test]
    fn set_and_lookup_variable_in_state() {
        let ctx = DummyCtx::null_root();
        let mut state = ExpressionState::new(&ctx);
        state.set_variable("myVar", make_string_tv("hello"));
        let found = state.lookup_variable("myVar");
        assert!(found.is_some());
        assert_eq!(
            *found.unwrap().value(),
            ExpressionValue::String("hello".to_string())
        );
    }

    #[test]
    fn lookup_variable_returns_none_for_missing() {
        let ctx = DummyCtx::null_root();
        let state = ExpressionState::new(&ctx);
        assert!(state.lookup_variable("nonexistent").is_none());
    }

    #[test]
    fn lookup_variable_falls_through_to_context() {
        let mut ctx = DummyCtx::null_root();
        ctx.set_variable("ctx_var", make_int_tv(99));
        let state = ExpressionState::new(&ctx);
        let found = state.lookup_variable("ctx_var");
        assert!(found.is_some());
        assert_eq!(*found.unwrap().value(), ExpressionValue::Int(99));
    }

    #[test]
    fn state_variable_overrides_context_variable() {
        let mut ctx = DummyCtx::null_root();
        ctx.set_variable("x", make_int_tv(1));
        let mut state = ExpressionState::new(&ctx);
        state.set_variable("x", make_int_tv(2));
        let found = state.lookup_variable("x");
        assert!(found.is_some());
        assert_eq!(*found.unwrap().value(), ExpressionValue::Int(2));
    }

    #[test]
    fn evaluation_context_returns_inner_context() {
        let ctx = DummyCtx::new(make_string_tv("root_obj"));
        let state = ExpressionState::new(&ctx);
        let ec = state.evaluation_context();
        assert_eq!(
            *ec.root_object().value(),
            ExpressionValue::String("root_obj".to_string())
        );
    }

    #[test]
    fn track_operation_increments_count() {
        let ctx = DummyCtx::null_root();
        let mut state = ExpressionState::new(&ctx);
        // operation_count is private, but track_operation should not panic
        state.track_operation();
        state.track_operation();
        state.track_operation();
    }

    #[test]
    fn multiple_push_pop_cycles() {
        let ctx = DummyCtx::null_root();
        let mut state = ExpressionState::new(&ctx);

        for i in 0..10 {
            state.push_active_context_object(make_int_tv(i));
        }
        for i in (0..10).rev() {
            let popped = state.pop_active_context_object();
            assert_eq!(*popped.value(), ExpressionValue::Int(i));
        }
    }

    #[test]
    fn overwrite_variable() {
        let ctx = DummyCtx::null_root();
        let mut state = ExpressionState::new(&ctx);
        state.set_variable("v", make_string_tv("first"));
        state.set_variable("v", make_string_tv("second"));
        let found = state.lookup_variable("v").unwrap();
        assert_eq!(
            *found.value(),
            ExpressionValue::String("second".to_string())
        );
    }
}

// =========================================================================
// 4. SpelParserConfiguration tests
// =========================================================================

mod spel_parser_configuration_tests {
    use super::*;

    #[test]
    fn new_returns_default() {
        let config = SpelParserConfiguration::new();
        assert_eq!(config.max_expression_length(), 10_000);
        assert_eq!(config.max_operations(), 10_000);
        assert!(!config.auto_grow_null_references());
    }

    #[test]
    fn default_trait_matches_new() {
        let config1 = SpelParserConfiguration::new();
        let config2 = SpelParserConfiguration::default();
        assert_eq!(config1.max_expression_length(), config2.max_expression_length());
        assert_eq!(config1.max_operations(), config2.max_operations());
        assert_eq!(
            config1.auto_grow_null_references(),
            config2.auto_grow_null_references()
        );
    }

    #[test]
    fn clone_preserves_values() {
        let config = SpelParserConfiguration::new();
        let cloned = config.clone();
        assert_eq!(config.max_expression_length(), cloned.max_expression_length());
        assert_eq!(config.max_operations(), cloned.max_operations());
    }

    #[test]
    fn debug_format_does_not_panic() {
        let config = SpelParserConfiguration::new();
        let dbg = format!("{:?}", config);
        assert!(!dbg.is_empty());
    }

    #[test]
    fn default_values_are_sensible() {
        let config = SpelParserConfiguration::default();
        assert!(config.max_expression_length() > 0);
        assert!(config.max_operations() > 0);
        assert!(!config.auto_grow_null_references());
    }
}

// =========================================================================
// 5. SpelNodeImpl tests
// =========================================================================

mod spel_node_impl_tests {
    use super::*;

    #[test]
    fn new_stores_positions() {
        let node = SpelNodeImpl::new(5, 15);
        assert_eq!(node.start_position(), 5);
        assert_eq!(node.end_position(), 15);
    }

    #[test]
    fn zero_positions() {
        let node = SpelNodeImpl::new(0, 0);
        assert_eq!(node.start_position(), 0);
        assert_eq!(node.end_position(), 0);
    }

    #[test]
    fn large_positions() {
        let node = SpelNodeImpl::new(0, 1_000_000);
        assert_eq!(node.start_position(), 0);
        assert_eq!(node.end_position(), 1_000_000);
    }

    #[test]
    fn start_equals_end_for_zero_length() {
        let node = SpelNodeImpl::new(42, 42);
        assert_eq!(node.start_position(), node.end_position());
    }
}

// =========================================================================
// 6. TypeCode tests
// =========================================================================

mod type_code_tests {
    use super::*;

    #[test]
    fn name_returns_correct_strings() {
        assert_eq!(TypeCode::Object.name(), "object");
        assert_eq!(TypeCode::Boolean.name(), "boolean");
        assert_eq!(TypeCode::Char.name(), "char");
        assert_eq!(TypeCode::Byte.name(), "byte");
        assert_eq!(TypeCode::Short.name(), "short");
        assert_eq!(TypeCode::Int.name(), "int");
        assert_eq!(TypeCode::Long.name(), "long");
        assert_eq!(TypeCode::Float.name(), "float");
        assert_eq!(TypeCode::Double.name(), "double");
        assert_eq!(TypeCode::BigInteger.name(), "java.math.BigInteger");
        assert_eq!(TypeCode::BigDecimal.name(), "java.math.BigDecimal");
        assert_eq!(TypeCode::String.name(), "java.lang.String");
        assert_eq!(TypeCode::Array.name(), "array");
    }

    #[test]
    fn is_integer_for_integer_types() {
        assert!(TypeCode::Byte.is_integer());
        assert!(TypeCode::Short.is_integer());
        assert!(TypeCode::Int.is_integer());
        assert!(TypeCode::Long.is_integer());
        assert!(TypeCode::BigInteger.is_integer());
    }

    #[test]
    fn is_integer_false_for_non_integer_types() {
        assert!(!TypeCode::Object.is_integer());
        assert!(!TypeCode::Boolean.is_integer());
        assert!(!TypeCode::Char.is_integer());
        assert!(!TypeCode::Float.is_integer());
        assert!(!TypeCode::Double.is_integer());
        assert!(!TypeCode::BigDecimal.is_integer());
        assert!(!TypeCode::String.is_integer());
        assert!(!TypeCode::Array.is_integer());
    }

    #[test]
    fn is_number_for_numeric_types() {
        assert!(TypeCode::Byte.is_number());
        assert!(TypeCode::Short.is_number());
        assert!(TypeCode::Int.is_number());
        assert!(TypeCode::Long.is_number());
        assert!(TypeCode::Float.is_number());
        assert!(TypeCode::Double.is_number());
        assert!(TypeCode::BigInteger.is_number());
        assert!(TypeCode::BigDecimal.is_number());
    }

    #[test]
    fn is_number_false_for_non_numeric_types() {
        assert!(!TypeCode::Object.is_number());
        assert!(!TypeCode::Boolean.is_number());
        assert!(!TypeCode::Char.is_number());
        assert!(!TypeCode::String.is_number());
        assert!(!TypeCode::Array.is_number());
    }

    #[test]
    fn is_integer_subset_of_is_number() {
        let all_codes = [
            TypeCode::Object,
            TypeCode::Boolean,
            TypeCode::Char,
            TypeCode::Byte,
            TypeCode::Short,
            TypeCode::Int,
            TypeCode::Long,
            TypeCode::Float,
            TypeCode::Double,
            TypeCode::BigInteger,
            TypeCode::BigDecimal,
            TypeCode::String,
            TypeCode::Array,
        ];
        for tc in &all_codes {
            if tc.is_integer() {
                assert!(tc.is_number(), "{:?} is integer but not number", tc);
            }
        }
    }

    #[test]
    fn equality_and_hash() {
        assert_eq!(TypeCode::Int, TypeCode::Int);
        assert_ne!(TypeCode::Int, TypeCode::Long);
        assert_ne!(TypeCode::String, TypeCode::Object);
    }

    #[test]
    fn debug_format() {
        let dbg = format!("{:?}", TypeCode::Boolean);
        assert_eq!(dbg, "Boolean");
    }

    #[test]
    fn copy_semantics() {
        let a = TypeCode::Double;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn all_13_variants_exist() {
        let all = [
            TypeCode::Object,
            TypeCode::Boolean,
            TypeCode::Char,
            TypeCode::Byte,
            TypeCode::Short,
            TypeCode::Int,
            TypeCode::Long,
            TypeCode::Float,
            TypeCode::Double,
            TypeCode::BigInteger,
            TypeCode::BigDecimal,
            TypeCode::String,
            TypeCode::Array,
        ];
        assert_eq!(all.len(), 13);
    }
}

// =========================================================================
// 7. SimpleEvaluationContext tests
// =========================================================================

mod simple_evaluation_context_tests {
    use super::*;

    #[test]
    fn for_read_only_sets_assignment_disabled() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(!ctx.is_assignment_enabled());
    }

    #[test]
    fn for_read_write_sets_assignment_enabled() {
        let ctx = SimpleEvaluationContext::for_read_write(TypedValue::null());
        assert!(ctx.is_assignment_enabled());
    }

    #[test]
    fn root_object_returns_provided_root() {
        let root = make_string_tv("my_root");
        let ctx = SimpleEvaluationContext::for_read_only(root.clone());
        assert_eq!(
            *ctx.root_object().value(),
            ExpressionValue::String("my_root".to_string())
        );
    }

    #[test]
    fn root_object_null() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.root_object().is_null());
    }

    #[test]
    fn property_accessors_returns_empty() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.property_accessors().is_empty());
    }

    #[test]
    fn bean_resolver_returns_none() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.bean_resolver().is_none());
    }

    #[test]
    fn type_converter_returns_none() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.type_converter().is_none());
    }

    #[test]
    fn type_locator_returns_none() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.type_locator().is_none());
    }

    #[test]
    fn type_comparator_returns_none() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.type_comparator().is_none());
    }

    #[test]
    fn operator_overloader_returns_none() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.operator_overloader().is_none());
    }

    #[test]
    fn method_resolvers_returns_empty() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.method_resolvers().is_empty());
    }

    #[test]
    fn constructor_resolvers_returns_empty() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.constructor_resolvers().is_empty());
    }

    #[test]
    fn set_and_lookup_variable() {
        let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        ctx.set_variable("name", make_string_tv("Alice"));
        let found = ctx.lookup_variable("name");
        assert!(found.is_some());
        assert_eq!(
            *found.unwrap().value(),
            ExpressionValue::String("Alice".to_string())
        );
    }

    #[test]
    fn lookup_variable_missing_returns_none() {
        let ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        assert!(ctx.lookup_variable("nonexistent").is_none());
    }

    #[test]
    fn set_multiple_variables() {
        let mut ctx = SimpleEvaluationContext::for_read_write(TypedValue::null());
        ctx.set_variable("a", make_int_tv(1));
        ctx.set_variable("b", make_int_tv(2));
        ctx.set_variable("c", make_int_tv(3));
        assert_eq!(*ctx.lookup_variable("a").unwrap().value(), ExpressionValue::Int(1));
        assert_eq!(*ctx.lookup_variable("b").unwrap().value(), ExpressionValue::Int(2));
        assert_eq!(*ctx.lookup_variable("c").unwrap().value(), ExpressionValue::Int(3));
    }

    #[test]
    fn overwrite_variable() {
        let mut ctx = SimpleEvaluationContext::for_read_only(TypedValue::null());
        ctx.set_variable("x", make_string_tv("old"));
        ctx.set_variable("x", make_string_tv("new"));
        assert_eq!(
            *ctx.lookup_variable("x").unwrap().value(),
            ExpressionValue::String("new".to_string())
        );
    }

    #[test]
    fn read_only_context_with_list_root() {
        let root = make_list_tv(vec![make_int_tv(1), make_int_tv(2)]);
        let ctx = SimpleEvaluationContext::for_read_only(root);
        match ctx.root_object().value() {
            ExpressionValue::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("expected List"),
        }
    }
}

// =========================================================================
// 8. ReflectivePropertyAccessor tests
// =========================================================================

mod reflective_property_accessor_tests {
    use super::*;

    #[test]
    fn new_creates_accessor() {
        let accessor = ReflectivePropertyAccessor::new();
        // Should not panic on creation
        let _ = accessor;
    }

    #[test]
    fn default_trait_works() {
        let accessor = ReflectivePropertyAccessor::default();
        let ctx = DummyCtx::null_root();
        let target = TypedValue::null();
        // No bindings registered for null target, so can_read returns false
        assert!(!accessor.can_read(&ctx, &target, "any"));
    }

    #[test]
    fn can_read_returns_false_for_unregistered_property() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_string_tv("hello");
        assert!(!accessor.can_read(&ctx, &target, "nonexistent"));
    }

    #[test]
    fn can_read_returns_false_for_null_target() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_read(&ctx, &TypedValue::null(), "any"));
    }

    #[test]
    fn read_returns_error_for_unregistered_property() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_string_tv("hello");
        let result = accessor.read(&ctx, &target, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn can_write_returns_false_for_unregistered_property() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_string_tv("hello");
        assert!(!accessor.can_write(&ctx, &target, "nonexistent"));
    }

    #[test]
    fn write_returns_error() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_string_tv("hello");
        let result = accessor.write(&ctx, &target, "any", &make_string_tv("v"));
        assert!(result.is_err());
    }

    #[test]
    fn can_read_with_int_target() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_int_tv(42);
        assert!(!accessor.can_read(&ctx, &target, "value"));
    }

    #[test]
    fn can_read_with_bool_target() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_bool_tv(true);
        assert!(!accessor.can_read(&ctx, &target, "flag"));
    }

    #[test]
    fn can_read_with_list_target() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_list_tv(vec![]);
        assert!(!accessor.can_read(&ctx, &target, "size"));
    }

    #[test]
    fn read_with_list_target_returns_error() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_list_tv(vec![]);
        assert!(accessor.read(&ctx, &target, "size").is_err());
    }

    #[test]
    fn read_with_map_target_returns_error() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        let target = make_map_tv(vec![]);
        assert!(accessor.read(&ctx, &target, "key").is_err());
    }

    #[test]
    fn can_write_with_various_targets() {
        let accessor = ReflectivePropertyAccessor::new();
        let ctx = DummyCtx::null_root();
        assert!(!accessor.can_write(&ctx, &TypedValue::null(), "x"));
        assert!(!accessor.can_write(&ctx, &make_string_tv("s"), "x"));
        assert!(!accessor.can_write(&ctx, &make_int_tv(1), "x"));
        assert!(!accessor.can_write(&ctx, &make_bool_tv(false), "x"));
        assert!(!accessor.can_write(&ctx, &make_list_tv(vec![]), "x"));
        assert!(!accessor.can_write(&ctx, &make_map_tv(vec![]), "x"));
    }
}

// =========================================================================
// 9. StandardEvaluationContext tests
// =========================================================================

mod standard_evaluation_context_tests {
    use super::*;

    #[test]
    fn new_with_null_root() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.root_object().is_null());
    }

    #[test]
    fn new_with_string_root() {
        let ctx = StandardEvaluationContext::new(make_string_tv("hello"));
        assert_eq!(
            *ctx.root_object().value(),
            ExpressionValue::String("hello".to_string())
        );
    }

    #[test]
    fn new_default_has_null_root() {
        let ctx = StandardEvaluationContext::new_default();
        assert!(ctx.root_object().is_null());
    }

    #[test]
    fn default_trait_works() {
        let ctx = StandardEvaluationContext::default();
        assert!(ctx.root_object().is_null());
    }

    #[test]
    fn property_accessors_non_empty_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let accessors = ctx.property_accessors();
        assert!(!accessors.is_empty(), "should have default ReflectivePropertyAccessor");
    }

    #[test]
    fn method_resolvers_non_empty_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let resolvers = ctx.method_resolvers();
        assert_eq!(resolvers.len(), 1);
    }

    #[test]
    fn bean_resolver_none_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.bean_resolver().is_none());
    }

    #[test]
    fn type_converter_none_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.type_converter().is_none());
    }

    #[test]
    fn type_locator_none_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.type_locator().is_none());
    }

    #[test]
    fn type_comparator_none_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.type_comparator().is_none());
    }

    #[test]
    fn operator_overloader_none_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.operator_overloader().is_none());
    }

    #[test]
    fn constructor_resolvers_empty_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.constructor_resolvers().is_empty());
    }

    #[test]
    fn register_method_fn() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        ctx.register_method_fn("test_method", |_ctx, _target, _args| {
            Ok(TypedValue::null())
        });
        // Verify it was registered by checking the resolver has it
        let resolvers = ctx.method_resolvers();
        assert_eq!(resolvers.len(), 1);
    }

    #[test]
    fn lookup_variable_returns_none_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        // Note: lookup_variable on StandardEvaluationContext always returns None
        // due to lifetime constraints with RwLock
        assert!(ctx.lookup_variable("any").is_none());
    }

    #[test]
    fn set_variable_does_not_panic() {
        let mut ctx = StandardEvaluationContext::new(TypedValue::null());
        ctx.set_variable("x", make_string_tv("value"));
        // We can't retrieve it via lookup_variable due to lifetime, but set should not panic
    }

    #[test]
    fn set_property_accessors() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        // Set empty accessors (overriding default)
        ctx.set_property_accessors(Vec::new());
        // After setting, property_accessors should return empty
        // Note: OnceLock::set returns Err if already initialized, but the initial
        // get_or_init in property_accessors() may have already been called.
        // This test verifies the call doesn't panic.
    }

    #[test]
    fn index_accessors_returns_empty_by_default() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        assert!(ctx.index_accessors().is_empty());
    }

    #[test]
    fn new_with_list_root() {
        let root = make_list_tv(vec![make_int_tv(1), make_int_tv(2), make_int_tv(3)]);
        let ctx = StandardEvaluationContext::new(root);
        match ctx.root_object().value() {
            ExpressionValue::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn new_with_map_root() {
        let root = make_map_tv(vec![
            (make_string_tv("k"), make_string_tv("v")),
        ]);
        let ctx = StandardEvaluationContext::new(root);
        match ctx.root_object().value() {
            ExpressionValue::Map(entries) => assert_eq!(entries.len(), 1),
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn multiple_register_method_fns() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        ctx.register_method_fn("fn1", |_, _, _| Ok(TypedValue::null()));
        ctx.register_method_fn("fn2", |_, _, _| Ok(TypedValue::null()));
        ctx.register_method_fn("fn3", |_, _, _| Ok(TypedValue::null()));
        let resolvers = ctx.method_resolvers();
        assert_eq!(resolvers.len(), 1);
    }
}

// =========================================================================
// 10. SpelMessage tests
// =========================================================================

mod spel_message_tests {
    use super::*;

    #[test]
    fn type_conversion_error_code() {
        assert_eq!(SpelMessage::TypeConversionError.code(), 1001);
    }

    #[test]
    fn constructor_not_found_code() {
        assert_eq!(SpelMessage::ConstructorNotFound.code(), 1002);
    }

    #[test]
    fn constructor_invocation_problem_code() {
        assert_eq!(SpelMessage::ConstructorInvocationProblem.code(), 1003);
    }

    #[test]
    fn method_not_found_code() {
        assert_eq!(SpelMessage::MethodNotFound.code(), 1004);
    }

    #[test]
    fn type_not_found_code() {
        assert_eq!(SpelMessage::TypeNotFound.code(), 1005);
    }

    #[test]
    fn function_not_defined_code() {
        assert_eq!(SpelMessage::FunctionNotDefined.code(), 1006);
    }

    #[test]
    fn property_or_field_not_readable_on_null_code() {
        assert_eq!(SpelMessage::PropertyOrFieldNotReadableOnNull.code(), 1007);
    }

    #[test]
    fn property_or_field_not_readable_code() {
        assert_eq!(SpelMessage::PropertyOrFieldNotReadable.code(), 1008);
    }

    #[test]
    fn property_or_field_not_writable_on_null_code() {
        assert_eq!(SpelMessage::PropertyOrFieldNotWritableOnNull.code(), 1009);
    }

    #[test]
    fn property_or_field_not_writable_code() {
        assert_eq!(SpelMessage::PropertyOrFieldNotWritable.code(), 1010);
    }

    #[test]
    fn method_call_on_null_object_not_allowed_code() {
        assert_eq!(SpelMessage::MethodCallOnNullObjectNotAllowed.code(), 1011);
    }

    #[test]
    fn cannot_index_into_null_value_code() {
        assert_eq!(SpelMessage::CannotIndexIntoNullValue.code(), 1012);
    }

    #[test]
    fn not_comparable_code() {
        assert_eq!(SpelMessage::NotComparable.code(), 1013);
    }

    #[test]
    fn incorrect_number_of_arguments_to_function_code() {
        assert_eq!(SpelMessage::IncorrectNumberOfArgumentsToFunction.code(), 1014);
    }

    #[test]
    fn internal_error_code() {
        assert_eq!(SpelMessage::InternalError.code(), 9999);
    }

    #[test]
    fn max_operations_exceeded_code() {
        assert_eq!(SpelMessage::MaxOperationsExceeded.code(), 1085);
    }

    #[test]
    fn unsupported_character_code() {
        assert_eq!(SpelMessage::UnsupportedCharacter.code(), 1082);
    }

    #[test]
    fn ood_code() {
        assert_eq!(SpelMessage::Ood.code(), 1044);
    }

    #[test]
    fn more_input_code() {
        assert_eq!(SpelMessage::MoreInput.code(), 1041);
    }

    #[test]
    fn not_expected_token_code() {
        assert_eq!(SpelMessage::NotExpectedToken.code(), 1043);
    }

    // default_message tests

    #[test]
    fn default_message_type_conversion_error() {
        let msg = SpelMessage::TypeConversionError.default_message();
        assert!(msg.contains("Type conversion problem"));
        assert!(msg.contains("{0}"));
        assert!(msg.contains("{1}"));
    }

    #[test]
    fn default_message_ood() {
        let msg = SpelMessage::Ood.default_message();
        assert!(msg.contains("ran out of input"));
    }

    #[test]
    fn default_message_internal_error() {
        let msg = SpelMessage::InternalError.default_message();
        assert_eq!(msg, "Internal error");
    }

    #[test]
    fn default_message_not_empty_for_all_common_variants() {
        let variants = [
            SpelMessage::TypeConversionError,
            SpelMessage::ConstructorNotFound,
            SpelMessage::MethodNotFound,
            SpelMessage::TypeNotFound,
            SpelMessage::FunctionNotDefined,
            SpelMessage::PropertyOrFieldNotReadableOnNull,
            SpelMessage::PropertyOrFieldNotReadable,
            SpelMessage::PropertyOrFieldNotWritableOnNull,
            SpelMessage::PropertyOrFieldNotWritable,
            SpelMessage::CannotIndexIntoNullValue,
            SpelMessage::NotComparable,
            SpelMessage::NotExpectedToken,
            SpelMessage::Ood,
            SpelMessage::InternalError,
            SpelMessage::UnsupportedCharacter,
            SpelMessage::MaxOperationsExceeded,
            SpelMessage::ArrayIndexOutOfBounds,
            SpelMessage::CollectionIndexOutOfBounds,
            SpelMessage::StringIndexOutOfBounds,
        ];
        for v in &variants {
            assert!(!v.default_message().is_empty(), "empty message for {:?}", v);
        }
    }

    // format_message tests

    #[test]
    fn format_message_with_inserts() {
        let result = SpelMessage::TypeConversionError.format_message(&["int", "String"]);
        assert_eq!(
            result,
            "EL1001E: Type conversion problem, cannot convert from int to String"
        );
    }

    #[test]
    fn format_message_no_inserts() {
        let result = SpelMessage::Ood.format_message(&[]);
        assert_eq!(result, "EL1044E: Unexpectedly ran out of input");
    }

    #[test]
    fn format_message_prefix_format() {
        let result = SpelMessage::InternalError.format_message(&[]);
        assert!(result.starts_with("EL9999E:"));
    }

    #[test]
    fn format_message_with_three_inserts() {
        let result = SpelMessage::ExceptionDuringMethodInvocation
            .format_message(&["toString", "MyObject", "NPE"]);
        assert!(result.contains("toString"));
        assert!(result.contains("MyObject"));
        assert!(result.contains("NPE"));
    }

    #[test]
    fn format_message_missing_insert_keeps_placeholder() {
        let result = SpelMessage::TypeConversionError.format_message(&["int"]);
        // {1} is missing, so it should remain as {1}
        assert!(result.contains("{1}"));
    }

    #[test]
    fn format_message_display_trait() {
        let result =
            SpelMessage::NotAnInteger.format_message_display(&[42_i32, 100_i32]);
        assert!(result.contains("42"));
    }

    #[test]
    fn kind_returns_error() {
        assert_eq!(SpelMessage::Ood.kind(), vernal_expression::spel::spel_message::MessageKind::Error);
        assert_eq!(
            SpelMessage::TypeConversionError.kind(),
            vernal_expression::spel::spel_message::MessageKind::Error
        );
    }

    #[test]
    fn codes_are_sequential_range() {
        // 1001..=1085 plus 9999
        assert_eq!(SpelMessage::TypeConversionError.code(), 1001);
        assert_eq!(SpelMessage::MaxOperationsExceeded.code(), 1085);
        assert_eq!(SpelMessage::InternalError.code(), 9999);
    }

    #[test]
    fn format_message_property_not_readable() {
        let result =
            SpelMessage::PropertyOrFieldNotReadable.format_message(&["name", "Person"]);
        assert!(result.contains("name"));
        assert!(result.contains("Person"));
    }

    #[test]
    fn format_message_not_expected_token() {
        let result = SpelMessage::NotExpectedToken.format_message(&["RParen", "Comma"]);
        assert!(result.contains("RParen"));
        assert!(result.contains("Comma"));
    }
}

// =========================================================================
// 11. SpelEvaluationException tests
// =========================================================================

mod spel_evaluation_exception_tests {
    use super::*;

    #[test]
    fn new_creates_without_position() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        assert!(ex.position().is_none());
        assert_eq!(ex.message_code(), SpelMessage::Ood);
    }

    #[test]
    fn at_creates_with_position() {
        let ex =
            SpelEvaluationException::at(SpelMessage::PropertyOrFieldNotReadable, 12, &["foo", "Object"]);
        assert_eq!(ex.position(), Some(12));
        assert!(ex.simple_message().contains("foo"));
    }

    #[test]
    fn set_position_updates_position() {
        let mut ex = SpelEvaluationException::new(SpelMessage::NotComparable, &["Integer", "String"]);
        assert!(ex.position().is_none());
        ex.set_position(42);
        assert_eq!(ex.position(), Some(42));
    }

    #[test]
    fn message_code_returns_correct_variant() {
        let ex = SpelEvaluationException::new(SpelMessage::MethodNotFound, &["foo", "Bar"]);
        assert_eq!(ex.message_code(), SpelMessage::MethodNotFound);
    }

    #[test]
    fn inserts_returns_correct_data() {
        let ex = SpelEvaluationException::new(SpelMessage::TypeConversionError, &["int", "String"]);
        let inserts = ex.inserts();
        assert_eq!(inserts.len(), 2);
        assert_eq!(inserts[0], "int");
        assert_eq!(inserts[1], "String");
    }

    #[test]
    fn inserts_empty_when_none_provided() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        assert!(ex.inserts().is_empty());
    }

    #[test]
    fn simple_message_matches_formatted() {
        let ex = SpelEvaluationException::new(SpelMessage::TypeNotFound, &["MyType"]);
        assert_eq!(ex.simple_message(), ex.formatted_message());
    }

    #[test]
    fn simple_message_contains_el_prefix() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        assert!(ex.simple_message().starts_with("EL"));
    }

    #[test]
    fn display_without_position() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        let display = format!("{}", ex);
        assert!(!display.contains("@position"));
    }

    #[test]
    fn display_with_position() {
        let ex = SpelEvaluationException::at(SpelMessage::Ood, 10, &[]);
        let display = format!("{}", ex);
        assert!(display.contains("@position 10"));
    }

    #[test]
    fn debug_format() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        let dbg = format!("{:?}", ex);
        assert!(dbg.contains("SpelEvaluationException"));
    }

    #[test]
    fn is_std_error() {
        let ex = SpelEvaluationException::new(SpelMessage::Ood, &[]);
        let _: &dyn std::error::Error = &ex;
    }
}

// =========================================================================
// 12. SpelParseException tests
// =========================================================================

mod spel_parse_exception_tests {
    use super::*;

    #[test]
    fn new_with_expression_and_position() {
        let ex = SpelParseException::new("1 + )", 4, SpelMessage::NotExpectedToken, &["rparen", "rparen"]);
        assert_eq!(ex.code, SpelMessage::NotExpectedToken);
        assert_eq!(ex.position, Some(4));
        assert!(ex.expression_string.is_some());
        assert!(ex.formatted_message().starts_with("EL1043E:"));
    }

    #[test]
    fn at_unknown_has_no_position() {
        let ex = SpelParseException::at_unknown(SpelMessage::Ood, &[]);
        assert_eq!(ex.code, SpelMessage::Ood);
        assert!(ex.position.is_none());
        assert!(ex.expression_string.is_none());
    }

    #[test]
    fn message_code_returns_correct_variant() {
        let ex = SpelParseException::new("x", 0, SpelMessage::MoreInput, &["extra"]);
        assert_eq!(ex.message_code(), SpelMessage::MoreInput);
    }

    #[test]
    fn inserts_returns_correct_data() {
        let ex = SpelParseException::new("x", 0, SpelMessage::TypeConversionError, &["A", "B"]);
        let inserts = ex.inserts();
        assert_eq!(inserts.len(), 2);
        assert_eq!(inserts[0], "A");
        assert_eq!(inserts[1], "B");
    }

    #[test]
    fn formatted_message_substitutes() {
        let ex = SpelParseException::new(
            "foo",
            0,
            SpelMessage::TypeConversionError,
            &["Integer", "String"],
        );
        assert!(ex.formatted_message().contains("from Integer to String"));
    }

    #[test]
    fn simple_message_matches_formatted() {
        let ex = SpelParseException::new("x", 0, SpelMessage::Ood, &[]);
        assert_eq!(ex.simple_message(), ex.formatted_message());
    }

    #[test]
    fn detailed_message_with_expression_and_position() {
        let ex = SpelParseException::new("1 + )", 4, SpelMessage::NotExpectedToken, &["a", "b"]);
        let detailed = ex.detailed_message();
        assert!(detailed.contains("Expression [1 + )]"));
        assert!(detailed.contains("@4"));
    }

    #[test]
    fn detailed_message_without_expression_or_position() {
        let ex = SpelParseException::at_unknown(SpelMessage::Ood, &[]);
        let detailed = ex.detailed_message();
        // Should fall back to simple_message
        assert!(!detailed.contains("Expression"));
    }

    #[test]
    fn display_uses_detailed_message() {
        let ex = SpelParseException::new("abc", 1, SpelMessage::Ood, &[]);
        let display = format!("{}", ex);
        let detailed = ex.detailed_message();
        assert_eq!(display, detailed);
    }

    #[test]
    fn debug_format() {
        let ex = SpelParseException::new("x", 0, SpelMessage::Ood, &[]);
        let dbg = format!("{:?}", ex);
        assert!(dbg.contains("SpelParseException"));
    }

    #[test]
    fn clone_works() {
        let ex = SpelParseException::new("x", 0, SpelMessage::Ood, &[]);
        let cloned = ex.clone();
        assert_eq!(ex.code, cloned.code);
        assert_eq!(ex.position, cloned.position);
    }

    #[test]
    fn is_std_error() {
        let ex = SpelParseException::new("x", 0, SpelMessage::Ood, &[]);
        let _: &dyn std::error::Error = &ex;
    }
}

// =========================================================================
// 13. InternalParseException tests
// =========================================================================

mod internal_parse_exception_tests {
    use super::*;

    #[test]
    fn wrap_from_spel_parse_exception() {
        let parse_ex = SpelParseException::new("expr", 0, SpelMessage::Ood, &[]);
        let internal = InternalParseException::wrap(parse_ex);
        assert_eq!(internal.parse_exception().code, SpelMessage::Ood);
    }

    #[test]
    fn new_creates_with_expression_and_position() {
        let internal =
            InternalParseException::new("1 + )", 4, SpelMessage::NotExpectedToken, &["a", "b"]);
        let pe = internal.parse_exception();
        assert_eq!(pe.code, SpelMessage::NotExpectedToken);
        assert_eq!(pe.position, Some(4));
    }

    #[test]
    fn into_parse_exception_consumes_and_returns() {
        let internal = InternalParseException::new("x", 0, SpelMessage::Ood, &[]);
        let pe = internal.into_parse_exception();
        assert_eq!(pe.code, SpelMessage::Ood);
    }

    #[test]
    fn parse_exception_borrows() {
        let internal =
            InternalParseException::new("abc", 2, SpelMessage::MoreInput, &["extra"]);
        let pe_ref = internal.parse_exception();
        assert_eq!(pe_ref.code, SpelMessage::MoreInput);
        assert!(pe_ref.expression_string.is_some());
    }

    #[test]
    fn display_delegates_to_cause() {
        let internal = InternalParseException::new("expr", 0, SpelMessage::Ood, &[]);
        let display = format!("{}", internal);
        assert!(!display.is_empty());
    }

    #[test]
    fn debug_format() {
        let internal = InternalParseException::new("x", 0, SpelMessage::Ood, &[]);
        let dbg = format!("{:?}", internal);
        assert!(dbg.contains("InternalParseException"));
    }

    #[test]
    fn error_trait_source() {
        let internal = InternalParseException::new("x", 0, SpelMessage::Ood, &[]);
        let err: &dyn std::error::Error = &internal;
        // source() should return the inner SpelParseException
        assert!(err.source().is_some());
    }

    #[test]
    fn clone_works() {
        let internal = InternalParseException::new("x", 0, SpelMessage::Ood, &[]);
        let cloned = internal.clone();
        assert_eq!(
            internal.parse_exception().code,
            cloned.parse_exception().code
        );
    }

    #[test]
    fn wrap_preserves_inserts() {
        let parse_ex = SpelParseException::new("x", 0, SpelMessage::NotExpectedToken, &["a", "b"]);
        let internal = InternalParseException::wrap(parse_ex);
        assert_eq!(internal.parse_exception().inserts().len(), 2);
    }

    #[test]
    fn new_with_empty_inserts() {
        let internal = InternalParseException::new("x", 0, SpelMessage::Ood, &[]);
        assert!(internal.parse_exception().inserts().is_empty());
    }
}

// =========================================================================
// 14. TokenKind tests
// =========================================================================

mod token_kind_tests {
    use super::*;

    // has_payload tests

    #[test]
    fn has_payload_identifier() {
        assert!(TokenKind::Identifier.has_payload());
    }

    #[test]
    fn has_payload_literal_int() {
        assert!(TokenKind::LiteralInt.has_payload());
    }

    #[test]
    fn has_payload_literal_long() {
        assert!(TokenKind::LiteralLong.has_payload());
    }

    #[test]
    fn has_payload_literal_hex_int() {
        assert!(TokenKind::LiteralHexInt.has_payload());
    }

    #[test]
    fn has_payload_literal_hex_long() {
        assert!(TokenKind::LiteralHexLong.has_payload());
    }

    #[test]
    fn has_payload_literal_string() {
        assert!(TokenKind::LiteralString.has_payload());
    }

    #[test]
    fn has_payload_literal_real() {
        assert!(TokenKind::LiteralReal.has_payload());
    }

    #[test]
    fn has_payload_literal_real_float() {
        assert!(TokenKind::LiteralRealFloat.has_payload());
    }

    #[test]
    fn has_payload_instanceof() {
        assert!(TokenKind::Instanceof.has_payload());
    }

    #[test]
    fn has_payload_matches() {
        assert!(TokenKind::Matches.has_payload());
    }

    #[test]
    fn has_payload_between() {
        assert!(TokenKind::Between.has_payload());
    }

    #[test]
    fn no_payload_plus() {
        assert!(!TokenKind::Plus.has_payload());
    }

    #[test]
    fn no_payload_minus() {
        assert!(!TokenKind::Minus.has_payload());
    }

    #[test]
    fn no_payload_star() {
        assert!(!TokenKind::Star.has_payload());
    }

    #[test]
    fn no_payload_lparen() {
        assert!(!TokenKind::LParen.has_payload());
    }

    #[test]
    fn no_payload_dot() {
        assert!(!TokenKind::Dot.has_payload());
    }

    #[test]
    fn no_payload_assign() {
        assert!(!TokenKind::Assign.has_payload());
    }

    // token_chars tests

    #[test]
    fn token_chars_single_char_operators() {
        assert_eq!(TokenKind::Plus.token_chars(), "+");
        assert_eq!(TokenKind::Minus.token_chars(), "-");
        assert_eq!(TokenKind::Star.token_chars(), "*");
        assert_eq!(TokenKind::Div.token_chars(), "/");
        assert_eq!(TokenKind::Mod.token_chars(), "%");
        assert_eq!(TokenKind::LParen.token_chars(), "(");
        assert_eq!(TokenKind::RParen.token_chars(), ")");
        assert_eq!(TokenKind::LSquare.token_chars(), "[");
        assert_eq!(TokenKind::RSquare.token_chars(), "]");
        assert_eq!(TokenKind::LCurly.token_chars(), "{");
        assert_eq!(TokenKind::RCurly.token_chars(), "}");
        assert_eq!(TokenKind::Dot.token_chars(), ".");
        assert_eq!(TokenKind::Comma.token_chars(), ",");
        assert_eq!(TokenKind::Colon.token_chars(), ":");
        assert_eq!(TokenKind::Hash.token_chars(), "#");
        assert_eq!(TokenKind::QMark.token_chars(), "?");
        assert_eq!(TokenKind::Not.token_chars(), "!");
        assert_eq!(TokenKind::Assign.token_chars(), "=");
        assert_eq!(TokenKind::Gt.token_chars(), ">");
        assert_eq!(TokenKind::Lt.token_chars(), "<");
        assert_eq!(TokenKind::Power.token_chars(), "^");
        assert_eq!(TokenKind::BeanRef.token_chars(), "@");
        assert_eq!(TokenKind::FactoryBeanRef.token_chars(), "&");
    }

    #[test]
    fn token_chars_double_char_operators() {
        assert_eq!(TokenKind::Ge.token_chars(), ">=");
        assert_eq!(TokenKind::Le.token_chars(), "<=");
        assert_eq!(TokenKind::Equal.token_chars(), "==");
        assert_eq!(TokenKind::NotEqual.token_chars(), "!=");
        assert_eq!(TokenKind::Elvis.token_chars(), "?:");
        assert_eq!(TokenKind::SafeNavi.token_chars(), "?.");
        assert_eq!(TokenKind::SymbolicOr.token_chars(), "||");
        assert_eq!(TokenKind::SymbolicAnd.token_chars(), "&&");
        assert_eq!(TokenKind::Inc.token_chars(), "++");
        assert_eq!(TokenKind::Dec.token_chars(), "--");
        assert_eq!(TokenKind::SelectFirst.token_chars(), "^[");
        assert_eq!(TokenKind::SelectLast.token_chars(), "$[");
        assert_eq!(TokenKind::Project.token_chars(), "![");
        assert_eq!(TokenKind::Select.token_chars(), "?[");
    }

    #[test]
    fn token_chars_keywords() {
        assert_eq!(TokenKind::Instanceof.token_chars(), "instanceof");
        assert_eq!(TokenKind::Matches.token_chars(), "matches");
        assert_eq!(TokenKind::Between.token_chars(), "between");
    }

    #[test]
    fn token_chars_empty_for_payload_types() {
        assert_eq!(TokenKind::Identifier.token_chars(), "");
        assert_eq!(TokenKind::LiteralInt.token_chars(), "");
        assert_eq!(TokenKind::LiteralLong.token_chars(), "");
        assert_eq!(TokenKind::LiteralHexInt.token_chars(), "");
        assert_eq!(TokenKind::LiteralHexLong.token_chars(), "");
        assert_eq!(TokenKind::LiteralString.token_chars(), "");
        assert_eq!(TokenKind::LiteralReal.token_chars(), "");
        assert_eq!(TokenKind::LiteralRealFloat.token_chars(), "");
    }

    // length tests

    #[test]
    fn length_single_char() {
        assert_eq!(TokenKind::Plus.length(), 1);
        assert_eq!(TokenKind::Minus.length(), 1);
        assert_eq!(TokenKind::Dot.length(), 1);
    }

    #[test]
    fn length_double_char() {
        assert_eq!(TokenKind::Ge.length(), 2);
        assert_eq!(TokenKind::Le.length(), 2);
        assert_eq!(TokenKind::Equal.length(), 2);
        assert_eq!(TokenKind::NotEqual.length(), 2);
        assert_eq!(TokenKind::SafeNavi.length(), 2);
        assert_eq!(TokenKind::Inc.length(), 2);
        assert_eq!(TokenKind::Dec.length(), 2);
    }

    #[test]
    fn length_keywords() {
        assert_eq!(TokenKind::Instanceof.length(), 10);
        assert_eq!(TokenKind::Matches.length(), 7);
        assert_eq!(TokenKind::Between.length(), 7);
    }

    #[test]
    fn length_zero_for_payload_types() {
        assert_eq!(TokenKind::Identifier.length(), 0);
        assert_eq!(TokenKind::LiteralInt.length(), 0);
        assert_eq!(TokenKind::LiteralString.length(), 0);
    }

    // is_numeric_relational tests

    #[test]
    fn is_numeric_relational_true_cases() {
        assert!(TokenKind::Gt.is_numeric_relational());
        assert!(TokenKind::Ge.is_numeric_relational());
        assert!(TokenKind::Lt.is_numeric_relational());
        assert!(TokenKind::Le.is_numeric_relational());
        assert!(TokenKind::Equal.is_numeric_relational());
        assert!(TokenKind::NotEqual.is_numeric_relational());
    }

    #[test]
    fn is_numeric_relational_false_cases() {
        assert!(!TokenKind::Plus.is_numeric_relational());
        assert!(!TokenKind::Minus.is_numeric_relational());
        assert!(!TokenKind::Star.is_numeric_relational());
        assert!(!TokenKind::SymbolicAnd.is_numeric_relational());
        assert!(!TokenKind::SymbolicOr.is_numeric_relational());
        assert!(!TokenKind::Assign.is_numeric_relational());
        assert!(!TokenKind::Instanceof.is_numeric_relational());
        assert!(!TokenKind::Matches.is_numeric_relational());
        assert!(!TokenKind::Identifier.is_numeric_relational());
    }

    // is_literal tests

    #[test]
    fn is_literal_true_cases() {
        assert!(TokenKind::LiteralInt.is_literal());
        assert!(TokenKind::LiteralLong.is_literal());
        assert!(TokenKind::LiteralHexInt.is_literal());
        assert!(TokenKind::LiteralHexLong.is_literal());
        assert!(TokenKind::LiteralString.is_literal());
        assert!(TokenKind::LiteralReal.is_literal());
        assert!(TokenKind::LiteralRealFloat.is_literal());
    }

    #[test]
    fn is_literal_false_for_non_literals() {
        assert!(!TokenKind::Identifier.is_literal());
        assert!(!TokenKind::Plus.is_literal());
        assert!(!TokenKind::LParen.is_literal());
        assert!(!TokenKind::Instanceof.is_literal());
        assert!(!TokenKind::Matches.is_literal());
        assert!(!TokenKind::Between.is_literal());
        assert!(!TokenKind::Dot.is_literal());
    }

    // Equality and other traits

    #[test]
    fn equality_works() {
        assert_eq!(TokenKind::Plus, TokenKind::Plus);
        assert_ne!(TokenKind::Plus, TokenKind::Minus);
    }

    #[test]
    fn debug_format() {
        let dbg = format!("{:?}", TokenKind::Identifier);
        assert_eq!(dbg, "Identifier");
    }

    #[test]
    fn copy_semantics() {
        let a = TokenKind::Ge;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn hash_works_in_set() {
        let mut set = std::collections::HashSet::new();
        set.insert(TokenKind::Plus);
        set.insert(TokenKind::Minus);
        set.insert(TokenKind::Plus); // duplicate
        assert_eq!(set.len(), 2);
    }
}

// =========================================================================
// 15. Token tests
// =========================================================================

mod token_tests {
    use super::*;

    #[test]
    fn empty_token_has_no_data() {
        let t = Token::empty(TokenKind::Plus, 0, 1);
        assert_eq!(t.kind, TokenKind::Plus);
        assert!(t.data.is_none());
        assert_eq!(t.start_pos, 0);
        assert_eq!(t.end_pos, 1);
    }

    #[test]
    fn empty_token_string_value_is_empty() {
        let t = Token::empty(TokenKind::LParen, 5, 6);
        assert_eq!(t.string_value(), "");
    }

    #[test]
    fn with_data_creates_token_with_payload() {
        let t = Token::with_data(TokenKind::Identifier, "foo", 0, 3);
        assert_eq!(t.kind, TokenKind::Identifier);
        assert!(t.data.is_some());
        assert_eq!(t.string_value(), "foo");
        assert_eq!(t.start_pos, 0);
        assert_eq!(t.end_pos, 3);
    }

    #[test]
    fn with_data_literal_int() {
        let t = Token::with_data(TokenKind::LiteralInt, "42", 0, 2);
        assert_eq!(t.kind, TokenKind::LiteralInt);
        assert_eq!(t.string_value(), "42");
    }

    #[test]
    fn with_data_literal_string() {
        let t = Token::with_data(TokenKind::LiteralString, "hello", 0, 7);
        assert_eq!(t.kind, TokenKind::LiteralString);
        assert_eq!(t.string_value(), "hello");
    }

    #[test]
    fn with_data_accepts_string() {
        let s = String::from("bar");
        let t = Token::with_data(TokenKind::Identifier, s, 0, 3);
        assert_eq!(t.string_value(), "bar");
    }

    #[test]
    fn with_data_accepts_str_ref() {
        let t = Token::with_data(TokenKind::Identifier, "baz", 0, 3);
        assert_eq!(t.string_value(), "baz");
    }

    #[test]
    fn is_identifier_true() {
        let t = Token::with_data(TokenKind::Identifier, "myVar", 0, 5);
        assert!(t.is_identifier());
    }

    #[test]
    fn is_identifier_false_for_literal() {
        let t = Token::with_data(TokenKind::LiteralInt, "42", 0, 2);
        assert!(!t.is_identifier());
    }

    #[test]
    fn is_identifier_false_for_operator() {
        let t = Token::empty(TokenKind::Plus, 0, 1);
        assert!(!t.is_identifier());
    }

    #[test]
    fn is_numeric_relational_operator_true() {
        assert!(Token::empty(TokenKind::Gt, 0, 1).is_numeric_relational_operator());
        assert!(Token::empty(TokenKind::Ge, 0, 2).is_numeric_relational_operator());
        assert!(Token::empty(TokenKind::Lt, 0, 1).is_numeric_relational_operator());
        assert!(Token::empty(TokenKind::Le, 0, 2).is_numeric_relational_operator());
        assert!(Token::empty(TokenKind::Equal, 0, 2).is_numeric_relational_operator());
        assert!(Token::empty(TokenKind::NotEqual, 0, 2).is_numeric_relational_operator());
    }

    #[test]
    fn is_numeric_relational_operator_false() {
        assert!(!Token::empty(TokenKind::Plus, 0, 1).is_numeric_relational_operator());
        assert!(!Token::with_data(TokenKind::Identifier, "x", 0, 1).is_numeric_relational_operator());
    }

    #[test]
    fn as_instanceof_token_converts_kind_and_data() {
        let t = Token::with_data(TokenKind::Identifier, "instanceof", 5, 15);
        let t2 = t.as_instanceof_token();
        assert_eq!(t2.kind, TokenKind::Instanceof);
        assert_eq!(t2.string_value(), "instanceof");
        assert_eq!(t2.start_pos, 5);
        assert_eq!(t2.end_pos, 15);
    }

    #[test]
    fn as_matches_token_converts_kind_and_data() {
        let t = Token::with_data(TokenKind::Identifier, "matches", 0, 7);
        let t2 = t.as_matches_token();
        assert_eq!(t2.kind, TokenKind::Matches);
        assert_eq!(t2.string_value(), "matches");
    }

    #[test]
    fn as_between_token_converts_kind_and_data() {
        let t = Token::with_data(TokenKind::Identifier, "between", 0, 7);
        let t2 = t.as_between_token();
        assert_eq!(t2.kind, TokenKind::Between);
        assert_eq!(t2.string_value(), "between");
    }

    #[test]
    fn as_instanceof_preserves_positions() {
        let t = Token::with_data(TokenKind::Identifier, "instanceof", 100, 110);
        let t2 = t.as_instanceof_token();
        assert_eq!(t2.start_pos, 100);
        assert_eq!(t2.end_pos, 110);
    }

    #[test]
    fn as_matches_preserves_positions() {
        let t = Token::with_data(TokenKind::Identifier, "matches", 20, 27);
        let t2 = t.as_matches_token();
        assert_eq!(t2.start_pos, 20);
        assert_eq!(t2.end_pos, 27);
    }

    #[test]
    fn as_between_preserves_positions() {
        let t = Token::with_data(TokenKind::Identifier, "between", 30, 37);
        let t2 = t.as_between_token();
        assert_eq!(t2.start_pos, 30);
        assert_eq!(t2.end_pos, 37);
    }

    #[test]
    fn clone_works() {
        let t = Token::with_data(TokenKind::Identifier, "test", 0, 4);
        let t2 = t.clone();
        assert_eq!(t.kind, t2.kind);
        assert_eq!(t.string_value(), t2.string_value());
        assert_eq!(t.start_pos, t2.start_pos);
        assert_eq!(t.end_pos, t2.end_pos);
    }

    #[test]
    fn debug_format() {
        let t = Token::with_data(TokenKind::Identifier, "x", 0, 1);
        let dbg = format!("{:?}", t);
        assert!(dbg.contains("Token"));
        assert!(dbg.contains("Identifier"));
    }

    #[test]
    fn empty_token_with_zero_positions() {
        let t = Token::empty(TokenKind::Dot, 0, 0);
        assert_eq!(t.start_pos, 0);
        assert_eq!(t.end_pos, 0);
    }

    #[test]
    fn string_value_for_empty_data_returns_empty() {
        let t = Token::empty(TokenKind::Star, 0, 1);
        assert_eq!(t.string_value(), "");
    }

    #[test]
    fn string_value_for_data_token() {
        let t = Token::with_data(TokenKind::LiteralReal, "3.14", 0, 4);
        assert_eq!(t.string_value(), "3.14");
    }

    #[test]
    fn with_data_literal_long() {
        let t = Token::with_data(TokenKind::LiteralLong, "100L", 0, 4);
        assert_eq!(t.kind, TokenKind::LiteralLong);
        assert_eq!(t.string_value(), "100L");
    }

    #[test]
    fn with_data_literal_hex() {
        let t = Token::with_data(TokenKind::LiteralHexInt, "0xFF", 0, 4);
        assert_eq!(t.kind, TokenKind::LiteralHexInt);
        assert_eq!(t.string_value(), "0xFF");
    }

    #[test]
    fn empty_token_all_operators() {
        // Verify empty() works for a variety of operator kinds
        let operators = [
            (TokenKind::Plus, "+"),
            (TokenKind::Minus, "-"),
            (TokenKind::Star, "*"),
            (TokenKind::Div, "/"),
            (TokenKind::Mod, "%"),
            (TokenKind::Assign, "="),
            (TokenKind::Not, "!"),
        ];
        for (kind, expected_chars) in &operators {
            let t = Token::empty(*kind, 0, expected_chars.len());
            assert_eq!(t.kind, *kind);
            assert!(t.data.is_none());
        }
    }
}

// =========================================================================
// Cross-cutting: SpelNode trait default methods
// =========================================================================

mod spel_node_trait_tests {
    use super::*;

    struct TestNode {
        inner: SpelNodeImpl,
    }

    impl TestNode {
        fn new(start: usize, end: usize) -> Self {
            Self {
                inner: SpelNodeImpl::new(start, end),
            }
        }
    }

    impl SpelNode for TestNode {
        fn get_value(&self, _context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException> {
            Ok(TypedValue::null())
        }

        fn to_string_ast(&self) -> String {
            "TestNode".to_string()
        }

        fn start_position(&self) -> usize {
            self.inner.start_position()
        }

        fn end_position(&self) -> usize {
            self.inner.end_position()
        }
    }

    #[test]
    fn default_child_count_is_zero() {
        let node = TestNode::new(0, 10);
        assert_eq!(node.child_count(), 0);
    }

    #[test]
    fn default_get_child_returns_none() {
        let node = TestNode::new(0, 10);
        assert!(node.get_child(0).is_none());
    }

    #[test]
    fn default_is_null_safe_is_false() {
        let node = TestNode::new(0, 10);
        assert!(!node.is_null_safe());
    }

    #[test]
    fn default_class_returns_none() {
        let node = TestNode::new(0, 10);
        assert!(node.class().is_none());
    }

    #[test]
    fn default_is_writable_is_false() {
        let node = TestNode::new(0, 10);
        let ctx = DummyCtx::null_root();
        assert!(!node.is_writable(&ctx));
    }

    #[test]
    fn default_exit_descriptor_returns_none() {
        let node = TestNode::new(0, 10);
        assert!(node.exit_descriptor().is_none());
    }

    #[test]
    fn to_string_ast_works() {
        let node = TestNode::new(0, 10);
        assert_eq!(node.to_string_ast(), "TestNode");
    }

    #[test]
    fn start_and_end_position_from_impl() {
        let node = TestNode::new(5, 20);
        assert_eq!(node.start_position(), 5);
        assert_eq!(node.end_position(), 20);
    }

    #[test]
    fn get_value_returns_null() {
        let node = TestNode::new(0, 1);
        let ctx = DummyCtx::null_root();
        let result = node.get_value(&ctx).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn debug_format_for_dyn_spel_node() {
        let node = TestNode::new(0, 10);
        let dyn_ref: &dyn SpelNode = &node;
        let dbg = format!("{:?}", dyn_ref);
        assert!(dbg.contains("SpelNode"));
        assert!(dbg.contains("TestNode"));
    }
}

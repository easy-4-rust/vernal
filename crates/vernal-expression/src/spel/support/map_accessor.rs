//! Map 键访问器。
//!
//! 对标 Spring 的 `MapAccessor`：通过键访问 Map 值。

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::property_accessor::PropertyAccessor;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// Map 键访问器。
///
/// 对标 Spring 的 `org.springframework.expression.spel.support.MapAccessor`。
pub struct MapAccessor;

impl PropertyAccessor for MapAccessor {
    fn can_read(&self, _context: &dyn EvaluationContext, target: &TypedValue, _name: &str) -> bool {
        matches!(target.value(), ExpressionValue::Map(_))
    }

    fn read(
        &self,
        _context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
    ) -> Result<TypedValue, AccessException> {
        if let ExpressionValue::Map(pairs) = target.value() {
            let key = TypedValue::new(
                ExpressionValue::String(name.to_string()),
                TypeDescriptor::STRING,
            );
            for (k, v) in pairs {
                if k.value() == key.value() {
                    return Ok(v.clone());
                }
            }
            Ok(TypedValue::null())
        } else {
            Err(AccessException::new("目标不是 Map"))
        }
    }

    fn can_write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
    ) -> bool {
        false
    }

    fn write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _name: &str,
        _value: &TypedValue,
    ) -> Result<(), AccessException> {
        Err(AccessException::new("MapAccessor 不支持写入"))
    }

    fn specific_target_classes(&self) -> &[&str] {
        &["HashMap", "BTreeMap"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;

    fn ctx() -> StandardEvaluationContext {
        StandardEvaluationContext::new(TypedValue::null())
    }

    fn make_map(pairs: Vec<(&str, i64)>) -> TypedValue {
        let entries: Vec<(TypedValue, TypedValue)> = pairs
            .into_iter()
            .map(|(k, v)| {
                (
                    TypedValue::new(ExpressionValue::String(k.into()), TypeDescriptor::STRING),
                    TypedValue::new(ExpressionValue::Int(v), TypeDescriptor::INT),
                )
            })
            .collect();
        TypedValue::new(ExpressionValue::Map(entries), TypeDescriptor::from_type_name("Map"))
    }

    #[test]
    fn can_read_map_target() {
        let accessor = MapAccessor;
        let target = make_map(vec![("a", 1)]);
        assert!(accessor.can_read(&ctx(), &target, "a"));
    }

    #[test]
    fn cannot_read_non_map_target() {
        let accessor = MapAccessor;
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        assert!(!accessor.can_read(&ctx(), &target, "a"));
    }

    #[test]
    fn read_existing_key() {
        let accessor = MapAccessor;
        let target = make_map(vec![("name", 42), ("age", 100)]);
        let result = accessor.read(&ctx(), &target, "name").unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(42));
    }

    #[test]
    fn read_missing_key_returns_null() {
        let accessor = MapAccessor;
        let target = make_map(vec![("a", 1)]);
        let result = accessor.read(&ctx(), &target, "missing").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_empty_map_returns_null() {
        let accessor = MapAccessor;
        let target = make_map(vec![]);
        let result = accessor.read(&ctx(), &target, "key").unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_non_map_returns_error() {
        let accessor = MapAccessor;
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = accessor.read(&ctx(), &target, "key");
        assert!(result.is_err());
    }

    #[test]
    fn can_write_always_false() {
        let accessor = MapAccessor;
        let target = make_map(vec![("a", 1)]);
        assert!(!accessor.can_write(&ctx(), &target, "a"));
    }

    #[test]
    fn write_always_returns_error() {
        let accessor = MapAccessor;
        let target = make_map(vec![("a", 1)]);
        let value = TypedValue::new(ExpressionValue::Int(99), TypeDescriptor::INT);
        let result = accessor.write(&ctx(), &target, "a", &value);
        assert!(result.is_err());
    }

    #[test]
    fn specific_target_classes() {
        let accessor = MapAccessor;
        let classes = accessor.specific_target_classes();
        assert_eq!(classes.len(), 2);
        assert!(classes.contains(&"HashMap"));
        assert!(classes.contains(&"BTreeMap"));
    }

    #[test]
    fn read_multiple_keys() {
        let accessor = MapAccessor;
        let target = make_map(vec![("x", 10), ("y", 20), ("z", 30)]);

        let rx = accessor.read(&ctx(), &target, "x").unwrap();
        assert_eq!(*rx.value(), ExpressionValue::Int(10));

        let ry = accessor.read(&ctx(), &target, "y").unwrap();
        assert_eq!(*ry.value(), ExpressionValue::Int(20));

        let rz = accessor.read(&ctx(), &target, "z").unwrap();
        assert_eq!(*rz.value(), ExpressionValue::Int(30));
    }
}

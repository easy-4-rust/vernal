//! 反射索引访问器。
//!
//! 对标 Spring 的 `ReflectiveIndexAccessor`：通过反射访问数组/列表/Map。

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::property_accessor::IndexAccessor;
use crate::typed_value::{ExpressionValue, TypedValue};

/// 反射索引访问器。
///
/// 对标 Spring 的 `ReflectiveIndexAccessor`。
pub struct ReflectiveIndexAccessor;

impl IndexAccessor for ReflectiveIndexAccessor {
    fn can_read(
        &self,
        _context: &dyn EvaluationContext,
        target: &TypedValue,
        _index: &TypedValue,
    ) -> bool {
        matches!(
            target.value(),
            ExpressionValue::List(_) | ExpressionValue::Map(_)
        )
    }

    fn read(
        &self,
        _context: &dyn EvaluationContext,
        target: &TypedValue,
        index: &TypedValue,
    ) -> Result<TypedValue, AccessException> {
        match (target.value(), index.value()) {
            (ExpressionValue::List(items), ExpressionValue::Int(i)) => {
                let idx = *i as usize;
                if idx < items.len() {
                    Ok(items[idx].clone())
                } else {
                    Err(AccessException::new(format!("索引 {} 超出列表长度", idx)))
                }
            }
            (ExpressionValue::Map(entries), key) => {
                for (k, v) in entries {
                    if k.value() == key {
                        return Ok(v.clone());
                    }
                }
                Ok(TypedValue::null())
            }
            _ => Err(AccessException::new("反射索引访问器不支持该操作数类型")),
        }
    }

    fn can_write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _index: &TypedValue,
    ) -> bool {
        false
    }

    fn write(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        _index: &TypedValue,
        _value: &TypedValue,
    ) -> Result<(), AccessException> {
        Err(AccessException::new("反射索引访问器不支持写入"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::TypeDescriptor;

    fn ctx() -> StandardEvaluationContext {
        StandardEvaluationContext::new(TypedValue::null())
    }

    #[test]
    fn can_read_list() {
        let accessor = ReflectiveIndexAccessor;
        let target = TypedValue::new(
            ExpressionValue::List(vec![TypedValue::new(
                ExpressionValue::Int(1),
                TypeDescriptor::INT,
            )]),
            TypeDescriptor::from_type_name("List"),
        );
        let index = TypedValue::new(ExpressionValue::Int(0), TypeDescriptor::INT);
        assert!(accessor.can_read(&ctx(), &target, &index));
    }

    #[test]
    fn can_read_map() {
        let accessor = ReflectiveIndexAccessor;
        let target = TypedValue::new(
            ExpressionValue::Map(vec![(
                TypedValue::new(ExpressionValue::String("k".into()), TypeDescriptor::STRING),
                TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
            )]),
            TypeDescriptor::from_type_name("Map"),
        );
        let index = TypedValue::new(ExpressionValue::String("k".into()), TypeDescriptor::STRING);
        assert!(accessor.can_read(&ctx(), &target, &index));
    }

    #[test]
    fn cannot_read_int_target() {
        let accessor = ReflectiveIndexAccessor;
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let index = TypedValue::new(ExpressionValue::Int(0), TypeDescriptor::INT);
        assert!(!accessor.can_read(&ctx(), &target, &index));
    }

    #[test]
    fn read_list_by_index() {
        let accessor = ReflectiveIndexAccessor;
        let items = vec![
            TypedValue::new(ExpressionValue::Int(10), TypeDescriptor::INT),
            TypedValue::new(ExpressionValue::Int(20), TypeDescriptor::INT),
            TypedValue::new(ExpressionValue::Int(30), TypeDescriptor::INT),
        ];
        let target = TypedValue::new(
            ExpressionValue::List(items),
            TypeDescriptor::from_type_name("List"),
        );

        let index = TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT);
        let result = accessor.read(&ctx(), &target, &index).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(20));
    }

    #[test]
    fn read_list_out_of_bounds() {
        let accessor = ReflectiveIndexAccessor;
        let items = vec![TypedValue::new(
            ExpressionValue::Int(10),
            TypeDescriptor::INT,
        )];
        let target = TypedValue::new(
            ExpressionValue::List(items),
            TypeDescriptor::from_type_name("List"),
        );

        let index = TypedValue::new(ExpressionValue::Int(5), TypeDescriptor::INT);
        let result = accessor.read(&ctx(), &target, &index);
        assert!(result.is_err());
    }

    #[test]
    fn read_map_by_key() {
        let accessor = ReflectiveIndexAccessor;
        let entries = vec![
            (
                TypedValue::new(ExpressionValue::String("a".into()), TypeDescriptor::STRING),
                TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
            ),
            (
                TypedValue::new(ExpressionValue::String("b".into()), TypeDescriptor::STRING),
                TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT),
            ),
        ];
        let target = TypedValue::new(
            ExpressionValue::Map(entries),
            TypeDescriptor::from_type_name("Map"),
        );

        let index = TypedValue::new(ExpressionValue::String("b".into()), TypeDescriptor::STRING);
        let result = accessor.read(&ctx(), &target, &index).unwrap();
        assert_eq!(*result.value(), ExpressionValue::Int(2));
    }

    #[test]
    fn read_map_missing_key_returns_null() {
        let accessor = ReflectiveIndexAccessor;
        let entries = vec![(
            TypedValue::new(ExpressionValue::String("a".into()), TypeDescriptor::STRING),
            TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
        )];
        let target = TypedValue::new(
            ExpressionValue::Map(entries),
            TypeDescriptor::from_type_name("Map"),
        );

        let index = TypedValue::new(
            ExpressionValue::String("missing".into()),
            TypeDescriptor::STRING,
        );
        let result = accessor.read(&ctx(), &target, &index).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn read_unsupported_type_returns_error() {
        let accessor = ReflectiveIndexAccessor;
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let index = TypedValue::new(ExpressionValue::Int(0), TypeDescriptor::INT);
        let result = accessor.read(&ctx(), &target, &index);
        assert!(result.is_err());
    }

    #[test]
    fn can_write_always_false() {
        let accessor = ReflectiveIndexAccessor;
        let target = TypedValue::new(
            ExpressionValue::List(vec![]),
            TypeDescriptor::from_type_name("List"),
        );
        let index = TypedValue::new(ExpressionValue::Int(0), TypeDescriptor::INT);
        assert!(!accessor.can_write(&ctx(), &target, &index));
    }

    #[test]
    fn write_always_returns_error() {
        let accessor = ReflectiveIndexAccessor;
        let target = TypedValue::new(
            ExpressionValue::List(vec![]),
            TypeDescriptor::from_type_name("List"),
        );
        let index = TypedValue::new(ExpressionValue::Int(0), TypeDescriptor::INT);
        let value = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = accessor.write(&ctx(), &target, &index, &value);
        assert!(result.is_err());
    }
}

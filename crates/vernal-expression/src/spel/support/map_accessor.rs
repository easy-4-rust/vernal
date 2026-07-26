//! Map 键访问器。
//!
//! 对标 Spring 的 `MapAccessor`：通过键访问 Map 值。

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::property_accessor::PropertyAccessor;
use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};
use std::collections::HashMap;

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

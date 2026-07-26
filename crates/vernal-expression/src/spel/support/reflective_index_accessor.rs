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

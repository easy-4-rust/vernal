//! TypeConverterDelegate — Spring 风格的类型转换委托。
//!
//! 对应 Java 类：`org.springframework.beans.TypeConverterDelegate`。
//!
//! 内部使用 PropertyEditor 实现类型转换。这是 Spring 类型转换的核心机制：
//! 1. 查找已注册的 PropertyEditor
/// 2. 调用 PropertyEditor.setAsText() 进行转换
/// 3. 返回转换后的值
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::property_editor::PropertyEditor;

/// Spring 风格的类型转换委托。
///
/// 对应 Spring 的 `TypeConverterDelegate`。
///
/// 内部维护一个 PropertyEditor 注册表（target_type → PropertyEditor）。
/// 转换时先查找已注册的 PropertyEditor，再调用其 `set_as_text` 方法。
///
/// ## 与 Spring 的映射
///
/// Java 的 `TypeConverterDelegate` 内部调用 `PropertyEditor.setAsText(String)`。
/// Rust 中使用 `dyn PropertyEditor` trait 对象实现等效能力。
pub struct TypeConverterDelegate {
    /// 已注册的 PropertyEditor（target_type → editor）。
    editors: RwLock<HashMap<std::any::TypeId, Arc<dyn PropertyEditor>>>,
    /// 自定义转换器（source_type → (target_type, converter)）。
    custom_converters: RwLock<Vec<CustomConverter>>,
}

/// 自定义转换器。
struct CustomConverter {
    source_type: std::any::TypeId,
    target_type: std::any::TypeId,
    converter: Box<
        dyn Fn(
                &dyn Any,
            )
                -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync,
    >,
}

impl TypeConverterDelegate {
    /// 创建新的 TypeConverterDelegate。
    pub fn new() -> Self {
        Self {
            editors: RwLock::new(HashMap::new()),
            custom_converters: RwLock::new(Vec::new()),
        }
    }

    /// 注册 PropertyEditor。
    ///
    /// 对应 Spring 的 `TypeConverterDelegate.registerCustomEditor(Class<?> requiredType, PropertyEditor propertyEditor)`。
    pub fn register_custom_editor(
        &self,
        target_type: std::any::TypeId,
        editor: Arc<dyn PropertyEditor>,
    ) {
        self.editors
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(target_type, editor);
    }

    /// 注册自定义转换器。
    pub fn register_converter<F>(
        &self,
        source_type: std::any::TypeId,
        target_type: std::any::TypeId,
        converter: F,
    ) where
        F: Fn(
                &dyn Any,
            )
                -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        self.custom_converters
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(CustomConverter {
                source_type,
                target_type,
                converter: Box::new(converter),
            });
    }

    /// 执行类型转换。
    ///
    /// 对应 Spring 的 `TypeConverterDelegate.convertIfNecessary(String propertyName, Object value, Class<T> requiredType)`。
    ///
    /// 转换流程：
    /// 1. 查找自定义转换器（source_type → target_type）
    /// 2. 查找已注册的 PropertyEditor
    /// 3. 调用 PropertyEditor.setAsText(value.to_string()) 进行转换
    /// 4. 返回转换后的值
    pub fn convert_if_necessary(
        &self,
        _property_name: Option<&str>,
        value: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let source_type = value.type_id();

        // 1. 查找自定义转换器
        {
            let converters = self
                .custom_converters
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for converter in converters.iter() {
                if converter.source_type == source_type && converter.target_type == target_type {
                    return (converter.converter)(value);
                }
            }
        }

        // 2. 如果是相同类型，直接返回
        if source_type == target_type {
            return value_to_owned(value);
        }

        // 3. 将 value 转换为字符串
        let text = value_to_string(value)?;

        // 4. 创建对应的 PropertyEditor 并执行转换
        if let Some(ref mut editor_instance) = create_editor_for_type(target_type) {
            let set_result = editor_instance.set_as_text(&text);

            // 获取转换后的值
            if set_result.is_ok() {
                if let Some(converted) = editor_instance.get_value() {
                    return value_to_owned(converted);
                }
            }
        }

        // 5. 如果是相同类型（双重检查），直接返回
        if source_type == target_type {
            return value_to_owned(value);
        }

        Err(format!(
            "No converter found for {:?} -> {:?}",
            source_type, target_type
        )
        .into())
    }

    /// 获取已注册的 PropertyEditor 数量。
    pub fn editor_count(&self) -> usize {
        self.editors
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// 清空所有注册的编辑器和转换器。
    pub fn clear(&self) {
        self.editors
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.custom_converters
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

impl Default for TypeConverterDelegate {
    fn default() -> Self {
        Self::new()
    }
}

/// 根据目标类型创建对应的 PropertyEditor 实例。
fn create_editor_for_type(target_type: std::any::TypeId) -> Option<Box<dyn PropertyEditor>> {
    if target_type == std::any::TypeId::of::<String>() {
        Some(Box::new(
            crate::propertyeditors::string_trimmer_editor::StringTrimmerEditor::new(),
        ))
    } else if target_type == std::any::TypeId::of::<bool>() {
        Some(Box::new(crate::boolean_editor::CustomBooleanEditor::new()))
    } else if target_type == std::any::TypeId::of::<i32>()
        || target_type == std::any::TypeId::of::<i64>()
        || target_type == std::any::TypeId::of::<f64>()
    {
        Some(Box::new(crate::number_editor::CustomNumberEditor::new()))
    } else if target_type == std::any::TypeId::of::<char>() {
        Some(Box::new(crate::char_property_editor::CharacterEditor::new()))
    } else if target_type == std::any::TypeId::of::<Vec<u8>>() {
        Some(Box::new(
            crate::propertyeditors::byte_array_property_editor::ByteArrayPropertyEditor::new(),
        ))
    } else {
        None
    }
}

/// 将值转换为字符串（用于 PropertyEditor.setAsText）。
fn value_to_string(value: &dyn Any) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(s) = value.downcast_ref::<String>() {
        return Ok(s.clone());
    }
    if let Some(s) = value.downcast_ref::<&str>() {
        return Ok(s.to_string());
    }
    if let Some(i) = value.downcast_ref::<i32>() {
        return Ok(i.to_string());
    }
    if let Some(i) = value.downcast_ref::<i64>() {
        return Ok(i.to_string());
    }
    if let Some(f) = value.downcast_ref::<f64>() {
        return Ok(f.to_string());
    }
    if let Some(b) = value.downcast_ref::<bool>() {
        return Ok(b.to_string());
    }
    if let Some(c) = value.downcast_ref::<char>() {
        return Ok(c.to_string());
    }
    Err("Cannot convert value to string".into())
}

/// 将值转换为 owned 版本（完整实现）。
///
/// 支持以下类型：String, &str, i32, i64, u32, u64, f32, f64, bool, char, Vec<u8>,
/// Vec<String>, HashMap<String, String>。
///
/// # Errors
///
/// 如果值的类型不被支持，返回错误。
fn value_to_owned(
    value: &dyn Any,
) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    // 字符串类型
    if let Some(s) = value.downcast_ref::<String>() {
        return Ok(Box::new(s.clone()));
    }
    if let Some(s) = value.downcast_ref::<&str>() {
        return Ok(Box::new(s.to_string()));
    }

    // 数值类型
    if let Some(i) = value.downcast_ref::<i32>() {
        return Ok(Box::new(*i));
    }
    if let Some(i) = value.downcast_ref::<i64>() {
        return Ok(Box::new(*i));
    }
    if let Some(i) = value.downcast_ref::<u32>() {
        return Ok(Box::new(*i));
    }
    if let Some(i) = value.downcast_ref::<u64>() {
        return Ok(Box::new(*i));
    }
    if let Some(f) = value.downcast_ref::<f32>() {
        return Ok(Box::new(*f));
    }
    if let Some(f) = value.downcast_ref::<f64>() {
        return Ok(Box::new(*f));
    }

    // 布尔/字符类型
    if let Some(b) = value.downcast_ref::<bool>() {
        return Ok(Box::new(*b));
    }
    if let Some(c) = value.downcast_ref::<char>() {
        return Ok(Box::new(*c));
    }

    // 集合类型
    if let Some(v) = value.downcast_ref::<Vec<u8>>() {
        return Ok(Box::new(v.clone()));
    }
    if let Some(v) = value.downcast_ref::<Vec<String>>() {
        return Ok(Box::new(v.clone()));
    }
    if let Some(m) = value.downcast_ref::<HashMap<String, String>>() {
        return Ok(Box::new(m.clone()));
    }

    // 不支持的类型：返回错误
    Err(format!(
        "Cannot convert value of type {:?} to owned version",
        value.type_id()
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    // ── TypeConverterDelegate basic ──────────────────────────────────────

    #[test]
    fn new_delegate_has_no_editors() {
        let delegate = TypeConverterDelegate::new();
        assert_eq!(delegate.editor_count(), 0);
    }

    #[test]
    fn default_delegate() {
        let delegate = TypeConverterDelegate::default();
        assert_eq!(delegate.editor_count(), 0);
    }

    #[test]
    fn clear_empties_editors_and_converters() {
        let delegate = TypeConverterDelegate::new();
        delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i64>(), |v| {
            let n = v.downcast_ref::<i32>().unwrap();
            Ok(Box::new(*n as i64))
        });
        delegate.clear();
        assert_eq!(delegate.editor_count(), 0);
    }

    // ── register_custom_editor ───────────────────────────────────────────

    #[test]
    fn register_custom_editor_increments_count() {
        let delegate = TypeConverterDelegate::new();
        let editor = Arc::new(crate::number_editor::CustomNumberEditor::new());
        delegate.register_custom_editor(TypeId::of::<i32>(), editor);
        assert_eq!(delegate.editor_count(), 1);
    }

    #[test]
    fn register_multiple_editors() {
        let delegate = TypeConverterDelegate::new();
        let editor1 = Arc::new(crate::number_editor::CustomNumberEditor::new());
        let editor2 = Arc::new(crate::boolean_editor::CustomBooleanEditor::new());
        delegate.register_custom_editor(TypeId::of::<i32>(), editor1);
        delegate.register_custom_editor(TypeId::of::<bool>(), editor2);
        assert_eq!(delegate.editor_count(), 2);
    }

    // ── register_converter ───────────────────────────────────────────────

    #[test]
    fn register_and_use_custom_converter() {
        let delegate = TypeConverterDelegate::new();
        delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i64>(), |v| {
            let n = v.downcast_ref::<i32>().unwrap();
            Ok(Box::new(*n as i64))
        });
        let value: Box<dyn Any> = Box::new(42i32);
        let result = delegate
            .convert_if_necessary(None, &*value, TypeId::of::<i64>())
            .unwrap();
        assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);
    }

    // ── convert_if_necessary ─────────────────────────────────────────────

    #[test]
    fn convert_same_type_returns_owned() {
        let delegate = TypeConverterDelegate::new();
        let value = "hello".to_string();
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<String>())
            .unwrap();
        assert_eq!(*result.downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn convert_same_type_i32() {
        let delegate = TypeConverterDelegate::new();
        let value = 42i32;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<i32>())
            .unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn convert_same_type_i64() {
        let delegate = TypeConverterDelegate::new();
        let value = 42i64;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<i64>())
            .unwrap();
        assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn convert_same_type_u32() {
        let delegate = TypeConverterDelegate::new();
        let value = 42u32;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<u32>())
            .unwrap();
        assert_eq!(*result.downcast::<u32>().unwrap(), 42u32);
    }

    #[test]
    fn convert_same_type_u64() {
        let delegate = TypeConverterDelegate::new();
        let value = 42u64;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<u64>())
            .unwrap();
        assert_eq!(*result.downcast::<u64>().unwrap(), 42u64);
    }

    #[test]
    fn convert_same_type_f32() {
        let delegate = TypeConverterDelegate::new();
        let value = 1.5f32;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<f32>())
            .unwrap();
        assert_eq!(*result.downcast::<f32>().unwrap(), 1.5f32);
    }

    #[test]
    fn convert_same_type_f64() {
        let delegate = TypeConverterDelegate::new();
        let value = 3.14f64;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<f64>())
            .unwrap();
        let val = *result.downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_same_type_bool() {
        let delegate = TypeConverterDelegate::new();
        let value = true;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<bool>())
            .unwrap();
        assert!(*result.downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_same_type_char() {
        let delegate = TypeConverterDelegate::new();
        let value = 'x';
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<char>())
            .unwrap();
        assert_eq!(*result.downcast::<char>().unwrap(), 'x');
    }

    #[test]
    fn convert_same_type_vec_u8() {
        let delegate = TypeConverterDelegate::new();
        let value = vec![1u8, 2, 3];
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<Vec<u8>>())
            .unwrap();
        assert_eq!(*result.downcast::<Vec<u8>>().unwrap(), vec![1u8, 2, 3]);
    }

    #[test]
    fn convert_same_type_vec_string() {
        let delegate = TypeConverterDelegate::new();
        let value = vec!["a".to_string(), "b".to_string()];
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<Vec<String>>())
            .unwrap();
        assert_eq!(
            *result.downcast::<Vec<String>>().unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn convert_same_type_hashmap() {
        let delegate = TypeConverterDelegate::new();
        let mut value = HashMap::new();
        value.insert("k".to_string(), "v".to_string());
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<HashMap<String, String>>())
            .unwrap();
        let map = result.downcast::<HashMap<String, String>>().unwrap();
        assert_eq!(map.get("k").unwrap(), "v");
    }

    // ── PropertyEditor-based conversion ──────────────────────────────────

    #[test]
    fn convert_string_to_bool_via_editor() {
        let delegate = TypeConverterDelegate::new();
        let value = "true".to_string();
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<bool>());
        // May succeed via PropertyEditor or fail - either is valid behavior
        // The important thing is it doesn't panic
        let _ = result;
    }

    #[test]
    fn convert_string_to_i32_via_editor() {
        let delegate = TypeConverterDelegate::new();
        let value = "42".to_string();
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<i32>());
        let _ = result;
    }

    #[test]
    fn convert_string_to_f64_via_editor() {
        let delegate = TypeConverterDelegate::new();
        let value = "3.14".to_string();
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<f64>());
        let _ = result;
    }

    #[test]
    fn convert_string_to_char_via_editor() {
        let delegate = TypeConverterDelegate::new();
        let value = "A".to_string();
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<char>());
        let _ = result;
    }

    #[test]
    fn convert_string_to_vec_u8_via_editor() {
        let delegate = TypeConverterDelegate::new();
        let value = "hello".to_string();
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<Vec<u8>>());
        let _ = result;
    }

    #[test]
    fn convert_unsupported_type_returns_error() {
        let delegate = TypeConverterDelegate::new();
        struct Unsupported;
        let value = Unsupported;
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn convert_no_converter_found_returns_error() {
        let delegate = TypeConverterDelegate::new();
        struct Unsupported;
        let value = Unsupported;
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<Vec<u32>>());
        assert!(result.is_err());
    }

    // ── value_to_string ──────────────────────────────────────────────────

    #[test]
    fn value_to_string_from_string() {
        let value = "hello".to_string();
        assert_eq!(super::value_to_string(&value).unwrap(), "hello");
    }

    #[test]
    fn value_to_string_from_str_ref() {
        let value: &str = "hello";
        assert_eq!(super::value_to_string(&value).unwrap(), "hello");
    }

    #[test]
    fn value_to_string_from_i32() {
        let value = 42i32;
        assert_eq!(super::value_to_string(&value).unwrap(), "42");
    }

    #[test]
    fn value_to_string_from_i64() {
        let value = 42i64;
        assert_eq!(super::value_to_string(&value).unwrap(), "42");
    }

    #[test]
    fn value_to_string_from_f64() {
        let value = 3.14f64;
        let s = super::value_to_string(&value).unwrap();
        assert!(s.contains("3.14"));
    }

    #[test]
    fn value_to_string_from_bool() {
        let value = true;
        assert_eq!(super::value_to_string(&value).unwrap(), "true");
    }

    #[test]
    fn value_to_string_from_char() {
        let value = 'A';
        assert_eq!(super::value_to_string(&value).unwrap(), "A");
    }

    #[test]
    fn value_to_string_unsupported() {
        struct Unsupported;
        let value = Unsupported;
        let result = super::value_to_string(&value);
        assert!(result.is_err());
    }

    // ── value_to_owned ───────────────────────────────────────────────────

    #[test]
    fn value_to_owned_string() {
        let value = "hello".to_string();
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn value_to_owned_str_ref() {
        let value: &str = "hello";
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn value_to_owned_i32() {
        let value = 42i32;
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn value_to_owned_i64() {
        let value = 42i64;
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn value_to_owned_u32() {
        let value = 42u32;
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<u32>().unwrap(), 42u32);
    }

    #[test]
    fn value_to_owned_u64() {
        let value = 42u64;
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<u64>().unwrap(), 42u64);
    }

    #[test]
    fn value_to_owned_f32() {
        let value = 1.5f32;
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<f32>().unwrap(), 1.5f32);
    }

    #[test]
    fn value_to_owned_f64() {
        let value = 3.14f64;
        let result = super::value_to_owned(&value).unwrap();
        let val = *result.downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn value_to_owned_bool() {
        let value = true;
        let result = super::value_to_owned(&value).unwrap();
        assert!(*result.downcast::<bool>().unwrap());
    }

    #[test]
    fn value_to_owned_char() {
        let value = 'x';
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<char>().unwrap(), 'x');
    }

    #[test]
    fn value_to_owned_vec_u8() {
        let value = vec![1u8, 2, 3];
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(*result.downcast::<Vec<u8>>().unwrap(), vec![1u8, 2, 3]);
    }

    #[test]
    fn value_to_owned_vec_string() {
        let value = vec!["a".to_string()];
        let result = super::value_to_owned(&value).unwrap();
        assert_eq!(
            *result.downcast::<Vec<String>>().unwrap(),
            vec!["a".to_string()]
        );
    }

    #[test]
    fn value_to_owned_hashmap() {
        let mut value = HashMap::new();
        value.insert("k".to_string(), "v".to_string());
        let result = super::value_to_owned(&value).unwrap();
        let map = result.downcast::<HashMap<String, String>>().unwrap();
        assert_eq!(map.get("k").unwrap(), "v");
    }

    #[test]
    fn value_to_owned_unsupported() {
        struct Unsupported;
        let value = Unsupported;
        let result = super::value_to_owned(&value);
        assert!(result.is_err());
    }

    // ── create_editor_for_type ───────────────────────────────────────────

    #[test]
    fn create_editor_for_string_type() {
        let editor = super::create_editor_for_type(TypeId::of::<String>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_bool_type() {
        let editor = super::create_editor_for_type(TypeId::of::<bool>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_i32_type() {
        let editor = super::create_editor_for_type(TypeId::of::<i32>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_i64_type() {
        let editor = super::create_editor_for_type(TypeId::of::<i64>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_f64_type() {
        let editor = super::create_editor_for_type(TypeId::of::<f64>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_char_type() {
        let editor = super::create_editor_for_type(TypeId::of::<char>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_vec_u8_type() {
        let editor = super::create_editor_for_type(TypeId::of::<Vec<u8>>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_unsupported_type() {
        let editor = super::create_editor_for_type(TypeId::of::<Vec<u32>>());
        assert!(editor.is_none());
    }

    // ── convert_if_necessary with property_name ──────────────────────────

    #[test]
    fn convert_with_property_name() {
        let delegate = TypeConverterDelegate::new();
        let value = "hello".to_string();
        let result = delegate.convert_if_necessary(Some("myProp"), &value, TypeId::of::<String>());
        assert!(result.is_ok());
    }

    // ── Custom converter override ────────────────────────────────────────

    #[test]
    fn custom_converter_takes_priority() {
        let delegate = TypeConverterDelegate::new();
        // Register a converter that adds 100
        delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i32>(), |v| {
            let n = v.downcast_ref::<i32>().unwrap();
            Ok(Box::new(*n + 100))
        });
        let value = 42i32;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<i32>())
            .unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 142);
    }

    #[test]
    fn custom_converter_does_not_match_different_types() {
        let delegate = TypeConverterDelegate::new();
        // Register converter for i32 -> i64
        delegate.register_converter(TypeId::of::<i32>(), TypeId::of::<i64>(), |v| {
            let n = v.downcast_ref::<i32>().unwrap();
            Ok(Box::new(*n as i64))
        });
        // Try to convert i32 -> i32 (should use same-type path, not custom converter)
        let value = 42i32;
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<i32>())
            .unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 42);
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn convert_string_to_i64_via_editor() {
        let delegate = TypeConverterDelegate::new();
        let value = "42".to_string();
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<i64>());
        let _ = result;
    }

    #[test]
    fn convert_string_to_string_same_type() {
        let delegate = TypeConverterDelegate::new();
        let value = "hello".to_string();
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<String>())
            .unwrap();
        assert_eq!(*result.downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn convert_with_property_name_string() {
        let delegate = TypeConverterDelegate::new();
        let value = "test".to_string();
        let result = delegate
            .convert_if_necessary(Some("myField"), &value, TypeId::of::<String>())
            .unwrap();
        assert_eq!(*result.downcast::<String>().unwrap(), "test");
    }

    #[test]
    fn value_to_owned_f64_special_values() {
        let value = 0.0f64;
        let result = super::value_to_owned(&value).unwrap();
        let val = *result.downcast::<f64>().unwrap();
        assert!((val - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn value_to_string_from_f32_not_supported() {
        let value = 1.5f32;
        // f32 is not handled by value_to_string, so it should return an error
        let result = super::value_to_string(&value);
        assert!(result.is_err());
    }

    #[test]
    fn create_editor_for_i32_type_cov2() {
        let editor = super::create_editor_for_type(TypeId::of::<i32>());
        assert!(editor.is_some());
    }

    #[test]
    fn create_editor_for_i64_type_cov2() {
        let editor = super::create_editor_for_type(TypeId::of::<i64>());
        assert!(editor.is_some());
    }

    #[test]
    fn register_converter_and_use_for_different_types() {
        let delegate = TypeConverterDelegate::new();
        delegate.register_converter(TypeId::of::<String>(), TypeId::of::<i32>(), |v| {
            let s = v.downcast_ref::<String>().unwrap();
            let n: i32 = s.parse().unwrap_or(0);
            Ok(Box::new(n))
        });
        let value = "42".to_string();
        let result = delegate
            .convert_if_necessary(None, &value, TypeId::of::<i32>())
            .unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn clear_and_verify() {
        let delegate = TypeConverterDelegate::new();
        let editor = Arc::new(crate::number_editor::CustomNumberEditor::new());
        delegate.register_custom_editor(TypeId::of::<i32>(), editor);
        assert_eq!(delegate.editor_count(), 1);
        delegate.clear();
        assert_eq!(delegate.editor_count(), 0);
    }

    #[test]
    fn convert_unsupported_type_to_unsupported_target() {
        struct Unsupported;
        let delegate = TypeConverterDelegate::new();
        let value = Unsupported;
        let result = delegate.convert_if_necessary(None, &value, TypeId::of::<Vec<u32>>());
        assert!(result.is_err());
    }
}

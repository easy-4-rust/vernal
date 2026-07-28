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
        property_name: Option<&str>,
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
            crate::string_trimmer_editor::StringTrimmerEditor::new(),
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
            crate::byte_array_property_editor::ByteArrayPropertyEditor::new(),
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

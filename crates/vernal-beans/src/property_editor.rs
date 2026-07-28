//! PropertyEditor — Spring 风格的属性编辑器 trait。
//!
//! 对应 Java 类：`java.beans.PropertyEditor` + `org.springframework.beans.PropertyEditorRegistry`。
//!
//! 属性编辑器用于将字符串值转换为目标类型，或将目标类型转换为字符串表示。
//! 这是 Spring 配置绑定（XML/Properties → Bean）的核心机制。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的属性编辑器 trait。
///
/// 对应 Java 的 `PropertyEditor` 接口。
///
/// 在 Rust 中，属性编辑器的核心能力是：
/// - `set_as_text` — 将字符串转换为目标类型
/// - `get_as_text` — 将目标类型转换为字符串
/// - `set_value` / `get_value` — 直接操作目标值
///
/// ## 与 Rust 的映射
///
/// Java 的 `PropertyEditor` 依赖 `Class<?>` 反射。
/// Rust 中使用 `std::any::TypeId` + trait 对象实现等效能力。
pub trait PropertyEditor: Send + Sync {
    /// 获取编辑器支持的目标类型。
    fn target_type(&self) -> std::any::TypeId;

    /// 将字符串值转换为目标类型。
    ///
    /// 对应 Spring 的 `PropertyEditor.setAsText(String text)`。
    ///
    /// # 错误
    ///
    /// 转换失败时返回 `Err`（对标 Spring 的 `IllegalArgumentException`）。
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 将目标类型转换为字符串。
    ///
    /// 对应 Spring 的 `PropertyEditor.getAsText()`。
    ///
    /// # 返回
    ///
    /// - `Some(string)` — 字符串表示
    /// - `None` — 无法转换为字符串
    fn get_as_text(&self) -> Option<String>;

    /// 设置目标值。
    ///
    /// 对应 Spring 的 `PropertyEditor.setValue(Object value)`。
    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>);

    /// 获取目标值。
    ///
    /// 对应 Spring 的 `PropertyEditor.getValue()`。
    fn get_value(&self) -> Option<&dyn Any>;

    /// 获取值的类型。
    ///
    /// 对应 Spring 的 `PropertyEditor.getValue()` 返回的类型。
    fn get_value_type(&self) -> std::any::TypeId;

    /// 是否支持从字符串转换。
    ///
    /// 对应 Spring 的 `PropertyEditor.isPaintable()` 的简化版本。
    fn supports_text(&self) -> bool {
        true
    }

    /// 是否支持自定义编辑。
    ///
    /// 对应 Spring 的 `PropertyEditor.supportsCustomEditor()`。
    fn supports_custom_editor(&self) -> bool {
        false
    }

    /// 获取 Java 格式化标记（用于错误报告）。
    ///
    /// 对应 Spring 的 `PropertyEditor.getJavaInitializationString()`。
    /// 在 Rust 中返回一个调试字符串。
    fn java_initialization_string(&self) -> Option<&str> {
        None
    }
}

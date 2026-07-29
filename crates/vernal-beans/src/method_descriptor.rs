//! MethodDescriptor — Spring 风格的方法描述符。
//!
//! 对应 Java 类：`org.springframework.core.MethodParameter` / `java.lang.reflect.Method`
//! 的可移植描述。
//!
//! 描述一个方法的元数据：名称、声明类、返回类型、参数类型列表。
//! 用于方法注入、方法覆盖等不依赖反射 API 的场景。

/// 方法描述符。
///
/// 对应 Spring 的方法元数据载体（如 `MethodParameter` 的简化形态）。
///
/// 所有字段均为字符串形式（类型名），避免依赖 Rust 的 `TypeId` 反射，
/// 以便序列化与跨模块传递。
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    /// 方法名。
    name: String,
    /// 声明该方法的类名。
    declaring_class: String,
    /// 返回类型名。
    return_type: String,
    /// 参数类型名列表（按声明顺序）。
    parameter_types: Vec<String>,
}

impl MethodDescriptor {
    /// 创建方法描述符。
    pub fn new(name: impl Into<String>, declaring_class: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            declaring_class: declaring_class.into(),
            return_type: String::new(),
            parameter_types: Vec::new(),
        }
    }

    /// 创建完整的方法描述符。
    pub fn with_all(
        name: impl Into<String>,
        declaring_class: impl Into<String>,
        return_type: impl Into<String>,
        parameter_types: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            declaring_class: declaring_class.into(),
            return_type: return_type.into(),
            parameter_types,
        }
    }

    /// 获取方法名。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取声明类名。
    pub fn declaring_class(&self) -> &str {
        &self.declaring_class
    }

    /// 获取返回类型名。
    pub fn return_type(&self) -> &str {
        &self.return_type
    }

    /// 设置返回类型名。
    pub fn set_return_type(&mut self, return_type: impl Into<String>) {
        self.return_type = return_type.into();
    }

    /// 获取参数类型名列表。
    pub fn parameter_types(&self) -> &[String] {
        &self.parameter_types
    }

    /// 参数数量。
    pub fn parameter_count(&self) -> usize {
        self.parameter_types.len()
    }

    /// 添加参数类型名。
    pub fn add_parameter_type(&mut self, type_name: impl Into<String>) {
        self.parameter_types.push(type_name.into());
    }

    /// 是否无参方法。
    pub fn is_no_arg(&self) -> bool {
        self.parameter_types.is_empty()
    }

    /// 全限定签名字符串（声明类#方法名(参数类型, ...)）。
    pub fn signature(&self) -> String {
        format!(
            "{}#{}({})",
            self.declaring_class,
            self.name,
            self.parameter_types.join(", ")
        )
    }
}

impl Default for MethodDescriptor {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

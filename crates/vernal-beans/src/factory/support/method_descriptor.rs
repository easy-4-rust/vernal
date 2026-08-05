//! MethodDescriptor — Spring 风格的方法描述符。
//!
//! 对应 Java 类：`java.beans.MethodDescriptor`。
//!
//! 在 Spring 中，`MethodDescriptor` 描述一个方法的元数据，
//! 包括方法名、返回类型、参数类型等。
//! 用于 Bean 描述符和方法覆盖机制中。

/// 方法描述符。
///
/// 对应 Java 的 `MethodDescriptor`。
///
/// 描述一个方法的基本信息，用于 Bean 定义的方法覆盖和查找。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MethodDescriptor {
    /// 方法名
    name: String,
    /// 返回类型名
    return_type: String,
    /// 参数类型名列表
    parameter_types: Vec<String>,
    /// 方法是否公开
    is_public: bool,
}

impl MethodDescriptor {
    /// 创建新的方法描述符。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            return_type: String::new(),
            parameter_types: Vec::new(),
            is_public: true,
        }
    }

    /// 设置返回类型。
    pub fn with_return_type(mut self, rt: impl Into<String>) -> Self {
        self.return_type = rt.into();
        self
    }

    /// 设置参数类型列表。
    pub fn with_parameters(mut self, params: Vec<String>) -> Self {
        self.parameter_types = params;
        self
    }

    /// 设置是否公开。
    pub fn with_public(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self
    }

    /// 获取方法名。
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// 获取返回类型名。
    pub fn get_return_type(&self) -> &str {
        &self.return_type
    }

    /// 获取参数类型列表。
    pub fn get_parameter_types(&self) -> &[String] {
        &self.parameter_types
    }

    /// 获取参数数量。
    pub fn parameter_count(&self) -> usize {
        self.parameter_types.len()
    }

    /// 是否无参数方法。
    pub fn is_no_arg(&self) -> bool {
        self.parameter_types.is_empty()
    }

    /// 是否公开方法。
    pub fn is_public(&self) -> bool {
        self.is_public
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_method_descriptor() {
        let md = MethodDescriptor::new("doSomething");
        assert_eq!(md.get_name(), "doSomething");
        assert_eq!(md.get_return_type(), "");
        assert_eq!(md.parameter_count(), 0);
        assert!(md.is_no_arg());
        assert!(md.is_public());
    }

    #[test]
    fn with_return_type_and_parameters() {
        let md = MethodDescriptor::new("calculate")
            .with_return_type("i32")
            .with_parameters(vec!["f64".to_string(), "f64".to_string()]);

        assert_eq!(md.get_return_type(), "i32");
        assert_eq!(md.parameter_count(), 2);
        assert!(!md.is_no_arg());
    }

    #[test]
    fn with_public_flag() {
        let md = MethodDescriptor::new("internal").with_public(false);
        assert!(!md.is_public());
    }

    #[test]
    fn equality() {
        let md1 = MethodDescriptor::new("test").with_return_type("bool");
        let md2 = MethodDescriptor::new("test").with_return_type("bool");
        assert_eq!(md1, md2);
    }
}

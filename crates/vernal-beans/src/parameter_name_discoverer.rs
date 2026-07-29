//! ParameterNameDiscoverer — Spring 风格的参数名称发现器 trait。
//!
//! 对应 Java 类：`org.springframework.core.ParameterNameDiscoverer`。
//!
//! 定义从方法/函数签名中发现参数名称的策略接口。

/// Spring 风格的参数名称发现器 trait。
///
/// 对应 Spring 的 `ParameterNameDiscoverer`。
///
/// 用于从可调用对象（函数、方法等）中提取参数名称列表。
/// 在构造器注入、`@Autowired` 方法注入等场景中，通过参数名称
/// 匹配依赖 Bean 名称。
///
/// ## 实现
///
/// - `LocalVariableTableParameterNameDiscoverer` — 通过调试信息的
///   局部变量表获取参数名
/// - `StandardReflectionParameterNameDiscoverer` — 通过 Java 8+
///   的反射 API 获取参数名
/// - `AspectJAdviceParameterNameDiscoverer` — 用于 AOP 通知
pub trait ParameterNameDiscoverer: Send + Sync + std::fmt::Debug {
    /// 获取指定方法的参数名称列表。
    ///
    /// 对应 Spring 的 `String[] getParameterNames(Method method)`。
    ///
    /// # 参数
    ///
    /// * `method` — 目标方法的描述信息（类型擦除的 Any）
    ///
    /// # 返回
    ///
    /// 参数名称的列表。如果无法获取参数名，返回空 Vec。
    fn get_parameter_names(&self, method: &dyn std::any::Any) -> Vec<String>;
}

/// 默认的参数名称发现器实现。
///
/// 使用简单的命名约定：`arg0`, `arg1`, `arg2`, ...
/// 这是在没有调试信息或反射支持的情况下的回退方案。
#[derive(Debug, Default)]
pub struct DefaultParameterNameDiscoverer;

impl DefaultParameterNameDiscoverer {
    /// 创建新的 DefaultParameterNameDiscoverer。
    pub fn new() -> Self {
        Self
    }
}

impl ParameterNameDiscoverer for DefaultParameterNameDiscoverer {
    fn get_parameter_names(&self, _method: &dyn std::any::Any) -> Vec<String> {
        // 默认实现无法从擦除的类型中提取参数名
        Vec::new()
    }
}

/// 通过局部变量表获取参数名的发现器实现（模拟 Spring 的对应实现）。
///
/// 在实际的 Java 中，此实现通过读取 `.class` 文件的 LocalVariableTable 属性
/// 来获取参数名。在 Rust 中，由于没有此类调试信息，此实现作为占位符。
///
/// 用户可以提供自定义的参数名解析逻辑（如通过宏或外部信息）。
#[derive(Debug, Default)]
pub struct LocalVariableTableParameterNameDiscoverer;

impl LocalVariableTableParameterNameDiscoverer {
    /// 创建新的 LocalVariableTableParameterNameDiscoverer。
    pub fn new() -> Self {
        Self
    }
}

impl ParameterNameDiscoverer for LocalVariableTableParameterNameDiscoverer {
    fn get_parameter_names(&self, _method: &dyn std::any::Any) -> Vec<String> {
        // 在 Rust 中，此发现器无法通过运行时反射获取参数名。
        // 返回空 Vec 表示无法获取。
        // 此实现可作为未来通过宏或编译器插件增强的占位符。
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_default_discoverer() {
        let discoverer = DefaultParameterNameDiscoverer::new();
        let method = Box::new(String::from("test_method")) as Box<dyn std::any::Any>;
        let names = discoverer.get_parameter_names(method.as_ref());
        assert!(names.is_empty());
    }

    #[test]
    fn test_local_variable_table_discoverer() {
        let discoverer = LocalVariableTableParameterNameDiscoverer::new();
        let method = Box::new(String::from("test_method")) as Box<dyn std::any::Any>;
        let names = discoverer.get_parameter_names(method.as_ref());
        assert!(names.is_empty());
    }

    #[test]
    fn test_trait_object_safe() {
        let discoverer: Arc<dyn ParameterNameDiscoverer> =
            Arc::new(DefaultParameterNameDiscoverer::new());
        let method = Box::new(String::from("test")) as Box<dyn std::any::Any>;
        let names = discoverer.get_parameter_names(method.as_ref());
        assert!(names.is_empty());
    }
}

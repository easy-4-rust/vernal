//! BeanDefinitionResource — Spring 风格的 Bean 定义资源 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionResource`。
//!
//! 表示 Bean 定义的来源（文件路径、类路径、URL 等）。

/// Spring 风格的 Bean 定义资源 trait。
///
/// 对应 Spring 的 `BeanDefinitionResource`。
///
/// 表示 Bean 定义的来源，BeanDefinitionReader 使用它加载定义。
pub trait BeanDefinitionResource: Send + Sync {
    /// 获取资源描述。
    fn description(&self) -> &str;

    /// 获取资源名称（如文件名）。
    fn name(&self) -> &str;

    /// 读取资源内容为字符串。
    fn read_to_string(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

//! XmlBeanDefinitionReader — XML Bean 定义读取器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.XmlBeanDefinitionReader`。
//!
//! 负责从 XML 资源读取 Bean 定义并注册到 BeanDefinitionRegistry。

/// XML Bean 定义读取器接口。
///
/// 对应 Spring 的 `XmlBeanDefinitionReader`。
///
/// 从 XML 资源读取 Bean 定义，解析后注册到 `BeanDefinitionRegistry`。
/// 在 vernal-beans 中，XML 解析使用 `quick-xml` crate。
pub trait XmlBeanDefinitionReader: Send + Sync {
    /// 从 XML 字符串加载 Bean 定义。
    ///
    /// # 参数
    /// - `xml_content` — XML 内容字符串
    ///
    /// # 返回
    /// 加载的 Bean 定义数量。
    fn load_bean_definitions_from_xml(
        &self,
        xml_content: &str,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// 从 XML 文件路径加载 Bean 定义。
    ///
    /// # 参数
    /// - `file_path` — XML 文件路径
    ///
    /// # 返回
    /// 加载的 Bean 定义数量。
    fn load_bean_definitions_from_file(
        &self,
        file_path: &str,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取验证模式。
    fn validation_mode(&self) -> crate::factory::xml::document_loader::ValidationMode;

    /// 设置验证模式。
    fn set_validation_mode(&mut self, mode: crate::factory::xml::document_loader::ValidationMode);

    /// 是否支持命名空间。
    fn namespace_aware(&self) -> bool;

    /// 设置命名空间支持。
    fn set_namespace_aware(&mut self, namespace_aware: bool);
}

//! BeanDefinitionParserDelegate — Bean 定义解析委托接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeanDefinitionParserDelegate`。
//!
//! 负责解析 XML 中的 `<bean>` 元素及其子元素。

/// Bean 定义解析委托接口。
///
/// 对应 Spring 的 `BeanDefinitionParserDelegate`。
///
/// 解析 XML 中的 `<bean>` 元素，包括：
/// - 属性（scope, lazy-init, autowire, depends-on 等）
/// - 构造器参数（`<constructor-arg>`）
/// - 属性值（`<property>`）
/// - 集合（`<list>`, `<set>`, `<map>`, `<props>`）
/// - 方法覆盖（`<lookup-method>`, `<replaced-method>`）
pub trait BeanDefinitionParserDelegate: Send + Sync {
    /// 解析 Bean 元素。
    ///
    /// # 参数
    /// - `element_name` — 元素名称
    /// - `attributes` — 元素属性
    ///
    /// # 返回
    /// 解析结果（Bean 定义名称）。
    fn parse_bean_element(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// 解析构造器参数元素。
    fn parse_constructor_arg_element(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 解析属性元素。
    fn parse_property_element(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 解析限定符元素。
    fn parse_qualifier_element(
        &self,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

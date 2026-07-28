//! AutowireCapableBeanFactory — Spring 风格的自动装配能力 BeanFactory。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.AutowireCapableBeanFactory`。
//!
//! 扩展 BeanFactory，提供自动装配、Bean 创建和初始化的能力。

use std::any::Any;
use std::sync::Arc;

use crate::bean_factory::BeanFactory;
use crate::dependency_descriptor::DependencyDescriptor;
use crate::named_bean_holder::NamedBeanHolder;
use crate::type_converter::TypeConverter;

/// Spring 风格的自动装配能力 BeanFactory 接口。
///
/// 对应 Spring 的 `AutowireCapableBeanFactory`。
///
/// 扩展 `BeanFactory`，提供自动装配、Bean 创建和初始化的能力。
/// 这是 Spring AOP、`@Autowired` 注解处理的核心接口。
///
/// ## 常量
///
/// - `AUTOWIRE_NO = 0` — 不自动装配
/// - `AUTOWIRE_BY_NAME = 1` — 按名称自动装配
/// - `AUTOWIRE_BY_TYPE = 2` — 按类型自动装配
/// - `AUTOWIRE_CONSTRUCTOR = 3` — 按构造器自动装配
/// - `AUTOWIRE_AUTODETECT = 4` — 已废弃
/// - `ORIGINAL_INSTANCE_SUFFIX = ".ORIGINAL"` — 原始实例后缀
pub trait AutowireCapableBeanFactory: BeanFactory {
    /// 不自动装配。
    const AUTOWIRE_NO: i32 = 0;
    /// 按名称自动装配。
    const AUTOWIRE_BY_NAME: i32 = 1;
    /// 按类型自动装配。
    const AUTOWIRE_BY_TYPE: i32 = 2;
    /// 按构造器自动装配。
    const AUTOWIRE_CONSTRUCTOR: i32 = 3;
    /// 原始实例后缀。
    const ORIGINAL_INSTANCE_SUFFIX: &str = ".ORIGINAL";

    /// 创建一个新的 Bean 实例。
    ///
    /// 对应 Spring 的 `<T> T createBean(Class<T> beanClass) throws BeansException`。
    ///
    /// 完整的 Bean 创建流程：实例化 → 属性注入 → 初始化。
    fn create_bean(
        &self,
        bean_class_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 自动装配现有 Bean 的属性。
    ///
    /// 对应 Spring 的 `void autowireBean(Object existingBean) throws BeansException`。
    ///
    /// 根据 Bean 的属性类型自动注入匹配的依赖。
    fn autowire_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 配置现有 Bean（应用属性值 + 初始化）。
    ///
    /// 对应 Spring 的 `Object configureBean(Object existingBean, String beanName) throws BeansException`。
    fn configure_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 按指定 autowire 模式自动装配。
    ///
    /// 对应 Spring 的 `Object autowire(Class<?> beanClass, int autowireMode, boolean dependencyCheck) throws BeansException`。
    fn autowire(
        &self,
        bean_class_name: &str,
        autowire_mode: i32,
        dependency_check: bool,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 自动装配现有 Bean 的属性（指定模式）。
    ///
    /// 对应 Spring 的 `void autowireBeanProperties(Object existingBean, int autowireMode, boolean dependencyCheck) throws BeansException`。
    fn autowire_bean_properties(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        autowire_mode: i32,
        dependency_check: bool,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 应用属性值到现有 Bean。
    ///
    /// 对应 Spring 的 `void applyBeanPropertyValues(Object existingBean, String beanName) throws BeansException`。
    fn apply_bean_property_values(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 初始化现有 Bean（应用初始化回调）。
    ///
    /// 对应 Spring 的 `Object initializeBean(Object existingBean, String beanName) throws BeansException`。
    fn initialize_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 销毁 Bean。
    ///
    /// 对应 Spring 的 `void destroyBean(Object existingBean)`。
    fn destroy_bean_instance(
        &self,
        bean_name: &str,
        bean_instance: &dyn Any,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 按类型解析命名 Bean。
    ///
    /// 对应 Spring 的 `<T> NamedBeanHolder<T> resolveNamedBean(Class<T> requiredType) throws BeansException`。
    fn resolve_named_bean(
        &self,
        type_id: std::any::TypeId,
    ) -> Result<NamedBeanHolder<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;

    /// 解析依赖。
    ///
    /// 对应 Spring 的 `Object resolveDependency(DependencyDescriptor descriptor, String requestingBeanName) throws BeansException`。
    fn resolve_dependency(
        &self,
        descriptor: &DependencyDescriptor,
        requesting_bean_name: Option<&str>,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>;

    /// 设置类型转换器。
    ///
    /// 对应 Spring 的 `void setTypeConverter(TypeConverter typeConverter)`。
    fn set_type_converter(&mut self, converter: Option<Arc<dyn TypeConverter>>);

    /// 获取类型转换器。
    ///
    /// 对应 Spring 的 `TypeConverter getTypeConverter()`。
    fn type_converter(&self) -> Option<&dyn TypeConverter>;
}

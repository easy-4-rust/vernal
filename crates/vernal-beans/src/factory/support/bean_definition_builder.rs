//! BeanDefinitionBuilder — Spring 风格的 Bean 定义构建器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionBuilder`。
//!
//! 提供流式 API 构建 BeanDefinition，是注册 Bean 定义的主要入口。

use crate::component_scope::Scope;
use crate::factory::annotation::autowire::Autowire;
use crate::factory::config::constructor_argument_values::ValueHolder;
use crate::factory::support::generic_bean_definition::GenericBeanDefinition;
use crate::factory::support::root_bean_definition::RootBeanDefinition;

/// Spring 风格的 Bean 定义构建器。
///
/// 对应 Spring 的 `BeanDefinitionBuilder`。
///
/// 提供流式 API 构建 `GenericBeanDefinition` 或 `RootBeanDefinition`。
///
/// ## 使用示例
///
/// ```rust,ignore
/// use vernal_beans::BeanDefinitionBuilder;
///
/// let bd = BeanDefinitionBuilder::generic("MyService")
///     .set_scope(Scope::Singleton)
///     .set_lazy_init(false)
///     .add_constructor_arg_value(42i32)
///     .set_init_method("init")
///     .set_destroy_method("cleanup")
///     .add_depends_on("DataSource")
///     .set_primary(true)
///     .build();
/// ```
///
/// ## 与 Spring 的差异
///
/// - Spring 的 ` BeanDefinitionBuilder.root()` / `generic()` 接受 `Class<?>` 或 `String`
/// - vernal 的 `BeanDefinitionBuilder` 接受 `impl Into<String>` 类名
/// - Spring 的 `getBeanDefinition()` 返回 `AbstractBeanDefinition`
/// - vernal 的 `build()` 返回 `GenericBeanDefinition`
pub struct BeanDefinitionBuilder {
    definition: GenericBeanDefinition,
}

impl BeanDefinitionBuilder {
    /// 创建 GenericBeanDefinition 构建器。
    ///
    /// 对应 Spring 的 `BeanDefinitionBuilder.genericBeanDefinition(Class<?> beanClass)`。
    pub fn generic(bean_class_name: impl Into<String>) -> Self {
        let mut def = GenericBeanDefinition::new();
        def.set_bean_class_name(bean_class_name);
        Self { definition: def }
    }

    /// 创建 RootBeanDefinition 构建器。
    ///
    /// 对应 Spring 的 `BeanDefinitionBuilder.rootBeanDefinition(Class<?> beanClass)`。
    pub fn root(bean_class_name: impl Into<String>) -> RootBeanDefinitionBuilder {
        RootBeanDefinitionBuilder {
            definition: RootBeanDefinition::new().with_bean_class_name(bean_class_name),
        }
    }

    // ── 流式设置方法 ────────────────────────────────────────────────────

    /// 设置 Bean 类名。
    pub fn set_bean_class_name(mut self, name: impl Into<String>) -> Self {
        self.definition.set_bean_class_name(name);
        self
    }

    /// 设置父 Bean 定义名称。
    pub fn set_parent_name(mut self, name: impl Into<String>) -> Self {
        self.definition.set_parent_name(name);
        self
    }

    /// 设置作用域。
    pub fn set_scope(mut self, scope: Scope) -> Self {
        self.definition.set_scope(scope);
        self
    }

    /// 设置惰性初始化。
    pub fn set_lazy_init(mut self, lazy: bool) -> Self {
        self.definition.set_lazy_init(lazy);
        self
    }

    /// 设置 abstract 标志。
    pub fn set_abstract(mut self, abstract_flag: bool) -> Self {
        self.definition.set_abstract(abstract_flag);
        self
    }

    /// 设置 autowire candidate。
    pub fn set_autowire_candidate(mut self, candidate: bool) -> Self {
        self.definition.set_autowire_candidate(candidate);
        self
    }

    /// 设置 primary。
    pub fn set_primary(mut self, primary: bool) -> Self {
        self.definition.set_primary(primary);
        self
    }

    /// 设置 fallback。
    pub fn set_fallback(mut self, fallback: bool) -> Self {
        self.definition.set_fallback(fallback);
        self
    }

    /// 设置角色。
    pub fn set_role(mut self, role: i32) -> Self {
        self.definition.set_role(role);
        self
    }

    /// 设置描述。
    pub fn set_description(mut self, description: impl Into<String>) -> Self {
        self.definition.set_description(description);
        self
    }

    /// 添加依赖。
    pub fn add_depends_on(mut self, name: impl Into<String>) -> Self {
        self.definition.add_depends_on(name);
        self
    }

    /// 设置 autowire 模式。
    pub fn set_autowire_mode(mut self, mode: Autowire) -> Self {
        self.definition.set_autowire_mode(mode);
        self
    }

    /// 设置初始化方法名。
    pub fn set_init_method(mut self, name: impl Into<String>) -> Self {
        self.definition.set_init_method_name(name);
        self
    }

    /// 设置销毁方法名。
    pub fn set_destroy_method(mut self, name: impl Into<String>) -> Self {
        self.definition.set_destroy_method_name(name);
        self
    }

    /// 设置工厂 Bean 名称。
    pub fn set_factory_bean_name(mut self, name: impl Into<String>) -> Self {
        self.definition.set_factory_bean_name(name);
        self
    }

    /// 设置工厂方法名。
    pub fn set_factory_method_name(mut self, name: impl Into<String>) -> Self {
        self.definition.set_factory_method_name(name);
        self
    }

    /// 添加按索引的构造参数。
    ///
    /// 对应 Spring 的 `BeanDefinitionBuilder.addConstructorArgValue(Object value)`。
    pub fn add_constructor_arg_value(
        mut self,
        value: impl std::any::Any + Send + Sync + 'static,
    ) -> Self {
        self.definition
            .constructor_argument_values_mut()
            .add_generic_argument_value(ValueHolder::new(std::sync::Arc::new(value)));
        self
    }

    /// 添加按索引的构造参数（带类型名）。
    pub fn add_constructor_arg_typed(
        mut self,
        value: impl std::any::Any + Send + Sync + 'static,
        type_name: impl Into<String>,
    ) -> Self {
        self.definition
            .constructor_argument_values_mut()
            .add_generic_argument_value(ValueHolder::with_type(
                std::sync::Arc::new(value),
                type_name,
            ));
        self
    }

    /// 添加属性值。
    ///
    /// 对应 Spring 的 `BeanDefinitionBuilder.addPropertyValue(String name, Object value)`。
    pub fn add_property_value(
        mut self,
        name: impl Into<String>,
        value: impl std::any::Any + Send + Sync + 'static,
    ) -> Self {
        self.definition
            .property_values_mut()
            .add_value(name, std::sync::Arc::new(value));
        self
    }

    /// 构建 GenericBeanDefinition。
    ///
    /// 对应 Spring 的 `BeanDefinitionBuilder.getBeanDefinition()`。
    pub fn build(self) -> GenericBeanDefinition {
        self.definition
    }

    /// 获取引用（不消耗构建器）。
    pub fn get_definition(&self) -> &GenericBeanDefinition {
        &self.definition
    }
}

/// RootBeanDefinition 构建器。
pub struct RootBeanDefinitionBuilder {
    definition: RootBeanDefinition,
}

impl RootBeanDefinitionBuilder {
    /// 设置作用域。
    pub fn set_scope(mut self, scope: Scope) -> Self {
        self.definition.set_scope(scope);
        self
    }

    /// 设置惰性初始化。
    pub fn set_lazy_init(mut self, lazy: bool) -> Self {
        self.definition.set_lazy_init(lazy);
        self
    }

    /// 设置 primary。
    pub fn set_primary(mut self, primary: bool) -> Self {
        self.definition.set_primary(primary);
        self
    }

    /// 设置初始化方法名。
    pub fn set_init_method(mut self, name: impl Into<String>) -> Self {
        self.definition.set_init_method_name(name);
        self
    }

    /// 设置销毁方法名。
    pub fn set_destroy_method(mut self, name: impl Into<String>) -> Self {
        self.definition.set_destroy_method_name(name);
        self
    }

    /// 添加依赖。
    pub fn add_depends_on(mut self, name: impl Into<String>) -> Self {
        self.definition.add_depends_on(name);
        self
    }

    /// 添加按索引的构造参数。
    pub fn add_constructor_arg_value(
        mut self,
        value: impl std::any::Any + Send + Sync + 'static,
    ) -> Self {
        self.definition
            .get_constructor_argument_values_mut()
            .add_generic_argument_value(ValueHolder::new(std::sync::Arc::new(value)));
        self
    }

    /// 添加属性值。
    pub fn add_property_value(
        mut self,
        name: impl Into<String>,
        value: impl std::any::Any + Send + Sync + 'static,
    ) -> Self {
        self.definition
            .get_property_values_mut()
            .add_value(name, std::sync::Arc::new(value));
        self
    }

    /// 构建 RootBeanDefinition。
    pub fn build(self) -> RootBeanDefinition {
        self.definition
    }
}

// ── RootBeanDefinition 流式扩展 ──────────────────────────────────────────

/// RootBeanDefinition 的流式设置扩展。
impl RootBeanDefinition {
    /// 流式设置 Bean 类名。
    pub fn with_bean_class_name(mut self, name: impl Into<String>) -> Self {
        self.set_bean_class_name(name);
        self
    }

    /// 流式设置作用域。
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.set_scope(scope);
        self
    }

    /// 流式设置惰性初始化。
    pub fn with_lazy_init(mut self, lazy: bool) -> Self {
        self.set_lazy_init(lazy);
        self
    }

    /// 流式设置 primary。
    pub fn with_primary(mut self, primary: bool) -> Self {
        self.set_primary(primary);
        self
    }

    /// 流式设置初始化方法。
    pub fn with_init_method(mut self, name: impl Into<String>) -> Self {
        self.set_init_method_name(name);
        self
    }

    /// 流式设置销毁方法。
    pub fn with_destroy_method(mut self, name: impl Into<String>) -> Self {
        self.set_destroy_method_name(name);
        self
    }

    /// 流式设置初始化排序值。
    pub fn with_init_order(mut self, order: i32) -> Self {
        self.set_init_order(order);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_builder_basic() {
        let bd = BeanDefinitionBuilder::generic("MyService").build();
        assert_eq!(bd.get_bean_class_name(), Some("MyService"));
    }

    #[test]
    fn generic_builder_with_scope() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_scope(Scope::Transient)
            .build();
        assert_eq!(bd.scope(), Scope::Transient);
    }

    #[test]
    fn generic_builder_with_lazy_init() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_lazy_init(true)
            .build();
        assert!(bd.is_lazy_init());
    }

    #[test]
    fn generic_builder_with_primary() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_primary(true)
            .build();
        assert!(bd.is_primary());
    }

    #[test]
    fn generic_builder_with_abstract() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_abstract(true)
            .build();
        assert!(bd.is_abstract());
    }

    #[test]
    fn generic_builder_with_autowire_candidate() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_autowire_candidate(false)
            .build();
        assert!(!bd.is_autowire_candidate());
    }

    #[test]
    fn generic_builder_with_fallback() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_fallback(true)
            .build();
        assert!(bd.is_fallback());
    }

    #[test]
    fn generic_builder_with_role() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_role(crate::factory::config::bean_definition::ROLE_SUPPORT)
            .build();
        assert_eq!(
            bd.role(),
            crate::factory::config::bean_definition::ROLE_SUPPORT
        );
    }

    #[test]
    fn generic_builder_with_description() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_description("A test service")
            .build();
        assert_eq!(bd.description(), Some("A test service"));
    }

    #[test]
    fn generic_builder_with_parent_name() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_parent_name("ParentBean")
            .build();
        assert_eq!(bd.get_parent_name(), Some("ParentBean"));
    }

    #[test]
    fn generic_builder_with_depends_on() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .add_depends_on("DataSource")
            .add_depends_on("CacheManager")
            .build();
        assert!(bd.depends_on().contains(&"DataSource".to_string()));
        assert!(bd.depends_on().contains(&"CacheManager".to_string()));
    }

    #[test]
    fn generic_builder_with_init_method() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_init_method("init")
            .build();
        assert_eq!(bd.init_method_name(), Some("init"));
    }

    #[test]
    fn generic_builder_with_destroy_method() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_destroy_method("cleanup")
            .build();
        assert_eq!(bd.destroy_method_name(), Some("cleanup"));
    }

    #[test]
    fn generic_builder_with_factory() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_factory_bean_name("myFactory")
            .set_factory_method_name("create")
            .build();
        assert_eq!(bd.factory_bean_name(), Some("myFactory"));
        assert_eq!(bd.factory_method_name(), Some("create"));
    }

    #[test]
    fn generic_builder_with_constructor_arg() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .add_constructor_arg_value(42i32)
            .add_constructor_arg_typed("hello".to_string(), "String")
            .build();
        assert!(bd.constructor_argument_values().argument_count() >= 2);
    }

    #[test]
    fn generic_builder_with_property_value() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .add_property_value("name", "test".to_string())
            .build();
        assert!(bd.property_values().contains("name"));
    }

    #[test]
    fn generic_builder_fluent_chaining() {
        let bd = BeanDefinitionBuilder::generic("MyService")
            .set_scope(Scope::Singleton)
            .set_lazy_init(false)
            .set_primary(true)
            .set_role(crate::factory::config::bean_definition::ROLE_APPLICATION)
            .set_description("Main service")
            .add_depends_on("DB")
            .set_init_method("start")
            .set_destroy_method("stop")
            .build();

        assert_eq!(bd.get_bean_class_name(), Some("MyService"));
        assert_eq!(bd.scope(), Scope::Singleton);
        assert!(!bd.is_lazy_init());
        assert!(bd.is_primary());
        assert_eq!(
            bd.role(),
            crate::factory::config::bean_definition::ROLE_APPLICATION
        );
    }

    #[test]
    fn generic_builder_get_definition() {
        let builder = BeanDefinitionBuilder::generic("MyService");
        let bd = builder.get_definition();
        assert_eq!(bd.get_bean_class_name(), Some("MyService"));
    }

    #[test]
    fn root_builder_basic() {
        let bd = BeanDefinitionBuilder::root("MyService").build();
        assert_eq!(bd.bean_class_name(), "MyService");
    }

    #[test]
    fn root_builder_with_scope() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .set_scope(Scope::Transient)
            .build();
        assert_eq!(bd.scope(), Scope::Transient);
    }

    #[test]
    fn root_builder_with_lazy_init() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .set_lazy_init(true)
            .build();
        assert!(bd.is_lazy_init());
    }

    #[test]
    fn root_builder_with_primary() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .set_primary(true)
            .build();
        assert!(bd.is_primary());
    }

    #[test]
    fn root_builder_with_init_and_destroy() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .set_init_method("init")
            .set_destroy_method("destroy")
            .build();
        assert_eq!(bd.init_method_name(), Some("init"));
        assert_eq!(bd.destroy_method_name(), Some("destroy"));
    }

    #[test]
    fn root_builder_with_depends_on() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .add_depends_on("DB")
            .build();
        assert!(bd.depends_on().contains(&"DB".to_string()));
    }

    #[test]
    fn root_builder_with_constructor_arg() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .add_constructor_arg_value(42i32)
            .build();
        assert!(bd.constructor_argument_values().argument_count() >= 1);
    }

    #[test]
    fn root_builder_with_property_value() {
        let bd = BeanDefinitionBuilder::root("MyService")
            .add_property_value("name", "test".to_string())
            .build();
        assert!(bd.property_values().contains("name"));
    }

    #[test]
    fn root_definition_fluent_methods() {
        let bd = RootBeanDefinition::new()
            .with_bean_class_name("TestBean")
            .with_scope(Scope::Singleton)
            .with_lazy_init(false)
            .with_primary(true)
            .with_init_method("init")
            .with_destroy_method("destroy")
            .with_init_order(10);

        assert_eq!(bd.bean_class_name(), "TestBean");
        assert_eq!(bd.scope(), Scope::Singleton);
        assert!(!bd.is_lazy_init());
        assert!(bd.is_primary());
        assert_eq!(bd.init_method_name(), Some("init"));
        assert_eq!(bd.destroy_method_name(), Some("destroy"));
    }
}

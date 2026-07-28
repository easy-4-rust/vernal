//! GenericBeanDefinition — Spring 风格的通用 Bean 定义。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.GenericBeanDefinition`。
//!
//! 通用的 Bean 定义实现，支持 Parent-Child 继承。
//! 与 RootBeanDefinition 不同，GenericBeanDefinition 可以有父级。

use crate::autowire::Autowire;
use crate::bean_definition;
use crate::component_scope::Scope;
use crate::constructor_argument_values::ConstructorArgumentValues;
use crate::mutable_property_values::MutablePropertyValues;

/// Spring 风格的通用 Bean 定义。
///
/// 对应 Spring 的 `GenericBeanDefinition`。
///
/// 通用的 Bean 定义实现，支持 Parent-Child 继承：
/// - 可以设置 parent_name 指向父 Bean 定义
/// - 子定义的属性覆盖父定义的属性
/// - 合并后生成 RootBeanDefinition
///
/// ## 与 RootBeanDefinition 的区别
///
/// - `GenericBeanDefinition` 可以有父级，用于继承场景
/// - `RootBeanDefinition` 是合并后的最终定义，无父级
/// - Spring 内部在 `getMergedBeanDefinition` 时将 Generic 合并为 Root
///
/// ## 使用场景
///
/// - XML/注解解析后创建的 Bean 定义（支持继承）
/// - `BeanDefinitionBuilder.build()` 的默认返回值
#[derive(Clone, Debug)]
pub struct GenericBeanDefinition {
    /// Bean 类名。
    pub(crate) bean_class_name: Option<String>,
    /// 父 Bean 定义名称。
    pub(crate) parent_name: Option<String>,
    /// 作用域。
    pub(crate) scope: Scope,
    /// 是否惰性初始化。
    pub(crate) lazy_init: bool,
    /// 是否 abstract。
    pub(crate) abstract_flag: bool,
    /// 是否 autowire candidate。
    pub(crate) autowire_candidate: bool,
    /// 是否 primary。
    pub(crate) primary: bool,
    /// 是否 fallback。
    pub(crate) fallback: bool,
    /// 是否 synthetic。
    pub(crate) synthetic: bool,
    /// 角色。
    pub(crate) role: i32,
    /// 描述。
    pub(crate) description: Option<String>,
    /// 依赖列表。
    pub(crate) depends_on: Vec<String>,
    /// autowire 模式。
    pub(crate) autowire_mode: Autowire,
    /// 初始化方法名。
    pub(crate) init_method_name: Option<String>,
    /// 销毁方法名。
    pub(crate) destroy_method_name: Option<String>,
    /// 工厂 Bean 名称。
    pub(crate) factory_bean_name: Option<String>,
    /// 工厂方法名。
    pub(crate) factory_method_name: Option<String>,
    /// 构造参数值。
    pub(crate) constructor_argument_values: ConstructorArgumentValues,
    /// 属性值。
    pub(crate) property_values: MutablePropertyValues,
}

impl GenericBeanDefinition {
    /// 创建一个新的 GenericBeanDefinition。
    pub fn new() -> Self {
        Self {
            bean_class_name: None,
            parent_name: None,
            scope: Scope::Singleton,
            lazy_init: false,
            abstract_flag: false,
            autowire_candidate: true,
            primary: false,
            fallback: false,
            synthetic: false,
            role: bean_definition::ROLE_APPLICATION,
            description: None,
            depends_on: Vec::new(),
            autowire_mode: Autowire::No,
            init_method_name: None,
            destroy_method_name: None,
            factory_bean_name: None,
            factory_method_name: None,
            constructor_argument_values: ConstructorArgumentValues::new(),
            property_values: MutablePropertyValues::new(),
        }
    }

    /// 从 RootBeanDefinition 创建（降级为可继承版本）。
    pub fn from_root(root: &super::root_bean_definition::RootBeanDefinition) -> Self {
        Self {
            bean_class_name: Some(root.bean_class_name().to_string()),
            parent_name: root.parent_name().map(String::from),
            scope: root.scope(),
            lazy_init: root.is_lazy_init(),
            abstract_flag: root.is_abstract(),
            autowire_candidate: root.is_autowire_candidate(),
            primary: root.is_primary(),
            fallback: root.is_fallback(),
            synthetic: false,
            role: root.role(),
            description: root.description().map(String::from),
            depends_on: root.depends_on().to_vec(),
            autowire_mode: root.autowire_mode(),
            init_method_name: root.init_method_name().map(String::from),
            destroy_method_name: root.destroy_method_name().map(String::from),
            factory_bean_name: root.factory_bean_name().map(String::from),
            factory_method_name: root.factory_method_name().map(String::from),
            constructor_argument_values: root.constructor_argument_values().clone(),
            property_values: root.property_values().clone(),
        }
    }

    // ── Builder 方法 ────────────────────────────────────────────────────

    /// 设置 Bean 类名。
    pub fn set_bean_class_name(&mut self, name: impl Into<String>) {
        self.bean_class_name = Some(name.into());
    }

    /// 设置父 Bean 定义名称。
    pub fn set_parent_name(&mut self, name: impl Into<String>) {
        self.parent_name = Some(name.into());
    }

    /// 设置作用域。
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// 设置惰性初始化。
    pub fn set_lazy_init(&mut self, lazy: bool) {
        self.lazy_init = lazy;
    }

    /// 设置 abstract 标志。
    pub fn set_abstract(&mut self, abstract_flag: bool) {
        self.abstract_flag = abstract_flag;
    }

    /// 设置 autowire candidate。
    pub fn set_autowire_candidate(&mut self, candidate: bool) {
        self.autowire_candidate = candidate;
    }

    /// 设置 primary。
    pub fn set_primary(&mut self, primary: bool) {
        self.primary = primary;
    }

    /// 设置 fallback。
    pub fn set_fallback(&mut self, fallback: bool) {
        self.fallback = fallback;
    }

    /// 设置 synthetic 标志。
    pub fn set_synthetic(&mut self, synthetic: bool) {
        self.synthetic = synthetic;
    }

    /// 设置角色。
    pub fn set_role(&mut self, role: i32) {
        self.role = role;
    }

    /// 设置描述。
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// 添加依赖。
    pub fn add_depends_on(&mut self, name: impl Into<String>) {
        self.depends_on.push(name.into());
    }

    /// 设置 autowire 模式。
    pub fn set_autowire_mode(&mut self, mode: Autowire) {
        self.autowire_mode = mode;
    }

    /// 设置初始化方法名。
    pub fn set_init_method_name(&mut self, name: impl Into<String>) {
        self.init_method_name = Some(name.into());
    }

    /// 设置销毁方法名。
    pub fn set_destroy_method_name(&mut self, name: impl Into<String>) {
        self.destroy_method_name = Some(name.into());
    }

    /// 设置工厂 Bean 名称。
    pub fn set_factory_bean_name(&mut self, name: impl Into<String>) {
        self.factory_bean_name = Some(name.into());
    }

    /// 设置工厂方法名。
    pub fn set_factory_method_name(&mut self, name: impl Into<String>) {
        self.factory_method_name = Some(name.into());
    }

    /// 设置初始化排序值。
    pub fn set_init_order(&mut self, _order: i32) {
        // GenericBeanDefinition 暂不存储 init_order（由 RootBeanDefinition 管理）
    }

    /// 添加属性值。
    pub fn add_property_value(
        &mut self,
        name: impl Into<String>,
        value: impl std::any::Any + Send + Sync + 'static,
    ) {
        self.property_values
            .add_value(name, std::sync::Arc::new(value));
    }

    /// 获取构造参数值（可变引用）。
    pub fn constructor_argument_values_mut(&mut self) -> &mut ConstructorArgumentValues {
        &mut self.constructor_argument_values
    }

    /// 获取属性值（可变引用）。
    pub fn property_values_mut(&mut self) -> &mut MutablePropertyValues {
        &mut self.property_values
    }

    // ── Getter 方法 ─────────────────────────────────────────────────────

    /// 获取构造参数值。
    pub fn constructor_argument_values(&self) -> &ConstructorArgumentValues {
        &self.constructor_argument_values
    }

    /// 获取属性值。
    pub fn property_values(&self) -> &MutablePropertyValues {
        &self.property_values
    }

    /// 是否 abstract。
    pub fn is_abstract(&self) -> bool {
        self.abstract_flag
    }

    /// 是否 synthetic。
    pub fn is_synthetic(&self) -> bool {
        self.synthetic
    }

    /// 获取 autowire 模式。
    pub fn autowire_mode(&self) -> Autowire {
        self.autowire_mode
    }

    /// 获取依赖列表。
    pub fn depends_on(&self) -> &[String] {
        &self.depends_on
    }

    /// 获取父 Bean 定义名称。
    pub fn get_parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    /// 获取 Bean 类名。
    pub fn get_bean_class_name(&self) -> Option<&str> {
        self.bean_class_name.as_deref()
    }

    /// 获取作用域。
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// 是否惰性初始化。
    pub fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    /// 是否 primary。
    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// 是否 fallback。
    pub fn is_fallback(&self) -> bool {
        self.fallback
    }

    /// 是否 autowire candidate。
    pub fn is_autowire_candidate(&self) -> bool {
        self.autowire_candidate
    }

    /// 获取角色。
    pub fn role(&self) -> i32 {
        self.role
    }

    /// 获取描述。
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// 获取初始化方法名。
    pub fn init_method_name(&self) -> Option<&str> {
        self.init_method_name.as_deref()
    }

    /// 获取销毁方法名。
    pub fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    /// 获取工厂 Bean 名称。
    pub fn factory_bean_name(&self) -> Option<&str> {
        self.factory_bean_name.as_deref()
    }

    /// 获取工厂方法名。
    pub fn factory_method_name(&self) -> Option<&str> {
        self.factory_method_name.as_deref()
    }
}

impl Default for GenericBeanDefinition {
    fn default() -> Self {
        Self::new()
    }
}

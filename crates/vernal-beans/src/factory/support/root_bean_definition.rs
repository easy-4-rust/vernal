//! RootBeanDefinition — Spring 风格的根 Bean 定义。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.RootBeanDefinition`。
//!
//! 表示一个完整的、无父级的 Bean 定义。这是 Bean 定义层次结构中最常用的类型，
//! 包含所有 Bean 元数据：作用域、依赖、初始化/销毁方法、构造参数等。

use crate::factory::config::bean_definition::{self, BeanDefinition};
use crate::component_scope::Scope;
use crate::factory::config::constructor_argument_values::ConstructorArgumentValues;
use crate::mutable_property_values::MutablePropertyValues;

/// Spring 风格的根 Bean 定义。
///
/// 对应 Spring 的 `RootBeanDefinition`。
///
/// 表示一个完整的、无父级的 Bean 定义。包含：
/// - Bean 类名
/// - 作用域（singleton/prototype）
/// - 惰性初始化标志
/// - 依赖列表
/// - 初始化/销毁方法名
/// - 构造参数
/// - 属性值
/// - autowire 模式
/// - primary / fallback 标志
///
/// ## 与 Parent-Child 的关系
///
/// `RootBeanDefinition` 是合并后的最终 Bean 定义。当有 Parent-Child 继承时，
/// `ChildBeanDefinition` 的属性会覆盖 `RootBeanDefinition` 的属性，
/// 最终生成一个新的 `RootBeanDefinition`。
///
/// ## 使用场景
///
/// - XML/注解解析后生成的 Bean 定义
/// - `BeanDefinitionBuilder.build()` 的返回值
/// - `BeanFactory.getMergedBeanDefinition()` 的返回值
#[derive(Clone, Debug)]
pub struct RootBeanDefinition {
    /// Bean 类名。
    bean_class_name: Option<String>,
    /// 父 Bean 定义名称（无父级时为 None）。
    parent_name: Option<String>,
    /// 作用域。
    scope: Scope,
    /// 是否惰性初始化。
    lazy_init: bool,
    /// 是否 abstract（不能直接实例化）。
    abstract_flag: bool,
    /// 是否 autowire candidate。
    autowire_candidate: bool,
    /// 是否 primary。
    primary: bool,
    /// 是否 fallback（6.2+）。
    fallback: bool,
    /// 是否 synthetic（框架内部）。
    synthetic: bool,
    /// 角色（APPLICATION/SUPPORT/INFRASTRUCTURE）。
    role: i32,
    /// 描述信息。
    description: Option<String>,
    /// 依赖列表。
    depends_on: Vec<String>,
    /// autowire 模式。
    autowire_mode: crate::factory::annotation::autowire::Autowire,
    /// 初始化方法名。
    init_method_name: Option<String>,
    /// 销毁方法名。
    destroy_method_name: Option<String>,
    /// 工厂 Bean 名称。
    factory_bean_name: Option<String>,
    /// 工厂方法名。
    factory_method_name: Option<String>,
    /// 构造参数值。
    constructor_argument_values: ConstructorArgumentValues,
    /// 属性值。
    property_values: MutablePropertyValues,
    /// 初始化排序值。
    init_order: i32,
}

impl RootBeanDefinition {
    /// 创建一个新的 RootBeanDefinition。
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
            autowire_mode: crate::factory::annotation::autowire::Autowire::No,
            init_method_name: None,
            destroy_method_name: None,
            factory_bean_name: None,
            factory_method_name: None,
            constructor_argument_values: ConstructorArgumentValues::new(),
            property_values: MutablePropertyValues::new(),
            init_order: 0,
        }
    }

    /// 从 GenericBeanDefinition 创建（合并后）。
    pub fn from_generic(generic: GenericBeanDefinition) -> Self {
        Self {
            bean_class_name: generic.bean_class_name,
            parent_name: generic.parent_name,
            scope: generic.scope,
            lazy_init: generic.lazy_init,
            abstract_flag: generic.abstract_flag,
            autowire_candidate: generic.autowire_candidate,
            primary: generic.primary,
            fallback: generic.fallback,
            synthetic: generic.synthetic,
            role: generic.role,
            description: generic.description,
            depends_on: generic.depends_on,
            autowire_mode: generic.autowire_mode,
            init_method_name: generic.init_method_name,
            destroy_method_name: generic.destroy_method_name,
            factory_bean_name: generic.factory_bean_name,
            factory_method_name: generic.factory_method_name,
            constructor_argument_values: generic.constructor_argument_values,
            property_values: generic.property_values,
            init_order: 0,
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
    pub fn set_autowire_mode(&mut self, mode: crate::factory::annotation::autowire::Autowire) {
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

    /// 获取构造参数值（可变引用）。
    pub fn get_constructor_argument_values_mut(&mut self) -> &mut ConstructorArgumentValues {
        &mut self.constructor_argument_values
    }

    /// 获取属性值（可变引用）。
    pub fn get_property_values_mut(&mut self) -> &mut MutablePropertyValues {
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
    pub fn autowire_mode(&self) -> crate::factory::annotation::autowire::Autowire {
        self.autowire_mode
    }

    /// 获取依赖列表。
    pub fn depends_on(&self) -> &[String] {
        &self.depends_on
    }

    /// 获取 Bean 类名。
    pub fn bean_class_name(&self) -> &str {
        self.bean_class_name.as_deref().unwrap_or("unknown")
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

    /// 获取父 Bean 定义名称。
    pub fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    /// 获取初始化排序值。
    pub fn init_order_value(&self) -> i32 {
        self.init_order
    }

    /// 设置初始化排序值。
    pub fn set_init_order(&mut self, order: i32) {
        self.init_order = order;
    }
}

impl Default for RootBeanDefinition {
    fn default() -> Self {
        Self::new()
    }
}

/// BeanDefinition trait 实现。
impl BeanDefinition for RootBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        // RootBeanDefinition 不持有 ComponentKey（它是纯 Bean 定义）
        // 这里用一个 static 占位，实际使用时通过 Container 的 key 管理
        // 注意：这是对 Spring 语义的适配——Spring 的 BeanDefinition 也不持有 key
        unimplemented!(
            "RootBeanDefinition does not hold ComponentKey; use Container's key management"
        )
    }

    fn bean_class_name(&self) -> &str {
        self.bean_class_name.as_deref().unwrap_or("unknown")
    }

    fn scope(&self) -> Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    fn is_primary(&self) -> bool {
        self.primary
    }

    fn is_fallback(&self) -> bool {
        self.fallback
    }

    fn is_autowire_candidate(&self) -> bool {
        self.autowire_candidate
    }

    fn role(&self) -> i32 {
        self.role
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    fn factory_bean_name(&self) -> Option<&str> {
        self.factory_bean_name.as_deref()
    }

    fn factory_method_name(&self) -> Option<&str> {
        self.factory_method_name.as_deref()
    }

    fn init_method_name(&self) -> Option<&str> {
        self.init_method_name.as_deref()
    }

    fn destroy_method_name(&self) -> Option<&str> {
        self.destroy_method_name.as_deref()
    }

    fn is_abstract(&self) -> bool {
        self.abstract_flag
    }
}

use super::generic_bean_definition::GenericBeanDefinition;

/// 使用 GenericBeanDefinition 创建 RootBeanDefinition 的便捷方法。
impl From<GenericBeanDefinition> for RootBeanDefinition {
    fn from(generic: GenericBeanDefinition) -> Self {
        Self::from_generic(generic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::annotation::autowire::Autowire;

    #[test]
    fn test_new_defaults() {
        let def = RootBeanDefinition::new();
        assert_eq!(def.bean_class_name, None);
        assert_eq!(def.parent_name, None);
        assert_eq!(def.scope, Scope::Singleton);
        assert!(!def.lazy_init);
        assert!(!def.abstract_flag);
        assert!(def.autowire_candidate);
        assert!(!def.primary);
        assert!(!def.fallback);
        assert!(!def.synthetic);
        assert_eq!(def.role, bean_definition::ROLE_APPLICATION);
        assert_eq!(def.description, None);
        assert!(def.depends_on.is_empty());
        assert_eq!(def.autowire_mode, Autowire::No);
        assert_eq!(def.init_method_name, None);
        assert_eq!(def.destroy_method_name, None);
        assert_eq!(def.factory_bean_name, None);
        assert_eq!(def.factory_method_name, None);
        assert_eq!(def.init_order, 0);
    }

    #[test]
    fn test_default_trait() {
        let def = RootBeanDefinition::default();
        assert_eq!(def.scope, Scope::Singleton);
    }

    #[test]
    fn test_set_bean_class_name() {
        let mut def = RootBeanDefinition::new();
        def.set_bean_class_name("com.example.MyBean");
        assert_eq!(def.bean_class_name(), "com.example.MyBean");
    }

    #[test]
    fn test_bean_class_name_default() {
        let def = RootBeanDefinition::new();
        assert_eq!(def.bean_class_name(), "unknown");
    }

    #[test]
    fn test_set_parent_name() {
        let mut def = RootBeanDefinition::new();
        def.set_parent_name("parentBean");
        assert_eq!(def.parent_name(), Some("parentBean"));
    }

    #[test]
    fn test_parent_name_default() {
        let def = RootBeanDefinition::new();
        assert_eq!(def.parent_name(), None);
    }

    #[test]
    fn test_set_scope() {
        let mut def = RootBeanDefinition::new();
        def.set_scope(Scope::Transient);
        assert_eq!(def.scope(), Scope::Transient);
    }

    #[test]
    fn test_set_lazy_init() {
        let mut def = RootBeanDefinition::new();
        def.set_lazy_init(true);
        assert!(def.is_lazy_init());
    }

    #[test]
    fn test_set_abstract() {
        let mut def = RootBeanDefinition::new();
        def.set_abstract(true);
        assert!(def.is_abstract());
    }

    #[test]
    fn test_set_autowire_candidate() {
        let mut def = RootBeanDefinition::new();
        def.set_autowire_candidate(false);
        assert!(!def.is_autowire_candidate());
    }

    #[test]
    fn test_set_primary() {
        let mut def = RootBeanDefinition::new();
        def.set_primary(true);
        assert!(def.is_primary());
    }

    #[test]
    fn test_set_fallback() {
        let mut def = RootBeanDefinition::new();
        def.set_fallback(true);
        assert!(def.is_fallback());
    }

    #[test]
    fn test_set_synthetic() {
        let mut def = RootBeanDefinition::new();
        def.set_synthetic(true);
        assert!(def.is_synthetic());
    }

    #[test]
    fn test_set_role() {
        let mut def = RootBeanDefinition::new();
        def.set_role(2);
        assert_eq!(def.role(), 2);
    }

    #[test]
    fn test_set_description() {
        let mut def = RootBeanDefinition::new();
        def.set_description("A test bean");
        assert_eq!(def.description(), Some("A test bean"));
    }

    #[test]
    fn test_add_depends_on() {
        let mut def = RootBeanDefinition::new();
        def.add_depends_on("bean1");
        def.add_depends_on("bean2");
        assert_eq!(def.depends_on(), &["bean1", "bean2"]);
    }

    #[test]
    fn test_set_autowire_mode() {
        let mut def = RootBeanDefinition::new();
        def.set_autowire_mode(Autowire::ByName);
        assert_eq!(def.autowire_mode(), Autowire::ByName);
    }

    #[test]
    fn test_set_init_method_name() {
        let mut def = RootBeanDefinition::new();
        def.set_init_method_name("init");
        assert_eq!(def.init_method_name(), Some("init"));
    }

    #[test]
    fn test_set_destroy_method_name() {
        let mut def = RootBeanDefinition::new();
        def.set_destroy_method_name("destroy");
        assert_eq!(def.destroy_method_name(), Some("destroy"));
    }

    #[test]
    fn test_set_factory_bean_name() {
        let mut def = RootBeanDefinition::new();
        def.set_factory_bean_name("myFactory");
        assert_eq!(def.factory_bean_name(), Some("myFactory"));
    }

    #[test]
    fn test_set_factory_method_name() {
        let mut def = RootBeanDefinition::new();
        def.set_factory_method_name("createBean");
        assert_eq!(def.factory_method_name(), Some("createBean"));
    }

    #[test]
    fn test_set_init_order() {
        let mut def = RootBeanDefinition::new();
        def.set_init_order(10);
        assert_eq!(def.init_order_value(), 10);
    }

    #[test]
    fn test_constructor_argument_values_mut() {
        let mut def = RootBeanDefinition::new();
        let cav = def.get_constructor_argument_values_mut();
        assert!(cav.is_empty());
    }

    #[test]
    fn test_property_values_mut() {
        let mut def = RootBeanDefinition::new();
        let pv = def.get_property_values_mut();
        assert!(pv.is_empty());
    }

    #[test]
    fn test_from_generic() {
        let mut generic = GenericBeanDefinition::new();
        generic.set_bean_class_name("com.example.Generic");
        generic.set_parent_name("parent");
        generic.set_scope(Scope::Transient);
        generic.set_lazy_init(true);
        generic.set_abstract(true);
        generic.set_autowire_candidate(false);
        generic.set_primary(true);
        generic.set_fallback(true);
        generic.set_synthetic(true);
        generic.set_role(1);
        generic.set_description("generic desc");
        generic.add_depends_on("dep1");
        generic.set_autowire_mode(Autowire::ByType);
        generic.set_init_method_name("init");
        generic.set_destroy_method_name("destroy");
        generic.set_factory_bean_name("factory");
        generic.set_factory_method_name("create");

        let root = RootBeanDefinition::from_generic(generic);
        assert_eq!(root.bean_class_name(), "com.example.Generic");
        assert_eq!(root.parent_name(), Some("parent"));
        assert_eq!(root.scope(), Scope::Transient);
        assert!(root.is_lazy_init());
        assert!(root.is_abstract());
        assert!(!root.is_autowire_candidate());
        assert!(root.is_primary());
        assert!(root.is_fallback());
        assert!(root.is_synthetic());
        assert_eq!(root.role(), 1);
        assert_eq!(root.description(), Some("generic desc"));
        assert_eq!(root.depends_on(), &["dep1"]);
        assert_eq!(root.autowire_mode(), Autowire::ByType);
        assert_eq!(root.init_method_name(), Some("init"));
        assert_eq!(root.destroy_method_name(), Some("destroy"));
        assert_eq!(root.factory_bean_name(), Some("factory"));
        assert_eq!(root.factory_method_name(), Some("create"));
        assert_eq!(root.init_order_value(), 0);
    }

    #[test]
    fn test_from_trait() {
        let mut generic = GenericBeanDefinition::new();
        generic.set_bean_class_name("com.example.FromTrait");
        let root = RootBeanDefinition::from(generic);
        assert_eq!(root.bean_class_name(), "com.example.FromTrait");
    }

    #[test]
    fn test_getters_before_set() {
        let def = RootBeanDefinition::new();
        assert_eq!(def.description(), None);
        assert_eq!(def.init_method_name(), None);
        assert_eq!(def.destroy_method_name(), None);
        assert_eq!(def.factory_bean_name(), None);
        assert_eq!(def.factory_method_name(), None);
    }
}

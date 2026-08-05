//! factory — 对应 Spring beans.factory 包根模块。
//!
//! 包含 factory 子包：
//! - support: Bean 工厂支持实现
//! - config: Bean 工厂配置接口
//! - annotation: 注解驱动配置
//! - aot: Ahead-of-Time 处理
//! - parsing: Bean 定义解析
//! - xml: XML 配置解析
//! - wiring: Bean 装配信息
//! - serviceloader: ServiceLoader 集成
//! - groovy: Groovy DSL 集成

pub mod annotation;
pub mod aot;
pub mod config;
pub mod groovy;
pub mod parsing;
pub mod serviceloader;
pub mod support;
pub mod wiring;
pub mod xml;

// ── factory 根模块对象（从根 src/ 移入） ─────────────────────────────

/// Spring Aware 标记接口。
pub mod aware;

/// Spring BeanFactory 顶层客户端视图。
pub mod bean_factory;

/// Spring BeanFactoryAware 接口。
pub mod bean_factory_aware;

/// Spring BeanFactoryUtils 工具类。
pub mod bean_factory_utils;

/// Bean 定义为 abstract 时抛出的异常。
pub mod bean_is_abstract_exception;

/// Bean 不是 FactoryBean 时抛出的异常。
pub mod bean_is_not_a_factory_exception;

/// Spring BeanNameAware 接口。
pub mod bean_name_aware;

/// Spring DisposableBean 接口。
pub mod disposable_bean;

/// Spring FactoryBean 接口。
pub mod factory_bean;

/// FactoryBean 未初始化时抛出的异常。
pub mod factory_bean_not_initialized_exception;

/// Spring HierarchicalBeanFactory 接口。
pub mod hierarchical_bean_factory;

/// Spring InitializingBean 接口。
pub mod initializing_bean;

/// Spring InjectionPoint 类。
pub mod injection_point;

/// Spring ListableBeanFactory 接口。
pub mod listable_bean_factory;

/// Spring ObjectProvider 接口。
pub mod object_provider;

/// Spring SmartInitializingSingleton 接口。
pub mod smart_initializing_singleton;

/// Spring SmartFactoryBean 接口。
pub mod smart_factory_bean;

/// Spring NamedBean 接口。
pub mod named_bean;

/// Spring BeanRegistry 接口。
pub mod bean_registry;

/// Spring BeanRegistrar 接口。
pub mod bean_registrar;

/// Spring BeanFactoryInitializer 接口。
pub mod bean_factory_initializer;

/// Spring ObjectFactory 接口。
pub mod object_factory;

/// Bean 类加载器感知管理器。
pub mod bean_class_loader_aware;

// ── 新增工厂异常类型 ───────────────────────────────────────────────

/// Bean 创建失败时抛出的异常。
pub mod bean_creation_exception;

/// Bean 创建被禁止时抛出的异常。
pub mod bean_creation_not_allowed_exception;

/// Bean 正在创建中（循环依赖）时抛出的异常。
pub mod bean_currently_in_creation_exception;

/// Bean 定义存储异常。
pub mod bean_definition_store_exception;

/// Bean 表达式求值失败时抛出的异常。
pub mod bean_expression_exception;

/// Bean 初始化失败时抛出的异常。
pub mod bean_initialization_exception;

/// Bean 类型不匹配时抛出的异常。
pub mod bean_not_of_required_type_exception;

/// 无法加载 Bean 类时抛出的异常。
pub mod cannot_load_bean_class_exception;

/// 没有找到 Bean 定义时抛出的异常。
pub mod no_such_bean_definition_exception;

/// 当有多个 Bean 匹配但期望唯一时抛出的异常。
pub mod no_unique_bean_definition_exception;

/// 依赖注入失败时抛出的异常。
pub mod unsatisfied_dependency_exception;

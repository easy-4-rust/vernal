//! factory::support — 对应 Spring beans.factory.support 包。

pub mod abstract_autowire_capable_bean_factory;
pub mod abstract_bean_definition;
pub mod abstract_bean_definition_reader;
pub mod abstract_bean_factory;
pub mod autowire_candidate_qualifier;
pub mod autowire_candidate_resolver;
pub mod autowire_utils;
pub mod bean_definition_builder;
pub mod bean_definition_defaults;
pub mod bean_definition_override_exception;
pub mod bean_definition_reader;
pub mod bean_definition_reader_utils;
pub mod bean_definition_registry;
pub mod bean_definition_registry_post_processor;
pub mod bean_definition_resource;
pub mod bean_definition_validation_exception;
pub mod bean_definition_value_resolver;
pub mod bean_name_generator;
pub mod bean_registry_adapter;
pub mod child_bean_definition;
pub mod constructor_resolver;
pub mod default_bean_name_generator;
pub mod default_listable_bean_factory;
pub mod default_singleton_bean_registry;
pub mod dependency_descriptor;
pub mod disposable_bean_adapter;
pub mod factory_bean_registry_support;
pub mod generic_bean_definition;
pub mod generic_type_aware_autowire_candidate_resolver;
pub mod instantiation_strategy;
pub mod lookup_override;
pub mod managed_array;
pub mod managed_list;
pub mod managed_map;
pub mod managed_properties;
pub mod managed_set;
pub mod method_descriptor;
pub mod method_override;
pub mod method_overrides;
pub mod replace_override;
pub mod root_bean_definition;
pub mod scope_not_active_exception;
pub mod simple_bean_definition_registry;
pub mod simple_instantiation_strategy;
pub mod static_listable_bean_factory;

// ── 新增 support 子模块 ────────────────────────────────────────────

/// CGLIB 子类实例化策略。
pub mod cglib_subclassing_instantiation_strategy;

/// 隐式单例异常。
pub mod implicitly_appeared_singleton_exception;

/// 实例供应器接口。
pub mod instance_supplier;

/// 合并 Bean 定义后处理器。
pub mod merged_bean_definition_post_processor;

/// 方法替换器接口。
pub mod method_replacer;

/// 空 Bean 标记。
pub mod null_bean;

/// Properties Bean 定义读取器。
pub mod properties_bean_definition_reader;

/// 已注册 Bean 句柄。
pub mod registered_bean;

/// 简单自动装配候选解析器。
pub mod simple_autowire_candidate_resolver;

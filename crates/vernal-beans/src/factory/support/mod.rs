//! factory::support — 对应 Spring beans.factory.support 包。

pub mod abstract_autowire_capable_bean_factory;
pub mod abstract_bean_definition_reader;
pub mod abstract_bean_factory;
pub mod autowire_candidate_qualifier;
pub mod autowire_candidate_resolver;
pub mod autowire_utils;
pub mod bean_definition_defaults;
pub mod bean_definition_reader;
pub mod bean_definition_reader_utils;
pub mod bean_definition_resource;
pub mod bean_definition_value_resolver;
pub mod bean_name_generator;
pub mod bean_registry_adapter;
pub mod default_bean_name_generator;
pub mod default_listable_bean_factory;
pub mod dependency_descriptor;
pub mod disposable_bean_adapter;
pub mod generic_type_aware_autowire_candidate_resolver;
pub mod managed_array;
pub mod managed_list;
pub mod managed_map;
pub mod managed_properties;
pub mod managed_set;
pub mod default_singleton_bean_registry;

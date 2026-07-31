//! factory::annotation — 对应 Spring beans.factory.annotation 包。

pub mod annotated_bean_definition;
/// 带注解的通用Bean定义 模块。
pub mod annotated_generic_bean_definition;
pub mod annotation_bean_wiring_info_resolver;
pub mod autowire;
pub mod autowired;
pub mod autowired_annotation_bean_post_processor;
pub mod bean_factory_annotation_utils;
pub mod configurable;
pub mod custom_autowire_configurer;
pub mod init_destroy_annotation_bean_post_processor;
pub mod injection_metadata;
pub mod jakarta_annotations_runtime_hints;
pub mod lookup;
pub mod parameter_resolution_delegate;
pub mod qualifier;
pub mod qualifier_annotation_autowire_candidate_resolver;
pub mod value;

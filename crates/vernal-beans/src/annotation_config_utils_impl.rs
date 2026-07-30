//! AnnotationConfigUtilsImpl — 注解配置工具实现。
use crate::annotation_config_utils::AnnotationConfigUtils;

/// 注解配置工具实现。
pub struct AnnotationConfigUtilsImpl;
impl AnnotationConfigUtilsImpl {
    pub fn new() -> Self { Self }
    pub fn is_configuration_class(name: &str) -> bool { AnnotationConfigUtils::is_configuration_class(name) }
    pub fn is_component_class(name: &str) -> bool { AnnotationConfigUtils::is_component_class(name) }
}

//! AnnotationConfigUtils — 注解配置工具。
/// 注解配置工具。
pub struct AnnotationConfigUtils;
impl AnnotationConfigUtils {
    pub fn is_configuration_class(class_name: &str) -> bool {
        class_name.contains("Configuration") || class_name.contains("Config")
    }
    pub fn is_component_class(class_name: &str) -> bool {
        class_name.contains("Component") || class_name.contains("Service") || class_name.contains("Repository")
    }
}

//! ParameterResolutionDelegate — Spring 风格参数解析委托。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.ParameterResolutionDelegate`。
//!
//! 在自动装配过程中，此类负责将方法参数的类型映射到容器中的 Bean，
//! 并跟踪解析进度。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

/// Spring 风格参数解析委托。
///
/// 对应 Spring 的 `ParameterResolutionDelegate`。
///
/// 管理参数名称到类型的依赖映射，支持自动装配时的参数解析。
pub struct ParameterResolutionDelegate {
    dependencies: Mutex<HashMap<String, TypeId>>,
    resolved_count: Mutex<usize>,
}

impl ParameterResolutionDelegate {
    /// 创建新的参数解析委托。
    pub fn new() -> Self {
        Self {
            dependencies: Mutex::new(HashMap::new()),
            resolved_count: Mutex::new(0),
        }
    }

    /// 注册一个参数名称到类型的依赖映射。
    pub fn register_dependency(&self, name: String, type_id: TypeId) {
        self.dependencies.lock().unwrap().insert(name, type_id);
    }

    /// 获取指定参数名称对应的 `TypeId`。
    pub fn get_dependency(&self, name: &str) -> Option<TypeId> {
        self.dependencies.lock().unwrap().get(name).copied()
    }

    /// 已注册的依赖数量。
    pub fn dependency_count(&self) -> usize {
        self.dependencies.lock().unwrap().len()
    }

    /// 递增已解析计数。
    pub fn increment_resolved(&self) {
        *self.resolved_count.lock().unwrap() += 1;
    }

    /// 已解析的参数数量。
    pub fn resolved_count(&self) -> usize {
        *self.resolved_count.lock().unwrap()
    }

    /// 是否所有依赖都已解析。
    pub fn all_resolved(&self) -> bool {
        let deps = self.dependencies.lock().unwrap().len();
        let resolved = *self.resolved_count.lock().unwrap();
        resolved >= deps
    }

    /// 列出所有已注册的参数名称。
    pub fn parameter_names(&self) -> Vec<String> {
        self.dependencies.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for ParameterResolutionDelegate { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get_dependency() {
        let delegate = ParameterResolutionDelegate::new();
        delegate.register_dependency("dataSource".to_string(), TypeId::of::<String>());
        assert_eq!(
            delegate.get_dependency("dataSource"),
            Some(TypeId::of::<String>())
        );
        assert_eq!(delegate.dependency_count(), 1);
    }

    #[test]
    fn resolved_count_tracking() {
        let delegate = ParameterResolutionDelegate::new();
        delegate.register_dependency("a".to_string(), TypeId::of::<i32>());
        delegate.register_dependency("b".to_string(), TypeId::of::<String>());

        assert!(!delegate.all_resolved());
        delegate.increment_resolved();
        delegate.increment_resolved();
        assert!(delegate.all_resolved());
        assert_eq!(delegate.resolved_count(), 2);
    }

    #[test]
    fn parameter_names_list() {
        let delegate = ParameterResolutionDelegate::new();
        delegate.register_dependency("x".to_string(), TypeId::of::<i32>());
        delegate.register_dependency("y".to_string(), TypeId::of::<i32>());
        let mut names = delegate.parameter_names();
        names.sort();
        assert_eq!(names, vec!["x", "y"]);
    }
}

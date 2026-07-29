//! MethodOverrides — Spring 风格的方法覆盖集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MethodOverrides`。
//!
//! 一个 Bean 上所有 `MethodOverride` 的容器，提供增删查与统计能力。

use std::sync::Arc;

use crate::method_override::MethodOverride;

/// 方法覆盖集合。
///
/// 对应 Spring 的 `MethodOverrides`。
///
/// 每个 `BeanDefinition` 关联一个 `MethodOverrides`，记录该 Bean 上所有
/// lookup 注入与方法替换。`MethodOverride` 以 `Arc` 共享存储。
#[derive(Debug, Clone, Default)]
pub struct MethodOverrides {
    /// 所有覆盖（按方法名索引以便快速查找）。
    overrides: Vec<Arc<dyn MethodOverride>>,
}

impl MethodOverrides {
    /// 创建空的覆盖集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个方法覆盖。
    ///
    /// 对应 Spring 的 `addOverride(MethodOverride override)`。
    /// 同名覆盖会被替换（保留最后添加的）。
    pub fn add(&mut self, method_override: Arc<dyn MethodOverride>) {
        let name = method_override.get_method_name();
        // 移除同名的旧覆盖。
        self.overrides.retain(|o| o.get_method_name() != name);
        self.overrides.push(method_override);
    }

    /// 添加覆盖的便捷方法（接受任意实现 `MethodOverride` 的值）。
    pub fn add_override<T: MethodOverride + 'static>(&mut self, method_override: T) {
        self.add(Arc::new(method_override));
    }

    /// 获取所有覆盖。
    pub fn get_overrides(&self) -> &[Arc<dyn MethodOverride>] {
        &self.overrides
    }

    /// 按方法名查找覆盖。
    ///
    /// 对应 Spring 的 `getOverride(String methodName)`。
    pub fn get_override(&self, method_name: &str) -> Option<&Arc<dyn MethodOverride>> {
        self.overrides
            .iter()
            .find(|o| o.get_method_name() == method_name)
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    /// 覆盖数量。
    ///
    /// 对应 Spring 的 `size()`。
    pub fn size(&self) -> usize {
        self.overrides.len()
    }

    /// 是否包含指定方法的覆盖。
    pub fn contains(&self, method_name: &str) -> bool {
        self.get_override(method_name).is_some()
    }

    /// 移除指定方法的覆盖。
    pub fn remove(&mut self, method_name: &str) -> bool {
        let before = self.overrides.len();
        self.overrides
            .retain(|o| o.get_method_name() != method_name);
        self.overrides.len() != before
    }

    /// 清空所有覆盖。
    pub fn clear(&mut self) {
        self.overrides.clear();
    }
}

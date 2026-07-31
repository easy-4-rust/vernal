//! AnnotationBeanWiringInfoResolver — Spring 风格注解 Bean 装配信息解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AnnotationBeanWiringInfoResolver`。
//!
//! 在 Spring 中，此类负责从注解元数据中解析 Bean 的装配信息，
//! 包括需要注入的字段和方法。它是 `@Autowired` 注解处理链的一部分。
//!
//! ## 主要功能
//!
//! - 注册类型到装配信息的映射
//! - 查询已注册的装配信息
//! - 跟踪解析进度

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

/// 注解 Bean 装配信息解析器。
///
/// 对应 Spring 的 `AnnotationBeanWiringInfoResolver`。
///
/// 管理类型到装配信息的映射，支持自动装配时的信息查找。
pub struct AnnotationBeanWiringInfoResolver {
    /// 类型到装配信息的映射
    wiring_infos: Mutex<HashMap<TypeId, Vec<String>>>,
    /// 已解析的类型数量
    resolved_count: Mutex<usize>,
}

impl AnnotationBeanWiringInfoResolver {
    /// 创建新的装配信息解析器。
    pub fn new() -> Self {
        Self {
            wiring_infos: Mutex::new(HashMap::new()),
            resolved_count: Mutex::new(0),
        }
    }

    /// 注册类型到装配信息的映射。
    ///
    /// # 参数
    /// - `type_id` — Bean 类型
    /// - `info` — 装配信息列表（字段名或方法名）
    pub fn register_wiring_info(&self, type_id: TypeId, info: Vec<String>) {
        self.wiring_infos.lock().unwrap().insert(type_id, info);
    }

    /// 获取指定类型的装配信息。
    pub fn get_wiring_info(&self, type_id: TypeId) -> Option<Vec<String>> {
        self.wiring_infos.lock().unwrap().get(&type_id).cloned()
    }

    /// 已注册的类型数量。
    pub fn registered_count(&self) -> usize {
        self.wiring_infos.lock().unwrap().len()
    }

    /// 是否包含指定类型的装配信息。
    pub fn has_wiring_info(&self, type_id: TypeId) -> bool {
        self.wiring_infos.lock().unwrap().contains_key(&type_id)
    }

    /// 标记指定类型已解析。
    pub fn mark_resolved(&self) {
        *self.resolved_count.lock().unwrap() += 1;
    }

    /// 获取已解析数量。
    pub fn resolved_count(&self) -> usize {
        *self.resolved_count.lock().unwrap()
    }

    /// 清空所有装配信息。
    pub fn clear(&self) {
        self.wiring_infos.lock().unwrap().clear();
        *self.resolved_count.lock().unwrap() = 0;
    }
}

impl Default for AnnotationBeanWiringInfoResolver {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_resolver_is_empty() {
        let resolver = AnnotationBeanWiringInfoResolver::new();
        assert_eq!(resolver.registered_count(), 0);
        assert_eq!(resolver.resolved_count(), 0);
    }

    #[test]
    fn register_and_get_wiring_info() {
        let resolver = AnnotationBeanWiringInfoResolver::new();
        let tid = TypeId::of::<String>();
        resolver.register_wiring_info(tid, vec!["field1".to_string(), "setter".to_string()]);

        assert!(resolver.has_wiring_info(tid));
        let info = resolver.get_wiring_info(tid).unwrap();
        assert_eq!(info, vec!["field1", "setter"]);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let resolver = AnnotationBeanWiringInfoResolver::new();
        assert!(resolver.get_wiring_info(TypeId::of::<i32>()).is_none());
        assert!(!resolver.has_wiring_info(TypeId::of::<i32>()));
    }

    #[test]
    fn mark_resolved_tracking() {
        let resolver = AnnotationBeanWiringInfoResolver::new();
        resolver.mark_resolved();
        resolver.mark_resolved();
        assert_eq!(resolver.resolved_count(), 2);
    }

    #[test]
    fn clear_removes_all() {
        let resolver = AnnotationBeanWiringInfoResolver::new();
        resolver.register_wiring_info(TypeId::of::<String>(), vec!["a".to_string()]);
        resolver.mark_resolved();

        resolver.clear();
        assert_eq!(resolver.registered_count(), 0);
        assert_eq!(resolver.resolved_count(), 0);
    }
}

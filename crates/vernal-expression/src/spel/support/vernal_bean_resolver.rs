//! Vernal IoC 容器 Bean 解析器。
//!
//! 从 Vernal Container 解析 Bean，对标 Spring 的 BeanResolver 集成。

use crate::access_exception::AccessException;
use crate::bean_resolver::BeanResolver;
use crate::evaluation_context::EvaluationContext;
use crate::typed_value::TypedValue;

/// Vernal Bean 解析器。
///
/// 从 Vernal IoC 容器解析 Bean。
/// 这是 Rust 侧新增功能，对标 Spring 的 BeanFactory 集成。
pub struct VernalBeanResolver;

impl BeanResolver for VernalBeanResolver {
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        bean_name: &str,
    ) -> Result<TypedValue, AccessException> {
        // 简化实现：返回 Bean 名称
        // 完整实现需要从 Vernal Container 解析
        Err(AccessException::new(format!("Bean '{}' 未找到", bean_name)))
    }
}

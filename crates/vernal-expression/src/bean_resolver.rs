//! Bean 解析器 trait。
//!
//! 对标 Spring 的 `BeanResolver`。

use super::access_exception::AccessException;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypedValue;

/// Bean 解析器 trait。
///
/// 解析 `@beanName` 引用。
/// 对标 Spring 的 `org.springframework.expression.BeanResolver`。
pub trait BeanResolver: Send + Sync {
    /// 解析 Bean 引用。
    fn resolve(
        &self,
        context: &dyn EvaluationContext,
        bean_name: &str,
    ) -> Result<TypedValue, AccessException>;
}

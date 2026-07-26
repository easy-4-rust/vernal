//! 构造器解析器 trait。
//!
//! 对标 Spring 的 `ConstructorResolver`。

use super::access_exception::AccessException;
use super::constructor_executor::ConstructorExecutor;
use super::evaluation_context::EvaluationContext;
use super::typed_value::TypeDescriptor;

/// 构造器解析器 trait。
///
/// 定位构造器并返回 [`ConstructorExecutor`]。
/// 对标 Spring 的 `org.springframework.expression.ConstructorResolver`。
pub trait ConstructorResolver: Send + Sync {
    /// 解析构造器。
    fn resolve(
        &self,
        context: &dyn EvaluationContext,
        type_name: &str,
        argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn ConstructorExecutor>>, AccessException>;
}

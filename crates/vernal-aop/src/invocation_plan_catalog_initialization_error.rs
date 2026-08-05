//! 调用计划目录封存错误对象。

use std::{error::Error, fmt};

/// 调用计划目录已经封存，拒绝再次写入。
///
/// Vernal 的高层应用建造器会先把一个待封存目录作为原生对象注册进 `IoC` 图，
/// 再使用同一个 Container 解析由组件实现的拦截器，最后一次性写入预编译计划。
/// 该错误用于阻止框架缺陷或错误的自定义装配路径在运行期替换既有计划。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvocationPlanCatalogInitializationError;

impl fmt::Display for InvocationPlanCatalogInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invocation plan catalog is already initialized")
    }
}

impl Error for InvocationPlanCatalogInitializationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = InvocationPlanCatalogInitializationError;
        assert_eq!(
            format!("{}", err),
            "invocation plan catalog is already initialized"
        );
    }

    #[test]
    fn error_debug() {
        let err = InvocationPlanCatalogInitializationError;
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty());
    }

    #[test]
    fn error_clone() {
        let err = InvocationPlanCatalogInitializationError;
        let _cloned = err;
    }

    #[test]
    fn error_copy() {
        let err = InvocationPlanCatalogInitializationError;
        let copied = err;
        assert_eq!(err, copied);
    }

    #[test]
    fn error_trait() {
        let err = InvocationPlanCatalogInitializationError;
        let _: &dyn Error = &err;
    }
}

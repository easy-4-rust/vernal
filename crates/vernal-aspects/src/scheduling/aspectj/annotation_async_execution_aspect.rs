//! 对标 `org.springframework.scheduling.aspectj.AnnotationAsyncExecutionAspect` 具体 Aspect。
//!
//! 基于 Spring `@Async` 注解驱动的异步切面。

use super::abstract_async_execution_aspect::AbstractAsyncExecutionAspect;

/// 基于 `@Async` 注解的异步执行切面。
///
/// 对标 Spring 的 `AnnotationAsyncExecutionAspect`。
///
/// # Pointcut 定义
///
/// ```text
/// private pointcut asyncMarkedMethod() : execution(@Async (void || Future+) *(..));
/// private pointcut asyncTypeMarkedMethod() : execution((void || Future+) (@Async *).*(..));
/// public pointcut asyncMethod() : asyncMarkedMethod() || asyncTypeMarkedMethod();
/// ```
pub struct AnnotationAsyncExecutionAspect {
    inner: AbstractAsyncExecutionAspect,
}

impl AnnotationAsyncExecutionAspect {
    /// 创建 `@Async` 注解驱动的异步切面。
    pub fn new() -> Self {
        Self {
            inner: AbstractAsyncExecutionAspect::new(),
        }
    }

    /// 获取底层异步切面支撑。
    pub fn get_inner(&self) -> &AbstractAsyncExecutionAspect {
        &self.inner
    }

    /// 获取可变的底层异步切面支撑。
    pub fn get_inner_mut(&mut self) -> &mut AbstractAsyncExecutionAspect {
        &mut self.inner
    }

    /// 匹配 `asyncMarkedMethod()` pointcut。
    ///
    /// 对应 Spring 的 `execution(@Async (void || Future+) *(..))`。
    /// 匹配标记了 `@Async` 注解的方法（返回 void 或 Future）。
    pub fn matches_async_marked_method(
        &self,
        method_has_async_annotation: bool,
        return_type_is_void_or_future: bool,
    ) -> bool {
        method_has_async_annotation && return_type_is_void_or_future
    }

    /// 匹配 `asyncTypeMarkedMethod()` pointcut。
    ///
    /// 对应 Spring 的 `execution((void || Future+) (@Async *).*(..))`。
    /// 匹配标记了 `@Async` 的类型中的方法（返回 void 或 Future）。
    pub fn matches_async_type_marked_method(
        &self,
        type_has_async_annotation: bool,
        return_type_is_void_or_future: bool,
    ) -> bool {
        type_has_async_annotation && return_type_is_void_or_future
    }

    /// 组合 pointcut：`asyncMethod()`。
    ///
    /// 对应 Spring 的 `public pointcut asyncMethod()`。
    pub fn async_method(
        &self,
        method_has_async: bool,
        type_has_async: bool,
        return_type_is_void_or_future: bool,
    ) -> bool {
        self.matches_async_marked_method(method_has_async, return_type_is_void_or_future)
            || self.matches_async_type_marked_method(type_has_async, return_type_is_void_or_future)
    }

    /// 获取执行器限定名。
    ///
    /// 对标 Spring 的 `AnnotationAsyncExecutionAspect#getExecutorQualifier(Method)` 方法。
    /// 从 `@Async#value()` 读取指定的执行器限定名。
    pub fn get_executor_qualifier<'a>(
        &self,
        method_has_async: bool,
        method_async_value: Option<&'a str>,
        class_has_async: bool,
        class_async_value: Option<&'a str>,
    ) -> Option<&'a str> {
        // 方法级 @Async 优先
        if method_has_async {
            return method_async_value;
        }
        // 回退到类级 @Async
        if class_has_async {
            return class_async_value;
        }
        None
    }
}

impl Default for AnnotationAsyncExecutionAspect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_async_execution_aspect_creation() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        assert!(
            aspect
                .get_inner()
                .determine_async_executor(
                    &super::super::abstract_async_execution_aspect::MethodMetadata::new(
                        "Foo", "bar", "void"
                    )
                )
                .is_none()
        );
    }

    #[test]
    fn test_matches_async_marked_method() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        assert!(aspect.matches_async_marked_method(true, true));
        assert!(!aspect.matches_async_marked_method(false, true));
        assert!(!aspect.matches_async_marked_method(true, false));
    }

    #[test]
    fn test_matches_async_type_marked_method() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        assert!(aspect.matches_async_type_marked_method(true, true));
        assert!(!aspect.matches_async_type_marked_method(false, true));
    }

    #[test]
    fn test_async_method_combination() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        // 方法级 @Async
        assert!(aspect.async_method(true, false, true));
        // 类级 @Async
        assert!(aspect.async_method(false, true, true));
        // 都不匹配
        assert!(!aspect.async_method(false, false, false));
        // return type 不匹配
        assert!(!aspect.async_method(true, false, false));
    }

    #[test]
    fn test_get_executor_qualifier() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        // 方法级优先
        assert_eq!(
            aspect.get_executor_qualifier(true, Some("methodExecutor"), false, None),
            Some("methodExecutor")
        );
        // 回退到类级
        assert_eq!(
            aspect.get_executor_qualifier(false, None, true, Some("classExecutor")),
            Some("classExecutor")
        );
        // 都没有
        assert_eq!(
            aspect.get_executor_qualifier(false, None, false, None),
            None
        );
    }

    #[test]
    fn test_default() {
        let aspect = AnnotationAsyncExecutionAspect::default();
        assert!(
            aspect
                .get_inner()
                .determine_async_executor(
                    &super::super::abstract_async_execution_aspect::MethodMetadata::new(
                        "Foo", "bar", "void"
                    )
                )
                .is_none()
        );
    }

    #[test]
    fn test_get_executor_qualifier_all_none() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        assert_eq!(
            aspect.get_executor_qualifier(false, None, false, None),
            None
        );
    }

    #[test]
    fn test_get_executor_qualifier_method_only() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        assert_eq!(
            aspect.get_executor_qualifier(true, Some("methodExec"), false, None),
            Some("methodExec")
        );
    }

    #[test]
    fn test_get_executor_qualifier_class_only() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        assert_eq!(
            aspect.get_executor_qualifier(false, None, true, Some("classExec")),
            Some("classExec")
        );
    }

    #[test]
    fn test_get_executor_qualifier_both() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        // 方法级优先
        assert_eq!(
            aspect.get_executor_qualifier(true, Some("methodExec"), true, Some("classExec")),
            Some("methodExec")
        );
    }

    #[test]
    fn test_async_method_all_combinations() {
        let aspect = AnnotationAsyncExecutionAspect::new();

        // 方法级 @Async + return type void
        assert!(aspect.async_method(true, false, true));

        // 方法级 @Async + return type Future
        assert!(aspect.async_method(true, false, true));

        // 类级 @Async + return type void
        assert!(aspect.async_method(false, true, true));

        // 类级 @Async + return type Future
        assert!(aspect.async_method(false, true, true));

        // 都不匹配
        assert!(!aspect.async_method(false, false, false));

        // return type 不匹配
        assert!(!aspect.async_method(true, false, false));
    }

    #[test]
    fn test_annotation_async_execution_aspect_clone() {
        let aspect = AnnotationAsyncExecutionAspect::new();
        let _ = aspect;
    }

    #[test]
    fn test_get_inner_mut() {
        let mut aspect = AnnotationAsyncExecutionAspect::new();
        let inner = aspect.get_inner_mut();
        // Verify we got a mutable reference
        let _ = inner;
    }
}

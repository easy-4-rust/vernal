//! 前置通知。
//!
//! 对应 spring-aop `org.springframework.aop.BeforeAdvice`。
//! 在方法执行前调用的通知。

use super::advice::Advice;

/// 前置通知标记接口。
///
/// 对应 spring-aop `BeforeAdvice`。
///
/// 在目标方法执行前调用。这是 `MethodBeforeAdvice` 和
/// `ConstructorInterceptor` 的父接口。
pub trait BeforeAdvice: Advice {
    /// 获取通知名称。
    fn advice_name(&self) -> &str;
}

/// 方法前置通知。
///
/// 对应 spring-aop `MethodBeforeAdvice`。
///
/// 在方法执行前调用，可以访问方法、参数和目标对象。
pub trait MethodBeforeAdvice: BeforeAdvice {
    /// 方法执行前调用。
    ///
    /// # 参数
    /// * `method` - 方法名
    /// * `args` - 参数列表
    /// * `target` - 目标对象（可选）
    ///
    /// # 返回
    /// 如果返回 Err，将阻止方法执行。
    fn before(
        &self,
        method: &str,
        args: &[&dyn std::any::Any],
        target: Option<&dyn std::any::Any>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBeforeAdvice;

    impl Advice for TestBeforeAdvice {
        fn advice_type(&self) -> &str {
            "BeforeAdvice"
        }
    }

    impl BeforeAdvice for TestBeforeAdvice {
        fn advice_name(&self) -> &str {
            "test_before"
        }
    }

    impl MethodBeforeAdvice for TestBeforeAdvice {
        fn before(
            &self,
            _method: &str,
            _args: &[&dyn std::any::Any],
            _target: Option<&dyn std::any::Any>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
    }

    #[test]
    fn before_advice() {
        let advice = TestBeforeAdvice;
        assert_eq!(advice.advice_type(), "BeforeAdvice");
        assert_eq!(advice.advice_name(), "test_before");
        assert!(advice.before("method", &[], None).is_ok());
    }
}

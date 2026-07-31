//! 后置通知。
//!
//! 对应 spring-aop `org.springframework.aop.AfterAdvice`、`AfterReturningAdvice`、`ThrowsAdvice`。

use super::advice::Advice;

/// 后置通知标记接口。
///
/// 对应 spring-aop `AfterAdvice`。
///
/// 在目标方法执行后调用的通知。这是 `AfterReturningAdvice` 和
/// `ThrowsAdvice` 的父接口。
pub trait AfterAdvice: Advice {}

/// 返回后通知。
///
/// 对应 spring-aop `AfterReturningAdvice`。
///
/// 在方法成功返回后调用。
pub trait AfterReturningAdvice: AfterAdvice {
    /// 方法成功返回后调用。
    ///
    /// # 参数
    /// * `return_value` - 返回值
    /// * `method` - 方法名
    /// * `args` - 参数列表
    /// * `target` - 目标对象（可选）
    fn after_returning(
        &self,
        return_value: &dyn std::any::Any,
        method: &str,
        args: &[&dyn std::any::Any],
        target: Option<&dyn std::any::Any>,
    );
}

/// 异常通知。
///
/// 对应 spring-aop `ThrowsAdvice`。
///
/// 在方法抛出异常时调用。
pub trait ThrowsAdvice: AfterAdvice {
    /// 方法抛出异常时调用。
    ///
    /// # 参数
    /// * `error` - 异常
    /// * `method` - 方法名
    /// * `args` - 参数列表
    /// * `target` - 目标对象（可选）
    fn after_throwing(
        &self,
        error: &(dyn std::error::Error + Send + Sync),
        method: &str,
        args: &[&dyn std::any::Any],
        target: Option<&dyn std::any::Any>,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAfterReturningAdvice;

    impl Advice for TestAfterReturningAdvice {
        fn advice_type(&self) -> &str {
            "AfterReturningAdvice"
        }
    }

    impl AfterAdvice for TestAfterReturningAdvice {}

    impl AfterReturningAdvice for TestAfterReturningAdvice {
        fn after_returning(
            &self,
            _return_value: &dyn std::any::Any,
            _method: &str,
            _args: &[&dyn std::any::Any],
            _target: Option<&dyn std::any::Any>,
        ) {
            // 成功后置通知
        }
    }

    #[test]
    fn after_returning_advice() {
        let advice = TestAfterReturningAdvice;
        assert_eq!(advice.advice_type(), "AfterReturningAdvice");
    }

    struct TestThrowsAdvice;

    impl Advice for TestThrowsAdvice {
        fn advice_type(&self) -> &str {
            "ThrowsAdvice"
        }
    }

    impl AfterAdvice for TestThrowsAdvice {}

    impl ThrowsAdvice for TestThrowsAdvice {
        fn after_throwing(
            &self,
            _error: &(dyn std::error::Error + Send + Sync),
            _method: &str,
            _args: &[&dyn std::any::Any],
            _target: Option<&dyn std::any::Any>,
        ) {
            // 异常通知
        }
    }

    #[test]
    fn throws_advice() {
        let advice = TestThrowsAdvice;
        assert_eq!(advice.advice_type(), "ThrowsAdvice");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    struct TestAfterReturningAdvice;
    impl Advice for TestAfterReturningAdvice {
        fn advice_type(&self) -> &str {
            "AfterReturningAdvice"
        }
    }
    impl AfterAdvice for TestAfterReturningAdvice {}
    impl AfterReturningAdvice for TestAfterReturningAdvice {
        fn after_returning(
            &self,
            _return_value: &dyn std::any::Any,
            _method: &str,
            _args: &[&dyn std::any::Any],
            _target: Option<&dyn std::any::Any>,
        ) {
        }
    }

    struct TestThrowsAdvice;
    impl Advice for TestThrowsAdvice {
        fn advice_type(&self) -> &str {
            "ThrowsAdvice"
        }
    }
    impl AfterAdvice for TestThrowsAdvice {}
    impl ThrowsAdvice for TestThrowsAdvice {
        fn after_throwing(
            &self,
            _error: &(dyn std::error::Error + Send + Sync),
            _method: &str,
            _args: &[&dyn std::any::Any],
            _target: Option<&dyn std::any::Any>,
        ) {
        }
    }

    #[test]
    fn after_returning_advice_trait_object() {
        let advice: Box<dyn AfterReturningAdvice> = Box::new(TestAfterReturningAdvice);
        assert_eq!(advice.advice_type(), "AfterReturningAdvice");
    }

    #[test]
    fn throws_advice_trait_object() {
        let advice: Box<dyn ThrowsAdvice> = Box::new(TestThrowsAdvice);
        assert_eq!(advice.advice_type(), "ThrowsAdvice");
    }

    #[test]
    fn after_returning_advice_call() {
        let advice = TestAfterReturningAdvice;
        advice.after_returning(&42i32, "method", &[], None);
    }

    #[test]
    fn throws_advice_call() {
        let advice = TestThrowsAdvice;
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        advice.after_throwing(&error, "method", &[], None);
    }
}

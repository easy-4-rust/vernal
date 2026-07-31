//! 引入拦截器。
//!
//! 对应 spring-aop `org.springframework.aop.IntroductionInterceptor`。

use super::advice::Advice;

/// 引入拦截器接口。
///
/// 对应 spring-aop `IntroductionInterceptor`。
///
/// 拦截器可以引入新的接口实现。
pub trait IntroductionInterceptor: Advice {
    /// 检查是否实现了给定接口。
    fn implements_interface(&self, interface_type: &str) -> bool;

    /// 获取实现的接口列表。
    fn get_interfaces(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestIntroductionInterceptor {
        interfaces: Vec<String>,
    }

    impl Advice for TestIntroductionInterceptor {
        fn advice_type(&self) -> &str {
            "IntroductionInterceptor"
        }
    }

    impl IntroductionInterceptor for TestIntroductionInterceptor {
        fn implements_interface(&self, interface_type: &str) -> bool {
            self.interfaces.contains(&interface_type.to_string())
        }

        fn get_interfaces(&self) -> Vec<String> {
            self.interfaces.clone()
        }
    }

    #[test]
    fn introduction_interceptor() {
        let interceptor = TestIntroductionInterceptor {
            interfaces: vec!["com.example.MyInterface".to_string()],
        };
        assert!(interceptor.implements_interface("com.example.MyInterface"));
        assert!(!interceptor.implements_interface("com.example.OtherInterface"));
    }
}

//! 动态引入通知。
//!
//! 对应 spring-aop `org.springframework.aop.DynamicIntroductionAdvice`。
//! 可以动态添加接口引入的通知。

use super::advice::Advice;

/// 动态引入通知接口。
///
/// 对应 spring-aop `DynamicIntroductionAdvice`。
///
/// 可以在运行时动态添加接口引入的通知。
pub trait DynamicIntroductionAdvice: Advice {
    /// 检查是否实现了给定接口。
    fn implements_interface(&self, interface_type: &str) -> bool;

    /// 获取实现的接口列表。
    fn get_interfaces(&self) -> Vec<String>;
}

/// 引入通知接口。
///
/// 对应 spring-aop `IntroductionInfo`。
pub trait IntroductionAdvice: Advice {
    /// 获取引入的接口列表。
    fn get_interfaces(&self) -> Vec<String>;
}

/// 引入拦截器接口。
///
/// 对应 spring-aop `IntroductionInterceptor`。
///
/// 拦截器可以引入新的接口实现。
pub trait IntroductionInterceptor: Advice {
    /// 检查是否实现了给定接口。
    fn implements_interface(&self, interface_type: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDynamicIntroductionAdvice {
        interfaces: Vec<String>,
    }

    impl Advice for TestDynamicIntroductionAdvice {
        fn advice_type(&self) -> &str {
            "DynamicIntroductionAdvice"
        }
    }

    impl DynamicIntroductionAdvice for TestDynamicIntroductionAdvice {
        fn implements_interface(&self, interface_type: &str) -> bool {
            self.interfaces.contains(&interface_type.to_string())
        }

        fn get_interfaces(&self) -> Vec<String> {
            self.interfaces.clone()
        }
    }

    #[test]
    fn dynamic_introduction_advice() {
        let advice = TestDynamicIntroductionAdvice {
            interfaces: vec!["com.example.MyInterface".to_string()],
        };
        assert!(advice.implements_interface("com.example.MyInterface"));
        assert!(!advice.implements_interface("com.example.OtherInterface"));
        assert_eq!(advice.get_interfaces().len(), 1);
    }
}

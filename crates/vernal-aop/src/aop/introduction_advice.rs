//! 引入通知。
//!
//! 对应 spring-aop `org.springframework.aop.IntroductionAdvisor`。

use super::advice::Advice;

/// 引入通知接口。
///
/// 对应 spring-aop `IntroductionAdvisor`。
///
/// 用于将额外的接口引入到目标对象中。
pub trait IntroductionAdvice: Advice {
    /// 获取引入的接口列表。
    fn get_interfaces(&self) -> Vec<String>;

    /// 检查是否支持给定接口。
    fn supports_interface(&self, interface_type: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestIntroductionAdvice {
        interfaces: Vec<String>,
    }

    impl Advice for TestIntroductionAdvice {
        fn advice_type(&self) -> &str {
            "IntroductionAdvice"
        }
    }

    impl IntroductionAdvice for TestIntroductionAdvice {
        fn get_interfaces(&self) -> Vec<String> {
            self.interfaces.clone()
        }

        fn supports_interface(&self, interface_type: &str) -> bool {
            self.interfaces.contains(&interface_type.to_string())
        }
    }

    #[test]
    fn introduction_advice() {
        let advice = TestIntroductionAdvice {
            interfaces: vec!["com.example.MyInterface".to_string()],
        };
        assert!(advice.supports_interface("com.example.MyInterface"));
        assert!(!advice.supports_interface("com.example.OtherInterface"));
    }
}

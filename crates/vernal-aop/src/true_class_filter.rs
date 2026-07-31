//! 始终匹配的类过滤器。
//!
//! 对应 spring-aop `org.springframework.aop.TrueClassFilter`。

use crate::class_filter::ClassFilter;

/// 始终匹配的类过滤器单例。
///
/// 对应 spring-aop `TrueClassFilter`。
///
/// 这是一个单例，始终返回 `true`。
// 对标 Spring AOP 的 API 脚手架：TrueClassFilter 为 TruePointcut 的类过滤单例，暂未被内部调用。
#[allow(dead_code)]
pub struct TrueClassFilter;

impl ClassFilter for TrueClassFilter {
    fn matches(&self, _class_name: &str) -> bool {
        true
    }
}

impl std::fmt::Debug for TrueClassFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrueClassFilter")
    }
}

impl std::fmt::Display for TrueClassFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClassFilter.TRUE")
    }
}

/// 获取 TrueClassFilter 单例。
// 对标 Spring AOP 的 API 脚手架：true_class_filter 返回 TrueClassFilter 单例引用，暂未被内部调用。
#[allow(dead_code)]
pub fn true_class_filter() -> &'static TrueClassFilter {
    &TrueClassFilter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_class_filter_matches_all() {
        let filter = TrueClassFilter;
        assert!(filter.matches("any.Class"));
        assert!(filter.matches(""));
        assert!(filter.matches("com.example.Service"));
    }

    #[test]
    fn true_class_filter_display() {
        let filter = TrueClassFilter;
        assert_eq!(format!("{}", filter), "ClassFilter.TRUE");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn true_class_filter_matches_various() {
        let filter = TrueClassFilter;
        assert!(filter.matches("com.example.Service"));
        assert!(filter.matches("org.springframework.Bean"));
        assert!(filter.matches(""));
        assert!(filter.matches("any_class"));
    }

    #[test]
    fn true_class_filter_display() {
        let filter = TrueClassFilter;
        assert_eq!(format!("{}", filter), "ClassFilter.TRUE");
    }

    #[test]
    fn true_class_filter_debug() {
        let filter = TrueClassFilter;
        let debug = format!("{:?}", filter);
        assert!(!debug.is_empty());
    }

    #[test]
    fn true_class_filter_singleton() {
        let f1 = true_class_filter();
        let f2 = true_class_filter();
        assert!(std::ptr::eq(f1, f2));
    }
}

//! 类过滤器。
//!
//! 对应 spring-aop `ClassFilter`。
//! 限制切点或引入顾问只匹配指定的目标类。

use std::fmt;

/// 类过滤器，判断切点是否应用于给定的类。
///
/// 对应 spring-aop `ClassFilter`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::ClassFilter;
///
/// // 匹配所有类
/// let filter = ClassFilter::TRUE;
///
/// // 自定义过滤器
/// let filter = ClassFilter::from_fn(|class_name| class_name.starts_with("com.example"));
/// ```
pub trait ClassFilter: Send + Sync + 'static {
    /// 判断是否匹配给定的类名。
    fn matches(&self, class_name: &str) -> bool;
}

/// 始终匹配的类过滤器。
///
/// 对应 spring-aop `TrueClassFilter`。
#[derive(Clone, Copy, Debug)]
pub struct TrueClassFilter;

impl ClassFilter for TrueClassFilter {
    fn matches(&self, _class_name: &str) -> bool {
        true
    }
}

impl fmt::Display for TrueClassFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClassFilter.TRUE")
    }
}

/// 基于闭包的类过滤器。
#[derive(Clone)]
pub struct FnClassFilter<F: Fn(&str) -> bool + Send + Sync + 'static> {
    predicate: F,
}

impl<F: Fn(&str) -> bool + Send + Sync + 'static> ClassFilter for FnClassFilter<F> {
    fn matches(&self, class_name: &str) -> bool {
        (self.predicate)(class_name)
    }
}

/// 类过滤器工厂。
pub struct ClassFilterFactory;

impl ClassFilterFactory {
    /// 始终匹配的实例。
    pub const TRUE: TrueClassFilter = TrueClassFilter;

    /// 从闭包创建类过滤器。
    pub fn from_fn<F: Fn(&str) -> bool + Send + Sync + 'static>(predicate: F) -> FnClassFilter<F> {
        FnClassFilter { predicate }
    }
}

/// 闭包自动实现 ClassFilter。
impl<F> ClassFilter for F
where
    F: Fn(&str) -> bool + Send + Sync + 'static,
{
    fn matches(&self, class_name: &str) -> bool {
        self(class_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_class_filter_matches_all() {
        let filter = TrueClassFilter;
        assert!(filter.matches("any.Class"));
        assert!(filter.matches(""));
    }

    #[test]
    fn closure_as_class_filter() {
        let filter = |name: &str| name.starts_with("com.example");
        assert!(filter.matches("com.example.Service"));
        assert!(!filter.matches("org.other.Service"));
    }

    #[test]
    fn class_filter_from_fn() {
        let filter = ClassFilterFactory::from_fn(|name: &str| name.ends_with("Service"));
        assert!(filter.matches("UserService"));
        assert!(!filter.matches("UserController"));
    }
}

//! 对应 spring-aop：org.springframework.aop.IntroductionAdvisor
//!
//! 引入顾问 trait（spring-aop 语义，aspect-rs 无对偶）。
//! 扩展 `IntroductionInfo`，为引入添加类过滤器。

use crate::{IntroductionInfo, Pointcut};

/// 引入顾问：执行一个或多个 AOP 引入的顾问。
///
/// 对应 spring-aop `IntroductionAdvisor`。
/// 引入是通过 AOP 通知实现目标未实现的额外接口。
pub trait IntroductionAdvisor: IntroductionInfo + Send + Sync + 'static {
    /// 返回确定此引入应应用于哪些目标类的过滤器。
    fn class_filter(&self) -> &dyn Fn(&str) -> bool;

    /// 返回执行顺序。
    fn order(&self) -> i32;

    /// 返回用于引入匹配的切点。
    fn pointcut(&self) -> &dyn Pointcut;
}

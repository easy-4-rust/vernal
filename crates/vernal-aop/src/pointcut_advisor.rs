//! 对应 spring-aop：org.springframework.aop.PointcutAdvisor
//!
//! 切点顾问 trait（spring-aop 语义，aspect-rs 无对偶）。
//! 由切点驱动的顾问的超接口。

use crate::{Interceptor, Pointcut};

/// 切点顾问：由切点驱动的顾问超接口。
///
/// 对应 spring-aop `PointcutAdvisor`。
/// 覆盖除引入顾问之外的几乎所有顾问。
pub trait PointcutAdvisor: Send + Sync + 'static {
    /// 返回驱动此顾问的切点。
    fn pointcut(&self) -> &dyn Pointcut;

    /// 返回此顾问的环绕拦截器。
    fn interceptor(&self) -> &dyn Interceptor;

    /// 返回执行顺序（越小越先执行）。
    fn order(&self) -> i32;
}

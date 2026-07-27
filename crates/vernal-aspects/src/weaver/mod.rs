//! 对标 `META-INF/aop.xml` + AspectJ advice 类型。
//!
//! 切面织入机制：定义 advice 类型、pointcut 模式匹配、aop.xml 等价物。

mod advice_kind;
mod pointcut_matcher;

pub use advice_kind::AdviceKind;
pub use pointcut_matcher::PointcutMatcher;

//! 对应 spring-aop：org.springframework.aop.IntroductionInfo
//!
//! 引入信息 trait（spring-aop 语义，aspect-rs 无对偶）。
//! 描述通过 AOP 介绍引入的额外接口。

/// 引入信息：描述通过 AOP 介绍引入的额外接口。
///
/// 对应 spring-aop `IntroductionInfo`。
/// `IntroductionAdvisor` 必须实现此 trait。
pub trait IntroductionInfo: Send + Sync + 'static {
    /// 返回通过此顾问或通知引入的额外接口标识列表。
    fn interface_names(&self) -> &[&'static str];
}

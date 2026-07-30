//! AutowireCandidateResolver — Spring 风格的自动装配候选解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireCandidateResolver`。
//!
//! 判断某个 Bean 是否符合自动装配条件。

use std::any::TypeId;

/// 自动装配候选解析器接口。
///
/// 对应 Spring 的 `AutowireCandidateResolver`。
///
/// 判断某个 Bean 是否符合自动装配条件。
pub trait AutowireCandidateResolver: Send + Sync {
    /// 判断是否为自动装配候选
    ///
    /// # 参数
    /// - `type_id` — Bean 类型
    /// - `bean_name` — Bean 名称
    ///
    /// # 返回
    /// 是否为自动装配候选
    fn is_autowire_candidate(&self, type_id: TypeId, bean_name: &str) -> bool;
}

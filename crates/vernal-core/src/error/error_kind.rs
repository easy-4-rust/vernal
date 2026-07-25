//! 错误分类枚举。
//!
//! 将 VernalError 按照错误性质分为四大类，便于上层统一处理：
//! - 业务错误：用户代码或配置导致的可预期错误
//! - 基础设施错误：IO、网络、第三方库导致的不可预期错误
//! - 验证错误：输入数据不满足约束
//! - 内部错误：框架自身的 bug

/// 错误分类枚举。
///
/// 用于在不暴露具体错误细节的情况下，快速判断错误的处理策略。
/// 例如：业务错误可以重试，基础设施错误需要降级，验证错误需要返回 400。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// 业务错误：用户代码或配置导致的可预期错误。
    ///
    /// 典型场景：组件未找到、依赖歧义、配置缺失。
    /// 处理策略：可以向用户暴露错误详情，通常不需要重试。
    Business,

    /// 基础设施错误：IO、网络、第三方库导致的不可预期错误。
    ///
    /// 典型场景：数据库连接失败、文件读取超时、序列化错误。
    /// 处理策略：不应向用户暴露内部细节，可能需要重试或降级。
    Infrastructure,

    /// 验证错误：输入数据不满足约束。
    ///
    /// 典型场景：配置格式错误、参数类型不匹配、边界值越界。
    /// 处理策略：返回 400 或 422，附带字段级错误说明。
    Validation,

    /// 内部错误：框架自身的 bug 或不变量违反。
    ///
    /// 典型场景：不可达代码被触发、状态机非法转换。
    /// 处理策略：记录 panic 级日志，不应向用户暴露。
    Internal,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Business => write!(f, "business"),
            Self::Infrastructure => write!(f, "infrastructure"),
            Self::Validation => write!(f, "validation"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

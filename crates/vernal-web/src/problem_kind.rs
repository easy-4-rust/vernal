//! Web 问题分类对象。

/// 跨框架稳定的应用问题分类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProblemKind {
    /// 请求参数、格式或校验错误。
    ClientInput,
    /// 未登录或缺少访问凭证。
    Unauthenticated,
    /// 已登录但策略拒绝访问。
    PolicyDenied,
    /// Handler 或用例返回业务失败。
    Application,
    /// 组件解析、上下文或其他基础设施失败。
    Infrastructure,
    /// Body、连接或流式传输失败。
    Transport,
}

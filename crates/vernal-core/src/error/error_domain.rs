//! 错误域常量定义。
//!
//! 每个 Vernal 子系统拥有一个唯一的域标识符，用于在结构化错误中标识错误来源。
//! 域标识符是静态字符串，零分配，可在日志、指标和 HTTP 响应中直接使用。

/// 错误域常量集合。
///
/// 所有常量遵循 `snake_case` 命名，长度不超过 16 字节。
/// 新增子系统时必须在此处注册域标识符。
pub struct ErrorDomain;

impl ErrorDomain {
    /// IoC 容器子系统（组件注册、依赖解析、作用域管理）
    pub const IOC: &'static str = "ioc";

    /// AOP 子系统（拦截器、切入点、调用计划）
    pub const AOP: &'static str = "aop";

    /// 应用上下文子系统（生命周期、事件总线、配置属性）
    pub const CONTEXT: &'static str = "context";

    /// Web 协议子系统（请求上下文、响应映射、安全主体）
    pub const WEB: &'static str = "web";

    /// HTTP 传输子系统（请求/响应体、流式传输）
    pub const HTTP: &'static str = "http";

    /// Tower 集成子系统（Layer、Service、错误映射）
    pub const TOWER: &'static str = "tower";

    /// 组件发现子系统（linkme 自动注册）
    pub const DISCOVERY: &'static str = "discovery";

    /// 过程宏子系统（Component derive、ConfigurationProperties derive）
    pub const MACROS: &'static str = "macros";

    /// 框架核心契约子系统（BoxError、SharedError、版本常量）
    pub const CORE: &'static str = "core";

    /// 桥接层子系统（Hutool、Sa-Token 等外部库集成）
    pub const BRIDGE: &'static str = "bridge";
}

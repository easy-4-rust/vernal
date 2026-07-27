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
    /// `IoC` 容器子系统（组件注册、依赖解析、作用域管理）
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

    /// 数据库迁移子系统。
    ///
    /// 对标 Spring `org.springframework.jdbc.datasource.init`（Flyway / Liquibase 集成）。
    pub const MIGRATION: &'static str = "migration";

    /// 安全子系统。
    ///
    /// 对标 Spring Security `org.springframework.security`（认证、授权、CSRF）。
    pub const SECURITY: &'static str = "security";

    /// 监控指标子系统。
    ///
    /// 对标 Spring Boot Actuator `org.springframework.boot.actuate.metrics`。
    pub const METRICS: &'static str = "metrics";

    /// 所有已注册域的完整列表（不含重复，按定义顺序）。
    ///
    /// 用于测试与文档生成。
    #[must_use]
    pub const fn all() -> &'static [&'static str] {
        &[
            Self::IOC,
            Self::AOP,
            Self::CONTEXT,
            Self::WEB,
            Self::HTTP,
            Self::TOWER,
            Self::DISCOVERY,
            Self::MACROS,
            Self::CORE,
            Self::BRIDGE,
            Self::MIGRATION,
            Self::SECURITY,
            Self::METRICS,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorDomain;
    use std::collections::HashSet;

    #[test]
    fn all_domains_are_unique() {
        let mut seen = HashSet::new();
        for d in ErrorDomain::all() {
            assert!(seen.insert(d), "域标识符重复: {d}");
        }
        assert_eq!(seen.len(), 13, "应注册 13 个域");
    }

    #[test]
    fn all_domains_are_lowercase_snake_case() {
        for d in ErrorDomain::all() {
            assert!(
                d.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
                "域标识符应仅含小写字母/下划线/数字: {d}"
            );
        }
    }

    #[test]
    fn all_domains_are_short_enough_for_metric_labels() {
        // Prometheus 标签值虽无硬长度限制，但常见工具（Grafana/Loki）按 16 字节对齐优化
        for d in ErrorDomain::all() {
            assert!(
                d.len() <= 16,
                "域标识符不应超过 16 字节（指标标签最佳实践）: {d} (len={})",
                d.len()
            );
        }
    }

    #[test]
    fn new_domains_are_present() {
        assert_eq!(ErrorDomain::MIGRATION, "migration");
        assert_eq!(ErrorDomain::SECURITY, "security");
        assert_eq!(ErrorDomain::METRICS, "metrics");
    }

    #[test]
    fn core_domains_remain_stable() {
        // 验证 v1.0 已发布的域常量签名不变
        assert_eq!(ErrorDomain::IOC, "ioc");
        assert_eq!(ErrorDomain::AOP, "aop");
        assert_eq!(ErrorDomain::CONTEXT, "context");
        assert_eq!(ErrorDomain::CORE, "core");
    }
}

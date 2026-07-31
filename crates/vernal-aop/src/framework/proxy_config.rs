//! 代理配置。
//!
//! 对应 spring-aop `org.springframework.aop.framework.ProxyConfig`。
//! 创建代理时使用的配置基类。

/// 代理配置。
///
/// 对应 spring-aop `ProxyConfig`。
///
/// 所有代理创建器的配置基类，确保配置属性一致。
///
/// # 属性
///
/// - `proxy_target_class` - 是否直接代理目标类（而非仅代理接口）
/// - `optimize` - 是否执行激进优化
/// - `opaque` - 是否阻止代理转换为 `Advised`
/// - `expose_proxy` - 是否将代理暴露为 ThreadLocal
/// - `frozen` - 是否冻结配置（不允许修改通知）
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 是否直接代理目标类。
    proxy_target_class: bool,
    /// 是否执行激进优化。
    optimize: bool,
    /// 是否阻止代理转换为 Advised。
    opaque: bool,
    /// 是否将代理暴露为 ThreadLocal。
    expose_proxy: bool,
    /// 是否冻结配置。
    frozen: bool,
}

impl ProxyConfig {
    /// 创建新的代理配置。
    pub fn new() -> Self {
        Self {
            proxy_target_class: false,
            optimize: false,
            opaque: false,
            expose_proxy: false,
            frozen: false,
        }
    }

    /// 设置是否直接代理目标类。
    pub fn set_proxy_target_class(&mut self, proxy_target_class: bool) {
        self.proxy_target_class = proxy_target_class;
    }

    /// 是否直接代理目标类。
    pub fn is_proxy_target_class(&self) -> bool {
        self.proxy_target_class
    }

    /// 设置是否执行激进优化。
    pub fn set_optimize(&mut self, optimize: bool) {
        self.optimize = optimize;
    }

    /// 是否执行激进优化。
    pub fn is_optimize(&self) -> bool {
        self.optimize
    }

    /// 设置是否阻止代理转换为 Advised。
    pub fn set_opaque(&mut self, opaque: bool) {
        self.opaque = opaque;
    }

    /// 是否阻止代理转换为 Advised。
    pub fn is_opaque(&self) -> bool {
        self.opaque
    }

    /// 设置是否将代理暴露为 ThreadLocal。
    pub fn set_expose_proxy(&mut self, expose_proxy: bool) {
        self.expose_proxy = expose_proxy;
    }

    /// 是否将代理暴露为 ThreadLocal。
    pub fn is_expose_proxy(&self) -> bool {
        self.expose_proxy
    }

    /// 设置是否冻结配置。
    pub fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
    }

    /// 是否冻结配置。
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// 从另一个配置复制。
    pub fn copy_from(&mut self, other: &ProxyConfig) {
        self.proxy_target_class = other.proxy_target_class;
        self.optimize = other.optimize;
        self.opaque = other.opaque;
        self.expose_proxy = other.expose_proxy;
        self.frozen = other.frozen;
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProxyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "proxyTargetClass={}; optimize={}; opaque={}; exposeProxy={}; frozen={}",
            self.proxy_target_class, self.optimize, self.opaque, self.expose_proxy, self.frozen
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_config_default() {
        let config = ProxyConfig::new();
        assert!(!config.is_proxy_target_class());
        assert!(!config.is_optimize());
        assert!(!config.is_opaque());
        assert!(!config.is_expose_proxy());
        assert!(!config.is_frozen());
    }

    #[test]
    fn proxy_config_setters() {
        let mut config = ProxyConfig::new();
        config.set_proxy_target_class(true);
        config.set_optimize(true);
        config.set_opaque(true);
        config.set_expose_proxy(true);
        config.set_frozen(true);

        assert!(config.is_proxy_target_class());
        assert!(config.is_optimize());
        assert!(config.is_opaque());
        assert!(config.is_expose_proxy());
        assert!(config.is_frozen());
    }

    #[test]
    fn proxy_config_copy_from() {
        let mut config1 = ProxyConfig::new();
        config1.set_proxy_target_class(true);
        config1.set_optimize(true);

        let mut config2 = ProxyConfig::new();
        config2.copy_from(&config1);

        assert!(config2.is_proxy_target_class());
        assert!(config2.is_optimize());
    }

    #[test]
    fn proxy_config_display() {
        let config = ProxyConfig::new();
        let display = format!("{}", config);
        assert!(display.contains("proxyTargetClass=false"));
        assert!(display.contains("frozen=false"));
    }
}

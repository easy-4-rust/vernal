//! AOP 基础设施 Bean。
//!
//! 对应 spring-aop `org.springframework.aop.framework.AopInfrastructureBean`。

/// AOP 基础设施 Bean 标记接口。
///
/// 对应 spring-aop `AopInfrastructureBean`。
///
/// 标记一个 Bean 为 AOP 基础设施。Spring 框架使用此标记
/// 来识别和排除 AOP 基础设施 Bean，避免循环依赖。
pub trait AopInfrastructureBean: Send + Sync + 'static {
    /// 是否为 AOP 基础设施。
    fn is_aop_infrastructure(&self) -> bool {
        true
    }
}

/// 默认 AOP 基础设施 Bean 实现。
pub struct DefaultAopInfrastructureBean;

impl AopInfrastructureBean for DefaultAopInfrastructureBean {
    fn is_aop_infrastructure(&self) -> bool {
        true
    }
}

impl std::fmt::Debug for DefaultAopInfrastructureBean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultAopInfrastructureBean").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aop_infrastructure_bean() {
        let bean = DefaultAopInfrastructureBean;
        assert!(bean.is_aop_infrastructure());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn default_aop_infrastructure_bean() {
        let bean = DefaultAopInfrastructureBean;
        assert!(bean.is_aop_infrastructure());
    }

    #[test]
    fn aop_infrastructure_bean_trait_object() {
        let bean: Box<dyn AopInfrastructureBean> = Box::new(DefaultAopInfrastructureBean);
        assert!(bean.is_aop_infrastructure());
    }
}

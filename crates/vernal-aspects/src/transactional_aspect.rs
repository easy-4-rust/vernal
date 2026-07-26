//! 事务切面（对标 Spring 的 @Transactional）。
//!
//! 通过 AOP 拦截器实现声明式事务管理。
//! 与 `vernal-tx` 的 `PlatformTransactionManager` 配合使用。
//!
//! # 设计来源
//!
//! 对标 Spring 的 `TransactionInterceptor` + `TransactionAspectSupport`。
//! 但 vernal 使用编译期织入（`#[intercept]` 宏）而非运行时代理。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};
use vernal_core::BoxError;

/// 事务传播行为。
///
/// 对标 Spring 的 `Propagation` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    /// 支持当前事务，如果不存在则创建新事务（默认）。
    Required,
    /// 总是创建新事务，挂起当前事务。
    RequiresNew,
    /// 支持当前事务，如果不存在则抛出异常。
    Mandatory,
    /// 如果当前事务存在，则在嵌套事务中执行。
    Nested,
    /// 支持当前事务，如果不存在则以非事务方式执行。
    Supports,
    /// 以非事务方式执行，如果存在当前事务则挂起。
    NotSupported,
    /// 以非事务方式执行，如果存在当前事务则抛出异常。
    Never,
}

impl Default for Propagation {
    fn default() -> Self {
        Self::Required
    }
}

/// 事务隔离级别。
///
/// 对标 Spring 的 `Isolation` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    /// 使用数据库默认隔离级别（默认）。
    Default,
    /// 读未提交。
    ReadUncommitted,
    /// 读已提交。
    ReadCommitted,
    /// 可重复读。
    RepeatableRead,
    /// 串行化。
    Serializable,
}

impl Default for Isolation {
    fn default() -> Self {
        Self::Default
    }
}

/// 事务配置。
///
/// 描述一个事务方法的属性，对标 Spring 的 `TransactionAttribute`。
#[derive(Debug, Clone)]
pub struct TransactionConfig {
    /// 事务传播行为
    pub propagation: Propagation,
    /// 事务隔离级别
    pub isolation: Isolation,
    /// 是否只读事务
    pub read_only: bool,
    /// 超时时间（秒），0 表示使用数据库默认
    pub timeout_secs: u64,
    /// 需要回滚的异常类型名（空 = 默认：unchecked 异常回滚）
    pub rollback_for: Vec<&'static str>,
    /// 不需要回滚的异常类型名
    pub no_rollback_for: Vec<&'static str>,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            read_only: false,
            timeout_secs: 0,
            rollback_for: Vec::new(),
            no_rollback_for: Vec::new(),
        }
    }
}

/// 事务切面（对标 Spring 的 TransactionInterceptor）。
///
/// 通过 AOP 拦截器实现声明式事务管理。
/// 在方法执行前开启事务，方法执行后根据结果提交或回滚。
///
/// # 使用方式
///
/// ```rust,ignore
/// use vernal_aspects::{TransactionalAspect, TransactionConfig, Propagation};
///
/// let config = TransactionConfig {
///     propagation: Propagation::Required,
///     read_only: false,
///     ..Default::default()
/// };
///
/// let aspect = TransactionalAspect::new(config);
/// // 注册到 AOP Advisor
/// ```
pub struct TransactionalAspect {
    /// 事务配置
    config: TransactionConfig,
}

impl TransactionalAspect {
    /// 创建事务切面。
    #[must_use]
    pub fn new(config: TransactionConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建事务切面。
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(TransactionConfig::default())
    }

    /// 获取事务配置。
    #[must_use]
    pub fn config(&self) -> &TransactionConfig {
        &self.config
    }

    /// 设置传播行为。
    #[must_use]
    pub fn with_propagation(mut self, propagation: Propagation) -> Self {
        self.config.propagation = propagation;
        self
    }

    /// 设置隔离级别。
    #[must_use]
    pub fn with_isolation(mut self, isolation: Isolation) -> Self {
        self.config.isolation = isolation;
        self
    }

    /// 设置只读事务。
    #[must_use]
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.config.read_only = read_only;
        self
    }

    /// 设置超时时间（秒）。
    #[must_use]
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.timeout_secs = timeout_secs;
        self
    }
}

impl Interceptor for TransactionalAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            // TODO: 集成 vernal-tx 的 PlatformTransactionManager
            // 当前实现：直接执行目标方法，不做事务管理
            // 完整实现需要：
            // 1. 从 invocation context 获取 TransactionManager
            // 2. 根据 propagation 决定事务行为
            // 3. 开启事务
            // 4. 执行目标方法
            // 5. 根据结果提交或回滚

            // 临时实现：直接调用 next
            let result = next.run(invocation).await;

            // TODO: 根据 config.rollback_for 和 result 决定是否回滚
            // TODO: 根据 config.propagation 决定是否挂起/恢复事务

            result
        })
    }
}

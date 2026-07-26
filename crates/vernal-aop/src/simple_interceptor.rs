//! 简化拦截器 trait。
//!
//! 对标 tx_di 的 `Interceptor` trait（before / after / around 三方法模式）。
//! 适用于不需要完整 Pointcut 表达式的简单场景（日志、指标、鉴权）。
//!
//! # 与完整 AOP 体系的区别
//!
//! | 特性 | SimpleInterceptor | Interceptor |
//! |------|-------------------|-------------|
//! | 配置 | 三注解即用 | 需要 Pointcut / Advisor / Plan / Catalog |
//! | 执行模型 | before → 原始方法 → after | 环绕 + Next 链 |
//! | 参数访问 | 仅方法名 | 完整 Invocation（含参数） |
//! | 返回值修改 | 不支持 | 支持 |
//! | 异步 | before/after 同步，around 可异步 | 全异步 |
//!
//! # 使用方式
//!
/// ```rust,ignore
/// use vernal_aop::{SimpleInterceptor, SimpleInvocationContext, SimpleCallResult};
/// use vernal_core::BoxError;
///
/// struct LoggingInterceptor;
///
/// impl SimpleInterceptor for LoggingInterceptor {
///     fn before(&self, ctx: &SimpleInvocationContext) -> Result<(), BoxError> {
///         println!("调用方法: {}", ctx.method());
///         Ok(())
///     }
///
///     fn after(&self, ctx: &SimpleInvocationContext, result: &SimpleCallResult) {
///         if result.is_err() {
///             println!("方法 {} 执行失败", ctx.method());
///         }
///     }
/// }
/// ```
use vernal_core::BoxError;

use super::{SimpleCallResult, SimpleInvocationContext};

/// 简化拦截器 trait。
///
/// 提供 before / after / around 三个可选钩子，均有默认实现。
/// 适用于日志、指标、鉴权等不需要完整 Pointcut 的简单场景。
pub trait SimpleInterceptor: Send + Sync + 'static {
    /// 方法执行前的钩子。
    ///
    /// 返回 `Ok(())` 继续执行原始方法，返回 `Err` 阻止执行。
    /// 默认实现：放行。
    fn before(&self, _context: &SimpleInvocationContext) -> Result<(), BoxError> {
        Ok(())
    }

    /// 方法执行后的钩子。
    ///
    /// 可检查结果状态（成功/失败），但不能修改返回值。
    /// 默认实现：无操作。
    fn after(&self, _context: &SimpleInvocationContext, _result: &SimpleCallResult) {}

    /// 环绕拦截。
    ///
    /// 默认实现调用 `before` → 原始方法 → `after`。
    /// 重写此方法可完全控制拦截流程（如缓存命中直接返回）。
    fn around(
        &self,
        context: &SimpleInvocationContext,
        proceed: Box<dyn FnOnce() -> SimpleCallResult + Send>,
    ) -> SimpleCallResult {
        // 默认：before → proceed → after
        if let Err(e) = self.before(context) {
            let result = SimpleCallResult::err(e);
            self.after(context, &result);
            return result;
        }
        let result = proceed();
        self.after(context, &result);
        result
    }
}

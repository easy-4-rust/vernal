//! Web 请求作用域关闭观察对象。

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{task::yield_now, time::timeout};
use vernal_ioc::ScopeState;
use vernal_web::WebRequestScope;

use crate::WebAdapterContract;

/// 在 Adapter 合同测试中跨 Handler 与响应 Body 边界观察同一个请求作用域。
///
/// Probe 只保存 Adapter 原生提取路径实际暴露的 [`WebRequestScope`]，不会注册
/// 关闭钩子、主动取消请求或调用 `close()`。因此 Scope 最终进入 Closed 只能由
/// Adapter 自身的成功、错误或取消清理路径触发，避免测试工具替被测实现完成清理。
///
/// 每个 Probe 只对应一个请求。并行请求应分别创建 Probe，借此保留请求间隔离，
/// 也避免测试代码通过进程级全局变量传递 Scope。
pub struct ScopeCloseProbe {
    observed_scope: Mutex<Option<Arc<WebRequestScope>>>,
}

impl ScopeCloseProbe {
    /// 创建尚未观察任何请求的 Probe。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observed_scope: Mutex::new(None),
        }
    }

    /// 保存 Handler、Extractor 或 Middleware 实际取得的请求作用域。
    ///
    /// # Panics
    ///
    /// 同一个 Probe 被用于多个请求时 panic。合同测试必须为每个请求建立独立 Probe，
    /// 防止后一次请求覆盖前一次请求的生命周期证据。
    pub fn observe(&self, scope: &Arc<WebRequestScope>) {
        let mut observed = self
            .observed_scope
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            observed.is_none(),
            "ScopeCloseProbe must observe exactly one request"
        );
        *observed = Some(Arc::clone(scope));
    }

    /// 验证响应 Body 尚未结束时，请求作用域依然处于 Open。
    ///
    /// # Panics
    ///
    /// 尚未观察 Scope，或 Adapter 在 Body 完成前提前关闭 Scope 时 panic。
    pub fn assert_open(&self) {
        WebAdapterContract::assert_scope_open(&self.scope());
    }

    /// 在给定时间内等待 Adapter 关闭请求作用域并验证取消状态。
    ///
    /// 该方法只轮询只读状态并向 Tokio 调度器让出执行权，不参与关闭过程。这样既能
    /// 等待 Adapter 派生的异步清理任务，也不会用固定 sleep 掩盖竞态。
    ///
    /// # Panics
    ///
    /// 尚未观察 Scope，或超时后 Scope 仍未进入 Closed 时 panic。
    pub async fn assert_closed_within(&self, maximum_wait: Duration) {
        let scope = self.scope();
        timeout(maximum_wait, async {
            while scope.state() != ScopeState::Closed {
                yield_now().await;
            }
        })
        .await
        .expect("Adapter must close request scope within the contract timeout");
        WebAdapterContract::assert_scope_closed(&scope);
    }

    /// 返回已经观察到的请求作用域。
    ///
    /// # Panics
    ///
    /// Adapter 尚未把 Scope 暴露给 Probe 时 panic。
    fn scope(&self) -> Arc<WebRequestScope> {
        self.observed_scope
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .expect("Adapter handler must expose its request scope to the probe")
    }
}

impl Default for ScopeCloseProbe {
    fn default() -> Self {
        Self::new()
    }
}

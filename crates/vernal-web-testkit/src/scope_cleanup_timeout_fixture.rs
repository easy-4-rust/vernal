//! 请求作用域清理超时合同夹具。

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use tokio::{sync::Notify, task::yield_now, time::timeout};
use vernal_context::{ApplicationContext, ScopeCleanupPolicy, VernalApplicationBuilder};
use vernal_ioc::ScopeState;
use vernal_web::WebRequestScope;

use crate::WebAdapterContract;

/// 为 Adapter 构造一个具有确定清理超时和可控阻塞钩子的真实应用请求作用域。
///
/// 夹具通过 [`VernalApplicationBuilder`] 创建真实 [`ApplicationContext`]，确保
/// `WebRequestScope` 读取的是应用冻结的 [`ScopeCleanupPolicy`]，而不是测试代码
/// 在外层额外套一层 timeout。关闭钩子只在测试显式调用 [`Self::release`] 后结束，
/// 因而可以稳定验证“当前响应等待超时、后台清理继续、最终进入 Closed”的完整语义。
///
/// 本对象不会主动调用 `WebRequestScope::close`。Scope 的第一次关闭必须由被测
/// Adapter 的响应 Body、Stream 或 Reader 发起，避免夹具替实现完成关键行为。
pub struct ScopeCleanupTimeoutFixture {
    context: Arc<ApplicationContext>,
    scope: Arc<WebRequestScope>,
    release: Arc<Notify>,
    hook_executions: Arc<AtomicUsize>,
}

impl ScopeCleanupTimeoutFixture {
    /// 创建采用指定等待上限的应用上下文，并注册一个可控阻塞关闭钩子。
    ///
    /// # Panics
    ///
    /// 应用构建、启动、请求作用域创建或关闭钩子注册失败时 panic；这些都属于测试
    /// 前置条件，而不是 Adapter 应返回的业务错误。
    pub async fn new(maximum_wait: Duration) -> Self {
        let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
        builder.scope_cleanup_policy(ScopeCleanupPolicy::bounded(maximum_wait));
        let context = Arc::new(builder.build().expect("timeout fixture context build"));
        context
            .refresh()
            .await
            .expect("timeout fixture context refresh");
        context
            .start()
            .await
            .expect("timeout fixture context start");

        let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
            &context,
        )));
        let release = Arc::new(Notify::new());
        let hook_executions = Arc::new(AtomicUsize::new(0));
        let hook_release = Arc::clone(&release);
        let executions = Arc::clone(&hook_executions);
        scope
            .on_close(move || async move {
                executions.fetch_add(1, Ordering::SeqCst);
                hook_release.notified().await;
                Ok::<(), std::convert::Infallible>(())
            })
            .expect("timeout fixture close hook registration");

        Self {
            context,
            scope,
            release,
            hook_executions,
        }
    }

    /// 返回交给 Adapter 原生响应包装器的同一个请求作用域。
    #[must_use]
    pub fn scope(&self) -> Arc<WebRequestScope> {
        Arc::clone(&self.scope)
    }

    /// 验证 Adapter 已启动唯一关闭协调器，并因阻塞钩子保持在 Closing。
    ///
    /// # Panics
    ///
    /// Adapter 没有执行钩子、重复执行钩子，或在钩子释放前错误进入 Closed 时 panic。
    pub fn assert_cleanup_timed_out(&self) {
        assert_eq!(
            self.hook_executions.load(Ordering::SeqCst),
            1,
            "Adapter must start the close hook exactly once"
        );
        assert_eq!(
            self.scope.state(),
            ScopeState::Closing,
            "timed-out cleanup must continue in the background"
        );
        assert!(
            self.scope.cancellation().is_cancelled(),
            "scope cancellation must be visible before cleanup completes"
        );
    }

    /// 验证 Context 只记录稳定、脱敏且去重的清理失败代码。
    ///
    /// # Panics
    ///
    /// 告警缺失，或诊断中出现动态错误正文时 panic。
    pub async fn assert_redacted_warning(&self) {
        assert_eq!(
            self.context.startup_report().await.warnings(),
            ["web.request-scope.cleanup-failed"]
        );
    }

    /// 允许阻塞关闭钩子结束。
    ///
    /// `notify_one` 会保留一个 permit，即使协调任务尚未轮询到 `notified()` 也不会
    /// 丢失释放信号，从而避免测试依赖调度时序。
    pub fn release(&self) {
        self.release.notify_one();
    }

    /// 等待后台协调器完成，并验证 Scope 终态以及关闭钩子的 exactly-once 语义。
    ///
    /// # Panics
    ///
    /// 后台清理没有在给定上限内结束，或关闭钩子被重复执行时 panic。
    pub async fn assert_closed_within(&self, maximum_wait: Duration) {
        timeout(maximum_wait, async {
            while self.scope.state() != ScopeState::Closed {
                yield_now().await;
            }
        })
        .await
        .expect("background cleanup must finish after the hook is released");
        WebAdapterContract::assert_scope_closed(&self.scope);
        assert_eq!(
            self.hook_executions.load(Ordering::SeqCst),
            1,
            "background cleanup must not execute the close hook twice"
        );
    }
}

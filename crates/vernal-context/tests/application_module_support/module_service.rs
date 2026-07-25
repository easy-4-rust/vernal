//! 应用模块测试服务对象。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationFuture, Next};
use vernal_context::{Lifecycle, LifecycleFuture};
use vernal_core::BoxError;
use vernal_ioc::{Component, ComponentDefinition};

use super::ModuleProbe;

/// 同时验证组件注入、生命周期登记与 `IoC` 管理 Advisor 的模块服务。
pub struct ModuleService {
    probe: Arc<ModuleProbe>,
}

impl ModuleService {
    /// 返回服务持有的模块级观测器。
    #[must_use]
    pub fn probe(&self) -> &Arc<ModuleProbe> {
        &self.probe
    }
}

impl Component for ModuleService {
    /// 声明对模块观测器的显式 Singleton 依赖。
    fn definition() -> ComponentDefinition {
        ComponentDefinition::try_singleton(|resolver| -> Result<Self, BoxError> {
            Ok(Self {
                probe: resolver.resolve::<ModuleProbe>()?,
            })
        })
        .depends_on::<ModuleProbe>()
    }
}

impl Lifecycle for ModuleService {
    /// 记录模块组件初始化。
    fn initialize(&self) -> LifecycleFuture<'_> {
        self.probe.push("initialize");
        Box::pin(async { Ok(()) })
    }

    /// 记录模块组件启动。
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        self.probe.push("start");
        Box::pin(async { Ok(()) })
    }

    /// 记录模块组件停止。
    fn stop(&self) -> LifecycleFuture<'_> {
        self.probe.push("stop");
        Box::pin(async { Ok(()) })
    }
}

impl Interceptor for ModuleService {
    /// 在业务目标前后记录同一个 `IoC` Singleton 的执行证据。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.probe.push("advisor:before");
            let result = next.run(invocation).await;
            self.probe.push("advisor:after");
            result
        })
    }
}

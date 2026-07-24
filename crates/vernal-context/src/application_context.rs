//! 应用上下文对象。

use std::sync::Arc;

use tokio::runtime::Handle;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use vernal_aop::InvocationPlanCatalog;
use vernal_ioc::{ComponentKey, Container};

use crate::{
    ContextError, ContextState, EventBus, Lifecycle, LifecyclePhase,
    application_context_builder::LifecycleResolver, context_resources::ContextResources,
};

/// 组合 `IoC` 容器与 Tokio 生命周期状态机的应用上下文。
///
/// 所有状态转换由异步互斥锁串行化；组件按依赖顺序 initialize/start，按逆序
/// stop。关闭先触发共享取消令牌，再继续释放所有组件，即使某个 stop 失败。
pub struct ApplicationContext {
    container: Container,
    lifecycle_resolvers: Arc<[(ComponentKey, Arc<LifecycleResolver>)]>,
    components: Mutex<Vec<Arc<dyn Lifecycle>>>,
    state: RwLock<ContextState>,
    operation: Mutex<()>,
    resources: ContextResources,
}

impl ApplicationContext {
    /// 由建造器创建尚未 refresh 的上下文。
    pub(crate) fn new(
        container: Container,
        lifecycle_resolvers: Vec<(ComponentKey, Arc<LifecycleResolver>)>,
        resources: ContextResources,
    ) -> Self {
        Self {
            container,
            lifecycle_resolvers: lifecycle_resolvers.into(),
            components: Mutex::new(Vec::new()),
            state: RwLock::new(ContextState::Created),
            operation: Mutex::new(()),
            resources,
        }
    }

    /// 预热单例、解析并初始化生命周期组件。
    ///
    /// # Errors
    ///
    /// 状态非法、组件解析失败或 initialize 失败时返回 [`ContextError`]。
    /// initialize 失败会逆序停止已成功初始化的组件并进入 Closed。
    pub async fn refresh(&self) -> Result<(), ContextError> {
        let _operation = self.operation.lock().await;
        self.require_state("refresh", ContextState::Created).await?;
        self.set_state(ContextState::Refreshing).await;

        if let Err(source) = self.container.warm_up() {
            self.set_state(ContextState::Failed).await;
            return Err(ContextError::ContainerWarmUp {
                source: Box::new(source),
            });
        }

        let mut initialized = Vec::with_capacity(self.lifecycle_resolvers.len());
        for (key, resolver) in self.lifecycle_resolvers.iter() {
            let component = match resolver(&self.container) {
                Ok(component) => component,
                Err(source) => {
                    self.set_state(ContextState::RollingBack).await;
                    Self::stop_all(&initialized).await;
                    self.set_state(ContextState::Closed).await;
                    return Err(ContextError::ComponentResolution {
                        component: key.clone(),
                        source: Box::new(source),
                    });
                }
            };
            if let Err(source) = component.initialize().await {
                self.set_state(ContextState::RollingBack).await;
                let error = ContextError::Lifecycle {
                    component: component.name(),
                    phase: LifecyclePhase::Initialize,
                    source,
                };
                let _ = component.stop().await;
                Self::stop_all(&initialized).await;
                self.set_state(ContextState::Closed).await;
                return Err(error);
            }
            initialized.push(component);
        }

        *self.components.lock().await = initialized;
        self.set_state(ContextState::Refreshed).await;
        Ok(())
    }

    /// 启动全部生命周期组件并进入 Ready。
    ///
    /// # Errors
    ///
    /// 状态非法或任一 start 失败时返回 [`ContextError`]。失败会取消上下文并
    /// 逆序停止所有已初始化组件。
    pub async fn start(&self) -> Result<(), ContextError> {
        let _operation = self.operation.lock().await;
        self.require_state("start", ContextState::Refreshed).await?;
        self.set_state(ContextState::Starting).await;

        let components = self.components.lock().await.clone();
        for component in &components {
            if let Err(source) = component.start(self.resources.cancellation().clone()).await {
                let error = ContextError::Lifecycle {
                    component: component.name(),
                    phase: LifecyclePhase::Start,
                    source,
                };
                self.resources.cancellation().cancel();
                self.set_state(ContextState::RollingBack).await;
                Self::stop_all(&components).await;
                self.components.lock().await.clear();
                self.set_state(ContextState::Closed).await;
                return Err(error);
            }
        }

        self.set_state(ContextState::Ready).await;
        Ok(())
    }

    /// 幂等关闭上下文。
    ///
    /// # Errors
    ///
    /// 返回第一个 stop 错误，但仍会尝试关闭其余组件并最终进入 Closed。
    pub async fn close(&self) -> Result<(), ContextError> {
        let _operation = self.operation.lock().await;
        if self.state().await == ContextState::Closed {
            return Ok(());
        }

        self.resources.cancellation().cancel();
        let draining_state = if self.state().await == ContextState::Ready {
            ContextState::Draining
        } else {
            ContextState::RollingBack
        };
        self.set_state(draining_state).await;

        let components = std::mem::take(&mut *self.components.lock().await);
        let error = Self::stop_all(&components).await;
        self.set_state(ContextState::Closed).await;
        error.map_or(Ok(()), Err)
    }

    /// 返回当前状态快照。
    pub async fn state(&self) -> ContextState {
        *self.state.read().await
    }

    /// 返回底层 `IoC` 容器。
    #[must_use]
    pub const fn container(&self) -> &Container {
        &self.container
    }

    /// 返回供组件和适配器派生子令牌的取消令牌。
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.resources.cancellation().clone()
    }

    /// 返回当前 Context 独占的类型化事件总线。
    #[must_use]
    pub fn events(&self) -> &EventBus {
        self.resources.events()
    }

    /// 返回高层应用建造器绑定的 Tokio Runtime Handle。
    ///
    /// 通过兼容性低层 API 创建的 Context 不隐式捕获 Runtime，因此返回
    /// `None`；通过 [`crate::VernalApplicationBuilder`] 创建时始终为 `Some`。
    #[must_use]
    pub fn runtime_handle(&self) -> Option<&Handle> {
        self.resources.runtime()
    }

    /// 返回应用构建阶段预编译的 AOP 调用计划目录。
    #[must_use]
    pub fn invocation_plans(&self) -> &InvocationPlanCatalog {
        self.resources.invocation_plans()
    }

    /// 校验当前状态是否符合操作前置条件。
    async fn require_state(
        &self,
        operation: &'static str,
        expected: ContextState,
    ) -> Result<(), ContextError> {
        let state = self.state().await;
        if state == expected {
            Ok(())
        } else {
            Err(ContextError::InvalidState { operation, state })
        }
    }

    /// 原子替换可观察状态。
    async fn set_state(&self, state: ContextState) {
        *self.state.write().await = state;
    }

    /// 逆序停止全部组件并保留第一个错误。
    async fn stop_all(components: &[Arc<dyn Lifecycle>]) -> Option<ContextError> {
        let mut first_error = None;
        for component in components.iter().rev() {
            let stop_error = component.stop().await.err();
            if first_error.is_none() {
                first_error = stop_error.map(|source| ContextError::Lifecycle {
                    component: component.name(),
                    phase: LifecyclePhase::Stop,
                    source,
                });
            }
        }
        first_error
    }
}

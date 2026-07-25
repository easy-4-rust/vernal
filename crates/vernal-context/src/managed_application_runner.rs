//! `IoC` 管理的应用 Runner 声明对象。

use std::{future::Future, pin::Pin, sync::Arc};

use tokio_util::sync::CancellationToken;
use vernal_core::SharedError;
use vernal_beans::{ComponentKey, Container, Qualifier, ResolveError};

use crate::{
    ApplicationRunner, ContextError, LifecycleExecutionPolicy,
    application_runner_task_executor::ApplicationRunnerTaskExecutor,
};

type ApplicationRunnerExecutionFuture =
    Pin<Box<dyn Future<Output = Result<(), ContextError>> + Send + 'static>>;
type ApplicationRunnerExecutor = dyn Fn(&Container, CancellationToken, LifecycleExecutionPolicy) -> ApplicationRunnerExecutionFuture
    + Send
    + Sync
    + 'static;

/// 保存一个具体 Runner 组件的解析与执行计划。
///
/// 类型擦除只存在于 Context 启动计划中。具体实现仍由最终应用 Container 解析，
/// 调用热路径直接持有同一个 `Arc<R>` 完成一次执行，不进行全局查找或按名称
/// downcast。
pub(crate) struct ManagedApplicationRunner {
    component: ComponentKey,
    executor: Arc<ApplicationRunnerExecutor>,
}

impl ManagedApplicationRunner {
    /// 声明一个无限定符 Singleton Runner。
    #[must_use]
    pub(crate) fn new<R>() -> Self
    where
        R: ApplicationRunner,
    {
        Self::with_resolver::<R, _>(ComponentKey::of::<R>(), |container| {
            container.resolve::<R>()
        })
    }

    /// 声明一个带精确限定符的 Singleton Runner。
    #[must_use]
    pub(crate) fn qualified<R>(qualifier: Qualifier) -> Self
    where
        R: ApplicationRunner,
    {
        let component = ComponentKey::qualified::<R>(qualifier.clone());
        Self::with_resolver::<R, _>(component, move |container| {
            container.resolve_qualified::<R>(&qualifier)
        })
    }

    /// 创建绑定具体组件解析方式的类型擦除执行计划。
    fn with_resolver<R, F>(component: ComponentKey, resolver: F) -> Self
    where
        R: ApplicationRunner,
        F: Fn(&Container) -> Result<Arc<R>, ResolveError> + Send + Sync + 'static,
    {
        let error_component = component.clone();
        let executor: Arc<ApplicationRunnerExecutor> =
            Arc::new(move |container, cancellation, policy| {
                let runner = match resolver(container) {
                    Ok(runner) => runner,
                    Err(source) => {
                        return Box::pin(std::future::ready(Err(
                            ContextError::ApplicationRunnerResolution {
                                component: error_component.clone(),
                                source: Box::new(source),
                            },
                        )));
                    }
                };
                let name = runner.name();
                Box::pin(async move {
                    // Runner 在独立任务中执行，业务错误先擦除为共享错误源。公开
                    // Display 不复制错误正文，显式 Error::source 链仍保留根因。
                    let task = tokio::spawn(async move {
                        runner
                            .run(cancellation)
                            .await
                            .map_err(|source| Arc::new(source) as SharedError)
                    });
                    ApplicationRunnerTaskExecutor::execute(
                        task,
                        name,
                        policy.start_timeout(),
                        policy.abort_timeout(),
                    )
                    .await
                })
            });

        Self {
            component,
            executor,
        }
    }

    /// 返回 Runner 组件的稳定身份。
    #[must_use]
    pub(crate) const fn component(&self) -> &ComponentKey {
        &self.component
    }

    /// 返回违反应用级一次性执行身份要求的组件作用域。
    pub(crate) fn invalid_scope(&self, container: &Container) -> Option<&'static str> {
        container
            .registry()
            .definitions()
            .iter()
            .find(|definition| definition.key() == &self.component)
            .map(|definition| definition.scope())
            .filter(|scope| !scope.is_singleton())
            .map(vernal_beans::Scope::as_str)
    }

    /// 使用最终 Container 解析并有界执行 Runner。
    pub(crate) fn execute(
        &self,
        container: &Container,
        cancellation: CancellationToken,
        policy: LifecycleExecutionPolicy,
    ) -> ApplicationRunnerExecutionFuture {
        (self.executor)(container, cancellation, policy)
    }
}

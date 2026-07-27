//! `IoC` 管理的应用事件监听声明对象。

use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use tokio::sync::broadcast;
use vernal_beans::{ComponentKey, Container, Qualifier, ResolveError};

use crate::{
    ApplicationEventListener, ContextError, EventBus, EventListenerError, ManagedTaskSupervisor,
};

type EventListenerStartFuture =
    Pin<Box<dyn Future<Output = Result<(), ContextError>> + Send + 'static>>;
type EventListenerStarter = dyn Fn(&Container, Arc<EventBus>, Arc<ManagedTaskSupervisor>) -> EventListenerStartFuture
    + Send
    + Sync
    + 'static;

/// 保存一个组件类型与一个事件类型之间的启动期监听声明。
///
/// 本对象只在 Context 装配与 refresh 阶段存在。具体监听器仍由应用 Container
/// 解析；后台任务捕获同一个 `Arc<L>`，不会额外构造实例或在每个事件上查询 `IoC`。
pub(crate) struct ManagedEventListener {
    component: ComponentKey,
    event: &'static str,
    starter: Arc<EventListenerStarter>,
}

impl ManagedEventListener {
    /// 声明一个无限定符 Singleton 监听器组件。
    #[must_use]
    pub(crate) fn new<E, L>() -> Self
    where
        E: Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        Self::with_resolver::<E, L, _>(ComponentKey::of::<L>(), |container| {
            container.resolve::<L>()
        })
    }

    /// 声明一个带精确限定符的 Singleton 监听器组件。
    #[must_use]
    pub(crate) fn qualified<E, L>(qualifier: Qualifier) -> Self
    where
        E: Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        let component = ComponentKey::qualified::<L>(qualifier.clone());
        Self::with_resolver::<E, L, _>(component, move |container| {
            container.resolve_qualified::<L>(&qualifier)
        })
    }

    /// 创建保存具体监听器解析方式的类型擦除启动声明。
    fn with_resolver<E, L, R>(component: ComponentKey, resolver: R) -> Self
    where
        E: Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
        R: Fn(&Container) -> Result<Arc<L>, ResolveError> + Send + Sync + 'static,
    {
        let event = std::any::type_name::<E>();
        let error_component = component.clone();
        let starter: Arc<EventListenerStarter> =
            Arc::new(move |container, events, managed_tasks| {
                let listener = match resolver(container) {
                    Ok(listener) => listener,
                    Err(source) => {
                        return Box::pin(std::future::ready(Err(
                            ContextError::EventListenerResolution {
                                component: error_component.clone(),
                                event,
                                source: Box::new(source),
                            },
                        )));
                    }
                };

                Box::pin(async move {
                    // 必须在提交受管任务前完成 subscribe。这样 refresh 可以保证所有
                    // 监听器已经拥有 Receiver，随后才允许 Lifecycle 初始化发布事件。
                    let receiver = events.subscribe::<E>().await;
                    let listener_name = listener.name();
                    let cancellation = managed_tasks.cancellation_token();
                    let task =
                        Self::consume::<E, L>(listener, receiver, cancellation, listener_name);
                    managed_tasks
                        .spawn(listener_name, task)
                        .map(|_| ())
                        .map_err(|source| ContextError::ManagedTask { source })
                })
            });

        Self {
            component,
            event,
            starter,
        }
    }

    /// 持续消费事件，直到应用取消、总线关闭或发生 fail-fast 错误。
    ///
    /// 在调用 `on_event` 之前先查询监听器的 SmartApplicationListener 钩子：
    /// `supports_event_type` 与 `supports_source` 任一返回 `false` 都会跳过本次
    /// 派发，对标 Spring `SimpleApplicationEventMulticaster` 在派发前调用
    /// `SmartApplicationListener#supportsEventType` /
    /// `#supportsSourceType` 的行为。
    async fn consume<E, L>(
        listener: Arc<L>,
        mut receiver: broadcast::Receiver<Arc<E>>,
        cancellation: tokio_util::sync::CancellationToken,
        listener_name: &'static str,
    ) -> Result<(), EventListenerError>
    where
        E: Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        let event = std::any::type_name::<E>();
        let event_type_id = std::any::TypeId::of::<E>();
        loop {
            tokio::select! {
                biased;
                () = cancellation.cancelled() => return Ok(()),
                message = receiver.recv() => {
                    match message {
                        Ok(value) => {
                            // SmartApplicationListener 过滤：先按事件类型，再按源类型。
                            // 对标 Spring `SimpleApplicationEventMulticaster#multicastEvent`
                            // 中的 `supportsEventType` + `supportsSourceType` 检查。
                            if !listener.supports_event_type(event_type_id) {
                                continue;
                            }
                            // 源类型过滤交给监听器：如果事件类型实现了
                            // ApplicationContextEvent，监听器可以在 supports_source
                            // 中通过 TypeId::of::<E>() 检查具体源类型。普通事件
                            // 传 None，让监听器自行决定。
                            if !listener.supports_source(None) {
                                continue;
                            }
                            listener.on_event(value).await.map_err(|source| {
                                EventListenerError::HandlerFailed {
                                    listener: listener_name,
                                    event,
                                    source: Arc::new(source),
                                }
                            })?
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            return Err(EventListenerError::Lagged {
                                listener: listener_name,
                                event,
                                skipped,
                            });
                        }
                        Err(broadcast::error::RecvError::Closed) => return Ok(()),
                    }
                }
            }
        }
    }

    /// 返回监听器组件稳定身份。
    #[must_use]
    pub(crate) const fn component(&self) -> &ComponentKey {
        &self.component
    }

    /// 返回事件 Rust 类型名。
    #[must_use]
    pub(crate) const fn event(&self) -> &'static str {
        self.event
    }

    /// 返回违反 Context 级监听任务生命周期要求的组件作用域。
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

    /// 解析组件、建立订阅并把消费 Future 交给统一任务监督器。
    pub(crate) fn start(
        &self,
        container: &Container,
        events: Arc<EventBus>,
        managed_tasks: Arc<ManagedTaskSupervisor>,
    ) -> EventListenerStartFuture {
        (self.starter)(container, events, managed_tasks)
    }
}

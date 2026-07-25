//! 应用上下文建造器对象。

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use vernal_ioc::{ComponentKey, Container, Qualifier, Registry, ResolveError};

use crate::{
    ApplicationContext, ApplicationEventListener, ContextError, Lifecycle,
    context_resources::ContextResources, managed_event_listener::ManagedEventListener,
};

pub(crate) type LifecycleResolver =
    dyn Fn(&Container) -> Result<Arc<dyn Lifecycle>, ResolveError> + Send + Sync + 'static;

/// 将 `IoC` 注册表与需要编排的生命周期组件组合成应用上下文。
///
/// 生命周期类型只保存解析函数，不提前持有实例。`build` 按 `IoC` 构建计划重新排序，
/// 从而保证 initialize/start 遵循依赖优先顺序。
pub struct ApplicationContextBuilder {
    container: Container,
    lifecycle_resolvers: Vec<(ComponentKey, Arc<LifecycleResolver>)>,
    event_listeners: Vec<ManagedEventListener>,
    resources: ContextResources,
}

impl ApplicationContextBuilder {
    /// 基于已校验注册表创建建造器。
    #[must_use]
    pub fn new(registry: Registry) -> Self {
        Self {
            container: registry.into_container(),
            lifecycle_resolvers: Vec::new(),
            event_listeners: Vec::new(),
            resources: ContextResources::standalone(),
        }
    }

    /// 基于高层应用建造器准备的内建资源创建 Context 建造器。
    ///
    /// Container 已经完成 `IoC` 管理拦截器的解析与 AOP 目录封存；Context 必须继续
    /// 持有同一个实例，不能从 Registry 再创建第二个 Container，否则会破坏
    /// Singleton 身份和组件使用追踪。
    pub(crate) fn managed(container: Container, resources: ContextResources) -> Self {
        Self {
            container,
            lifecycle_resolvers: Vec::new(),
            event_listeners: Vec::new(),
            resources,
        }
    }

    /// 注册一个无限定符生命周期组件类型。
    pub fn lifecycle<T>(&mut self) -> &mut Self
    where
        T: Lifecycle,
    {
        let resolver: Arc<LifecycleResolver> = Arc::new(|container| {
            let component: Arc<T> = container.resolve()?;
            let component: Arc<dyn Lifecycle> = component;
            Ok(component)
        });
        self.lifecycle_resolvers
            .push((ComponentKey::of::<T>(), resolver));
        self
    }

    /// 注册一个带限定符生命周期组件类型。
    pub fn lifecycle_qualified<T>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        T: Lifecycle,
    {
        let key = ComponentKey::qualified::<T>(qualifier.clone());
        let resolver: Arc<LifecycleResolver> = Arc::new(move |container| {
            let component: Arc<T> = container.resolve_qualified(&qualifier)?;
            let component: Arc<dyn Lifecycle> = component;
            Ok(component)
        });
        self.lifecycle_resolvers.push((key, resolver));
        self
    }

    /// 登记一个由无限定符 Singleton 组件实现的强类型事件监听器。
    pub(crate) fn event_listener<E, L>(&mut self) -> &mut Self
    where
        E: std::any::Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        self.event_listeners
            .push(ManagedEventListener::new::<E, L>());
        self
    }

    /// 登记一个由带限定符 Singleton 组件实现的强类型事件监听器。
    pub(crate) fn event_listener_qualified<E, L>(&mut self, qualifier: Qualifier) -> &mut Self
    where
        E: std::any::Any + Send + Sync + 'static,
        L: ApplicationEventListener<E>,
    {
        self.event_listeners
            .push(ManagedEventListener::qualified::<E, L>(qualifier));
        self
    }

    /// 校验生命周期绑定并创建 Context。
    ///
    /// # Errors
    ///
    /// 生命周期类型没有对应 `IoC` 定义时返回
    /// [`ContextError::LifecycleDefinitionNotFound`]。
    pub fn build(mut self) -> Result<ApplicationContext, ContextError> {
        let positions: HashMap<ComponentKey, usize> = self
            .container
            .registry()
            .plan()
            .keys()
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, key)| (key, index))
            .collect();

        for (key, _) in &self.lifecycle_resolvers {
            if !positions.contains_key(key) {
                return Err(ContextError::LifecycleDefinitionNotFound {
                    component: key.clone(),
                });
            }
        }
        let mut listener_declarations = HashSet::new();
        for listener in &self.event_listeners {
            if !positions.contains_key(listener.component()) {
                return Err(ContextError::EventListenerDefinitionNotFound {
                    component: listener.component().clone(),
                    event: listener.event(),
                });
            }
            if !listener_declarations.insert((listener.component().clone(), listener.event())) {
                return Err(ContextError::DuplicateEventListener {
                    component: listener.component().clone(),
                    event: listener.event(),
                });
            }
            if let Some(scope) = listener.invalid_scope(&self.container) {
                return Err(ContextError::EventListenerScope {
                    component: listener.component().clone(),
                    event: listener.event(),
                    scope,
                });
            }
        }

        self.lifecycle_resolvers
            .sort_by_key(|(key, _)| positions[key]);
        self.event_listeners
            .sort_by_key(|listener| positions[listener.component()]);
        Ok(ApplicationContext::new(
            self.container,
            self.lifecycle_resolvers,
            self.event_listeners,
            self.resources,
        ))
    }
}

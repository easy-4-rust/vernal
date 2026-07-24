//! 应用上下文建造器对象。

use std::{collections::HashMap, sync::Arc};

use vernal_ioc::{ComponentKey, Container, Qualifier, Registry, ResolveError};

use crate::{ApplicationContext, ContextError, Lifecycle, context_resources::ContextResources};

pub(crate) type LifecycleResolver =
    dyn Fn(&Container) -> Result<Arc<dyn Lifecycle>, ResolveError> + Send + Sync + 'static;

/// 将 `IoC` 注册表与需要编排的生命周期组件组合成应用上下文。
///
/// 生命周期类型只保存解析函数，不提前持有实例。`build` 按 `IoC` 构建计划重新排序，
/// 从而保证 initialize/start 遵循依赖优先顺序。
pub struct ApplicationContextBuilder {
    registry: Registry,
    lifecycle_resolvers: Vec<(ComponentKey, Arc<LifecycleResolver>)>,
    resources: ContextResources,
}

impl ApplicationContextBuilder {
    /// 基于已校验注册表创建建造器。
    #[must_use]
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            lifecycle_resolvers: Vec::new(),
            resources: ContextResources::standalone(),
        }
    }

    /// 基于高层应用建造器准备的内建资源创建 Context 建造器。
    pub(crate) fn managed(registry: Registry, resources: ContextResources) -> Self {
        Self {
            registry,
            lifecycle_resolvers: Vec::new(),
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

    /// 校验生命周期绑定并创建 Context。
    ///
    /// # Errors
    ///
    /// 生命周期类型没有对应 `IoC` 定义时返回
    /// [`ContextError::LifecycleDefinitionNotFound`]。
    pub fn build(mut self) -> Result<ApplicationContext, ContextError> {
        let positions: HashMap<ComponentKey, usize> = self
            .registry
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

        self.lifecycle_resolvers
            .sort_by_key(|(key, _)| positions[key]);
        Ok(ApplicationContext::new(
            self.registry.container(),
            self.lifecycle_resolvers,
            self.resources,
        ))
    }
}

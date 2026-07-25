//! `IoC` 管理的切面顾问声明对象。

use std::sync::Arc;

use vernal_aop::{Advisor, Interceptor, Pointcut};
use vernal_beans::{ComponentKey, Container, Qualifier, ResolveError};

type ManagedInterceptorResolver =
    dyn Fn(&Container) -> Result<Arc<dyn Interceptor>, ResolveError> + Send + Sync + 'static;

/// 描述一个需要由应用 Container 解析拦截器实例的切面顾问。
///
/// 声明只保存组件稳定身份、切点、顺序和类型安全解析函数，不提前构造拦截器。
/// 应用依赖图冻结后，Vernal 使用即将交给 `ApplicationContext` 的同一个 Container
/// 完成解析，因此拦截器可以注入 Tokio Handle、Environment、业务服务或其他
/// Singleton，并与普通组件共享完全相同的作用域和失败语义。
pub(crate) struct ManagedAdvisor {
    component: ComponentKey,
    pointcut: Arc<dyn Pointcut>,
    resolver: Arc<ManagedInterceptorResolver>,
    order: i32,
}

impl ManagedAdvisor {
    /// 声明由无限定符组件实现的线程安全拦截器。
    #[must_use]
    pub(crate) fn new<I, P>(pointcut: P, order: i32) -> Self
    where
        I: Interceptor,
        P: Pointcut,
    {
        let resolver: Arc<ManagedInterceptorResolver> = Arc::new(|container| {
            let interceptor: Arc<I> = container.resolve()?;
            let interceptor: Arc<dyn Interceptor> = interceptor;
            Ok(interceptor)
        });
        Self {
            component: ComponentKey::of::<I>(),
            pointcut: Arc::new(pointcut),
            resolver,
            order,
        }
    }

    /// 声明由精确限定符组件实现的线程安全拦截器。
    #[must_use]
    pub(crate) fn qualified<I, P>(qualifier: Qualifier, pointcut: P, order: i32) -> Self
    where
        I: Interceptor,
        P: Pointcut,
    {
        let component = ComponentKey::qualified::<I>(qualifier.clone());
        let resolver: Arc<ManagedInterceptorResolver> = Arc::new(move |container| {
            let interceptor: Arc<I> = container.resolve_qualified(&qualifier)?;
            let interceptor: Arc<dyn Interceptor> = interceptor;
            Ok(interceptor)
        });
        Self {
            component,
            pointcut: Arc::new(pointcut),
            resolver,
            order,
        }
    }

    /// 返回用于结构化构建错误与诊断的组件身份。
    #[must_use]
    pub(crate) const fn component(&self) -> &ComponentKey {
        &self.component
    }

    /// 返回违反应用级 Advisor 生命周期要求的作用域名称。
    ///
    /// 调用计划会长期持有解析出的拦截器，因此只有 Singleton 定义能准确表达其
    /// 生命周期。Transient 不应被静默提升为应用级对象，自定义 Scope 也不应在
    /// 没有活动上下文时尝试解析。
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

    /// 从应用 Container 取得拦截器并生成普通不可变 Advisor。
    ///
    /// 解析成功后生成的 Advisor 只保存 `Arc<dyn Interceptor>`；Container 不会进入
    /// AOP 热路径，组件选择与构造成本也不会在每次调用时重复发生。
    pub(crate) fn resolve(&self, container: &Container) -> Result<Advisor, ResolveError> {
        let interceptor = (self.resolver)(container)?;
        Ok(Advisor::shared(
            Arc::clone(&self.pointcut),
            interceptor,
            self.order,
        ))
    }
}

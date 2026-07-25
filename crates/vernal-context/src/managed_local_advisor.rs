//! `IoC` 管理的本地切面顾问声明对象。

use std::sync::Arc;

use vernal_aop::{LocalAdvisor, LocalInterceptor, Pointcut};
use vernal_ioc::{ComponentKey, Container, Qualifier, ResolveError};

type ManagedLocalInterceptorResolver =
    dyn Fn(&Container) -> Result<Arc<dyn LocalInterceptor>, ResolveError> + Send + Sync + 'static;

/// 描述一个需要从应用 Container 解析 `LocalInterceptor` 的本地切面顾问。
///
/// `LocalInterceptor` 对象仍满足 `Send + Sync`，只有每次调用产生的 Future、目标和
/// 返回值允许 `!Send`，所以它可以像普通组件一样注入依赖并在 Context 间隔离。
/// 解析只发生在构建阶段，运行期 Local 计划直接持有共享拦截器对象。
pub(crate) struct ManagedLocalAdvisor {
    component: ComponentKey,
    pointcut: Arc<dyn Pointcut>,
    resolver: Arc<ManagedLocalInterceptorResolver>,
    order: i32,
}

impl ManagedLocalAdvisor {
    /// 声明由无限定符组件实现的本地拦截器。
    #[must_use]
    pub(crate) fn new<I, P>(pointcut: P, order: i32) -> Self
    where
        I: LocalInterceptor,
        P: Pointcut,
    {
        let resolver: Arc<ManagedLocalInterceptorResolver> = Arc::new(|container| {
            let interceptor: Arc<I> = container.resolve()?;
            let interceptor: Arc<dyn LocalInterceptor> = interceptor;
            Ok(interceptor)
        });
        Self {
            component: ComponentKey::of::<I>(),
            pointcut: Arc::new(pointcut),
            resolver,
            order,
        }
    }

    /// 声明由精确限定符组件实现的本地拦截器。
    #[must_use]
    pub(crate) fn qualified<I, P>(qualifier: Qualifier, pointcut: P, order: i32) -> Self
    where
        I: LocalInterceptor,
        P: Pointcut,
    {
        let component = ComponentKey::qualified::<I>(qualifier.clone());
        let resolver: Arc<ManagedLocalInterceptorResolver> = Arc::new(move |container| {
            let interceptor: Arc<I> = container.resolve_qualified(&qualifier)?;
            let interceptor: Arc<dyn LocalInterceptor> = interceptor;
            Ok(interceptor)
        });
        Self {
            component,
            pointcut: Arc::new(pointcut),
            resolver,
            order,
        }
    }

    /// 返回用于构建错误的组件稳定身份。
    #[must_use]
    pub(crate) const fn component(&self) -> &ComponentKey {
        &self.component
    }

    /// 返回不适合被应用级 Local 计划长期持有的组件作用域。
    pub(crate) fn invalid_scope(&self, container: &Container) -> Option<&'static str> {
        container
            .registry()
            .definitions()
            .iter()
            .find(|definition| definition.key() == &self.component)
            .map(|definition| definition.scope())
            .filter(|scope| !scope.is_singleton())
            .map(vernal_ioc::Scope::as_str)
    }

    /// 解析组件并创建普通不可变 Local Advisor。
    pub(crate) fn resolve(&self, container: &Container) -> Result<LocalAdvisor, ResolveError> {
        let interceptor = (self.resolver)(container)?;
        Ok(LocalAdvisor::shared(
            Arc::clone(&self.pointcut),
            interceptor,
            self.order,
        ))
    }
}

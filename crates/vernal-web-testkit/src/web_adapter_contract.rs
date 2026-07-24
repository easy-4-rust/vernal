//! Web Adapter 请求绑定合同对象。

use std::{any::Any, sync::Arc};

use vernal_context::ApplicationContext;
use vernal_ioc::{ScopeKey, ScopeState};
use vernal_web::WebRequestScope;

/// 对所有 Web/RPC Adapter 执行相同的请求 Context、Scope 与组件绑定断言。
///
/// 各框架仍使用自己的 Router、Middleware、Extractor、Request Guard 或 Extension
/// 产生三个原生观察值；本对象只验证它们最终落在同一个 Vernal 语义上。这样
/// Axum、Actix Web、Rocket、Warp、Salvo、Poem、Ntex、Gotham、Tide 和 Tonic
/// 不会各自复制一套容易漂移的“看起来差不多”断言。
pub struct WebAdapterContract;

impl WebAdapterContract {
    /// 验证响应 Body 尚未结束时，请求作用域保持开放且未收到取消信号。
    ///
    /// # Panics
    ///
    /// Scope 已提前关闭或取消时 panic。
    pub fn assert_scope_open(scope: &Arc<WebRequestScope>) {
        assert_eq!(
            scope.state(),
            ScopeState::Open,
            "request scope must remain open while the native response body is alive"
        );
        assert!(
            !scope.cancellation().is_cancelled(),
            "live response body must retain an uncancelled request scope"
        );
    }

    /// 验证一个正在执行的请求绑定了正确应用、IoC Scope 与组件实例。
    ///
    /// `component` 必须是 Adapter 通过原生提取器或扩展接口得到的对象。本方法会
    /// 从同一个请求 Scope 再解析一次并校验 `Arc` 身份，从而证明提取器没有绕过
    /// 自定义作用域退回全局或无作用域解析。
    ///
    /// # Panics
    ///
    /// Context 身份、Scope 所有权/状态或组件 `Arc` 身份不符合公共合同时 panic；
    /// 该行为仅用于测试进程中的断言，不进入 Adapter 运行时代码。
    pub fn assert_request_binding<T>(
        expected_context: &Arc<ApplicationContext>,
        actual_context: &Arc<ApplicationContext>,
        scope: &Arc<WebRequestScope>,
        component: &Arc<T>,
    ) where
        T: Any + Send + Sync,
    {
        assert!(
            Arc::ptr_eq(expected_context, actual_context),
            "Adapter must expose the exact ApplicationContext instance"
        );
        let scope_context = scope
            .application_context()
            .expect("Adapter request scope must be application-bound");
        assert!(
            Arc::ptr_eq(expected_context, scope_context),
            "request scope must belong to the same ApplicationContext"
        );
        assert_eq!(
            scope.key(),
            ScopeKey::of::<WebRequestScope>(),
            "request scope must use the shared WebRequestScope identity"
        );
        Self::assert_scope_open(scope);

        // 第二次解析必须命中 ScopeContext 的同一个 OnceLock，借此证明 Adapter 的
        // 组件提取路径与公共 WebRequestScope 使用同一 IoC 缓存。
        let resolved = scope
            .resolve::<T>()
            .expect("request-scoped component must resolve through the shared scope");
        assert!(
            Arc::ptr_eq(component, &resolved),
            "Adapter component extractor must resolve through WebRequestScope"
        );
    }

    /// 验证响应 Body 完成或被丢弃后，请求作用域已进入关闭状态。
    ///
    /// # Panics
    ///
    /// Scope 没有关闭或没有发布取消信号时 panic。
    pub fn assert_scope_closed(scope: &Arc<WebRequestScope>) {
        assert_eq!(
            scope.state(),
            ScopeState::Closed,
            "response termination must close the request scope"
        );
        assert!(
            scope.cancellation().is_cancelled(),
            "closed request scope must publish cancellation"
        );
    }
}

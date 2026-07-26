//! `IoC` 管理 AOP 拦截器的应用装配合同测试。

use std::{
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use vernal_aop::{
    Advisor, Interceptor, Invocation, InvocationFuture, InvocationPlanCatalog, InvocationTarget,
    InvocationValue, LocalAdvisor, LocalInterceptor, LocalInvocationFuture,
    LocalInvocationPlanCatalog, LocalInvocationTarget, LocalInvocationValue, LocalNext, Next,
    Operation,
};
use vernal_beans::{Component, ComponentDefinition, ComponentKey};
use vernal_context::{ApplicationBuildError, ApplicationContext, VernalApplicationBuilder};
use vernal_core::BoxError;

/// 保存测试调用顺序的线程安全依赖组件。
#[derive(Default)]
struct AuditTrail {
    events: Mutex<Vec<&'static str>>,
}

impl AuditTrail {
    /// 追加一个静态阶段标识。
    fn push(&self, event: &'static str) {
        self.lock().push(event);
    }

    /// 返回当前调用顺序快照。
    fn snapshot(&self) -> Vec<&'static str> {
        self.lock().clone()
    }

    /// 获取记录集合，并在测试线程 panic 污染锁时保留已经写入的证据。
    fn lock(&self) -> MutexGuard<'_, Vec<&'static str>> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// 由调用方直接构造的传统拦截器。
struct DirectInterceptor {
    trail: Arc<AuditTrail>,
}

impl Interceptor for DirectInterceptor {
    /// 记录普通 Advisor 的进入与退出顺序。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.trail.push("direct:before");
            let result = next.run(invocation).await;
            self.trail.push("direct:after");
            result
        })
    }
}

/// 由调用方直接构造的传统 Local-AOP 拦截器。
struct DirectLocalInterceptor {
    trail: Arc<AuditTrail>,
}

impl LocalInterceptor for DirectLocalInterceptor {
    /// 记录普通 Local Advisor 的进入与退出顺序。
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async move {
            self.trail.push("direct-local:before");
            let result = next.run(invocation).await;
            self.trail.push("direct-local:after");
            result
        })
    }
}

/// 从 Vernal Container 构造并注入审计依赖的拦截器组件。
struct ManagedAuditInterceptor {
    trail: Arc<AuditTrail>,
}

impl Component for ManagedAuditInterceptor {
    /// 声明审计依赖并生成每 Container 隔离的 Singleton 定义。
    fn definition() -> ComponentDefinition {
        ComponentDefinition::try_singleton(
            |resolver| -> Result<ManagedAuditInterceptor, BoxError> {
                Ok(ManagedAuditInterceptor {
                    trail: resolver.resolve::<AuditTrail>()?,
                })
            },
        )
        .depends_on::<AuditTrail>()
    }
}

impl Interceptor for ManagedAuditInterceptor {
    /// 记录 `IoC` 管理 Advisor 的进入与退出顺序。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.trail.push("managed:before");
            let result = next.run(invocation).await;
            self.trail.push("managed:after");
            result
        })
    }
}

impl LocalInterceptor for ManagedAuditInterceptor {
    /// 使用同一个 `IoC` Singleton 环绕 Worker-local 目标。
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async move {
            self.trail.push("managed-local:before");
            let result = next.run(invocation).await;
            self.trail.push("managed-local:after");
            result
        })
    }
}

/// 只用于验证缺失组件会在构建期失败的拦截器类型。
struct MissingInterceptor;

impl Interceptor for MissingInterceptor {
    /// 该方法不应被调用，因为对应类型没有进入组件图。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        next.run(invocation)
    }
}

/// 只用于验证缺失 Local-AOP 组件会在构建期失败的类型。
struct MissingLocalInterceptor;

impl LocalInterceptor for MissingLocalInterceptor {
    /// 该方法不应被调用，因为对应类型没有进入组件图。
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        next.run(invocation)
    }
}

/// 验证组件化 Advisor 不能把 Transient 静默提升为应用级实例。
struct TransientInterceptor;

impl Interceptor for TransientInterceptor {
    /// 测试不会进入运行期调用。
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        next.run(invocation)
    }
}

impl LocalInterceptor for TransientInterceptor {
    /// 测试不会进入 Worker-local 运行期调用。
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        next.run(invocation)
    }
}

/// 组装同时包含直接和组件化 Send/Local Advisor 的应用。
fn configured_application(
    trail: &Arc<AuditTrail>,
    operation: &Operation,
    local_operation: &Operation,
) -> VernalApplicationBuilder {
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    application
        .register(ComponentDefinition::shared_arc(Arc::clone(trail)))
        .expect("audit trail definition");

    // 两类 Advisor 以相同 order 交错登记，必须按统一注册顺序嵌套，而不是按来源分组。
    application.advisor(Advisor::new(
        {
            let operation = operation.clone();
            move |candidate: &Operation| candidate == &operation
        },
        DirectInterceptor {
            trail: Arc::clone(trail),
        },
        0,
    ));
    application
        .register_advisor_component::<ManagedAuditInterceptor, _>(
            {
                let operation = operation.clone();
                move |candidate: &Operation| candidate == &operation
            },
            0,
        )
        .expect("managed advisor definition");
    application.local_advisor(LocalAdvisor::new(
        {
            let local_operation = local_operation.clone();
            move |candidate: &Operation| candidate == &local_operation
        },
        DirectLocalInterceptor {
            trail: Arc::clone(trail),
        },
        0,
    ));
    application.local_advisor_component::<ManagedAuditInterceptor, _>(
        {
            let local_operation = local_operation.clone();
            move |candidate: &Operation| candidate == &local_operation
        },
        0,
    );
    application
        .operation(operation.clone())
        .operation(local_operation.clone());
    application
}

/// 执行并校验线程安全 Advisor 的统一稳定顺序。
async fn assert_send_plan(
    context: &ApplicationContext,
    trail: &Arc<AuditTrail>,
    operation: Operation,
) {
    let catalog = context
        .container()
        .resolve::<InvocationPlanCatalog>()
        .expect("catalog component");
    assert!(
        std::ptr::eq(catalog.as_ref(), context.invocation_plans()),
        "IoC components and Context must observe the same sealed catalog"
    );
    let plan = catalog.get(&operation).expect("compiled operation");
    let target: Arc<InvocationTarget> = Arc::new({
        let trail = Arc::clone(trail);
        move |_| {
            let trail = Arc::clone(&trail);
            Box::pin(async move {
                trail.push("target");
                Ok(Box::new(7_u8) as InvocationValue)
            })
        }
    });

    let result = plan
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("intercepted call");
    assert_eq!(*result.downcast::<u8>().expect("u8 result"), 7);
    assert_eq!(
        trail.snapshot(),
        [
            "direct:before",
            "managed:before",
            "target",
            "managed:after",
            "direct:after",
        ]
    );
}

/// 执行并校验 Local Advisor 的统一稳定顺序。
async fn assert_local_plan(
    context: &ApplicationContext,
    trail: &Arc<AuditTrail>,
    local_operation: Operation,
) {
    let local_catalog = context
        .container()
        .resolve::<LocalInvocationPlanCatalog>()
        .expect("local catalog component");
    assert!(
        std::ptr::eq(local_catalog.as_ref(), context.local_invocation_plans()),
        "IoC components and Context must observe the same sealed local catalog"
    );
    let local_plan = local_catalog
        .get(&local_operation)
        .expect("compiled local operation");
    let local_target: Rc<LocalInvocationTarget> = Rc::new({
        let trail = Arc::clone(trail);
        move |_| {
            let trail = Arc::clone(&trail);
            Box::pin(async move {
                trail.push("local-target");
                Ok(Box::new(Rc::new(9_u8)) as LocalInvocationValue)
            })
        }
    });
    let local_result = local_plan
        .invoke(Invocation::new(local_operation).shared(), local_target)
        .await
        .expect("local intercepted call");
    assert_eq!(
        **local_result
            .downcast::<Rc<u8>>()
            .expect("worker-local Rc result"),
        9
    );
    assert_eq!(
        trail.snapshot(),
        [
            "direct:before",
            "managed:before",
            "target",
            "managed:after",
            "direct:after",
            "direct-local:before",
            "managed-local:before",
            "local-target",
            "managed-local:after",
            "direct-local:after",
        ]
    );
}

/// 校验计划构建期的真实解析会进入未使用定义诊断。
async fn assert_managed_definitions_are_used(context: &ApplicationContext) {
    // 组件化拦截器及其依赖在计划封存阶段已经真实解析，不应出现在未使用定义中。
    let report = context.startup_report().await;
    assert!(
        !report
            .unused_definitions()
            .contains(&ComponentKey::of::<ManagedAuditInterceptor>().to_string())
    );
    assert!(
        !report
            .unused_definitions()
            .contains(&ComponentKey::of::<AuditTrail>().to_string())
    );
}

#[tokio::test]
async fn managed_interceptor_uses_application_singleton_and_preserves_registration_order() {
    let operation = Operation::new("PaymentService", "pay");
    let local_operation = Operation::new("PaymentService", "pay_local");
    let trail = Arc::new(AuditTrail::default());
    let application = configured_application(&trail, &operation, &local_operation);
    let context = application.build().expect("application context");
    let managed = context
        .container()
        .resolve::<ManagedAuditInterceptor>()
        .expect("managed interceptor singleton");
    assert!(
        Arc::ptr_eq(&managed.trail, &trail),
        "advisor must use the dependency from the final application container"
    );

    assert_send_plan(&context, &trail, operation).await;
    assert_local_plan(&context, &trail, local_operation).await;
    assert_managed_definitions_are_used(&context).await;
    context.close().await.expect("close");
}

#[tokio::test]
async fn missing_managed_interceptor_fails_before_context_is_published() {
    let operation = Operation::new("PaymentService", "pay");
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    application.advisor_component::<MissingInterceptor, _>(
        {
            let operation = operation.clone();
            move |candidate: &Operation| candidate == &operation
        },
        0,
    );
    application.operation(operation);

    let Err(error) = application.build() else {
        panic!("missing interceptor component must fail closed");
    };
    assert!(matches!(
        error,
        ApplicationBuildError::AdvisorResolution { component, .. }
            if component == ComponentKey::of::<MissingInterceptor>()
    ));
}

#[tokio::test]
async fn missing_managed_local_interceptor_fails_before_context_is_published() {
    let operation = Operation::new("PaymentService", "pay_local");
    let mut application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    application.local_advisor_component::<MissingLocalInterceptor, _>(
        {
            let operation = operation.clone();
            move |candidate: &Operation| candidate == &operation
        },
        0,
    );
    application.operation(operation);

    let Err(error) = application.build() else {
        panic!("missing local interceptor component must fail closed");
    };
    assert!(matches!(
        error,
        ApplicationBuildError::LocalAdvisorResolution { component, .. }
            if component == ComponentKey::of::<MissingLocalInterceptor>()
    ));
}

#[tokio::test]
async fn component_advisors_reject_non_singleton_scope_in_both_execution_planes() {
    let operation = Operation::new("PaymentService", "scope");

    let mut send_application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    send_application
        .register(ComponentDefinition::transient(|_| TransientInterceptor))
        .expect("transient send interceptor");
    send_application
        .advisor_component::<TransientInterceptor, _>(|_: &Operation| true, 0)
        .operation(operation.clone());
    let Err(send_error) = send_application.build() else {
        panic!("transient send advisor must fail closed");
    };
    assert!(matches!(
        send_error,
        ApplicationBuildError::AdvisorScope {
            component,
            scope: "transient"
        } if component == ComponentKey::of::<TransientInterceptor>()
    ));

    let mut local_application = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    local_application
        .register(ComponentDefinition::transient(|_| TransientInterceptor))
        .expect("transient local interceptor");
    local_application
        .local_advisor_component::<TransientInterceptor, _>(|_: &Operation| true, 0)
        .operation(operation);
    let Err(local_error) = local_application.build() else {
        panic!("transient local advisor must fail closed");
    };
    assert!(matches!(
        local_error,
        ApplicationBuildError::AdvisorScope {
            component,
            scope: "transient"
        } if component == ComponentKey::of::<TransientInterceptor>()
    ));
}

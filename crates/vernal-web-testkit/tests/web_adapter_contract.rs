//! Web Adapter 公共绑定合同自身的可执行验证。

use std::{sync::Arc, time::Duration};

use vernal_context::ApplicationContextBuilder;
use vernal_beans::{ComponentDefinition, RegistryBuilder};
use vernal_web::WebRequestScope;
use vernal_web_testkit::{ScopeCloseProbe, WebAdapterContract};

struct RequestValue(&'static str);

#[tokio::test]
async fn contract_accepts_application_bound_scoped_component_and_closed_scope() {
    let mut registry = RegistryBuilder::new();
    registry
        .register(ComponentDefinition::scoped::<
            RequestValue,
            WebRequestScope,
            _,
        >(|_| RequestValue("request")))
        .expect("component registration");
    let context = Arc::new(
        ApplicationContextBuilder::new(registry.build().expect("registry build"))
            .build()
            .expect("context build"),
    );
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");
    let scope = Arc::new(WebRequestScope::from_application_context(Arc::clone(
        &context,
    )));
    let component = scope.resolve::<RequestValue>().expect("component");
    let probe = ScopeCloseProbe::new();

    WebAdapterContract::assert_request_binding(&context, &context, &scope, &component);
    probe.observe(&scope);
    probe.assert_open();
    assert_eq!(component.0, "request");
    scope.close().await.expect("scope close");
    probe.assert_closed_within(Duration::from_secs(1)).await;
}

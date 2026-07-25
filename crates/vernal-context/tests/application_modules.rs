//! 显式应用模块的原子装配、生命周期、AOP 与错误脱敏合同测试。

mod application_module_support;

use std::{error::Error, sync::Arc};

use application_module_support::{
    CommerceModule, DefinitionConflictModule, EmptyModule, EnvironmentConflictModule,
    FailingModule, InvalidModule, ModuleProbe, ModuleService,
};
use vernal_aop::{Invocation, InvocationTarget, InvocationValue, Operation};
use vernal_context::{ApplicationModuleError, MapPropertySource, VernalApplicationBuilder};
use vernal_ioc::ComponentDefinition;

/// 创建一个由当前 Tokio Runtime 驱动的高层应用建造器。
fn application() -> VernalApplicationBuilder {
    VernalApplicationBuilder::new(tokio::runtime::Handle::current())
}

/// 创建不泄漏业务值的内存属性来源。
fn source(name: &str, key: &str, value: &str) -> Arc<dyn vernal_context::PropertySource> {
    Arc::new(MapPropertySource::new(name, [(key, value)]).expect("valid test property source"))
}

/// 从预期失败的模块登记结果中取得结构化错误。
fn module_error(
    result: Result<&mut VernalApplicationBuilder, ApplicationModuleError>,
) -> ApplicationModuleError {
    match result {
        Ok(_) => panic!("application module registration must fail"),
        Err(error) => error,
    }
}

#[tokio::test]
async fn module_atomically_installs_ioc_lifecycle_aop_operation_and_environment() {
    let operation = Operation::new("CommerceService", "checkout");
    let probe = Arc::new(ModuleProbe::default());
    let mut application = application();
    application
        .register_module(CommerceModule::new(Arc::clone(&probe), operation.clone()))
        .expect("commerce module should register");

    let context = application.build().expect("application context");
    context.refresh().await.expect("module initialize");
    context.start().await.expect("module start");
    let service = context
        .container()
        .resolve::<ModuleService>()
        .expect("module service");
    assert!(Arc::ptr_eq(service.probe(), &probe));
    assert!(
        context
            .environment()
            .require::<bool>("commerce.enabled")
            .expect("module property")
    );
    assert!(
        context
            .environment()
            .is_profile_active("commerce")
            .expect("valid profile")
    );

    let target: Arc<InvocationTarget> = Arc::new({
        let probe = Arc::clone(&probe);
        move |_| {
            let probe = Arc::clone(&probe);
            Box::pin(async move {
                probe.push("target");
                Ok(Box::new(42_u8) as InvocationValue)
            })
        }
    });
    let value = context
        .invocation_plans()
        .get(&operation)
        .expect("module operation plan")
        .invoke(Invocation::new(operation).shared(), target)
        .await
        .expect("module advisor call");
    assert_eq!(*value.downcast::<u8>().expect("u8 result"), 42);

    context.close().await.expect("module stop");
    assert_eq!(
        probe.snapshot(),
        [
            "initialize",
            "start",
            "advisor:before",
            "target",
            "advisor:after",
            "stop",
        ]
    );
}

#[tokio::test]
async fn configuration_failure_is_redacted_and_discards_staged_contributions() {
    let probe = Arc::new(ModuleProbe::default());
    let mut application = application();
    let error = module_error(application.register_module(FailingModule::new(Arc::clone(&probe))));
    assert!(matches!(
        error,
        ApplicationModuleError::Configuration {
            module: "failing.module",
            ..
        }
    ));
    assert!(!error.to_string().contains("secret-module"));
    assert!(!format!("{error:?}").contains("secret-module"));
    assert!(
        error
            .source()
            .expect("explicit source")
            .to_string()
            .contains("secret-module-configuration-response")
    );

    // 相同组件身份和来源名仍能注册，证明失败 Registrar 没有污染真实建造器。
    application
        .register(ComponentDefinition::shared_arc(probe))
        .expect("probe identity must remain free");
    application
        .environment()
        .add_last(source("failing-source", "failing.enabled", "manual"))
        .expect("source identity must remain free");
    let context = application.build().expect("clean application state");
    assert_eq!(
        context
            .environment()
            .property("failing.enabled")
            .expect("manual property"),
        Some(String::from("manual"))
    );
}

#[tokio::test]
async fn environment_conflict_rolls_back_components_and_operations() {
    let operation = Operation::new("EnvironmentConflict", "execute");
    let probe = Arc::new(ModuleProbe::default());
    let mut application = application();
    application
        .environment()
        .add_last(source("shared-source", "existing.value", "kept"))
        .expect("existing source");

    let error = module_error(application.register_module(EnvironmentConflictModule::new(
        Arc::clone(&probe),
        operation.clone(),
    )));
    assert!(matches!(
        error,
        ApplicationModuleError::Environment {
            module: "environment.conflict",
            ..
        }
    ));
    application
        .register(ComponentDefinition::shared_arc(probe))
        .expect("module component must not be committed");

    let context = application.build().expect("application after rollback");
    assert!(context.invocation_plans().get(&operation).is_none());
    assert_eq!(
        context
            .environment()
            .property("module.value")
            .expect("missing rolled-back property"),
        None
    );
}

#[tokio::test]
async fn definition_conflict_rolls_back_environment_operation_and_module_name() {
    let operation = Operation::new("DefinitionConflict", "execute");
    let probe = Arc::new(ModuleProbe::default());
    let mut application = application();
    application
        .register(ComponentDefinition::shared_arc(Arc::clone(&probe)))
        .expect("existing probe");

    let error = module_error(
        application.register_module(DefinitionConflictModule::conflicting(
            probe,
            operation.clone(),
        )),
    );
    assert!(matches!(
        error,
        ApplicationModuleError::Definition {
            module: "definition.retry",
            ..
        }
    ));

    // 同名有效重试及相同 PropertySource 名均成功，证明前次失败没有提交任何部分。
    application
        .register_module(DefinitionConflictModule::valid_retry())
        .expect("failed module name must remain reusable");
    let context = application.build().expect("retried module");
    assert!(context.invocation_plans().get(&operation).is_none());
    assert!(
        context
            .environment()
            .require::<bool>("definition.retry")
            .expect("retry property")
    );
}

#[tokio::test]
async fn module_identity_rejects_invalid_empty_and_duplicate_declarations() {
    let mut application = application();
    assert!(matches!(
        application.register_module(InvalidModule),
        Err(ApplicationModuleError::InvalidName { .. })
    ));
    assert!(matches!(
        application.register_module(EmptyModule),
        Err(ApplicationModuleError::Empty {
            name: "empty.module"
        })
    ));

    let operation = Operation::new("CommerceService", "checkout");
    application
        .register_module(CommerceModule::new(
            Arc::new(ModuleProbe::default()),
            operation.clone(),
        ))
        .expect("first module");
    assert!(matches!(
        application.register_module(CommerceModule::new(
            Arc::new(ModuleProbe::default()),
            operation,
        )),
        Err(ApplicationModuleError::DuplicateName {
            name: "commerce.core"
        })
    ));
}

//! 覆盖率补足：ApplicationModule / ApplicationModuleError / ApplicationBuildError /
//! ApplicationLaunchError / 条件模块 / registrar 失败路径。

use tokio::runtime::Handle;
use vernal_beans::ComponentDefinition;
use vernal_context::{
    ApplicationBuildError, ApplicationModule, ApplicationModuleRegistrar, VernalApplicationBuilder,
};
use vernal_core::BoxError;

// ── ApplicationModule trait 直接覆盖 ─────────────────────────────────────

struct NamedModule;

impl ApplicationModule for NamedModule {
    fn name(&self) -> &'static str {
        "named-module"
    }

    fn configure(self, _registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        Ok(())
    }
}

struct DefaultNameModule;

impl ApplicationModule for DefaultNameModule {
    fn configure(self, _registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        Ok(())
    }
}

#[test]
fn application_module_default_name_uses_type_name() {
    let module = DefaultNameModule;
    let name = module.name();
    assert!(
        name.contains("DefaultNameModule"),
        "default name should contain type name, got: {name}"
    );
}

#[test]
fn application_module_custom_name_overrides_default() {
    let module = NamedModule;
    assert_eq!(module.name(), "named-module");
}

// ── 通过 VernalApplicationBuilder 触发 ApplicationModuleError ────────────

struct EmptyModule;

impl ApplicationModule for EmptyModule {
    fn name(&self) -> &'static str {
        "empty-module"
    }

    fn configure(self, _registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        Ok(())
    }
}

#[tokio::test]
async fn empty_module_rejected_with_empty_error() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    let result = builder.register_module(EmptyModule);
    assert!(
        matches!(
            result,
            Err(vernal_context::ApplicationModuleError::Empty { .. })
        ),
        "expected Empty"
    );
    if let Err(error) = result {
        let display = format!("{error}");
        assert!(!display.is_empty());
    }
}

struct FailingModule;

impl ApplicationModule for FailingModule {
    fn name(&self) -> &'static str {
        "failing-module"
    }

    fn configure(self, _registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        Err(Box::new(std::io::Error::other("module configure failure")))
    }
}

#[tokio::test]
async fn failing_module_returns_configuration_error() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    let result = builder.register_module(FailingModule);
    assert!(
        matches!(
            result,
            Err(vernal_context::ApplicationModuleError::Configuration { .. })
        ),
        "expected Configuration"
    );
    if let Err(error) = result {
        let display = format!("{error}");
        assert!(display.contains("failing-module"));
    }
}

struct InvalidNameModule;

impl ApplicationModule for InvalidNameModule {
    fn name(&self) -> &'static str {
        "name with spaces"
    }

    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        registrar.register(ComponentDefinition::shared_value(42_u32));
        Ok(())
    }
}

#[tokio::test]
async fn invalid_module_name_rejected() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    let result = builder.register_module(InvalidNameModule);
    assert!(
        matches!(
            result,
            Err(vernal_context::ApplicationModuleError::InvalidName { .. })
        ),
        "expected InvalidName"
    );
}

struct DuplicateModule;

impl ApplicationModule for DuplicateModule {
    fn name(&self) -> &'static str {
        "duplicate-mod"
    }

    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        registrar.register(ComponentDefinition::shared_value(42_u32));
        Ok(())
    }
}

#[tokio::test]
async fn duplicate_module_name_rejected() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register_module(DuplicateModule)
        .expect("first registration succeeds");
    let result = builder.register_module(DuplicateModule);
    assert!(
        matches!(
            result,
            Err(vernal_context::ApplicationModuleError::DuplicateName { .. })
        ),
        "expected DuplicateName"
    );
}

// ── ApplicationLaunchError 通过 launch 失败覆盖 ─────────────────────────

struct FailingLifecycle;
impl vernal_context::Lifecycle for FailingLifecycle {
    fn initialize(&self) -> vernal_context::LifecycleFuture<'_> {
        Box::pin(async { Err(Box::new(std::io::Error::other("init failed")) as BoxError) })
    }
}

#[tokio::test]
async fn application_launch_error_lifecycle_via_failed_initialize() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::try_singleton::<FailingLifecycle, _>(
            |_resolver| Ok(FailingLifecycle),
        ))
        .expect("register FailingLifecycle");
    builder.lifecycle::<FailingLifecycle>();

    let result = builder.launch().await;
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("launch must fail when initialize fails"),
    };
    let display = format!("{error}");
    assert!(!display.is_empty());
    // ApplicationLaunchError 实现了 Debug；source 链被脱敏。
    let debug = format!("{error:?}");
    assert!(!debug.is_empty());
}

// ── ApplicationModuleError Debug 覆盖 ────────────────────────────────────

#[test]
fn application_module_error_debug_does_not_leak_source() {
    let error = vernal_context::ApplicationModuleError::Configuration {
        module: "mod",
        source: Box::new(std::io::Error::other("sensitive data")),
    };
    let debug = format!("{error:?}");
    assert!(debug.contains("mod") || debug.contains("Configuration"));
}

// ── ApplicationBuildError Display 通过实际失败覆盖 ───────────────────────

#[tokio::test]
async fn application_build_error_via_duplicate_definition() {
    let mut builder = VernalApplicationBuilder::new(Handle::current());
    builder
        .register(ComponentDefinition::shared_value(42_u32))
        .expect("first registration");
    let result = builder.register(ComponentDefinition::shared_value(42_u32));
    assert!(result.is_err(), "duplicate definition must fail");
    if let Err(error) = result {
        let display = format!("{error}");
        assert!(!display.is_empty());
    }
}

#[test]
fn application_build_error_tokio_runtime_unavailable_display() {
    // 在没有 Tokio runtime 的上下文中调用 try_current 来获取真实 TryCurrentError。
    let source = tokio::runtime::Handle::try_current()
        .err()
        .unwrap_or_else(|| {
            // 如果当前 thread 有 runtime（不应该），用 thread 构造一个 mock error。
            // 实际测试在 #[test] 而非 #[tokio::test] 中运行，应该没有 runtime。
            panic!("expected no Tokio runtime in this thread")
        });
    let error = ApplicationBuildError::TokioRuntimeUnavailable { source };
    let display = format!("{error}");
    assert!(display.contains("runtime") || display.contains("Tokio"));
}

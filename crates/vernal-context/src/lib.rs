#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 应用上下文与生命周期编排。"]

mod advisor_registration;
mod application_build_error;
mod application_close_coordinator;
mod application_context;
mod application_context_builder;
mod application_context_event;
mod application_environment;
mod application_environment_builder;
mod application_event_listener;
mod application_launch_error;
mod application_module;
mod application_module_error;
mod application_module_parts;
mod application_module_registrar;
mod application_paused_event;
mod application_ready_event;
mod application_refreshed_event;
mod application_runner;
mod application_runner_failure;
mod application_runner_registrar;
mod application_runner_task_executor;
mod application_shutdown_signal;
mod application_startup_coordinator;
mod async_task;
mod component_condition;
mod component_lifecycle;
mod condition_contribution_counts;
mod condition_error;
mod condition_evaluation_snapshot;
mod conditional_component_module;
mod conditional_component_module_parts;
mod configuration_phase;
mod configuration_properties;
mod configuration_properties_error;
mod context_error;
/// 资源抽象（由 vernal-core 提供,此处保持旧路径兼容重导出）。
pub mod resource {
    pub use vernal_core::io::{
        ByteArrayResource, ClassPathResource, FileSystemResource, Resource, ResourceError,
        ResourceLoader, SimpleResourceLoader,
    };
}

mod context_resources;
mod context_state;
mod diagnostic_configuration;
mod diagnostic_outcome;
mod diagnostic_phase;
mod diagnostic_state;
mod environment_error;
mod environment_snapshot;
mod event_bus;
mod event_listener_error;
mod event_listener_registrar;
mod event_listener_registration;
mod expression_condition;
mod lifecycle_execution_policy;
mod lifecycle_future;
mod lifecycle_phase;
mod lifecycle_processor;
mod lifecycle_registrar;
mod lifecycle_task_executor;
mod local_advisor_registration;
mod managed_advisor;
mod managed_application_runner;
mod managed_event_listener;
mod managed_local_advisor;
mod managed_scheduled_task;
mod managed_task_error;
mod managed_task_id;
mod managed_task_registry;
mod managed_task_supervisor;
mod map_property_source;
mod module_environment_contribution;
mod payload_application_event;
mod predicate_condition;
mod profile_condition;
mod property_condition;
mod property_source;
mod scheduled_task;
mod scheduled_task_failure;
mod scheduled_task_registrar;
mod scope_cleanup_policy;
mod startup_observation;
mod startup_report;
mod subsystem_status;
mod system_shutdown_signal_listener;
mod task_options;
mod task_schedule;
mod task_schedule_error;
mod task_schedule_mode;
mod task_shutdown_policy;
/// Spring `@Value` 注解的 Rust 等价物。
pub mod value_binding;
mod vernal_application_builder;

pub use advisor_registration::AdvisorRegistration;
pub use application_build_error::ApplicationBuildError;
pub use application_context::ApplicationContext;
pub use application_context_builder::ApplicationContextBuilder;
pub use application_context_event::{
    ApplicationContextEvent, ApplicationContextEventBase, event_type_id,
};
pub use application_environment::ApplicationEnvironment;
pub use application_environment_builder::ApplicationEnvironmentBuilder;
pub use application_event_listener::ApplicationEventListener;
pub use application_launch_error::ApplicationLaunchError;
pub use application_module::ApplicationModule;
pub use application_module_error::ApplicationModuleError;
pub use application_module_parts::ApplicationModuleParts;
pub use application_module_registrar::ApplicationModuleRegistrar;
pub use application_paused_event::ApplicationPausedEvent;
pub use application_ready_event::ApplicationReadyEvent;
pub use application_refreshed_event::ApplicationRefreshedEvent;
pub use application_runner::ApplicationRunner;
pub use application_runner_failure::ApplicationRunnerFailure;
pub use application_shutdown_signal::ApplicationShutdownSignal;
pub use async_task::AsyncTask;
pub use component_condition::ComponentCondition;
pub use component_lifecycle::Lifecycle;
pub use condition_contribution_counts::ConditionContributionCounts;
pub use condition_error::ConditionError;
pub use condition_evaluation_snapshot::ConditionEvaluationSnapshot;
pub use conditional_component_module::ConditionalComponentModule;
pub use conditional_component_module_parts::ConditionalComponentModuleParts;
pub use configuration_phase::ConfigurationPhase;
pub use configuration_properties::ConfigurationProperties;
pub use configuration_properties_error::ConfigurationPropertiesError;
pub use context_error::ContextError;
pub use context_state::ContextState;
pub use diagnostic_configuration::DiagnosticConfiguration;
pub use diagnostic_outcome::DiagnosticOutcome;
pub use diagnostic_phase::DiagnosticPhase;
pub use diagnostic_state::DiagnosticState;
pub use environment_error::EnvironmentError;
pub use environment_snapshot::EnvironmentSnapshot;
pub use event_bus::EventBus;
pub use event_listener_error::EventListenerError;
pub use event_listener_registration::{
    ApplicationListenerRegistration, EventListenerRegistry, ListenerKey,
};
pub use expression_condition::ExpressionCondition;
pub use lifecycle_execution_policy::LifecycleExecutionPolicy;
pub use lifecycle_future::LifecycleFuture;
pub use lifecycle_phase::LifecyclePhase;
pub use lifecycle_processor::LifecycleProcessor;
pub use local_advisor_registration::LocalAdvisorRegistration;
pub use managed_task_error::ManagedTaskError;
pub use managed_task_id::ManagedTaskId;
pub use managed_task_supervisor::ManagedTaskSupervisor;
pub use map_property_source::MapPropertySource;
pub use module_environment_contribution::ModuleEnvironmentContribution;
pub use payload_application_event::PayloadApplicationEvent;
pub use predicate_condition::PredicateCondition;
pub use profile_condition::ProfileCondition;
pub use property_condition::PropertyCondition;
pub use property_source::PropertySource;
pub use scheduled_task::ScheduledTask;
pub use scheduled_task_failure::ScheduledTaskFailure;
pub use scope_cleanup_policy::ScopeCleanupPolicy;
pub use startup_observation::StartupObservation;
pub use startup_report::StartupReport;
pub use subsystem_status::SubsystemStatus;
pub use system_shutdown_signal_listener::SystemShutdownSignalListener;
pub use task_options::{TaskOptions, TaskPriority};
pub use task_schedule::TaskSchedule;
pub use task_schedule_error::TaskScheduleError;
pub use task_schedule_mode::TaskScheduleMode;
pub use task_shutdown_policy::TaskShutdownPolicy;
pub use vernal_application_builder::VernalApplicationBuilder;

/// 返回应用上下文当前成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}

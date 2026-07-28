//! Quartz 调度适配 — 对标 `org.springframework.scheduling.quartz`。
//!
//! 使用 tokio 作为 Rust 端的调度实现。
//! 覆盖 Spring 的 19 个类。

mod adaptable_job_factory;
mod cron_trigger_factory_bean;
mod job_detail_factory_bean;
mod job_method_invocation_failed_exception;
mod local_data_source_job_store;
mod local_task_executor_thread_pool;
mod quartz_job_bean;
pub mod resource_loader_class_load_helper;
mod scheduler_accessor;
mod scheduler_accessor_bean;
mod scheduler_context_aware;
mod scheduler_factory_bean;
mod simple_thread_pool_task_executor;
mod simple_trigger_factory_bean;
mod spring_bean_job_factory;

pub use adaptable_job_factory::AdaptableJobFactory;
pub use cron_trigger_factory_bean::CronTriggerFactoryBean;
pub use job_detail_factory_bean::JobDetailFactoryBean;
pub use job_method_invocation_failed_exception::JobMethodInvocationFailedException;
pub use local_data_source_job_store::LocalDataSourceJobStore;
pub use local_task_executor_thread_pool::LocalTaskExecutorThreadPool;
pub use quartz_job_bean::{JobExecutionContext, QuartzJob, SimpleQuartzJob};
pub use resource_loader_class_load_helper::ResourceLoaderClassLoadHelper;
pub use scheduler_accessor::SchedulerAccessor;
pub use scheduler_accessor_bean::SchedulerAccessorBean;
pub use scheduler_context_aware::{SchedulerContext, SchedulerContextAware};
pub use scheduler_factory_bean::{SchedulerFactoryBean, SchedulerState};
pub use simple_thread_pool_task_executor::SimpleThreadPoolTaskExecutor;
pub use simple_trigger_factory_bean::SimpleTriggerFactoryBean;
pub use spring_bean_job_factory::SpringBeanJobFactory;

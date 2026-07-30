<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-context-support 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 78 个 class/interface/enum/record；`package-info.java` 不计入 |
| 目录算法 | 去掉组织和模块根包，保留末 2 层包目录 |
| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |
| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |
| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |
| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |
| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |

本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。
<!-- current-migration-contract-end -->

## 汇总

| 指标 | 数量 |
|---|---:|
| Java 业务对象 | 78 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 6 |
| `MISSING` | 39 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 33 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `mail/javamail/mime_message_preparator.rs`：`MimeMessagePreparator`、`SimpleMimeMessagePreparator`
- 单文件多个公开对象位于 `mail/javamail/java_mail_sender.rs`：`JavaMailSender`、`LettreJavaMailSender`
- 单文件多个公开对象位于 `scheduling/quartz/scheduler_context_aware.rs`：`SchedulerContext`、`SchedulerContextAware`
- 单文件多个公开对象位于 `scheduling/quartz/local_data_source_job_store.rs`：`ConnectionProvider`、`LocalDataSourceJobStore`
- 单文件多个公开对象位于 `scheduling/quartz/job_detail_factory_bean.rs`：`JobDetail`、`JobDetailFactoryBean`
- 单文件多个公开对象位于 `scheduling/quartz/local_task_executor_thread_pool.rs`：`TaskExecutor`、`LocalTaskExecutorThreadPool`
- 单文件多个公开对象位于 `scheduling/quartz/resource_loader_class_load_helper.rs`：`ResourceLoader`、`FileSystemResourceLoader`、`ResourceLoaderClassLoadHelper`
- 单文件多个公开对象位于 `scheduling/quartz/scheduler_factory_bean.rs`：`SchedulerState`、`SchedulerFactoryBean`
- 单文件多个公开对象位于 `scheduling/quartz/cron_trigger_factory_bean.rs`：`CronTrigger`、`CronTriggerFactoryBean`
- 单文件多个公开对象位于 `scheduling/quartz/quartz_job_bean.rs`：`QuartzJob`、`JobExecutionContext`、`SimpleQuartzJob`
- 单文件多个公开对象位于 `scheduling/quartz/simple_trigger_factory_bean.rs`：`SimpleTrigger`、`SimpleTriggerFactoryBean`
- 单文件多个公开对象位于 `cache/transaction/transaction_aware_cache_decorator.rs`：`TransactionStatus`、`TransactionCallbackRegistrar`、`ImmediateCallbackRegistrar`、`TransactionAwareCacheDecorator`
- 单文件多个公开对象位于 `cache/caffeine/caffeine_spec.rs`：`CaffeineSpecParseError`、`CaffeineSpec`
- 单文件多个公开对象位于 `cache/caffeine/caffeine_cache_manager.rs`：`AsyncCacheMode`、`CaffeineCacheManager`
- 单文件多个公开对象位于 `cache/jcache/interceptor/jcache_operation_source.rs`：`JCacheOperationSource`、`JCacheOperation`、`JCacheOperationType`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.cache.caffeine.CaffeineCache` | `cache/caffeine/CaffeineCache.java` | `cache/caffeine/caffeine_cache.rs` | `cache/caffeine/caffeine_cache.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.cache.caffeine.CaffeineCacheManager` | `cache/caffeine/CaffeineCacheManager.java` | `cache/caffeine/caffeine_cache_manager.rs` | `cache/caffeine/caffeine_cache_manager.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.cache.jcache.JCacheCache` | `cache/jcache/JCacheCache.java` | `cache/jcache/j_cache_cache.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.JCacheCacheManager` | `cache/jcache/JCacheCacheManager.java` | `cache/jcache/j_cache_cache_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.JCacheManagerFactoryBean` | `cache/jcache/JCacheManagerFactoryBean.java` | `cache/jcache/j_cache_manager_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.config.AbstractJCacheConfiguration` | `cache/jcache/config/AbstractJCacheConfiguration.java` | `jcache/config/abstract_j_cache_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.config.JCacheConfigurer` | `cache/jcache/config/JCacheConfigurer.java` | `jcache/config/j_cache_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.config.JCacheConfigurerSupport` | `cache/jcache/config/JCacheConfigurerSupport.java` | `jcache/config/j_cache_configurer_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.config.ProxyJCacheConfiguration` | `cache/jcache/config/ProxyJCacheConfiguration.java` | `jcache/config/proxy_j_cache_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.AbstractCacheInterceptor` | `cache/jcache/interceptor/AbstractCacheInterceptor.java` | `jcache/interceptor/abstract_cache_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.AbstractFallbackJCacheOperationSource` | `cache/jcache/interceptor/AbstractFallbackJCacheOperationSource.java` | `jcache/interceptor/abstract_fallback_j_cache_operation_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.AbstractJCacheKeyOperation` | `cache/jcache/interceptor/AbstractJCacheKeyOperation.java` | `jcache/interceptor/abstract_j_cache_key_operation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.AbstractJCacheOperation` | `cache/jcache/interceptor/AbstractJCacheOperation.java` | `jcache/interceptor/abstract_j_cache_operation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.AbstractKeyCacheInterceptor` | `cache/jcache/interceptor/AbstractKeyCacheInterceptor.java` | `jcache/interceptor/abstract_key_cache_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.AnnotationJCacheOperationSource` | `cache/jcache/interceptor/AnnotationJCacheOperationSource.java` | `jcache/interceptor/annotation_j_cache_operation_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.BeanFactoryJCacheOperationSourceAdvisor` | `cache/jcache/interceptor/BeanFactoryJCacheOperationSourceAdvisor.java` | `jcache/interceptor/bean_factory_j_cache_operation_source_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.CachePutInterceptor` | `cache/jcache/interceptor/CachePutInterceptor.java` | `jcache/interceptor/cache_put_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.CachePutOperation` | `cache/jcache/interceptor/CachePutOperation.java` | `jcache/interceptor/cache_put_operation.rs` | `cache/jcache/interceptor/cache_put_operation.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.jcache.interceptor.CacheRemoveAllInterceptor` | `cache/jcache/interceptor/CacheRemoveAllInterceptor.java` | `jcache/interceptor/cache_remove_all_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.CacheRemoveAllOperation` | `cache/jcache/interceptor/CacheRemoveAllOperation.java` | `jcache/interceptor/cache_remove_all_operation.rs` | `cache/jcache/interceptor/cache_remove_all_operation.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.jcache.interceptor.CacheRemoveEntryInterceptor` | `cache/jcache/interceptor/CacheRemoveEntryInterceptor.java` | `jcache/interceptor/cache_remove_entry_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.CacheRemoveOperation` | `cache/jcache/interceptor/CacheRemoveOperation.java` | `jcache/interceptor/cache_remove_operation.rs` | `cache/jcache/interceptor/cache_remove_operation.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.jcache.interceptor.CacheResolverAdapter` | `cache/jcache/interceptor/CacheResolverAdapter.java` | `jcache/interceptor/cache_resolver_adapter.rs` | `cache/jcache/interceptor/cache_resolver_adapter.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.jcache.interceptor.CacheResultInterceptor` | `cache/jcache/interceptor/CacheResultInterceptor.java` | `jcache/interceptor/cache_result_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.CacheResultOperation` | `cache/jcache/interceptor/CacheResultOperation.java` | `jcache/interceptor/cache_result_operation.rs` | `cache/jcache/interceptor/cache_result_operation.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.jcache.interceptor.DefaultCacheInvocationContext` | `cache/jcache/interceptor/DefaultCacheInvocationContext.java` | `jcache/interceptor/default_cache_invocation_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.DefaultCacheKeyInvocationContext` | `cache/jcache/interceptor/DefaultCacheKeyInvocationContext.java` | `jcache/interceptor/default_cache_key_invocation_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.DefaultCacheMethodDetails` | `cache/jcache/interceptor/DefaultCacheMethodDetails.java` | `jcache/interceptor/default_cache_method_details.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.DefaultJCacheOperationSource` | `cache/jcache/interceptor/DefaultJCacheOperationSource.java` | `jcache/interceptor/default_j_cache_operation_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.JCacheAspectSupport` | `cache/jcache/interceptor/JCacheAspectSupport.java` | `jcache/interceptor/j_cache_aspect_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.JCacheInterceptor` | `cache/jcache/interceptor/JCacheInterceptor.java` | `jcache/interceptor/j_cache_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.JCacheOperation` | `cache/jcache/interceptor/JCacheOperation.java` | `jcache/interceptor/j_cache_operation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.JCacheOperationSource` | `cache/jcache/interceptor/JCacheOperationSource.java` | `jcache/interceptor/j_cache_operation_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.JCacheOperationSourcePointcut` | `cache/jcache/interceptor/JCacheOperationSourcePointcut.java` | `jcache/interceptor/j_cache_operation_source_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.KeyGeneratorAdapter` | `cache/jcache/interceptor/KeyGeneratorAdapter.java` | `jcache/interceptor/key_generator_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.jcache.interceptor.SimpleExceptionCacheResolver` | `cache/jcache/interceptor/SimpleExceptionCacheResolver.java` | `jcache/interceptor/simple_exception_cache_resolver.rs` | `cache/jcache/interceptor/simple_exception_cache_resolver.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.transaction.AbstractTransactionSupportingCacheManager` | `cache/transaction/AbstractTransactionSupportingCacheManager.java` | `cache/transaction/abstract_transaction_supporting_cache_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.transaction.TransactionAwareCacheDecorator` | `cache/transaction/TransactionAwareCacheDecorator.java` | `cache/transaction/transaction_aware_cache_decorator.rs` | `cache/transaction/transaction_aware_cache_decorator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.cache.transaction.TransactionAwareCacheManagerProxy` | `cache/transaction/TransactionAwareCacheManagerProxy.java` | `cache/transaction/transaction_aware_cache_manager_proxy.rs` | `cache/transaction/transaction_aware_cache_manager_proxy.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.MailAuthenticationException` | `mail/MailAuthenticationException.java` | `mail/mail_authentication_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.mail.MailException` | `mail/MailException.java` | `mail/mail_exception.rs` | `mail/mail_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型、测试引用 |
| `org.springframework.mail.MailMessage` | `mail/MailMessage.java` | `mail/mail_message.rs` | `mail/mail_message.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.MailParseException` | `mail/MailParseException.java` | `mail/mail_parse_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.mail.MailPreparationException` | `mail/MailPreparationException.java` | `mail/mail_preparation_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.mail.MailSendException` | `mail/MailSendException.java` | `mail/mail_send_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.mail.MailSender` | `mail/MailSender.java` | `mail/mail_sender.rs` | `mail/mail_sender.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.SimpleMailMessage` | `mail/SimpleMailMessage.java` | `mail/simple_mail_message.rs` | `mail/simple_mail_message.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.javamail.ConfigurableMimeFileTypeMap` | `mail/javamail/ConfigurableMimeFileTypeMap.java` | `mail/javamail/configurable_mime_file_type_map.rs` | `mail/javamail/configurable_mime_file_type_map.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.javamail.InternetAddressEditor` | `mail/javamail/InternetAddressEditor.java` | `mail/javamail/internet_address_editor.rs` | `mail/javamail/internet_address_editor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.javamail.JavaMailMimeTypesRuntimeHints` | `mail/javamail/JavaMailMimeTypesRuntimeHints.java` | `mail/javamail/java_mail_mime_types_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.mail.javamail.JavaMailSender` | `mail/javamail/JavaMailSender.java` | `mail/javamail/java_mail_sender.rs` | `mail/javamail/java_mail_sender.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.mail.javamail.JavaMailSenderImpl` | `mail/javamail/JavaMailSenderImpl.java` | `mail/javamail/java_mail_sender_impl.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.mail.javamail.MimeMailMessage` | `mail/javamail/MimeMailMessage.java` | `mail/javamail/mime_mail_message.rs` | `mail/javamail/mime_mail_message.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.javamail.MimeMessageHelper` | `mail/javamail/MimeMessageHelper.java` | `mail/javamail/mime_message_helper.rs` | `mail/javamail/mime_message_helper.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.javamail.MimeMessagePreparator` | `mail/javamail/MimeMessagePreparator.java` | `mail/javamail/mime_message_preparator.rs` | `mail/javamail/mime_message_preparator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.mail.javamail.SmartMimeMessage` | `mail/javamail/SmartMimeMessage.java` | `mail/javamail/smart_mime_message.rs` | `mail/javamail/smart_mime_message.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.AdaptableJobFactory` | `scheduling/quartz/AdaptableJobFactory.java` | `scheduling/quartz/adaptable_job_factory.rs` | `scheduling/quartz/adaptable_job_factory.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.scheduling.quartz.CronTriggerFactoryBean` | `scheduling/quartz/CronTriggerFactoryBean.java` | `scheduling/quartz/cron_trigger_factory_bean.rs` | `scheduling/quartz/cron_trigger_factory_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.DelegatingJob` | `scheduling/quartz/DelegatingJob.java` | `scheduling/quartz/delegating_job.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.scheduling.quartz.JobDetailFactoryBean` | `scheduling/quartz/JobDetailFactoryBean.java` | `scheduling/quartz/job_detail_factory_bean.rs` | `scheduling/quartz/job_detail_factory_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.JobMethodInvocationFailedException` | `scheduling/quartz/JobMethodInvocationFailedException.java` | `scheduling/quartz/job_method_invocation_failed_exception.rs` | `scheduling/quartz/job_method_invocation_failed_exception.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.scheduling.quartz.LocalDataSourceJobStore` | `scheduling/quartz/LocalDataSourceJobStore.java` | `scheduling/quartz/local_data_source_job_store.rs` | `scheduling/quartz/local_data_source_job_store.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.LocalTaskExecutorThreadPool` | `scheduling/quartz/LocalTaskExecutorThreadPool.java` | `scheduling/quartz/local_task_executor_thread_pool.rs` | `scheduling/quartz/local_task_executor_thread_pool.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.MethodInvokingJobDetailFactoryBean` | `scheduling/quartz/MethodInvokingJobDetailFactoryBean.java` | `scheduling/quartz/method_invoking_job_detail_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.scheduling.quartz.QuartzJobBean` | `scheduling/quartz/QuartzJobBean.java` | `scheduling/quartz/quartz_job_bean.rs` | `scheduling/quartz/quartz_job_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、同名公开主类型、测试引用 |
| `org.springframework.scheduling.quartz.ResourceLoaderClassLoadHelper` | `scheduling/quartz/ResourceLoaderClassLoadHelper.java` | `scheduling/quartz/resource_loader_class_load_helper.rs` | `scheduling/quartz/resource_loader_class_load_helper.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.SchedulerAccessor` | `scheduling/quartz/SchedulerAccessor.java` | `scheduling/quartz/scheduler_accessor.rs` | `scheduling/quartz/scheduler_accessor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.scheduling.quartz.SchedulerAccessorBean` | `scheduling/quartz/SchedulerAccessorBean.java` | `scheduling/quartz/scheduler_accessor_bean.rs` | `scheduling/quartz/scheduler_accessor_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.scheduling.quartz.SchedulerContextAware` | `scheduling/quartz/SchedulerContextAware.java` | `scheduling/quartz/scheduler_context_aware.rs` | `scheduling/quartz/scheduler_context_aware.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.SchedulerFactoryBean` | `scheduling/quartz/SchedulerFactoryBean.java` | `scheduling/quartz/scheduler_factory_bean.rs` | `scheduling/quartz/scheduler_factory_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.SchedulerFactoryBeanRuntimeHints` | `scheduling/quartz/SchedulerFactoryBeanRuntimeHints.java` | `scheduling/quartz/scheduler_factory_bean_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.scheduling.quartz.SimpleThreadPoolTaskExecutor` | `scheduling/quartz/SimpleThreadPoolTaskExecutor.java` | `scheduling/quartz/simple_thread_pool_task_executor.rs` | `scheduling/quartz/simple_thread_pool_task_executor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.scheduling.quartz.SimpleTriggerFactoryBean` | `scheduling/quartz/SimpleTriggerFactoryBean.java` | `scheduling/quartz/simple_trigger_factory_bean.rs` | `scheduling/quartz/simple_trigger_factory_bean.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.scheduling.quartz.SpringBeanJobFactory` | `scheduling/quartz/SpringBeanJobFactory.java` | `scheduling/quartz/spring_bean_job_factory.rs` | `scheduling/quartz/spring_bean_job_factory.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.ui.freemarker.FreeMarkerConfigurationFactory` | `ui/freemarker/FreeMarkerConfigurationFactory.java` | `ui/freemarker/free_marker_configuration_factory.rs` | `ui/freemarker/free_marker_configuration_factory.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.ui.freemarker.FreeMarkerConfigurationFactoryBean` | `ui/freemarker/FreeMarkerConfigurationFactoryBean.java` | `ui/freemarker/free_marker_configuration_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.ui.freemarker.FreeMarkerTemplateUtils` | `ui/freemarker/FreeMarkerTemplateUtils.java` | `ui/freemarker/free_marker_template_utils.rs` | `ui/freemarker/free_marker_template_utils.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.ui.freemarker.SpringTemplateLoader` | `ui/freemarker/SpringTemplateLoader.java` | `ui/freemarker/spring_template_loader.rs` | `ui/freemarker/spring_template_loader.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |

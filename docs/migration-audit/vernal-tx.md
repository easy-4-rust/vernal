<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-tx 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 121 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 121 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 0 |
| `MISSING` | 121 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `manager.rs`：`PlatformTransactionManager`、`TransactionError`
- 单文件多个公开对象位于 `definition.rs`：`Propagation`、`Isolation`、`TransactionDefinition`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.transaction.CannotCreateTransactionException` | `CannotCreateTransactionException.java` | `cannot_create_transaction_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.ConfigurableTransactionManager` | `ConfigurableTransactionManager.java` | `configurable_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.HeuristicCompletionException` | `HeuristicCompletionException.java` | `heuristic_completion_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.IllegalTransactionStateException` | `IllegalTransactionStateException.java` | `illegal_transaction_state_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.InvalidIsolationLevelException` | `InvalidIsolationLevelException.java` | `invalid_isolation_level_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.InvalidTimeoutException` | `InvalidTimeoutException.java` | `invalid_timeout_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.NestedTransactionNotSupportedException` | `NestedTransactionNotSupportedException.java` | `nested_transaction_not_supported_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.NoTransactionException` | `NoTransactionException.java` | `no_transaction_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.PlatformTransactionManager` | `PlatformTransactionManager.java` | `platform_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.ReactiveTransaction` | `ReactiveTransaction.java` | `reactive_transaction.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.ReactiveTransactionManager` | `ReactiveTransactionManager.java` | `reactive_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.SavepointManager` | `SavepointManager.java` | `savepoint_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.StaticTransactionDefinition` | `StaticTransactionDefinition.java` | `static_transaction_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionDefinition` | `TransactionDefinition.java` | `transaction_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionException` | `TransactionException.java` | `transaction_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionExecution` | `TransactionExecution.java` | `transaction_execution.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionExecutionListener` | `TransactionExecutionListener.java` | `transaction_execution_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionManager` | `TransactionManager.java` | `transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionStatus` | `TransactionStatus.java` | `transaction_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionSuspensionNotSupportedException` | `TransactionSuspensionNotSupportedException.java` | `transaction_suspension_not_supported_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionSystemException` | `TransactionSystemException.java` | `transaction_system_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionTimedOutException` | `TransactionTimedOutException.java` | `transaction_timed_out_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.TransactionUsageException` | `TransactionUsageException.java` | `transaction_usage_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.UnexpectedRollbackException` | `UnexpectedRollbackException.java` | `unexpected_rollback_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.AbstractTransactionManagementConfiguration` | `annotation/AbstractTransactionManagementConfiguration.java` | `annotation/abstract_transaction_management_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.AnnotationTransactionAttributeSource` | `annotation/AnnotationTransactionAttributeSource.java` | `annotation/annotation_transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.Ejb3TransactionAnnotationParser` | `annotation/Ejb3TransactionAnnotationParser.java` | `annotation/ejb3_transaction_annotation_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.EnableTransactionManagement` | `annotation/EnableTransactionManagement.java` | `annotation/enable_transaction_management.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.Isolation` | `annotation/Isolation.java` | `annotation/isolation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.JtaTransactionAnnotationParser` | `annotation/JtaTransactionAnnotationParser.java` | `annotation/jta_transaction_annotation_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.Propagation` | `annotation/Propagation.java` | `annotation/propagation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.ProxyTransactionManagementConfiguration` | `annotation/ProxyTransactionManagementConfiguration.java` | `annotation/proxy_transaction_management_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.RestrictedTransactionalEventListenerFactory` | `annotation/RestrictedTransactionalEventListenerFactory.java` | `annotation/restricted_transactional_event_listener_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.RollbackOn` | `annotation/RollbackOn.java` | `annotation/rollback_on.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.SpringTransactionAnnotationParser` | `annotation/SpringTransactionAnnotationParser.java` | `annotation/spring_transaction_annotation_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.TransactionAnnotationParser` | `annotation/TransactionAnnotationParser.java` | `annotation/transaction_annotation_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.TransactionBeanRegistrationAotProcessor` | `annotation/TransactionBeanRegistrationAotProcessor.java` | `annotation/transaction_bean_registration_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.TransactionManagementConfigurationSelector` | `annotation/TransactionManagementConfigurationSelector.java` | `annotation/transaction_management_configuration_selector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.TransactionManagementConfigurer` | `annotation/TransactionManagementConfigurer.java` | `annotation/transaction_management_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.TransactionRuntimeHints` | `annotation/TransactionRuntimeHints.java` | `annotation/transaction_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.annotation.Transactional` | `annotation/Transactional.java` | `annotation/transactional.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.config.AnnotationDrivenBeanDefinitionParser` | `config/AnnotationDrivenBeanDefinitionParser.java` | `config/annotation_driven_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.config.JtaTransactionManagerBeanDefinitionParser` | `config/JtaTransactionManagerBeanDefinitionParser.java` | `config/jta_transaction_manager_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.config.JtaTransactionManagerFactoryBean` | `config/JtaTransactionManagerFactoryBean.java` | `config/jta_transaction_manager_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.config.TransactionManagementConfigUtils` | `config/TransactionManagementConfigUtils.java` | `config/transaction_management_config_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.config.TxAdviceBeanDefinitionParser` | `config/TxAdviceBeanDefinitionParser.java` | `config/tx_advice_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.config.TxNamespaceHandler` | `config/TxNamespaceHandler.java` | `config/tx_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionPhase` | `event/TransactionPhase.java` | `event/transaction_phase.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionalApplicationListener` | `event/TransactionalApplicationListener.java` | `event/transactional_application_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionalApplicationListenerAdapter` | `event/TransactionalApplicationListenerAdapter.java` | `event/transactional_application_listener_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionalApplicationListenerMethodAdapter` | `event/TransactionalApplicationListenerMethodAdapter.java` | `event/transactional_application_listener_method_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionalApplicationListenerSynchronization` | `event/TransactionalApplicationListenerSynchronization.java` | `event/transactional_application_listener_synchronization.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionalEventListener` | `event/TransactionalEventListener.java` | `event/transactional_event_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.event.TransactionalEventListenerFactory` | `event/TransactionalEventListenerFactory.java` | `event/transactional_event_listener_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.AbstractFallbackTransactionAttributeSource` | `interceptor/AbstractFallbackTransactionAttributeSource.java` | `interceptor/abstract_fallback_transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.BeanFactoryTransactionAttributeSourceAdvisor` | `interceptor/BeanFactoryTransactionAttributeSourceAdvisor.java` | `interceptor/bean_factory_transaction_attribute_source_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.CompositeTransactionAttributeSource` | `interceptor/CompositeTransactionAttributeSource.java` | `interceptor/composite_transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.DefaultTransactionAttribute` | `interceptor/DefaultTransactionAttribute.java` | `interceptor/default_transaction_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.DelegatingTransactionAttribute` | `interceptor/DelegatingTransactionAttribute.java` | `interceptor/delegating_transaction_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.MatchAlwaysTransactionAttributeSource` | `interceptor/MatchAlwaysTransactionAttributeSource.java` | `interceptor/match_always_transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.MethodMapTransactionAttributeSource` | `interceptor/MethodMapTransactionAttributeSource.java` | `interceptor/method_map_transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.MethodRollbackEvent` | `interceptor/MethodRollbackEvent.java` | `interceptor/method_rollback_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.NameMatchTransactionAttributeSource` | `interceptor/NameMatchTransactionAttributeSource.java` | `interceptor/name_match_transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.NoRollbackRuleAttribute` | `interceptor/NoRollbackRuleAttribute.java` | `interceptor/no_rollback_rule_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.RollbackRuleAttribute` | `interceptor/RollbackRuleAttribute.java` | `interceptor/rollback_rule_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.RuleBasedTransactionAttribute` | `interceptor/RuleBasedTransactionAttribute.java` | `interceptor/rule_based_transaction_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAspectSupport` | `interceptor/TransactionAspectSupport.java` | `interceptor/transaction_aspect_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAttribute` | `interceptor/TransactionAttribute.java` | `interceptor/transaction_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAttributeEditor` | `interceptor/TransactionAttributeEditor.java` | `interceptor/transaction_attribute_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAttributeSource` | `interceptor/TransactionAttributeSource.java` | `interceptor/transaction_attribute_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAttributeSourceAdvisor` | `interceptor/TransactionAttributeSourceAdvisor.java` | `interceptor/transaction_attribute_source_advisor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAttributeSourceEditor` | `interceptor/TransactionAttributeSourceEditor.java` | `interceptor/transaction_attribute_source_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionAttributeSourcePointcut` | `interceptor/TransactionAttributeSourcePointcut.java` | `interceptor/transaction_attribute_source_pointcut.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionInterceptor` | `interceptor/TransactionInterceptor.java` | `interceptor/transaction_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionProxyFactoryBean` | `interceptor/TransactionProxyFactoryBean.java` | `interceptor/transaction_proxy_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.interceptor.TransactionalProxy` | `interceptor/TransactionalProxy.java` | `interceptor/transactional_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.JtaAfterCompletionSynchronization` | `jta/JtaAfterCompletionSynchronization.java` | `jta/jta_after_completion_synchronization.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.JtaTransactionManager` | `jta/JtaTransactionManager.java` | `jta/jta_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.JtaTransactionObject` | `jta/JtaTransactionObject.java` | `jta/jta_transaction_object.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.ManagedTransactionAdapter` | `jta/ManagedTransactionAdapter.java` | `jta/managed_transaction_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.SimpleTransactionFactory` | `jta/SimpleTransactionFactory.java` | `jta/simple_transaction_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.SpringJtaSynchronizationAdapter` | `jta/SpringJtaSynchronizationAdapter.java` | `jta/spring_jta_synchronization_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.TransactionFactory` | `jta/TransactionFactory.java` | `jta/transaction_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.UserTransactionAdapter` | `jta/UserTransactionAdapter.java` | `jta/user_transaction_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.jta.WebLogicJtaTransactionManager` | `jta/WebLogicJtaTransactionManager.java` | `jta/web_logic_jta_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.AbstractReactiveTransactionManager` | `reactive/AbstractReactiveTransactionManager.java` | `reactive/abstract_reactive_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.GenericReactiveTransaction` | `reactive/GenericReactiveTransaction.java` | `reactive/generic_reactive_transaction.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.ReactiveResourceSynchronization` | `reactive/ReactiveResourceSynchronization.java` | `reactive/reactive_resource_synchronization.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionCallback` | `reactive/TransactionCallback.java` | `reactive/transaction_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionContext` | `reactive/TransactionContext.java` | `reactive/transaction_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionContextHolder` | `reactive/TransactionContextHolder.java` | `reactive/transaction_context_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionContextManager` | `reactive/TransactionContextManager.java` | `reactive/transaction_context_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionSynchronization` | `reactive/TransactionSynchronization.java` | `reactive/transaction_synchronization.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionSynchronizationManager` | `reactive/TransactionSynchronizationManager.java` | `reactive/transaction_synchronization_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionSynchronizationUtils` | `reactive/TransactionSynchronizationUtils.java` | `reactive/transaction_synchronization_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionalEventPublisher` | `reactive/TransactionalEventPublisher.java` | `reactive/transactional_event_publisher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionalOperator` | `reactive/TransactionalOperator.java` | `reactive/transactional_operator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.reactive.TransactionalOperatorImpl` | `reactive/TransactionalOperatorImpl.java` | `reactive/transactional_operator_impl.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.AbstractPlatformTransactionManager` | `support/AbstractPlatformTransactionManager.java` | `support/abstract_platform_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.AbstractTransactionStatus` | `support/AbstractTransactionStatus.java` | `support/abstract_transaction_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.CallbackPreferringPlatformTransactionManager` | `support/CallbackPreferringPlatformTransactionManager.java` | `support/callback_preferring_platform_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.DefaultTransactionDefinition` | `support/DefaultTransactionDefinition.java` | `support/default_transaction_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.DefaultTransactionStatus` | `support/DefaultTransactionStatus.java` | `support/default_transaction_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.DelegatingTransactionDefinition` | `support/DelegatingTransactionDefinition.java` | `support/delegating_transaction_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.ResourceHolder` | `support/ResourceHolder.java` | `support/resource_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.ResourceHolderSupport` | `support/ResourceHolderSupport.java` | `support/resource_holder_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.ResourceHolderSynchronization` | `support/ResourceHolderSynchronization.java` | `support/resource_holder_synchronization.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.ResourceTransactionDefinition` | `support/ResourceTransactionDefinition.java` | `support/resource_transaction_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.ResourceTransactionManager` | `support/ResourceTransactionManager.java` | `support/resource_transaction_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.SimpleTransactionScope` | `support/SimpleTransactionScope.java` | `support/simple_transaction_scope.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.SimpleTransactionStatus` | `support/SimpleTransactionStatus.java` | `support/simple_transaction_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.SmartTransactionObject` | `support/SmartTransactionObject.java` | `support/smart_transaction_object.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionCallback` | `support/TransactionCallback.java` | `support/transaction_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionCallbackWithoutResult` | `support/TransactionCallbackWithoutResult.java` | `support/transaction_callback_without_result.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionOperations` | `support/TransactionOperations.java` | `support/transaction_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionSynchronization` | `support/TransactionSynchronization.java` | `support/transaction_synchronization.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionSynchronizationAdapter` | `support/TransactionSynchronizationAdapter.java` | `support/transaction_synchronization_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionSynchronizationManager` | `support/TransactionSynchronizationManager.java` | `support/transaction_synchronization_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionSynchronizationUtils` | `support/TransactionSynchronizationUtils.java` | `support/transaction_synchronization_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.TransactionTemplate` | `support/TransactionTemplate.java` | `support/transaction_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.support.WithoutTransactionOperations` | `support/WithoutTransactionOperations.java` | `support/without_transaction_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-context 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 196 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 196 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 3 |
| `MISSING` | 190 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 3 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `task_options.rs`：`TaskOptions`、`TaskPriority`
- 单文件多个公开对象位于 `event_listener_registration.rs`：`ListenerKey`、`ApplicationListenerRegistration`、`EventListenerRegistry`
- 单文件多个公开对象位于 `application_context_event.rs`：`ApplicationContextEvent`、`ApplicationContextEventBase`
- 单文件多个公开对象位于 `value_binding.rs`：`ValueBinding`、`ValueExpressionResolver`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.context.ApplicationContext` | `ApplicationContext.java` | `application_context.rs` | `application_context.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.context.ApplicationContextAware` | `ApplicationContextAware.java` | `application_context_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationContextException` | `ApplicationContextException.java` | `application_context_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationContextInitializer` | `ApplicationContextInitializer.java` | `application_context_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationEvent` | `ApplicationEvent.java` | `application_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationEventPublisher` | `ApplicationEventPublisher.java` | `application_event_publisher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationEventPublisherAware` | `ApplicationEventPublisherAware.java` | `application_event_publisher_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationListener` | `ApplicationListener.java` | `application_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ApplicationStartupAware` | `ApplicationStartupAware.java` | `application_startup_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ConfigurableApplicationContext` | `ConfigurableApplicationContext.java` | `configurable_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.EmbeddedValueResolverAware` | `EmbeddedValueResolverAware.java` | `embedded_value_resolver_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.EnvironmentAware` | `EnvironmentAware.java` | `environment_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.HierarchicalMessageSource` | `HierarchicalMessageSource.java` | `hierarchical_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.Lifecycle` | `Lifecycle.java` | `lifecycle.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.LifecycleProcessor` | `LifecycleProcessor.java` | `lifecycle_processor.rs` | `lifecycle_processor.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.context.MessageSource` | `MessageSource.java` | `message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.MessageSourceAware` | `MessageSourceAware.java` | `message_source_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.MessageSourceResolvable` | `MessageSourceResolvable.java` | `message_source_resolvable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.NoSuchMessageException` | `NoSuchMessageException.java` | `no_such_message_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.PayloadApplicationEvent` | `PayloadApplicationEvent.java` | `payload_application_event.rs` | `payload_application_event.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.context.Phased` | `Phased.java` | `phased.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.ResourceLoaderAware` | `ResourceLoaderAware.java` | `resource_loader_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.SmartLifecycle` | `SmartLifecycle.java` | `smart_lifecycle.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AdviceMode` | `annotation/AdviceMode.java` | `annotation/advice_mode.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AdviceModeImportSelector` | `annotation/AdviceModeImportSelector.java` | `annotation/advice_mode_import_selector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotatedBeanDefinitionReader` | `annotation/AnnotatedBeanDefinitionReader.java` | `annotation/annotated_bean_definition_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotationBeanNameGenerator` | `annotation/AnnotationBeanNameGenerator.java` | `annotation/annotation_bean_name_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotationConfigApplicationContext` | `annotation/AnnotationConfigApplicationContext.java` | `annotation/annotation_config_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotationConfigBeanDefinitionParser` | `annotation/AnnotationConfigBeanDefinitionParser.java` | `annotation/annotation_config_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotationConfigRegistry` | `annotation/AnnotationConfigRegistry.java` | `annotation/annotation_config_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotationConfigUtils` | `annotation/AnnotationConfigUtils.java` | `annotation/annotation_config_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AnnotationScopeMetadataResolver` | `annotation/AnnotationScopeMetadataResolver.java` | `annotation/annotation_scope_metadata_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AspectJAutoProxyRegistrar` | `annotation/AspectJAutoProxyRegistrar.java` | `annotation/aspect_j_auto_proxy_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.AutoProxyRegistrar` | `annotation/AutoProxyRegistrar.java` | `annotation/auto_proxy_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Bean` | `annotation/Bean.java` | `annotation/bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.BeanAnnotationHelper` | `annotation/BeanAnnotationHelper.java` | `annotation/bean_annotation_helper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.BeanMethod` | `annotation/BeanMethod.java` | `annotation/bean_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ClassPathBeanDefinitionScanner` | `annotation/ClassPathBeanDefinitionScanner.java` | `annotation/class_path_bean_definition_scanner.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ClassPathScanningCandidateComponentProvider` | `annotation/ClassPathScanningCandidateComponentProvider.java` | `annotation/class_path_scanning_candidate_component_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.CommonAnnotationBeanPostProcessor` | `annotation/CommonAnnotationBeanPostProcessor.java` | `annotation/common_annotation_bean_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ComponentScan` | `annotation/ComponentScan.java` | `annotation/component_scan.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ComponentScanAnnotationParser` | `annotation/ComponentScanAnnotationParser.java` | `annotation/component_scan_annotation_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ComponentScanBeanDefinitionParser` | `annotation/ComponentScanBeanDefinitionParser.java` | `annotation/component_scan_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ComponentScans` | `annotation/ComponentScans.java` | `annotation/component_scans.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Condition` | `annotation/Condition.java` | `annotation/condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConditionContext` | `annotation/ConditionContext.java` | `annotation/condition_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConditionEvaluator` | `annotation/ConditionEvaluator.java` | `annotation/condition_evaluator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Conditional` | `annotation/Conditional.java` | `annotation/conditional.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Configuration` | `annotation/Configuration.java` | `annotation/configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationBeanNameGenerator` | `annotation/ConfigurationBeanNameGenerator.java` | `annotation/configuration_bean_name_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationClass` | `annotation/ConfigurationClass.java` | `annotation/configuration_class.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationClassBeanDefinitionReader` | `annotation/ConfigurationClassBeanDefinitionReader.java` | `annotation/configuration_class_bean_definition_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationClassEnhancer` | `annotation/ConfigurationClassEnhancer.java` | `annotation/configuration_class_enhancer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationClassParser` | `annotation/ConfigurationClassParser.java` | `annotation/configuration_class_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationClassPostProcessor` | `annotation/ConfigurationClassPostProcessor.java` | `annotation/configuration_class_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationClassUtils` | `annotation/ConfigurationClassUtils.java` | `annotation/configuration_class_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationCondition` | `annotation/ConfigurationCondition.java` | `annotation/configuration_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConfigurationMethod` | `annotation/ConfigurationMethod.java` | `annotation/configuration_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ConflictingBeanDefinitionException` | `annotation/ConflictingBeanDefinitionException.java` | `annotation/conflicting_bean_definition_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ContextAnnotationAutowireCandidateResolver` | `annotation/ContextAnnotationAutowireCandidateResolver.java` | `annotation/context_annotation_autowire_candidate_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.DeferredImportSelector` | `annotation/DeferredImportSelector.java` | `annotation/deferred_import_selector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.DependsOn` | `annotation/DependsOn.java` | `annotation/depends_on.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Description` | `annotation/Description.java` | `annotation/description.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.EnableAspectJAutoProxy` | `annotation/EnableAspectJAutoProxy.java` | `annotation/enable_aspect_j_auto_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.EnableLoadTimeWeaving` | `annotation/EnableLoadTimeWeaving.java` | `annotation/enable_load_time_weaving.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.EnableMBeanExport` | `annotation/EnableMBeanExport.java` | `annotation/enable_m_bean_export.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Fallback` | `annotation/Fallback.java` | `annotation/fallback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.FilterType` | `annotation/FilterType.java` | `annotation/filter_type.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.FullyQualifiedAnnotationBeanNameGenerator` | `annotation/FullyQualifiedAnnotationBeanNameGenerator.java` | `annotation/fully_qualified_annotation_bean_name_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.FullyQualifiedConfigurationBeanNameGenerator` | `annotation/FullyQualifiedConfigurationBeanNameGenerator.java` | `annotation/fully_qualified_configuration_bean_name_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Import` | `annotation/Import.java` | `annotation/import.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportAware` | `annotation/ImportAware.java` | `annotation/import_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportAwareAotBeanPostProcessor` | `annotation/ImportAwareAotBeanPostProcessor.java` | `annotation/import_aware_aot_bean_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportBeanDefinitionRegistrar` | `annotation/ImportBeanDefinitionRegistrar.java` | `annotation/import_bean_definition_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportRegistry` | `annotation/ImportRegistry.java` | `annotation/import_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportResource` | `annotation/ImportResource.java` | `annotation/import_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportRuntimeHints` | `annotation/ImportRuntimeHints.java` | `annotation/import_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ImportSelector` | `annotation/ImportSelector.java` | `annotation/import_selector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Jsr330ScopeMetadataResolver` | `annotation/Jsr330ScopeMetadataResolver.java` | `annotation/jsr330_scope_metadata_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Lazy` | `annotation/Lazy.java` | `annotation/lazy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.LoadTimeWeavingConfiguration` | `annotation/LoadTimeWeavingConfiguration.java` | `annotation/load_time_weaving_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.LoadTimeWeavingConfigurer` | `annotation/LoadTimeWeavingConfigurer.java` | `annotation/load_time_weaving_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.MBeanExportConfiguration` | `annotation/MBeanExportConfiguration.java` | `annotation/m_bean_export_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ParserStrategyUtils` | `annotation/ParserStrategyUtils.java` | `annotation/parser_strategy_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Primary` | `annotation/Primary.java` | `annotation/primary.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Profile` | `annotation/Profile.java` | `annotation/profile.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ProfileCondition` | `annotation/ProfileCondition.java` | `annotation/profile_condition.rs` | `profile_condition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.annotation.PropertySource` | `annotation/PropertySource.java` | `annotation/property_source.rs` | `property_source.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.annotation.PropertySourceRegistry` | `annotation/PropertySourceRegistry.java` | `annotation/property_source_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.PropertySources` | `annotation/PropertySources.java` | `annotation/property_sources.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ProxyType` | `annotation/ProxyType.java` | `annotation/proxy_type.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Proxyable` | `annotation/Proxyable.java` | `annotation/proxyable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ReflectiveScan` | `annotation/ReflectiveScan.java` | `annotation/reflective_scan.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ResourceElementResolver` | `annotation/ResourceElementResolver.java` | `annotation/resource_element_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Role` | `annotation/Role.java` | `annotation/role.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ScannedGenericBeanDefinition` | `annotation/ScannedGenericBeanDefinition.java` | `annotation/scanned_generic_bean_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.Scope` | `annotation/Scope.java` | `annotation/scope.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ScopeMetadata` | `annotation/ScopeMetadata.java` | `annotation/scope_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ScopeMetadataResolver` | `annotation/ScopeMetadataResolver.java` | `annotation/scope_metadata_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ScopedProxyCreator` | `annotation/ScopedProxyCreator.java` | `annotation/scoped_proxy_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.ScopedProxyMode` | `annotation/ScopedProxyMode.java` | `annotation/scoped_proxy_mode.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.TypeFilterUtils` | `annotation/TypeFilterUtils.java` | `annotation/type_filter_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.AbstractAotProcessor` | `aot/AbstractAotProcessor.java` | `aot/abstract_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.AotApplicationContextInitializer` | `aot/AotApplicationContextInitializer.java` | `aot/aot_application_context_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.ApplicationContextAotGenerator` | `aot/ApplicationContextAotGenerator.java` | `aot/application_context_aot_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.ApplicationContextInitializationCodeGenerator` | `aot/ApplicationContextInitializationCodeGenerator.java` | `aot/application_context_initialization_code_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.BeanFactoryInitializationAotContributions` | `aot/BeanFactoryInitializationAotContributions.java` | `aot/bean_factory_initialization_aot_contributions.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.CglibClassHandler` | `aot/CglibClassHandler.java` | `aot/cglib_class_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.ContextAotProcessor` | `aot/ContextAotProcessor.java` | `aot/context_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.KotlinReflectionBeanRegistrationAotProcessor` | `aot/KotlinReflectionBeanRegistrationAotProcessor.java` | `aot/kotlin_reflection_bean_registration_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.ReflectiveProcessorAotContributionBuilder` | `aot/ReflectiveProcessorAotContributionBuilder.java` | `aot/reflective_processor_aot_contribution_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.ReflectiveProcessorBeanFactoryInitializationAotProcessor` | `aot/ReflectiveProcessorBeanFactoryInitializationAotProcessor.java` | `aot/reflective_processor_bean_factory_initialization_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.aot.RuntimeHintsBeanFactoryInitializationAotProcessor` | `aot/RuntimeHintsBeanFactoryInitializationAotProcessor.java` | `aot/runtime_hints_bean_factory_initialization_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.AbstractPropertyLoadingBeanDefinitionParser` | `config/AbstractPropertyLoadingBeanDefinitionParser.java` | `config/abstract_property_loading_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.ContextNamespaceHandler` | `config/ContextNamespaceHandler.java` | `config/context_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.LoadTimeWeaverBeanDefinitionParser` | `config/LoadTimeWeaverBeanDefinitionParser.java` | `config/load_time_weaver_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.MBeanExportBeanDefinitionParser` | `config/MBeanExportBeanDefinitionParser.java` | `config/m_bean_export_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.MBeanServerBeanDefinitionParser` | `config/MBeanServerBeanDefinitionParser.java` | `config/m_bean_server_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.PropertyOverrideBeanDefinitionParser` | `config/PropertyOverrideBeanDefinitionParser.java` | `config/property_override_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.PropertyPlaceholderBeanDefinitionParser` | `config/PropertyPlaceholderBeanDefinitionParser.java` | `config/property_placeholder_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.config.SpringConfiguredBeanDefinitionParser` | `config/SpringConfiguredBeanDefinitionParser.java` | `config/spring_configured_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.AbstractApplicationEventMulticaster` | `event/AbstractApplicationEventMulticaster.java` | `event/abstract_application_event_multicaster.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ApplicationContextEvent` | `event/ApplicationContextEvent.java` | `event/application_context_event.rs` | `application_context_event.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.context.event.ApplicationEventMulticaster` | `event/ApplicationEventMulticaster.java` | `event/application_event_multicaster.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ApplicationListenerMethodAdapter` | `event/ApplicationListenerMethodAdapter.java` | `event/application_listener_method_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ContextClosedEvent` | `event/ContextClosedEvent.java` | `event/context_closed_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ContextPausedEvent` | `event/ContextPausedEvent.java` | `event/context_paused_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ContextRefreshedEvent` | `event/ContextRefreshedEvent.java` | `event/context_refreshed_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ContextRestartedEvent` | `event/ContextRestartedEvent.java` | `event/context_restarted_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ContextStartedEvent` | `event/ContextStartedEvent.java` | `event/context_started_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.ContextStoppedEvent` | `event/ContextStoppedEvent.java` | `event/context_stopped_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.DefaultEventListenerFactory` | `event/DefaultEventListenerFactory.java` | `event/default_event_listener_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.EventExpressionEvaluator` | `event/EventExpressionEvaluator.java` | `event/event_expression_evaluator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.EventExpressionRootObject` | `event/EventExpressionRootObject.java` | `event/event_expression_root_object.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.EventListener` | `event/EventListener.java` | `event/event_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.EventListenerFactory` | `event/EventListenerFactory.java` | `event/event_listener_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.EventListenerMethodProcessor` | `event/EventListenerMethodProcessor.java` | `event/event_listener_method_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.EventPublicationInterceptor` | `event/EventPublicationInterceptor.java` | `event/event_publication_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.GenericApplicationListener` | `event/GenericApplicationListener.java` | `event/generic_application_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.GenericApplicationListenerAdapter` | `event/GenericApplicationListenerAdapter.java` | `event/generic_application_listener_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.GenericApplicationListenerDelegate` | `event/GenericApplicationListenerDelegate.java` | `event/generic_application_listener_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.MethodFailureEvent` | `event/MethodFailureEvent.java` | `event/method_failure_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.SimpleApplicationEventMulticaster` | `event/SimpleApplicationEventMulticaster.java` | `event/simple_application_event_multicaster.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.SmartApplicationListener` | `event/SmartApplicationListener.java` | `event/smart_application_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.event.SourceFilteringListener` | `event/SourceFilteringListener.java` | `event/source_filtering_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.AnnotatedElementKey` | `expression/AnnotatedElementKey.java` | `expression/annotated_element_key.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.BeanExpressionContextAccessor` | `expression/BeanExpressionContextAccessor.java` | `expression/bean_expression_context_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.BeanFactoryAccessor` | `expression/BeanFactoryAccessor.java` | `expression/bean_factory_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.BeanFactoryResolver` | `expression/BeanFactoryResolver.java` | `expression/bean_factory_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.CachedExpressionEvaluator` | `expression/CachedExpressionEvaluator.java` | `expression/cached_expression_evaluator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.EnvironmentAccessor` | `expression/EnvironmentAccessor.java` | `expression/environment_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.MapAccessor` | `expression/MapAccessor.java` | `expression/map_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.MethodBasedEvaluationContext` | `expression/MethodBasedEvaluationContext.java` | `expression/method_based_evaluation_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.expression.StandardBeanExpressionResolver` | `expression/StandardBeanExpressionResolver.java` | `expression/standard_bean_expression_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.i18n.LocaleContext` | `i18n/LocaleContext.java` | `i18n/locale_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.i18n.LocaleContextHolder` | `i18n/LocaleContextHolder.java` | `i18n/locale_context_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.i18n.LocaleContextThreadLocalAccessor` | `i18n/LocaleContextThreadLocalAccessor.java` | `i18n/locale_context_thread_local_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.i18n.SimpleLocaleContext` | `i18n/SimpleLocaleContext.java` | `i18n/simple_locale_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.i18n.SimpleTimeZoneAwareLocaleContext` | `i18n/SimpleTimeZoneAwareLocaleContext.java` | `i18n/simple_time_zone_aware_locale_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.i18n.TimeZoneAwareLocaleContext` | `i18n/TimeZoneAwareLocaleContext.java` | `i18n/time_zone_aware_locale_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.CandidateComponentsIndex` | `index/CandidateComponentsIndex.java` | `index/candidate_components_index.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.index.CandidateComponentsIndexLoader` | `index/CandidateComponentsIndexLoader.java` | `index/candidate_components_index_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.AbstractApplicationContext` | `support/AbstractApplicationContext.java` | `support/abstract_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.AbstractMessageSource` | `support/AbstractMessageSource.java` | `support/abstract_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.AbstractRefreshableApplicationContext` | `support/AbstractRefreshableApplicationContext.java` | `support/abstract_refreshable_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.AbstractRefreshableConfigApplicationContext` | `support/AbstractRefreshableConfigApplicationContext.java` | `support/abstract_refreshable_config_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.AbstractResourceBasedMessageSource` | `support/AbstractResourceBasedMessageSource.java` | `support/abstract_resource_based_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.AbstractXmlApplicationContext` | `support/AbstractXmlApplicationContext.java` | `support/abstract_xml_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ApplicationContextAwareProcessor` | `support/ApplicationContextAwareProcessor.java` | `support/application_context_aware_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ApplicationListenerDetector` | `support/ApplicationListenerDetector.java` | `support/application_listener_detector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ApplicationObjectSupport` | `support/ApplicationObjectSupport.java` | `support/application_object_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ClassPathXmlApplicationContext` | `support/ClassPathXmlApplicationContext.java` | `support/class_path_xml_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ContextTypeMatchClassLoader` | `support/ContextTypeMatchClassLoader.java` | `support/context_type_match_class_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ConversionServiceFactoryBean` | `support/ConversionServiceFactoryBean.java` | `support/conversion_service_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.DefaultLifecycleProcessor` | `support/DefaultLifecycleProcessor.java` | `support/default_lifecycle_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.DefaultMessageSourceResolvable` | `support/DefaultMessageSourceResolvable.java` | `support/default_message_source_resolvable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.DelegatingMessageSource` | `support/DelegatingMessageSource.java` | `support/delegating_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.EmbeddedValueResolutionSupport` | `support/EmbeddedValueResolutionSupport.java` | `support/embedded_value_resolution_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.FileSystemXmlApplicationContext` | `support/FileSystemXmlApplicationContext.java` | `support/file_system_xml_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.GenericApplicationContext` | `support/GenericApplicationContext.java` | `support/generic_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.GenericGroovyApplicationContext` | `support/GenericGroovyApplicationContext.java` | `support/generic_groovy_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.GenericXmlApplicationContext` | `support/GenericXmlApplicationContext.java` | `support/generic_xml_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.MessageSourceAccessor` | `support/MessageSourceAccessor.java` | `support/message_source_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.MessageSourceResourceBundle` | `support/MessageSourceResourceBundle.java` | `support/message_source_resource_bundle.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.MessageSourceSupport` | `support/MessageSourceSupport.java` | `support/message_source_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.PostProcessorRegistrationDelegate` | `support/PostProcessorRegistrationDelegate.java` | `support/post_processor_registration_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.PropertySourcesPlaceholderConfigurer` | `support/PropertySourcesPlaceholderConfigurer.java` | `support/property_sources_placeholder_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ReloadableResourceBundleMessageSource` | `support/ReloadableResourceBundleMessageSource.java` | `support/reloadable_resource_bundle_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.ResourceBundleMessageSource` | `support/ResourceBundleMessageSource.java` | `support/resource_bundle_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.SimpleThreadScope` | `support/SimpleThreadScope.java` | `support/simple_thread_scope.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.StaticApplicationContext` | `support/StaticApplicationContext.java` | `support/static_application_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.support.StaticMessageSource` | `support/StaticMessageSource.java` | `support/static_message_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.weaving.AspectJWeavingEnabler` | `weaving/AspectJWeavingEnabler.java` | `weaving/aspect_j_weaving_enabler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.weaving.DefaultContextLoadTimeWeaver` | `weaving/DefaultContextLoadTimeWeaver.java` | `weaving/default_context_load_time_weaver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.weaving.LoadTimeWeaverAware` | `weaving/LoadTimeWeaverAware.java` | `weaving/load_time_weaver_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.weaving.LoadTimeWeaverAwareProcessor` | `weaving/LoadTimeWeaverAwareProcessor.java` | `weaving/load_time_weaver_aware_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-beans 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 323 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 323 |
| 已处理（严格三类） | 42 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 42 |
| `MISPLACED` | 88 |
| `MISSING` | 139 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 16 |
| `UNVERIFIED` | 38 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `bean_wrapper_impl.rs`：`PropertyError`、`BeanWrapperImpl`
- 单文件多个公开对象位于 `cache.rs`：`Cache`、`SimpleCache`
- 单文件多个公开对象位于 `trait_binding.rs`：`ErasedTraitComponent`、`TraitBinding`
- 单文件多个公开对象位于 `transaction.rs`：`Transaction`、`TransactionStatus`
- 单文件多个公开对象位于 `application_event.rs`：`ApplicationEvent`、`GenericApplicationEvent`
- 单文件多个公开对象位于 `default_resource_loader.rs`：`ResourceLoader`、`DefaultResourceLoader`
- 单文件多个公开对象位于 `protocol_resolver.rs`：`ProtocolResolver`、`ClosureProtocolResolver`
- 单文件多个公开对象位于 `conversion_service.rs`：`ConversionService`、`Converter`、`DefaultConversionService`
- 单文件多个公开对象位于 `type_filter.rs`：`TypeFilter`、`RegexTypeFilter`
- 单文件多个公开对象位于 `constructor_argument_values.rs`：`ValueHolder`、`ConstructorArgumentValues`
- 单文件多个公开对象位于 `bean_descriptor.rs`：`BeanDescriptor`、`PropertyDescriptor`
- 单文件多个公开对象位于 `validation.rs`：`Validator`、`ValidationResult`
- 单文件多个公开对象位于 `configurable_property_accessor.rs`：`ConfigurablePropertyAccessor`、`ConfigurablePropertyAccessorImpl`
- 单文件多个公开对象位于 `field_metadata.rs`：`FieldDescriptor`、`TypeMetadata`
- 单文件多个公开对象位于 `document_loader.rs`：`Element`、`Document`、`DocumentLoader`
- 单文件多个公开对象位于 `bean_util.rs`：`BeanUtil`、`BeanError`
- 单文件多个公开对象位于 `method_override.rs`：`MethodOverride`、`SimpleMethodOverride`
- 单文件多个公开对象位于 `factory_bean.rs`：`FactoryBean`、`SmartFactoryBean`
- 单文件多个公开对象位于 `protocol_resolver_impl.rs`：`ProtocolResolver`、`ClosureProtocolResolver`
- 单文件多个公开对象位于 `container.rs`：`Container`、`ProxyBeanDefinition`
- 单文件多个公开对象位于 `component_definition.rs`：`ErasedComponent`、`ComponentDefinition`
- 单文件多个公开对象位于 `condition.rs`：`Condition`、`ConditionContext`
- 单文件多个公开对象位于 `xml/entity_resolver.rs`：`ResolvedEntity`、`EntityResolver`
- 单文件多个公开对象位于 `xml/default_bean_definition_document_reader.rs`：`BeanDefinitionDocumentReader`、`DefaultBeanDefinitionDocumentReader`
- 单文件多个公开对象位于 `xml/document_loader.rs`：`Document`、`XmlElement`、`DocumentLoader`、`ValidationMode`
- 单文件多个公开对象位于 `factory/annotation/autowired_annotation_bean_post_processor.rs`：`InjectionPoint`、`AutowiredAnnotationBeanPostProcessor`
- 单文件多个公开对象位于 `factory/annotation/annotated_bean_definition.rs`：`AnnotatedBeanDefinition`、`BeanMetadata`、`GenericAnnotatedBeanDefinition`
- 单文件多个公开对象位于 `factory/support/bean_name_generator.rs`：`BeanNameGenerator`、`DefaultBeanNameGenerator`
- 单文件多个公开对象位于 `factory/support/default_listable_bean_factory.rs`：`DefaultListableBeanFactory`、`SimpleAutowireCandidateResolver`
- 单文件多个公开对象位于 `factory/support/bean_definition_reader.rs`：`Resource`、`BeanDefinitionReader`、`AbstractBeanDefinitionReaderImpl`
- 单文件多个公开对象位于 `factory/aot/bean_registration_aot_processor.rs`：`AotContribution`、`BeanRegistrationAotProcessor`、`AotProcessingError`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.beans.AbstractNestablePropertyAccessor` | `AbstractNestablePropertyAccessor.java` | `abstract_nestable_property_accessor.rs` | `abstract_nestable_property_accessor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/abstract_nestable_property_accessor.rs#[cfg(test)]` |
| `org.springframework.beans.AbstractPropertyAccessor` | `AbstractPropertyAccessor.java` | `abstract_property_accessor.rs` | `abstract_property_accessor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/configurable_property_accessor.rs#[cfg(test)]` |
| `org.springframework.beans.BeanInfoFactory` | `BeanInfoFactory.java` | `bean_info_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.BeanInstantiationException` | `BeanInstantiationException.java` | `bean_instantiation_exception.rs` | `bean_instantiation_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.BeanMetadataAttribute` | `BeanMetadataAttribute.java` | `bean_metadata_attribute.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.BeanMetadataAttributeAccessor` | `BeanMetadataAttributeAccessor.java` | `bean_metadata_attribute_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.BeanMetadataElement` | `BeanMetadataElement.java` | `bean_metadata_element.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.BeanUtils` | `BeanUtils.java` | `bean_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.BeanUtilsRuntimeHints` | `BeanUtilsRuntimeHints.java` | `bean_utils_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.BeanWrapper` | `BeanWrapper.java` | `bean_wrapper.rs` | `bean_wrapper.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/bean_wrapper_impl.rs#[cfg(test)]` |
| `org.springframework.beans.BeanWrapperImpl` | `BeanWrapperImpl.java` | `bean_wrapper_impl.rs` | `bean_wrapper_impl.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/bean_wrapper_impl.rs#[cfg(test)]` |
| `org.springframework.beans.BeansException` | `BeansException.java` | `beans_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.CachedIntrospectionResults` | `CachedIntrospectionResults.java` | `cached_introspection_results.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.ConfigurablePropertyAccessor` | `ConfigurablePropertyAccessor.java` | `configurable_property_accessor.rs` | `configurable_property_accessor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/configurable_property_accessor.rs#[cfg(test)]` |
| `org.springframework.beans.ConversionNotSupportedException` | `ConversionNotSupportedException.java` | `conversion_not_supported_exception.rs` | `conversion_not_supported_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.DirectFieldAccessor` | `DirectFieldAccessor.java` | `direct_field_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.ExtendedBeanInfo` | `ExtendedBeanInfo.java` | `extended_bean_info.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.ExtendedBeanInfoFactory` | `ExtendedBeanInfoFactory.java` | `extended_bean_info_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.FatalBeanException` | `FatalBeanException.java` | `fatal_bean_exception.rs` | `fatal_bean_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.GenericTypeAwarePropertyDescriptor` | `GenericTypeAwarePropertyDescriptor.java` | `generic_type_aware_property_descriptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.InvalidPropertyException` | `InvalidPropertyException.java` | `invalid_property_exception.rs` | `invalid_property_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.Mergeable` | `Mergeable.java` | `mergeable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.MethodInvocationException` | `MethodInvocationException.java` | `method_invocation_exception.rs` | `method_invocation_exception.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.MutablePropertyValues` | `MutablePropertyValues.java` | `mutable_property_values.rs` | `mutable_property_values.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/mutable_property_values_differential.rs` |
| `org.springframework.beans.NotReadablePropertyException` | `NotReadablePropertyException.java` | `not_readable_property_exception.rs` | `not_readable_property_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.NotWritablePropertyException` | `NotWritablePropertyException.java` | `not_writable_property_exception.rs` | `not_writable_property_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.NullValueInNestedPathException` | `NullValueInNestedPathException.java` | `null_value_in_nested_path_exception.rs` | `null_value_in_nested_path_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.PropertyAccessException` | `PropertyAccessException.java` | `property_access_exception.rs` | `property_access_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.PropertyAccessor` | `PropertyAccessor.java` | `property_accessor.rs` | `property_accessor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/abstract_nestable_property_accessor.rs#[cfg(test)]` |
| `org.springframework.beans.PropertyAccessorFactory` | `PropertyAccessorFactory.java` | `property_accessor_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.PropertyAccessorUtils` | `PropertyAccessorUtils.java` | `property_accessor_utils.rs` | `property_accessor_utils.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.PropertyBatchUpdateException` | `PropertyBatchUpdateException.java` | `property_batch_update_exception.rs` | `property_batch_update_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.PropertyDescriptorUtils` | `PropertyDescriptorUtils.java` | `property_descriptor_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.PropertyEditorRegistrar` | `PropertyEditorRegistrar.java` | `property_editor_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.PropertyEditorRegistry` | `PropertyEditorRegistry.java` | `property_editor_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.PropertyEditorRegistrySupport` | `PropertyEditorRegistrySupport.java` | `property_editor_registry_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.PropertyMatches` | `PropertyMatches.java` | `property_matches.rs` | `property_matches.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.PropertyValue` | `PropertyValue.java` | `property_value.rs` | `property_value.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/mutable_property_values_differential.rs` |
| `org.springframework.beans.PropertyValues` | `PropertyValues.java` | `property_values.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.PropertyValuesEditor` | `PropertyValuesEditor.java` | `property_values_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.SimpleBeanInfoFactory` | `SimpleBeanInfoFactory.java` | `simple_bean_info_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.SimpleTypeConverter` | `SimpleTypeConverter.java` | `simple_type_converter.rs` | `simple_type_converter.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.StandardBeanInfoFactory` | `StandardBeanInfoFactory.java` | `standard_bean_info_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.TypeConverter` | `TypeConverter.java` | `type_converter.rs` | `type_converter.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/container_registry_type_converter.rs` |
| `org.springframework.beans.TypeConverterDelegate` | `TypeConverterDelegate.java` | `type_converter_delegate.rs` | `type_converter_delegate.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/container_registry_type_converter.rs` |
| `org.springframework.beans.TypeConverterSupport` | `TypeConverterSupport.java` | `type_converter_support.rs` | `type_converter_support.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.TypeMismatchException` | `TypeMismatchException.java` | `type_mismatch_exception.rs` | `type_mismatch_exception.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.factory.Aware` | `factory/Aware.java` | `factory/aware.rs` | `aware.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanClassLoaderAware` | `factory/BeanClassLoaderAware.java` | `factory/bean_class_loader_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanCreationException` | `factory/BeanCreationException.java` | `factory/bean_creation_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanCreationNotAllowedException` | `factory/BeanCreationNotAllowedException.java` | `factory/bean_creation_not_allowed_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanCurrentlyInCreationException` | `factory/BeanCurrentlyInCreationException.java` | `factory/bean_currently_in_creation_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanDefinitionStoreException` | `factory/BeanDefinitionStoreException.java` | `factory/bean_definition_store_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanExpressionException` | `factory/BeanExpressionException.java` | `factory/bean_expression_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanFactory` | `factory/BeanFactory.java` | `factory/bean_factory.rs` | `bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanFactoryAware` | `factory/BeanFactoryAware.java` | `factory/bean_factory_aware.rs` | `bean_factory_aware.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanFactoryInitializer` | `factory/BeanFactoryInitializer.java` | `factory/bean_factory_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanFactoryUtils` | `factory/BeanFactoryUtils.java` | `factory/bean_factory_utils.rs` | `bean_factory_utils.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanInitializationException` | `factory/BeanInitializationException.java` | `factory/bean_initialization_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanIsAbstractException` | `factory/BeanIsAbstractException.java` | `factory/bean_is_abstract_exception.rs` | `bean_is_abstract_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanIsNotAFactoryException` | `factory/BeanIsNotAFactoryException.java` | `factory/bean_is_not_a_factory_exception.rs` | `bean_is_not_a_factory_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanNameAware` | `factory/BeanNameAware.java` | `factory/bean_name_aware.rs` | `bean_name_aware.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.BeanNotOfRequiredTypeException` | `factory/BeanNotOfRequiredTypeException.java` | `factory/bean_not_of_required_type_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanRegistrar` | `factory/BeanRegistrar.java` | `factory/bean_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.BeanRegistry` | `factory/BeanRegistry.java` | `factory/bean_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.CannotLoadBeanClassException` | `factory/CannotLoadBeanClassException.java` | `factory/cannot_load_bean_class_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.DisposableBean` | `factory/DisposableBean.java` | `factory/disposable_bean.rs` | `disposable_bean.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.FactoryBean` | `factory/FactoryBean.java` | `factory/factory_bean.rs` | `factory_bean.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.FactoryBeanNotInitializedException` | `factory/FactoryBeanNotInitializedException.java` | `factory/factory_bean_not_initialized_exception.rs` | `factory_bean_not_initialized_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.HierarchicalBeanFactory` | `factory/HierarchicalBeanFactory.java` | `factory/hierarchical_bean_factory.rs` | `hierarchical_bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.InitializingBean` | `factory/InitializingBean.java` | `factory/initializing_bean.rs` | `initializing_bean.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.InjectionPoint` | `factory/InjectionPoint.java` | `factory/injection_point.rs` | `injection_point.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.ListableBeanFactory` | `factory/ListableBeanFactory.java` | `factory/listable_bean_factory.rs` | `listable_bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.NamedBean` | `factory/NamedBean.java` | `factory/named_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.NoSuchBeanDefinitionException` | `factory/NoSuchBeanDefinitionException.java` | `factory/no_such_bean_definition_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.NoUniqueBeanDefinitionException` | `factory/NoUniqueBeanDefinitionException.java` | `factory/no_unique_bean_definition_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.ObjectFactory` | `factory/ObjectFactory.java` | `factory/object_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.ObjectProvider` | `factory/ObjectProvider.java` | `factory/object_provider.rs` | `object_provider.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.SmartFactoryBean` | `factory/SmartFactoryBean.java` | `factory/smart_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.SmartInitializingSingleton` | `factory/SmartInitializingSingleton.java` | `factory/smart_initializing_singleton.rs` | `smart_initializing_singleton.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.UnsatisfiedDependencyException` | `factory/UnsatisfiedDependencyException.java` | `factory/unsatisfied_dependency_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.annotation.AnnotatedBeanDefinition` | `factory/annotation/AnnotatedBeanDefinition.java` | `factory/annotation/annotated_bean_definition.rs` | `factory/annotation/annotated_bean_definition.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.AnnotatedGenericBeanDefinition` | `factory/annotation/AnnotatedGenericBeanDefinition.java` | `factory/annotation/annotated_generic_bean_definition.rs` | `annotated_generic_bean_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.annotation.AnnotationBeanWiringInfoResolver` | `factory/annotation/AnnotationBeanWiringInfoResolver.java` | `factory/annotation/annotation_bean_wiring_info_resolver.rs` | `factory/annotation/annotation_bean_wiring_info_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.Autowire` | `factory/annotation/Autowire.java` | `factory/annotation/autowire.rs` | `autowire.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.annotation.Autowired` | `factory/annotation/Autowired.java` | `factory/annotation/autowired.rs` | `factory/annotation/autowired.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.AutowiredAnnotationBeanPostProcessor` | `factory/annotation/AutowiredAnnotationBeanPostProcessor.java` | `factory/annotation/autowired_annotation_bean_post_processor.rs` | `factory/annotation/autowired_annotation_bean_post_processor.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.annotation.BeanFactoryAnnotationUtils` | `factory/annotation/BeanFactoryAnnotationUtils.java` | `factory/annotation/bean_factory_annotation_utils.rs` | `factory/annotation/bean_factory_annotation_utils.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.Configurable` | `factory/annotation/Configurable.java` | `factory/annotation/configurable.rs` | `factory/annotation/configurable.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.CustomAutowireConfigurer` | `factory/annotation/CustomAutowireConfigurer.java` | `factory/annotation/custom_autowire_configurer.rs` | `factory/annotation/custom_autowire_configurer.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.InitDestroyAnnotationBeanPostProcessor` | `factory/annotation/InitDestroyAnnotationBeanPostProcessor.java` | `factory/annotation/init_destroy_annotation_bean_post_processor.rs` | `factory/annotation/init_destroy_annotation_bean_post_processor.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.factory.annotation.InjectionMetadata` | `factory/annotation/InjectionMetadata.java` | `factory/annotation/injection_metadata.rs` | `injection_metadata.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.annotation.JakartaAnnotationsRuntimeHints` | `factory/annotation/JakartaAnnotationsRuntimeHints.java` | `factory/annotation/jakarta_annotations_runtime_hints.rs` | `factory/annotation/jakarta_annotations_runtime_hints.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.Lookup` | `factory/annotation/Lookup.java` | `factory/annotation/lookup.rs` | `factory/annotation/lookup.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.ParameterResolutionDelegate` | `factory/annotation/ParameterResolutionDelegate.java` | `factory/annotation/parameter_resolution_delegate.rs` | `factory/annotation/parameter_resolution_delegate.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.annotation.Qualifier` | `factory/annotation/Qualifier.java` | `factory/annotation/qualifier.rs` | `qualifier.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.annotation.QualifierAnnotationAutowireCandidateResolver` | `factory/annotation/QualifierAnnotationAutowireCandidateResolver.java` | `factory/annotation/qualifier_annotation_autowire_candidate_resolver.rs` | `factory/annotation/qualifier_annotation_autowire_candidate_resolver.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.factory.annotation.Value` | `factory/annotation/Value.java` | `factory/annotation/value.rs` | `factory/annotation/value.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.aot.AotBeanProcessingException` | `factory/aot/AotBeanProcessingException.java` | `factory/aot/aot_bean_processing_exception.rs` | `aot_bean_processing_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.aot.AotException` | `factory/aot/AotException.java` | `factory/aot/aot_exception.rs` | `aot_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.aot.AotProcessingException` | `factory/aot/AotProcessingException.java` | `factory/aot/aot_processing_exception.rs` | `aot_processing_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.aot.AotServices` | `factory/aot/AotServices.java` | `factory/aot/aot_services.rs` | `factory/aot/aot_services.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.AutowiredArguments` | `factory/aot/AutowiredArguments.java` | `factory/aot/autowired_arguments.rs` | `factory/aot/autowired_arguments.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.AutowiredArgumentsCodeGenerator` | `factory/aot/AutowiredArgumentsCodeGenerator.java` | `factory/aot/autowired_arguments_code_generator.rs` | `factory/aot/autowired_arguments_code_generator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.AutowiredElementResolver` | `factory/aot/AutowiredElementResolver.java` | `factory/aot/autowired_element_resolver.rs` | `factory/aot/autowired_element_resolver.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.AutowiredFieldValueResolver` | `factory/aot/AutowiredFieldValueResolver.java` | `factory/aot/autowired_field_value_resolver.rs` | `factory/aot/autowired_field_value_resolver.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.AutowiredMethodArgumentsResolver` | `factory/aot/AutowiredMethodArgumentsResolver.java` | `factory/aot/autowired_method_arguments_resolver.rs` | `factory/aot/autowired_method_arguments_resolver.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanDefinitionMethodGenerator` | `factory/aot/BeanDefinitionMethodGenerator.java` | `factory/aot/bean_definition_method_generator.rs` | `factory/aot/bean_definition_method_generator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanDefinitionMethodGeneratorFactory` | `factory/aot/BeanDefinitionMethodGeneratorFactory.java` | `factory/aot/bean_definition_method_generator_factory.rs` | `factory/aot/bean_definition_method_generator_factory.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanDefinitionPropertiesCodeGenerator` | `factory/aot/BeanDefinitionPropertiesCodeGenerator.java` | `factory/aot/bean_definition_properties_code_generator.rs` | `factory/aot/bean_definition_properties_code_generator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanDefinitionPropertyValueCodeGeneratorDelegates` | `factory/aot/BeanDefinitionPropertyValueCodeGeneratorDelegates.java` | `factory/aot/bean_definition_property_value_code_generator_delegates.rs` | `factory/aot/bean_definition_property_value_code_generator_delegates.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanFactoryInitializationAotContribution` | `factory/aot/BeanFactoryInitializationAotContribution.java` | `factory/aot/bean_factory_initialization_aot_contribution.rs` | `factory/aot/bean_factory_initialization_aot_contribution.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanFactoryInitializationAotProcessor` | `factory/aot/BeanFactoryInitializationAotProcessor.java` | `factory/aot/bean_factory_initialization_aot_processor.rs` | `factory/aot/bean_factory_initialization_aot_processor.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.factory.aot.BeanFactoryInitializationCode` | `factory/aot/BeanFactoryInitializationCode.java` | `factory/aot/bean_factory_initialization_code.rs` | `factory/aot/bean_factory_initialization_code.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanInstanceSupplier` | `factory/aot/BeanInstanceSupplier.java` | `factory/aot/bean_instance_supplier.rs` | `factory/aot/bean_instance_supplier.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationAotContribution` | `factory/aot/BeanRegistrationAotContribution.java` | `factory/aot/bean_registration_aot_contribution.rs` | `factory/aot/bean_registration_aot_contribution.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationAotProcessor` | `factory/aot/BeanRegistrationAotProcessor.java` | `factory/aot/bean_registration_aot_processor.rs` | `factory/aot/bean_registration_aot_processor.rs` | `UNVERIFIED` | 缺少测试引用 |
| `org.springframework.beans.factory.aot.BeanRegistrationCode` | `factory/aot/BeanRegistrationCode.java` | `factory/aot/bean_registration_code.rs` | `factory/aot/bean_registration_code.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationCodeFragments` | `factory/aot/BeanRegistrationCodeFragments.java` | `factory/aot/bean_registration_code_fragments.rs` | `factory/aot/bean_registration_code_fragments.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationCodeFragmentsDecorator` | `factory/aot/BeanRegistrationCodeFragmentsDecorator.java` | `factory/aot/bean_registration_code_fragments_decorator.rs` | `factory/aot/bean_registration_code_fragments_decorator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationCodeGenerator` | `factory/aot/BeanRegistrationCodeGenerator.java` | `factory/aot/bean_registration_code_generator.rs` | `factory/aot/bean_registration_code_generator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationExcludeFilter` | `factory/aot/BeanRegistrationExcludeFilter.java` | `factory/aot/bean_registration_exclude_filter.rs` | `factory/aot/bean_registration_exclude_filter.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationsAotContribution` | `factory/aot/BeanRegistrationsAotContribution.java` | `factory/aot/bean_registrations_aot_contribution.rs` | `factory/aot/bean_registrations_aot_contribution.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationsAotProcessor` | `factory/aot/BeanRegistrationsAotProcessor.java` | `factory/aot/bean_registrations_aot_processor.rs` | `factory/aot/bean_registrations_aot_processor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.BeanRegistrationsCode` | `factory/aot/BeanRegistrationsCode.java` | `factory/aot/bean_registrations_code.rs` | `factory/aot/bean_registrations_code.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.CodeWarnings` | `factory/aot/CodeWarnings.java` | `factory/aot/code_warnings.rs` | `factory/aot/code_warnings.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.DefaultBeanRegistrationCodeFragments` | `factory/aot/DefaultBeanRegistrationCodeFragments.java` | `factory/aot/default_bean_registration_code_fragments.rs` | `factory/aot/default_bean_registration_code_fragments.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.aot.InstanceSupplierCodeGenerator` | `factory/aot/InstanceSupplierCodeGenerator.java` | `factory/aot/instance_supplier_code_generator.rs` | `factory/aot/instance_supplier_code_generator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.config.AbstractFactoryBean` | `factory/config/AbstractFactoryBean.java` | `factory/config/abstract_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.AutowireCapableBeanFactory` | `factory/config/AutowireCapableBeanFactory.java` | `factory/config/autowire_capable_bean_factory.rs` | `autowire_capable_bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.AutowiredPropertyMarker` | `factory/config/AutowiredPropertyMarker.java` | `factory/config/autowired_property_marker.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.BeanDefinition` | `factory/config/BeanDefinition.java` | `factory/config/bean_definition.rs` | `bean_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.BeanDefinitionCustomizer` | `factory/config/BeanDefinitionCustomizer.java` | `factory/config/bean_definition_customizer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.BeanDefinitionHolder` | `factory/config/BeanDefinitionHolder.java` | `factory/config/bean_definition_holder.rs` | `bean_definition_holder.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.BeanDefinitionVisitor` | `factory/config/BeanDefinitionVisitor.java` | `factory/config/bean_definition_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.BeanExpressionContext` | `factory/config/BeanExpressionContext.java` | `factory/config/bean_expression_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.BeanExpressionResolver` | `factory/config/BeanExpressionResolver.java` | `factory/config/bean_expression_resolver.rs` | `bean_expression_resolver.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.BeanFactoryPostProcessor` | `factory/config/BeanFactoryPostProcessor.java` | `factory/config/bean_factory_post_processor.rs` | `bean_factory_post_processor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.BeanPostProcessor` | `factory/config/BeanPostProcessor.java` | `factory/config/bean_post_processor.rs` | `bean_post_processor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.BeanReference` | `factory/config/BeanReference.java` | `factory/config/bean_reference.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.ConfigurableBeanFactory` | `factory/config/ConfigurableBeanFactory.java` | `factory/config/configurable_bean_factory.rs` | `configurable_bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.ConfigurableListableBeanFactory` | `factory/config/ConfigurableListableBeanFactory.java` | `factory/config/configurable_listable_bean_factory.rs` | `configurable_listable_bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.ConstructorArgumentValues` | `factory/config/ConstructorArgumentValues.java` | `factory/config/constructor_argument_values.rs` | `constructor_argument_values.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.CustomEditorConfigurer` | `factory/config/CustomEditorConfigurer.java` | `factory/config/custom_editor_configurer.rs` | `custom_editor_configurer.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.CustomScopeConfigurer` | `factory/config/CustomScopeConfigurer.java` | `factory/config/custom_scope_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.DependencyDescriptor` | `factory/config/DependencyDescriptor.java` | `factory/config/dependency_descriptor.rs` | `dependency_descriptor.rs, factory/support/dependency_descriptor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.DeprecatedBeanWarner` | `factory/config/DeprecatedBeanWarner.java` | `factory/config/deprecated_bean_warner.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.DestructionAwareBeanPostProcessor` | `factory/config/DestructionAwareBeanPostProcessor.java` | `factory/config/destruction_aware_bean_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.EmbeddedValueResolver` | `factory/config/EmbeddedValueResolver.java` | `factory/config/embedded_value_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.FieldRetrievingFactoryBean` | `factory/config/FieldRetrievingFactoryBean.java` | `factory/config/field_retrieving_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.InstantiationAwareBeanPostProcessor` | `factory/config/InstantiationAwareBeanPostProcessor.java` | `factory/config/instantiation_aware_bean_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.ListFactoryBean` | `factory/config/ListFactoryBean.java` | `factory/config/list_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.MapFactoryBean` | `factory/config/MapFactoryBean.java` | `factory/config/map_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.MethodInvokingBean` | `factory/config/MethodInvokingBean.java` | `factory/config/method_invoking_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.MethodInvokingFactoryBean` | `factory/config/MethodInvokingFactoryBean.java` | `factory/config/method_invoking_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.NamedBeanHolder` | `factory/config/NamedBeanHolder.java` | `factory/config/named_bean_holder.rs` | `named_bean_holder.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.ObjectFactoryCreatingFactoryBean` | `factory/config/ObjectFactoryCreatingFactoryBean.java` | `factory/config/object_factory_creating_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PlaceholderConfigurerSupport` | `factory/config/PlaceholderConfigurerSupport.java` | `factory/config/placeholder_configurer_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PreferencesPlaceholderConfigurer` | `factory/config/PreferencesPlaceholderConfigurer.java` | `factory/config/preferences_placeholder_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PropertiesFactoryBean` | `factory/config/PropertiesFactoryBean.java` | `factory/config/properties_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PropertyOverrideConfigurer` | `factory/config/PropertyOverrideConfigurer.java` | `factory/config/property_override_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PropertyPathFactoryBean` | `factory/config/PropertyPathFactoryBean.java` | `factory/config/property_path_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PropertyPlaceholderConfigurer` | `factory/config/PropertyPlaceholderConfigurer.java` | `factory/config/property_placeholder_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.PropertyResourceConfigurer` | `factory/config/PropertyResourceConfigurer.java` | `factory/config/property_resource_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.ProviderCreatingFactoryBean` | `factory/config/ProviderCreatingFactoryBean.java` | `factory/config/provider_creating_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.RuntimeBeanNameReference` | `factory/config/RuntimeBeanNameReference.java` | `factory/config/runtime_bean_name_reference.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.RuntimeBeanReference` | `factory/config/RuntimeBeanReference.java` | `factory/config/runtime_bean_reference.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.Scope` | `factory/config/Scope.java` | `factory/config/scope.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.ServiceLocatorFactoryBean` | `factory/config/ServiceLocatorFactoryBean.java` | `factory/config/service_locator_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.SetFactoryBean` | `factory/config/SetFactoryBean.java` | `factory/config/set_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.SingletonBeanRegistry` | `factory/config/SingletonBeanRegistry.java` | `factory/config/singleton_bean_registry.rs` | `singleton_bean_registry.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.config.SmartInstantiationAwareBeanPostProcessor` | `factory/config/SmartInstantiationAwareBeanPostProcessor.java` | `factory/config/smart_instantiation_aware_bean_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.TypedStringValue` | `factory/config/TypedStringValue.java` | `factory/config/typed_string_value.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.YamlMapFactoryBean` | `factory/config/YamlMapFactoryBean.java` | `factory/config/yaml_map_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.YamlProcessor` | `factory/config/YamlProcessor.java` | `factory/config/yaml_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.config.YamlPropertiesFactoryBean` | `factory/config/YamlPropertiesFactoryBean.java` | `factory/config/yaml_properties_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.groovy.GroovyBeanDefinitionReader` | `factory/groovy/GroovyBeanDefinitionReader.java` | `factory/groovy/groovy_bean_definition_reader.rs` | `factory/groovy/groovy_bean_definition_reader.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.groovy.GroovyBeanDefinitionWrapper` | `factory/groovy/GroovyBeanDefinitionWrapper.java` | `factory/groovy/groovy_bean_definition_wrapper.rs` | `factory/groovy/groovy_bean_definition_wrapper.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.groovy.GroovyDynamicElementReader` | `factory/groovy/GroovyDynamicElementReader.java` | `factory/groovy/groovy_dynamic_element_reader.rs` | `factory/groovy/groovy_dynamic_element_reader.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.parsing.AbstractComponentDefinition` | `factory/parsing/AbstractComponentDefinition.java` | `factory/parsing/abstract_component_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.AliasDefinition` | `factory/parsing/AliasDefinition.java` | `factory/parsing/alias_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.BeanComponentDefinition` | `factory/parsing/BeanComponentDefinition.java` | `factory/parsing/bean_component_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.BeanDefinitionParsingException` | `factory/parsing/BeanDefinitionParsingException.java` | `factory/parsing/bean_definition_parsing_exception.rs` | `bean_definition_parsing_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.parsing.BeanEntry` | `factory/parsing/BeanEntry.java` | `factory/parsing/bean_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ComponentDefinition` | `factory/parsing/ComponentDefinition.java` | `factory/parsing/component_definition.rs` | `component_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.parsing.CompositeComponentDefinition` | `factory/parsing/CompositeComponentDefinition.java` | `factory/parsing/composite_component_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ConstructorArgumentEntry` | `factory/parsing/ConstructorArgumentEntry.java` | `factory/parsing/constructor_argument_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.DefaultsDefinition` | `factory/parsing/DefaultsDefinition.java` | `factory/parsing/defaults_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.EmptyReaderEventListener` | `factory/parsing/EmptyReaderEventListener.java` | `factory/parsing/empty_reader_event_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.FailFastProblemReporter` | `factory/parsing/FailFastProblemReporter.java` | `factory/parsing/fail_fast_problem_reporter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ImportDefinition` | `factory/parsing/ImportDefinition.java` | `factory/parsing/import_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.Location` | `factory/parsing/Location.java` | `factory/parsing/location.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.NullSourceExtractor` | `factory/parsing/NullSourceExtractor.java` | `factory/parsing/null_source_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ParseState` | `factory/parsing/ParseState.java` | `factory/parsing/parse_state.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.PassThroughSourceExtractor` | `factory/parsing/PassThroughSourceExtractor.java` | `factory/parsing/pass_through_source_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.Problem` | `factory/parsing/Problem.java` | `factory/parsing/problem.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ProblemReporter` | `factory/parsing/ProblemReporter.java` | `factory/parsing/problem_reporter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.PropertyEntry` | `factory/parsing/PropertyEntry.java` | `factory/parsing/property_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.QualifierEntry` | `factory/parsing/QualifierEntry.java` | `factory/parsing/qualifier_entry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ReaderContext` | `factory/parsing/ReaderContext.java` | `factory/parsing/reader_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.ReaderEventListener` | `factory/parsing/ReaderEventListener.java` | `factory/parsing/reader_event_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.parsing.SourceExtractor` | `factory/parsing/SourceExtractor.java` | `factory/parsing/source_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.serviceloader.AbstractServiceLoaderBasedFactoryBean` | `factory/serviceloader/AbstractServiceLoaderBasedFactoryBean.java` | `factory/serviceloader/abstract_service_loader_based_factory_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.serviceloader.ServiceFactoryBean` | `factory/serviceloader/ServiceFactoryBean.java` | `factory/serviceloader/service_factory_bean.rs` | `factory/serviceloader/service_factory_bean.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.serviceloader.ServiceListFactoryBean` | `factory/serviceloader/ServiceListFactoryBean.java` | `factory/serviceloader/service_list_factory_bean.rs` | `factory/serviceloader/service_list_factory_bean.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.serviceloader.ServiceLoaderFactoryBean` | `factory/serviceloader/ServiceLoaderFactoryBean.java` | `factory/serviceloader/service_loader_factory_bean.rs` | `factory/serviceloader/service_loader_factory_bean.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.support.AbstractAutowireCapableBeanFactory` | `factory/support/AbstractAutowireCapableBeanFactory.java` | `factory/support/abstract_autowire_capable_bean_factory.rs` | `factory/support/abstract_autowire_capable_bean_factory.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.support.AbstractBeanDefinition` | `factory/support/AbstractBeanDefinition.java` | `factory/support/abstract_bean_definition.rs` | `abstract_bean_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.AbstractBeanDefinitionReader` | `factory/support/AbstractBeanDefinitionReader.java` | `factory/support/abstract_bean_definition_reader.rs` | `factory/support/abstract_bean_definition_reader.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.support.AbstractBeanFactory` | `factory/support/AbstractBeanFactory.java` | `factory/support/abstract_bean_factory.rs` | `factory/support/abstract_bean_factory.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.support.AutowireCandidateQualifier` | `factory/support/AutowireCandidateQualifier.java` | `factory/support/autowire_candidate_qualifier.rs` | `factory/support/autowire_candidate_qualifier.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.AutowireCandidateResolver` | `factory/support/AutowireCandidateResolver.java` | `factory/support/autowire_candidate_resolver.rs` | `factory/support/autowire_candidate_resolver.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.support.AutowireUtils` | `factory/support/AutowireUtils.java` | `factory/support/autowire_utils.rs` | `factory/support/autowire_utils.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.support.BeanDefinitionBuilder` | `factory/support/BeanDefinitionBuilder.java` | `factory/support/bean_definition_builder.rs` | `bean_definition_builder.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.BeanDefinitionDefaults` | `factory/support/BeanDefinitionDefaults.java` | `factory/support/bean_definition_defaults.rs` | `factory/support/bean_definition_defaults.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.BeanDefinitionOverrideException` | `factory/support/BeanDefinitionOverrideException.java` | `factory/support/bean_definition_override_exception.rs` | `bean_definition_override_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.BeanDefinitionReader` | `factory/support/BeanDefinitionReader.java` | `factory/support/bean_definition_reader.rs` | `factory/support/bean_definition_reader.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.support.BeanDefinitionReaderUtils` | `factory/support/BeanDefinitionReaderUtils.java` | `factory/support/bean_definition_reader_utils.rs` | `factory/support/bean_definition_reader_utils.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.BeanDefinitionRegistry` | `factory/support/BeanDefinitionRegistry.java` | `factory/support/bean_definition_registry.rs` | `bean_definition_registry.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor` | `factory/support/BeanDefinitionRegistryPostProcessor.java` | `factory/support/bean_definition_registry_post_processor.rs` | `bean_definition_registry_post_processor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.BeanDefinitionResource` | `factory/support/BeanDefinitionResource.java` | `factory/support/bean_definition_resource.rs` | `factory/support/bean_definition_resource.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.BeanDefinitionValidationException` | `factory/support/BeanDefinitionValidationException.java` | `factory/support/bean_definition_validation_exception.rs` | `bean_definition_validation_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.BeanDefinitionValueResolver` | `factory/support/BeanDefinitionValueResolver.java` | `factory/support/bean_definition_value_resolver.rs` | `factory/support/bean_definition_value_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.BeanNameGenerator` | `factory/support/BeanNameGenerator.java` | `factory/support/bean_name_generator.rs` | `factory/support/bean_name_generator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.BeanRegistryAdapter` | `factory/support/BeanRegistryAdapter.java` | `factory/support/bean_registry_adapter.rs` | `factory/support/bean_registry_adapter.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.CglibSubclassingInstantiationStrategy` | `factory/support/CglibSubclassingInstantiationStrategy.java` | `factory/support/cglib_subclassing_instantiation_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.ChildBeanDefinition` | `factory/support/ChildBeanDefinition.java` | `factory/support/child_bean_definition.rs` | `child_bean_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.ConstructorResolver` | `factory/support/ConstructorResolver.java` | `factory/support/constructor_resolver.rs` | `constructor_resolver.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.DefaultBeanNameGenerator` | `factory/support/DefaultBeanNameGenerator.java` | `factory/support/default_bean_name_generator.rs` | `factory/support/default_bean_name_generator.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.DefaultListableBeanFactory` | `factory/support/DefaultListableBeanFactory.java` | `factory/support/default_listable_bean_factory.rs` | `factory/support/default_listable_bean_factory.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/container_registry_type_converter.rs` |
| `org.springframework.beans.factory.support.DefaultSingletonBeanRegistry` | `factory/support/DefaultSingletonBeanRegistry.java` | `factory/support/default_singleton_bean_registry.rs` | `factory/support/default_singleton_bean_registry.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `tests/factory_real_impl_tests.rs` |
| `org.springframework.beans.factory.support.DisposableBeanAdapter` | `factory/support/DisposableBeanAdapter.java` | `factory/support/disposable_bean_adapter.rs` | `factory/support/disposable_bean_adapter.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.support.FactoryBeanRegistrySupport` | `factory/support/FactoryBeanRegistrySupport.java` | `factory/support/factory_bean_registry_support.rs` | `factory_bean_registry_support.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.GenericBeanDefinition` | `factory/support/GenericBeanDefinition.java` | `factory/support/generic_bean_definition.rs` | `generic_bean_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.GenericTypeAwareAutowireCandidateResolver` | `factory/support/GenericTypeAwareAutowireCandidateResolver.java` | `factory/support/generic_type_aware_autowire_candidate_resolver.rs` | `factory/support/generic_type_aware_autowire_candidate_resolver.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.ImplicitlyAppearedSingletonException` | `factory/support/ImplicitlyAppearedSingletonException.java` | `factory/support/implicitly_appeared_singleton_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.InstanceSupplier` | `factory/support/InstanceSupplier.java` | `factory/support/instance_supplier.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.InstantiationStrategy` | `factory/support/InstantiationStrategy.java` | `factory/support/instantiation_strategy.rs` | `instantiation_strategy.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.LookupOverride` | `factory/support/LookupOverride.java` | `factory/support/lookup_override.rs` | `lookup_override.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.ManagedArray` | `factory/support/ManagedArray.java` | `factory/support/managed_array.rs` | `factory/support/managed_array.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.ManagedList` | `factory/support/ManagedList.java` | `factory/support/managed_list.rs` | `factory/support/managed_list.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.ManagedMap` | `factory/support/ManagedMap.java` | `factory/support/managed_map.rs` | `factory/support/managed_map.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.ManagedProperties` | `factory/support/ManagedProperties.java` | `factory/support/managed_properties.rs` | `factory/support/managed_properties.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.ManagedSet` | `factory/support/ManagedSet.java` | `factory/support/managed_set.rs` | `factory/support/managed_set.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.beans.factory.support.MergedBeanDefinitionPostProcessor` | `factory/support/MergedBeanDefinitionPostProcessor.java` | `factory/support/merged_bean_definition_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.MethodDescriptor` | `factory/support/MethodDescriptor.java` | `factory/support/method_descriptor.rs` | `method_descriptor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.MethodOverride` | `factory/support/MethodOverride.java` | `factory/support/method_override.rs` | `method_override.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.MethodOverrides` | `factory/support/MethodOverrides.java` | `factory/support/method_overrides.rs` | `method_overrides.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.MethodReplacer` | `factory/support/MethodReplacer.java` | `factory/support/method_replacer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.NullBean` | `factory/support/NullBean.java` | `factory/support/null_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.PropertiesBeanDefinitionReader` | `factory/support/PropertiesBeanDefinitionReader.java` | `factory/support/properties_bean_definition_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.RegisteredBean` | `factory/support/RegisteredBean.java` | `factory/support/registered_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.ReplaceOverride` | `factory/support/ReplaceOverride.java` | `factory/support/replace_override.rs` | `replace_override.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.RootBeanDefinition` | `factory/support/RootBeanDefinition.java` | `factory/support/root_bean_definition.rs` | `root_bean_definition.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.ScopeNotActiveException` | `factory/support/ScopeNotActiveException.java` | `factory/support/scope_not_active_exception.rs` | `scope_not_active_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.SimpleAutowireCandidateResolver` | `factory/support/SimpleAutowireCandidateResolver.java` | `factory/support/simple_autowire_candidate_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.support.SimpleBeanDefinitionRegistry` | `factory/support/SimpleBeanDefinitionRegistry.java` | `factory/support/simple_bean_definition_registry.rs` | `simple_bean_definition_registry.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.SimpleInstantiationStrategy` | `factory/support/SimpleInstantiationStrategy.java` | `factory/support/simple_instantiation_strategy.rs` | `simple_instantiation_strategy.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.support.StaticListableBeanFactory` | `factory/support/StaticListableBeanFactory.java` | `factory/support/static_listable_bean_factory.rs` | `static_listable_bean_factory.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.wiring.BeanConfigurerSupport` | `factory/wiring/BeanConfigurerSupport.java` | `factory/wiring/bean_configurer_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.wiring.BeanWiringInfo` | `factory/wiring/BeanWiringInfo.java` | `factory/wiring/bean_wiring_info.rs` | `bean_wiring_info.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.wiring.BeanWiringInfoResolver` | `factory/wiring/BeanWiringInfoResolver.java` | `factory/wiring/bean_wiring_info_resolver.rs` | `factory/wiring/bean_wiring_info_resolver.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.wiring.ClassNameBeanWiringInfoResolver` | `factory/wiring/ClassNameBeanWiringInfoResolver.java` | `factory/wiring/class_name_bean_wiring_info_resolver.rs` | `factory/wiring/class_name_bean_wiring_info_resolver.rs` | `STUB` | 存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记 |
| `org.springframework.beans.factory.xml.AbstractBeanDefinitionParser` | `factory/xml/AbstractBeanDefinitionParser.java` | `factory/xml/abstract_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.AbstractSimpleBeanDefinitionParser` | `factory/xml/AbstractSimpleBeanDefinitionParser.java` | `factory/xml/abstract_simple_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.AbstractSingleBeanDefinitionParser` | `factory/xml/AbstractSingleBeanDefinitionParser.java` | `factory/xml/abstract_single_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.BeanDefinitionDecorator` | `factory/xml/BeanDefinitionDecorator.java` | `factory/xml/bean_definition_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.BeanDefinitionDocumentReader` | `factory/xml/BeanDefinitionDocumentReader.java` | `factory/xml/bean_definition_document_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.BeanDefinitionParser` | `factory/xml/BeanDefinitionParser.java` | `factory/xml/bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.BeanDefinitionParserDelegate` | `factory/xml/BeanDefinitionParserDelegate.java` | `factory/xml/bean_definition_parser_delegate.rs` | `xml/bean_definition_parser_delegate.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.BeansDtdResolver` | `factory/xml/BeansDtdResolver.java` | `factory/xml/beans_dtd_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.DefaultBeanDefinitionDocumentReader` | `factory/xml/DefaultBeanDefinitionDocumentReader.java` | `factory/xml/default_bean_definition_document_reader.rs` | `xml/default_bean_definition_document_reader.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.DefaultDocumentLoader` | `factory/xml/DefaultDocumentLoader.java` | `factory/xml/default_document_loader.rs` | `default_document_loader.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.DefaultNamespaceHandlerResolver` | `factory/xml/DefaultNamespaceHandlerResolver.java` | `factory/xml/default_namespace_handler_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.DelegatingEntityResolver` | `factory/xml/DelegatingEntityResolver.java` | `factory/xml/delegating_entity_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.DocumentDefaultsDefinition` | `factory/xml/DocumentDefaultsDefinition.java` | `factory/xml/document_defaults_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.DocumentLoader` | `factory/xml/DocumentLoader.java` | `factory/xml/document_loader.rs` | `document_loader.rs, xml/document_loader.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.NamespaceHandler` | `factory/xml/NamespaceHandler.java` | `factory/xml/namespace_handler.rs` | `xml/namespace_handler.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.NamespaceHandlerResolver` | `factory/xml/NamespaceHandlerResolver.java` | `factory/xml/namespace_handler_resolver.rs` | `xml/namespace_handler_resolver.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.NamespaceHandlerSupport` | `factory/xml/NamespaceHandlerSupport.java` | `factory/xml/namespace_handler_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.ParserContext` | `factory/xml/ParserContext.java` | `factory/xml/parser_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.PluggableSchemaResolver` | `factory/xml/PluggableSchemaResolver.java` | `factory/xml/pluggable_schema_resolver.rs` | `pluggable_schema_resolver.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.ResourceEntityResolver` | `factory/xml/ResourceEntityResolver.java` | `factory/xml/resource_entity_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.SimpleConstructorNamespaceHandler` | `factory/xml/SimpleConstructorNamespaceHandler.java` | `factory/xml/simple_constructor_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.SimplePropertyNamespaceHandler` | `factory/xml/SimplePropertyNamespaceHandler.java` | `factory/xml/simple_property_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.UtilNamespaceHandler` | `factory/xml/UtilNamespaceHandler.java` | `factory/xml/util_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.factory.xml.XmlBeanDefinitionReader` | `factory/xml/XmlBeanDefinitionReader.java` | `factory/xml/xml_bean_definition_reader.rs` | `xml/xml_bean_definition_reader.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.XmlBeanDefinitionStoreException` | `factory/xml/XmlBeanDefinitionStoreException.java` | `factory/xml/xml_bean_definition_store_exception.rs` | `xml_bean_definition_store_exception.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.factory.xml.XmlReaderContext` | `factory/xml/XmlReaderContext.java` | `factory/xml/xml_reader_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.ByteArrayPropertyEditor` | `propertyeditors/ByteArrayPropertyEditor.java` | `propertyeditors/byte_array_property_editor.rs` | `byte_array_property_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.CharArrayPropertyEditor` | `propertyeditors/CharArrayPropertyEditor.java` | `propertyeditors/char_array_property_editor.rs` | `char_array_property_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.CharacterEditor` | `propertyeditors/CharacterEditor.java` | `propertyeditors/character_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.CharsetEditor` | `propertyeditors/CharsetEditor.java` | `propertyeditors/charset_editor.rs` | `charset_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.ClassArrayEditor` | `propertyeditors/ClassArrayEditor.java` | `propertyeditors/class_array_editor.rs` | `class_array_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.ClassEditor` | `propertyeditors/ClassEditor.java` | `propertyeditors/class_editor.rs` | `class_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.CurrencyEditor` | `propertyeditors/CurrencyEditor.java` | `propertyeditors/currency_editor.rs` | `currency_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.CustomBooleanEditor` | `propertyeditors/CustomBooleanEditor.java` | `propertyeditors/custom_boolean_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.CustomCollectionEditor` | `propertyeditors/CustomCollectionEditor.java` | `propertyeditors/custom_collection_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.CustomDateEditor` | `propertyeditors/CustomDateEditor.java` | `propertyeditors/custom_date_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.CustomMapEditor` | `propertyeditors/CustomMapEditor.java` | `propertyeditors/custom_map_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.CustomNumberEditor` | `propertyeditors/CustomNumberEditor.java` | `propertyeditors/custom_number_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.FileEditor` | `propertyeditors/FileEditor.java` | `propertyeditors/file_editor.rs` | `file_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.InputSourceEditor` | `propertyeditors/InputSourceEditor.java` | `propertyeditors/input_source_editor.rs` | `input_source_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.InputStreamEditor` | `propertyeditors/InputStreamEditor.java` | `propertyeditors/input_stream_editor.rs` | `input_stream_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.LocaleEditor` | `propertyeditors/LocaleEditor.java` | `propertyeditors/locale_editor.rs` | `locale_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.PathEditor` | `propertyeditors/PathEditor.java` | `propertyeditors/path_editor.rs` | `path_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.PatternEditor` | `propertyeditors/PatternEditor.java` | `propertyeditors/pattern_editor.rs` | `pattern_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.PropertiesEditor` | `propertyeditors/PropertiesEditor.java` | `propertyeditors/properties_editor.rs` | `properties_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.ReaderEditor` | `propertyeditors/ReaderEditor.java` | `propertyeditors/reader_editor.rs` | `reader_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.ResourceBundleEditor` | `propertyeditors/ResourceBundleEditor.java` | `propertyeditors/resource_bundle_editor.rs` | `resource_bundle_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.StringArrayPropertyEditor` | `propertyeditors/StringArrayPropertyEditor.java` | `propertyeditors/string_array_property_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.StringTrimmerEditor` | `propertyeditors/StringTrimmerEditor.java` | `propertyeditors/string_trimmer_editor.rs` | `string_trimmer_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.TimeZoneEditor` | `propertyeditors/TimeZoneEditor.java` | `propertyeditors/time_zone_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.URIEditor` | `propertyeditors/URIEditor.java` | `propertyeditors/uri_editor.rs` | `uri_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.URLEditor` | `propertyeditors/URLEditor.java` | `propertyeditors/url_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.propertyeditors.UUIDEditor` | `propertyeditors/UUIDEditor.java` | `propertyeditors/uuid_editor.rs` | `uuid_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.propertyeditors.ZoneIdEditor` | `propertyeditors/ZoneIdEditor.java` | `propertyeditors/zone_id_editor.rs` | `zone_id_editor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.beans.support.ArgumentConvertingMethodInvoker` | `support/ArgumentConvertingMethodInvoker.java` | `support/argument_converting_method_invoker.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.support.MutableSortDefinition` | `support/MutableSortDefinition.java` | `support/mutable_sort_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.support.PagedListHolder` | `support/PagedListHolder.java` | `support/paged_list_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.support.PropertyComparator` | `support/PropertyComparator.java` | `support/property_comparator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.support.ResourceEditorRegistrar` | `support/ResourceEditorRegistrar.java` | `support/resource_editor_registrar.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.beans.support.SortDefinition` | `support/SortDefinition.java` | `support/sort_definition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

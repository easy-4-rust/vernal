<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-core 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 329 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 329 |
| 已处理（严格三类） | 25 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 25 |
| `MISPLACED` | 0 |
| `MISSING` | 304 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `properties_file.rs`：`FrameworkProperties`、`PropertiesFileError`
- 单文件多个公开对象位于 `failure.rs`：`BoxError`、`SharedError`
- 单文件多个公开对象位于 `util/mime_sniff.rs`：`SniffedMimeType`、`MismatchError`
- 单文件多个公开对象位于 `util/property_placeholder_helper.rs`：`PlaceholderError`、`PlaceholderResolver`、`PropertyPlaceholderHelper`、`StringValueResolver`
- 单文件多个公开对象位于 `util/multi_value_map_adapters.rs`：`MultiValueMapAdapter`、`MultiToSingleValueMapAdapter`、`SingleToMultiValueMapAdapter`、`MultiValueMapCollector`
- 单文件多个公开对象位于 `util/multi_value_map.rs`：`MultiValueMapTrait`、`MultiValueMap`、`UnmodifiableMultiValueMap`
- 单文件多个公开对象位于 `util/mime_type.rs`：`MimeType`、`InvalidMimeType`
- 单文件多个公开对象位于 `time/stop_watch.rs`：`StopWatchError`、`StopWatch`、`TaskInfo`
- 单文件多个公开对象位于 `diagnostics/span.rs`：`SpanStatus`、`Span`、`SpanReport`
- 单文件多个公开对象位于 `id/snowflake_id.rs`：`SnowflakeId`、`SnowflakeError`
- 单文件多个公开对象位于 `id/ulid_id.rs`：`UlidId`、`Ulid`
- 单文件多个公开对象位于 `convert/converter/converter_registry.rs`：`ErasedConverter`、`ConverterRegistry`、`TypeIdConverterRegistry`
- 单文件多个公开对象位于 `convert/converter/generic_converter.rs`：`ErasedGenericFn`、`GenericConverter`、`ClosureGenericConverter`
- 单文件多个公开对象位于 `util/unit/data_size.rs`：`DataSize`、`DataSizeParseError`
- 单文件多个公开对象位于 `util/unit/data_unit.rs`：`DataUnit`、`UnknownDataUnitSuffix`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.core.AliasRegistry` | `AliasRegistry.java` | `alias_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.AttributeAccessor` | `AttributeAccessor.java` | `attribute_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.AttributeAccessorSupport` | `AttributeAccessorSupport.java` | `attribute_accessor_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.BridgeMethodResolver` | `BridgeMethodResolver.java` | `bridge_method_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.CollectionFactory` | `CollectionFactory.java` | `collection_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ConfigurableObjectInputStream` | `ConfigurableObjectInputStream.java` | `configurable_object_input_stream.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.Constants` | `Constants.java` | `constants.rs` | `constants.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/constants.rs#[cfg(test)]` |
| `org.springframework.core.Conventions` | `Conventions.java` | `conventions.rs` | `conventions.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/conventions.rs#[cfg(test)]` |
| `org.springframework.core.CoroutinesUtils` | `CoroutinesUtils.java` | `coroutines_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.DecoratingClassLoader` | `DecoratingClassLoader.java` | `decorating_class_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.DecoratingProxy` | `DecoratingProxy.java` | `decorating_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.DefaultParameterNameDiscoverer` | `DefaultParameterNameDiscoverer.java` | `default_parameter_name_discoverer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ExceptionDepthComparator` | `ExceptionDepthComparator.java` | `exception_depth_comparator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.GenericTypeResolver` | `GenericTypeResolver.java` | `generic_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.InfrastructureProxy` | `InfrastructureProxy.java` | `infrastructure_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.KotlinDetector` | `KotlinDetector.java` | `kotlin_detector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.KotlinReflectionParameterNameDiscoverer` | `KotlinReflectionParameterNameDiscoverer.java` | `kotlin_reflection_parameter_name_discoverer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.MethodClassKey` | `MethodClassKey.java` | `method_class_key.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.MethodIntrospector` | `MethodIntrospector.java` | `method_introspector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.MethodParameter` | `MethodParameter.java` | `method_parameter.rs` | `method_parameter.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/method_parameter.rs#[cfg(test)]` |
| `org.springframework.core.NamedInheritableThreadLocal` | `NamedInheritableThreadLocal.java` | `named_inheritable_thread_local.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.NamedThreadLocal` | `NamedThreadLocal.java` | `named_thread_local.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.NativeDetector` | `NativeDetector.java` | `native_detector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.NestedCheckedException` | `NestedCheckedException.java` | `nested_checked_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.NestedExceptionUtils` | `NestedExceptionUtils.java` | `nested_exception_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.NestedRuntimeException` | `NestedRuntimeException.java` | `nested_runtime_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.Nullness` | `Nullness.java` | `nullness.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.OrderComparator` | `OrderComparator.java` | `order_comparator.rs` | `order_comparator.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/order_comparator.rs#[cfg(test)]` |
| `org.springframework.core.Ordered` | `Ordered.java` | `ordered.rs` | `ordered.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/order_comparator.rs#[cfg(test)]` |
| `org.springframework.core.OverridingClassLoader` | `OverridingClassLoader.java` | `overriding_class_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ParameterNameDiscoverer` | `ParameterNameDiscoverer.java` | `parameter_name_discoverer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ParameterizedTypeReference` | `ParameterizedTypeReference.java` | `parameterized_type_reference.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.PrioritizedParameterNameDiscoverer` | `PrioritizedParameterNameDiscoverer.java` | `prioritized_parameter_name_discoverer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.PriorityOrdered` | `PriorityOrdered.java` | `priority_ordered.rs` | `priority_ordered.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/order_comparator.rs#[cfg(test)]` |
| `org.springframework.core.PropagationContextElement` | `PropagationContextElement.java` | `propagation_context_element.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ReactiveAdapter` | `ReactiveAdapter.java` | `reactive_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ReactiveAdapterRegistry` | `ReactiveAdapterRegistry.java` | `reactive_adapter_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ReactiveTypeDescriptor` | `ReactiveTypeDescriptor.java` | `reactive_type_descriptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ResolvableType` | `ResolvableType.java` | `resolvable_type.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.ResolvableTypeProvider` | `ResolvableTypeProvider.java` | `resolvable_type_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.SerializableTypeWrapper` | `SerializableTypeWrapper.java` | `serializable_type_wrapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.SimpleAliasRegistry` | `SimpleAliasRegistry.java` | `simple_alias_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.SmartClassLoader` | `SmartClassLoader.java` | `smart_class_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.SortedProperties` | `SortedProperties.java` | `sorted_properties.rs` | `sorted_properties.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/sorted_properties.rs#[cfg(test)]` |
| `org.springframework.core.SpringProperties` | `SpringProperties.java` | `spring_properties.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.SpringVersion` | `SpringVersion.java` | `spring_version.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.StandardReflectionParameterNameDiscoverer` | `StandardReflectionParameterNameDiscoverer.java` | `standard_reflection_parameter_name_discoverer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AbstractMergedAnnotation` | `annotation/AbstractMergedAnnotation.java` | `annotation/abstract_merged_annotation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AliasFor` | `annotation/AliasFor.java` | `annotation/alias_for.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotatedElementAdapter` | `annotation/AnnotatedElementAdapter.java` | `annotation/annotated_element_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotatedElementUtils` | `annotation/AnnotatedElementUtils.java` | `annotation/annotated_element_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotatedMethod` | `annotation/AnnotatedMethod.java` | `annotation/annotated_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationAttributes` | `annotation/AnnotationAttributes.java` | `annotation/annotation_attributes.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationAwareOrderComparator` | `annotation/AnnotationAwareOrderComparator.java` | `annotation/annotation_aware_order_comparator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationConfigurationException` | `annotation/AnnotationConfigurationException.java` | `annotation/annotation_configuration_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationFilter` | `annotation/AnnotationFilter.java` | `annotation/annotation_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationTypeMapping` | `annotation/AnnotationTypeMapping.java` | `annotation/annotation_type_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationTypeMappings` | `annotation/AnnotationTypeMappings.java` | `annotation/annotation_type_mappings.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationUtils` | `annotation/AnnotationUtils.java` | `annotation/annotation_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationsProcessor` | `annotation/AnnotationsProcessor.java` | `annotation/annotations_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AnnotationsScanner` | `annotation/AnnotationsScanner.java` | `annotation/annotations_scanner.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.AttributeMethods` | `annotation/AttributeMethods.java` | `annotation/attribute_methods.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.IntrospectionFailureLogger` | `annotation/IntrospectionFailureLogger.java` | `annotation/introspection_failure_logger.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotation` | `annotation/MergedAnnotation.java` | `annotation/merged_annotation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotationCollectors` | `annotation/MergedAnnotationCollectors.java` | `annotation/merged_annotation_collectors.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotationPredicates` | `annotation/MergedAnnotationPredicates.java` | `annotation/merged_annotation_predicates.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotationSelector` | `annotation/MergedAnnotationSelector.java` | `annotation/merged_annotation_selector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotationSelectors` | `annotation/MergedAnnotationSelectors.java` | `annotation/merged_annotation_selectors.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotations` | `annotation/MergedAnnotations.java` | `annotation/merged_annotations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MergedAnnotationsCollection` | `annotation/MergedAnnotationsCollection.java` | `annotation/merged_annotations_collection.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.MissingMergedAnnotation` | `annotation/MissingMergedAnnotation.java` | `annotation/missing_merged_annotation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.Order` | `annotation/Order.java` | `annotation/order.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.OrderUtils` | `annotation/OrderUtils.java` | `annotation/order_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.PackagesAnnotationFilter` | `annotation/PackagesAnnotationFilter.java` | `annotation/packages_annotation_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.RepeatableContainers` | `annotation/RepeatableContainers.java` | `annotation/repeatable_containers.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.SynthesizedMergedAnnotationInvocationHandler` | `annotation/SynthesizedMergedAnnotationInvocationHandler.java` | `annotation/synthesized_merged_annotation_invocation_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.SynthesizingMethodParameter` | `annotation/SynthesizingMethodParameter.java` | `annotation/synthesizing_method_parameter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.TypeMappedAnnotation` | `annotation/TypeMappedAnnotation.java` | `annotation/type_mapped_annotation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.TypeMappedAnnotations` | `annotation/TypeMappedAnnotations.java` | `annotation/type_mapped_annotations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.annotation.ValueExtractor` | `annotation/ValueExtractor.java` | `annotation/value_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.AbstractCharSequenceDecoder` | `codec/AbstractCharSequenceDecoder.java` | `codec/abstract_char_sequence_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.AbstractDataBufferDecoder` | `codec/AbstractDataBufferDecoder.java` | `codec/abstract_data_buffer_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.AbstractDecoder` | `codec/AbstractDecoder.java` | `codec/abstract_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.AbstractEncoder` | `codec/AbstractEncoder.java` | `codec/abstract_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.AbstractSingleValueEncoder` | `codec/AbstractSingleValueEncoder.java` | `codec/abstract_single_value_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.ByteArrayDecoder` | `codec/ByteArrayDecoder.java` | `codec/byte_array_decoder.rs` | `codec/byte_array_decoder.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/codec/byte_array_decoder.rs#[cfg(test)]` |
| `org.springframework.core.codec.ByteArrayEncoder` | `codec/ByteArrayEncoder.java` | `codec/byte_array_encoder.rs` | `codec/byte_array_encoder.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/codec/byte_array_encoder.rs#[cfg(test)]` |
| `org.springframework.core.codec.ByteBufferDecoder` | `codec/ByteBufferDecoder.java` | `codec/byte_buffer_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.ByteBufferEncoder` | `codec/ByteBufferEncoder.java` | `codec/byte_buffer_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.CharBufferDecoder` | `codec/CharBufferDecoder.java` | `codec/char_buffer_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.CharSequenceEncoder` | `codec/CharSequenceEncoder.java` | `codec/char_sequence_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.CodecException` | `codec/CodecException.java` | `codec/codec_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.DataBufferDecoder` | `codec/DataBufferDecoder.java` | `codec/data_buffer_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.DataBufferEncoder` | `codec/DataBufferEncoder.java` | `codec/data_buffer_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.Decoder` | `codec/Decoder.java` | `codec/decoder.rs` | `codec/decoder.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/codec/string_decoder.rs#[cfg(test)]` |
| `org.springframework.core.codec.DecodingException` | `codec/DecodingException.java` | `codec/decoding_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.Encoder` | `codec/Encoder.java` | `codec/encoder.rs` | `codec/encoder.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/codec/codec_error.rs#[cfg(test)]` |
| `org.springframework.core.codec.EncodingException` | `codec/EncodingException.java` | `codec/encoding_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.Hints` | `codec/Hints.java` | `codec/hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.NettyByteBufDecoder` | `codec/NettyByteBufDecoder.java` | `codec/netty_byte_buf_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.NettyByteBufEncoder` | `codec/NettyByteBufEncoder.java` | `codec/netty_byte_buf_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.ResourceDecoder` | `codec/ResourceDecoder.java` | `codec/resource_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.ResourceEncoder` | `codec/ResourceEncoder.java` | `codec/resource_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.ResourceRegionEncoder` | `codec/ResourceRegionEncoder.java` | `codec/resource_region_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.codec.StringDecoder` | `codec/StringDecoder.java` | `codec/string_decoder.rs` | `codec/string_decoder.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/codec/string_decoder.rs#[cfg(test)]` |
| `org.springframework.core.convert.ConversionException` | `convert/ConversionException.java` | `convert/conversion_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.ConversionFailedException` | `convert/ConversionFailedException.java` | `convert/conversion_failed_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.ConversionService` | `convert/ConversionService.java` | `convert/conversion_service.rs` | `convert/conversion_service.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/convert/duration_converter.rs#[cfg(test)]` |
| `org.springframework.core.convert.ConverterNotFoundException` | `convert/ConverterNotFoundException.java` | `convert/converter_not_found_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.Property` | `convert/Property.java` | `convert/property.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.TypeDescriptor` | `convert/TypeDescriptor.java` | `convert/type_descriptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.converter.ConditionalConverter` | `convert/converter/ConditionalConverter.java` | `convert/converter/conditional_converter.rs` | `convert/converter/conditional_converter.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/convert/converter/type_pair_conditional_converter.rs#[cfg(test)]` |
| `org.springframework.core.convert.converter.ConditionalGenericConverter` | `convert/converter/ConditionalGenericConverter.java` | `convert/converter/conditional_generic_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.converter.Converter` | `convert/converter/Converter.java` | `convert/converter/converter.rs` | `convert/converter/converter.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/convert/duration_converter.rs#[cfg(test)]` |
| `org.springframework.core.convert.converter.ConverterFactory` | `convert/converter/ConverterFactory.java` | `convert/converter/converter_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.converter.ConverterRegistry` | `convert/converter/ConverterRegistry.java` | `convert/converter/converter_registry.rs` | `convert/converter/converter_registry.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/convert/converter/converter_registry.rs#[cfg(test)]` |
| `org.springframework.core.convert.converter.ConvertingComparator` | `convert/converter/ConvertingComparator.java` | `convert/converter/converting_comparator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.converter.GenericConverter` | `convert/converter/GenericConverter.java` | `convert/converter/generic_converter.rs` | `convert/converter/generic_converter.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/convert/converter/generic_converter.rs#[cfg(test)]` |
| `org.springframework.core.convert.support.AbstractConditionalEnumConverter` | `convert/support/AbstractConditionalEnumConverter.java` | `convert/support/abstract_conditional_enum_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ArrayToArrayConverter` | `convert/support/ArrayToArrayConverter.java` | `convert/support/array_to_array_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ArrayToCollectionConverter` | `convert/support/ArrayToCollectionConverter.java` | `convert/support/array_to_collection_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ArrayToObjectConverter` | `convert/support/ArrayToObjectConverter.java` | `convert/support/array_to_object_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ArrayToStringConverter` | `convert/support/ArrayToStringConverter.java` | `convert/support/array_to_string_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ByteBufferConverter` | `convert/support/ByteBufferConverter.java` | `convert/support/byte_buffer_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.CharacterToNumberFactory` | `convert/support/CharacterToNumberFactory.java` | `convert/support/character_to_number_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.CollectionToArrayConverter` | `convert/support/CollectionToArrayConverter.java` | `convert/support/collection_to_array_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.CollectionToCollectionConverter` | `convert/support/CollectionToCollectionConverter.java` | `convert/support/collection_to_collection_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.CollectionToObjectConverter` | `convert/support/CollectionToObjectConverter.java` | `convert/support/collection_to_object_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.CollectionToStringConverter` | `convert/support/CollectionToStringConverter.java` | `convert/support/collection_to_string_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ConfigurableConversionService` | `convert/support/ConfigurableConversionService.java` | `convert/support/configurable_conversion_service.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ConversionServiceFactory` | `convert/support/ConversionServiceFactory.java` | `convert/support/conversion_service_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ConversionUtils` | `convert/support/ConversionUtils.java` | `convert/support/conversion_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ConvertingPropertyEditorAdapter` | `convert/support/ConvertingPropertyEditorAdapter.java` | `convert/support/converting_property_editor_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.DateToInstantConverter` | `convert/support/DateToInstantConverter.java` | `convert/support/date_to_instant_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.DefaultConversionService` | `convert/support/DefaultConversionService.java` | `convert/support/default_conversion_service.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.EnumToIntegerConverter` | `convert/support/EnumToIntegerConverter.java` | `convert/support/enum_to_integer_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.EnumToStringConverter` | `convert/support/EnumToStringConverter.java` | `convert/support/enum_to_string_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.FallbackObjectToStringConverter` | `convert/support/FallbackObjectToStringConverter.java` | `convert/support/fallback_object_to_string_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.GenericConversionService` | `convert/support/GenericConversionService.java` | `convert/support/generic_conversion_service.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.IdToEntityConverter` | `convert/support/IdToEntityConverter.java` | `convert/support/id_to_entity_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.InstantToDateConverter` | `convert/support/InstantToDateConverter.java` | `convert/support/instant_to_date_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.IntegerToEnumConverterFactory` | `convert/support/IntegerToEnumConverterFactory.java` | `convert/support/integer_to_enum_converter_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.MapToMapConverter` | `convert/support/MapToMapConverter.java` | `convert/support/map_to_map_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.NumberToCharacterConverter` | `convert/support/NumberToCharacterConverter.java` | `convert/support/number_to_character_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.NumberToNumberConverterFactory` | `convert/support/NumberToNumberConverterFactory.java` | `convert/support/number_to_number_converter_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ObjectToArrayConverter` | `convert/support/ObjectToArrayConverter.java` | `convert/support/object_to_array_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ObjectToCollectionConverter` | `convert/support/ObjectToCollectionConverter.java` | `convert/support/object_to_collection_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ObjectToObjectConverter` | `convert/support/ObjectToObjectConverter.java` | `convert/support/object_to_object_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ObjectToOptionalConverter` | `convert/support/ObjectToOptionalConverter.java` | `convert/support/object_to_optional_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ObjectToStringConverter` | `convert/support/ObjectToStringConverter.java` | `convert/support/object_to_string_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.OptionalToObjectConverter` | `convert/support/OptionalToObjectConverter.java` | `convert/support/optional_to_object_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.PropertiesToStringConverter` | `convert/support/PropertiesToStringConverter.java` | `convert/support/properties_to_string_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StreamConverter` | `convert/support/StreamConverter.java` | `convert/support/stream_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToArrayConverter` | `convert/support/StringToArrayConverter.java` | `convert/support/string_to_array_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToBooleanConverter` | `convert/support/StringToBooleanConverter.java` | `convert/support/string_to_boolean_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToCharacterConverter` | `convert/support/StringToCharacterConverter.java` | `convert/support/string_to_character_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToCharsetConverter` | `convert/support/StringToCharsetConverter.java` | `convert/support/string_to_charset_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToCollectionConverter` | `convert/support/StringToCollectionConverter.java` | `convert/support/string_to_collection_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToCurrencyConverter` | `convert/support/StringToCurrencyConverter.java` | `convert/support/string_to_currency_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToEnumConverterFactory` | `convert/support/StringToEnumConverterFactory.java` | `convert/support/string_to_enum_converter_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToLocaleConverter` | `convert/support/StringToLocaleConverter.java` | `convert/support/string_to_locale_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToNumberConverterFactory` | `convert/support/StringToNumberConverterFactory.java` | `convert/support/string_to_number_converter_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToPatternConverter` | `convert/support/StringToPatternConverter.java` | `convert/support/string_to_pattern_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToPropertiesConverter` | `convert/support/StringToPropertiesConverter.java` | `convert/support/string_to_properties_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToRegexConverter` | `convert/support/StringToRegexConverter.java` | `convert/support/string_to_regex_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToTimeZoneConverter` | `convert/support/StringToTimeZoneConverter.java` | `convert/support/string_to_time_zone_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.StringToUUIDConverter` | `convert/support/StringToUUIDConverter.java` | `convert/support/string_to_uuid_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ZoneIdToTimeZoneConverter` | `convert/support/ZoneIdToTimeZoneConverter.java` | `convert/support/zone_id_to_time_zone_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.convert.support.ZonedDateTimeToCalendarConverter` | `convert/support/ZonedDateTimeToCalendarConverter.java` | `convert/support/zoned_date_time_to_calendar_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.AbstractEnvironment` | `env/AbstractEnvironment.java` | `env/abstract_environment.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.AbstractPropertyResolver` | `env/AbstractPropertyResolver.java` | `env/abstract_property_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.CommandLineArgs` | `env/CommandLineArgs.java` | `env/command_line_args.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.CommandLinePropertySource` | `env/CommandLinePropertySource.java` | `env/command_line_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.CompositePropertySource` | `env/CompositePropertySource.java` | `env/composite_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.ConfigurableEnvironment` | `env/ConfigurableEnvironment.java` | `env/configurable_environment.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.ConfigurablePropertyResolver` | `env/ConfigurablePropertyResolver.java` | `env/configurable_property_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.EnumerablePropertySource` | `env/EnumerablePropertySource.java` | `env/enumerable_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.Environment` | `env/Environment.java` | `env/environment.rs` | `env/environment.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/env/mod.rs#[cfg(test)]` |
| `org.springframework.core.env.EnvironmentCapable` | `env/EnvironmentCapable.java` | `env/environment_capable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.JOptCommandLinePropertySource` | `env/JOptCommandLinePropertySource.java` | `env/j_opt_command_line_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.MapPropertySource` | `env/MapPropertySource.java` | `env/map_property_source.rs` | `env/map_property_source.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/env/mod.rs#[cfg(test)]` |
| `org.springframework.core.env.MissingRequiredPropertiesException` | `env/MissingRequiredPropertiesException.java` | `env/missing_required_properties_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.MutablePropertySources` | `env/MutablePropertySources.java` | `env/mutable_property_sources.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.Profiles` | `env/Profiles.java` | `env/profiles.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.ProfilesParser` | `env/ProfilesParser.java` | `env/profiles_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.PropertiesPropertySource` | `env/PropertiesPropertySource.java` | `env/properties_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.PropertyResolver` | `env/PropertyResolver.java` | `env/property_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.PropertySource` | `env/PropertySource.java` | `env/property_source.rs` | `env/property_source.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/properties_file.rs#[cfg(test)]` |
| `org.springframework.core.env.PropertySources` | `env/PropertySources.java` | `env/property_sources.rs` | `env/property_sources.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/properties_file.rs#[cfg(test)]` |
| `org.springframework.core.env.PropertySourcesPropertyResolver` | `env/PropertySourcesPropertyResolver.java` | `env/property_sources_property_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.SimpleCommandLineArgsParser` | `env/SimpleCommandLineArgsParser.java` | `env/simple_command_line_args_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.SimpleCommandLinePropertySource` | `env/SimpleCommandLinePropertySource.java` | `env/simple_command_line_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.env.StandardEnvironment` | `env/StandardEnvironment.java` | `env/standard_environment.rs` | `env/standard_environment.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/env/mod.rs#[cfg(test)]` |
| `org.springframework.core.env.SystemEnvironmentPropertySource` | `env/SystemEnvironmentPropertySource.java` | `env/system_environment_property_source.rs` | `env/system_environment_property_source.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/env/mod.rs#[cfg(test)]` |
| `org.springframework.core.io.AbstractFileResolvingResource` | `io/AbstractFileResolvingResource.java` | `io/abstract_file_resolving_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.AbstractResource` | `io/AbstractResource.java` | `io/abstract_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ByteArrayResource` | `io/ByteArrayResource.java` | `io/byte_array_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ClassPathResource` | `io/ClassPathResource.java` | `io/class_path_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ClassRelativeResourceLoader` | `io/ClassRelativeResourceLoader.java` | `io/class_relative_resource_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ContextResource` | `io/ContextResource.java` | `io/context_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.DefaultResourceLoader` | `io/DefaultResourceLoader.java` | `io/default_resource_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.DescriptiveResource` | `io/DescriptiveResource.java` | `io/descriptive_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.FileSystemResource` | `io/FileSystemResource.java` | `io/file_system_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.FileSystemResourceLoader` | `io/FileSystemResourceLoader.java` | `io/file_system_resource_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.FileUrlResource` | `io/FileUrlResource.java` | `io/file_url_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.InputStreamResource` | `io/InputStreamResource.java` | `io/input_stream_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.InputStreamSource` | `io/InputStreamSource.java` | `io/input_stream_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ModuleResource` | `io/ModuleResource.java` | `io/module_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.PathResource` | `io/PathResource.java` | `io/path_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ProtocolResolver` | `io/ProtocolResolver.java` | `io/protocol_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.Resource` | `io/Resource.java` | `io/resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ResourceEditor` | `io/ResourceEditor.java` | `io/resource_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.ResourceLoader` | `io/ResourceLoader.java` | `io/resource_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.UrlResource` | `io/UrlResource.java` | `io/url_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.VfsResource` | `io/VfsResource.java` | `io/vfs_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.VfsUtils` | `io/VfsUtils.java` | `io/vfs_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.WritableResource` | `io/WritableResource.java` | `io/writable_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.CloseableDataBuffer` | `io/buffer/CloseableDataBuffer.java` | `io/buffer/closeable_data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBuffer` | `io/buffer/DataBuffer.java` | `io/buffer/data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBufferFactory` | `io/buffer/DataBufferFactory.java` | `io/buffer/data_buffer_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBufferInputStream` | `io/buffer/DataBufferInputStream.java` | `io/buffer/data_buffer_input_stream.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBufferLimitException` | `io/buffer/DataBufferLimitException.java` | `io/buffer/data_buffer_limit_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBufferOutputStream` | `io/buffer/DataBufferOutputStream.java` | `io/buffer/data_buffer_output_stream.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBufferUtils` | `io/buffer/DataBufferUtils.java` | `io/buffer/data_buffer_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DataBufferWrapper` | `io/buffer/DataBufferWrapper.java` | `io/buffer/data_buffer_wrapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DefaultDataBuffer` | `io/buffer/DefaultDataBuffer.java` | `io/buffer/default_data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.DefaultDataBufferFactory` | `io/buffer/DefaultDataBufferFactory.java` | `io/buffer/default_data_buffer_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.JettyDataBuffer` | `io/buffer/JettyDataBuffer.java` | `io/buffer/jetty_data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.JettyDataBufferFactory` | `io/buffer/JettyDataBufferFactory.java` | `io/buffer/jetty_data_buffer_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.LimitedDataBufferList` | `io/buffer/LimitedDataBufferList.java` | `io/buffer/limited_data_buffer_list.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.NettyDataBuffer` | `io/buffer/NettyDataBuffer.java` | `io/buffer/netty_data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.NettyDataBufferFactory` | `io/buffer/NettyDataBufferFactory.java` | `io/buffer/netty_data_buffer_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.OutputStreamPublisher` | `io/buffer/OutputStreamPublisher.java` | `io/buffer/output_stream_publisher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.PooledDataBuffer` | `io/buffer/PooledDataBuffer.java` | `io/buffer/pooled_data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.SubscriberInputStream` | `io/buffer/SubscriberInputStream.java` | `io/buffer/subscriber_input_stream.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.buffer.TouchableDataBuffer` | `io/buffer/TouchableDataBuffer.java` | `io/buffer/touchable_data_buffer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.DefaultPropertySourceFactory` | `io/support/DefaultPropertySourceFactory.java` | `io/support/default_property_source_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.EncodedResource` | `io/support/EncodedResource.java` | `io/support/encoded_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.LocalizedResourceHelper` | `io/support/LocalizedResourceHelper.java` | `io/support/localized_resource_helper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.PathMatchingResourcePatternResolver` | `io/support/PathMatchingResourcePatternResolver.java` | `io/support/path_matching_resource_pattern_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.PropertiesLoaderSupport` | `io/support/PropertiesLoaderSupport.java` | `io/support/properties_loader_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.PropertiesLoaderUtils` | `io/support/PropertiesLoaderUtils.java` | `io/support/properties_loader_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.PropertySourceDescriptor` | `io/support/PropertySourceDescriptor.java` | `io/support/property_source_descriptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.PropertySourceFactory` | `io/support/PropertySourceFactory.java` | `io/support/property_source_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.PropertySourceProcessor` | `io/support/PropertySourceProcessor.java` | `io/support/property_source_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.ResourceArrayPropertyEditor` | `io/support/ResourceArrayPropertyEditor.java` | `io/support/resource_array_property_editor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.ResourcePatternResolver` | `io/support/ResourcePatternResolver.java` | `io/support/resource_pattern_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.ResourcePatternUtils` | `io/support/ResourcePatternUtils.java` | `io/support/resource_pattern_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.ResourcePropertySource` | `io/support/ResourcePropertySource.java` | `io/support/resource_property_source.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.ResourceRegion` | `io/support/ResourceRegion.java` | `io/support/resource_region.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.SpringFactoriesLoader` | `io/support/SpringFactoriesLoader.java` | `io/support/spring_factories_loader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.io.support.VfsPatternUtils` | `io/support/VfsPatternUtils.java` | `io/support/vfs_pattern_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.log.CompositeLog` | `log/CompositeLog.java` | `log/composite_log.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.log.LogAccessor` | `log/LogAccessor.java` | `log/log_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.log.LogDelegateFactory` | `log/LogDelegateFactory.java` | `log/log_delegate_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.log.LogFormatUtils` | `log/LogFormatUtils.java` | `log/log_format_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.log.LogMessage` | `log/LogMessage.java` | `log/log_message.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.metrics.ApplicationStartup` | `metrics/ApplicationStartup.java` | `metrics/application_startup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.metrics.DefaultApplicationStartup` | `metrics/DefaultApplicationStartup.java` | `metrics/default_application_startup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.metrics.StartupStep` | `metrics/StartupStep.java` | `metrics/startup_step.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.metrics.jfr.FlightRecorderApplicationStartup` | `metrics/jfr/FlightRecorderApplicationStartup.java` | `metrics/jfr/flight_recorder_application_startup.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.metrics.jfr.FlightRecorderStartupEvent` | `metrics/jfr/FlightRecorderStartupEvent.java` | `metrics/jfr/flight_recorder_startup_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.metrics.jfr.FlightRecorderStartupStep` | `metrics/jfr/FlightRecorderStartupStep.java` | `metrics/jfr/flight_recorder_startup_step.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.DefaultRetryPolicy` | `retry/DefaultRetryPolicy.java` | `retry/default_retry_policy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.RetryException` | `retry/RetryException.java` | `retry/retry_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.RetryListener` | `retry/RetryListener.java` | `retry/retry_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.RetryOperations` | `retry/RetryOperations.java` | `retry/retry_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.RetryPolicy` | `retry/RetryPolicy.java` | `retry/retry_policy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.RetryState` | `retry/RetryState.java` | `retry/retry_state.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.RetryTemplate` | `retry/RetryTemplate.java` | `retry/retry_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.Retryable` | `retry/Retryable.java` | `retry/retryable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.support.CompositeRetryListener` | `retry/support/CompositeRetryListener.java` | `retry/support/composite_retry_listener.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.retry.support.RetryTask` | `retry/support/RetryTask.java` | `retry/support/retry_task.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.DefaultDeserializer` | `serializer/DefaultDeserializer.java` | `serializer/default_deserializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.DefaultSerializer` | `serializer/DefaultSerializer.java` | `serializer/default_serializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.Deserializer` | `serializer/Deserializer.java` | `serializer/deserializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.Serializer` | `serializer/Serializer.java` | `serializer/serializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.support.DeserializingConverter` | `serializer/support/DeserializingConverter.java` | `serializer/support/deserializing_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.support.SerializationDelegate` | `serializer/support/SerializationDelegate.java` | `serializer/support/serialization_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.support.SerializationFailedException` | `serializer/support/SerializationFailedException.java` | `serializer/support/serialization_failed_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.serializer.support.SerializingConverter` | `serializer/support/SerializingConverter.java` | `serializer/support/serializing_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.DefaultToStringStyler` | `style/DefaultToStringStyler.java` | `style/default_to_string_styler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.DefaultValueStyler` | `style/DefaultValueStyler.java` | `style/default_value_styler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.SimpleValueStyler` | `style/SimpleValueStyler.java` | `style/simple_value_styler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.StylerUtils` | `style/StylerUtils.java` | `style/styler_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.ToStringCreator` | `style/ToStringCreator.java` | `style/to_string_creator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.ToStringStyler` | `style/ToStringStyler.java` | `style/to_string_styler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.style.ValueStyler` | `style/ValueStyler.java` | `style/value_styler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.AsyncTaskExecutor` | `task/AsyncTaskExecutor.java` | `task/async_task_executor.rs` | `task/async_task_executor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/task/async_task_executor.rs#[cfg(test)]` |
| `org.springframework.core.task.SimpleAsyncTaskExecutor` | `task/SimpleAsyncTaskExecutor.java` | `task/simple_async_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.SyncTaskExecutor` | `task/SyncTaskExecutor.java` | `task/sync_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskCallback` | `task/TaskCallback.java` | `task/task_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskDecorator` | `task/TaskDecorator.java` | `task/task_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskExecutor` | `task/TaskExecutor.java` | `task/task_executor.rs` | `task/task_executor.rs` | `IMPLEMENTED` | 预期路径、公开主类型、中文 Java 来源注释均存在；测试证据 `src/task/async_task_executor.rs#[cfg(test)]` |
| `org.springframework.core.task.TaskRejectedException` | `task/TaskRejectedException.java` | `task/task_rejected_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskTimeoutException` | `task/TaskTimeoutException.java` | `task/task_timeout_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.VirtualThreadDelegate` | `task/VirtualThreadDelegate.java` | `task/virtual_thread_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.VirtualThreadTaskExecutor` | `task/VirtualThreadTaskExecutor.java` | `task/virtual_thread_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.CompositeTaskDecorator` | `task/support/CompositeTaskDecorator.java` | `task/support/composite_task_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.ContextPropagatingTaskDecorator` | `task/support/ContextPropagatingTaskDecorator.java` | `task/support/context_propagating_task_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.ExecutorServiceAdapter` | `task/support/ExecutorServiceAdapter.java` | `task/support/executor_service_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.TaskExecutorAdapter` | `task/support/TaskExecutorAdapter.java` | `task/support/task_executor_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.AnnotatedTypeMetadata` | `type/AnnotatedTypeMetadata.java` | `type/annotated_type_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.AnnotationMetadata` | `type/AnnotationMetadata.java` | `type/annotation_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.ClassMetadata` | `type/ClassMetadata.java` | `type/class_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.MethodMetadata` | `type/MethodMetadata.java` | `type/method_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.StandardAnnotationMetadata` | `type/StandardAnnotationMetadata.java` | `type/standard_annotation_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.StandardClassMetadata` | `type/StandardClassMetadata.java` | `type/standard_class_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.StandardMethodMetadata` | `type/StandardMethodMetadata.java` | `type/standard_method_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.AbstractMetadataReaderFactory` | `type/classreading/AbstractMetadataReaderFactory.java` | `type/classreading/abstract_metadata_reader_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.CachingMetadataReaderFactory` | `type/classreading/CachingMetadataReaderFactory.java` | `type/classreading/caching_metadata_reader_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.ClassFormatException` | `type/classreading/ClassFormatException.java` | `type/classreading/class_format_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.MergedAnnotationReadingVisitor` | `type/classreading/MergedAnnotationReadingVisitor.java` | `type/classreading/merged_annotation_reading_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.MetadataReader` | `type/classreading/MetadataReader.java` | `type/classreading/metadata_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.MetadataReaderFactory` | `type/classreading/MetadataReaderFactory.java` | `type/classreading/metadata_reader_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.MetadataReaderFactoryDelegate` | `type/classreading/MetadataReaderFactoryDelegate.java` | `type/classreading/metadata_reader_factory_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.SimpleAnnotationMetadata` | `type/classreading/SimpleAnnotationMetadata.java` | `type/classreading/simple_annotation_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.SimpleAnnotationMetadataReadingVisitor` | `type/classreading/SimpleAnnotationMetadataReadingVisitor.java` | `type/classreading/simple_annotation_metadata_reading_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.SimpleMetadataReader` | `type/classreading/SimpleMetadataReader.java` | `type/classreading/simple_metadata_reader.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.SimpleMetadataReaderFactory` | `type/classreading/SimpleMetadataReaderFactory.java` | `type/classreading/simple_metadata_reader_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.SimpleMethodMetadata` | `type/classreading/SimpleMethodMetadata.java` | `type/classreading/simple_method_metadata.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.classreading.SimpleMethodMetadataReadingVisitor` | `type/classreading/SimpleMethodMetadataReadingVisitor.java` | `type/classreading/simple_method_metadata_reading_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.AbstractClassTestingTypeFilter` | `type/filter/AbstractClassTestingTypeFilter.java` | `type/filter/abstract_class_testing_type_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.AbstractTypeHierarchyTraversingFilter` | `type/filter/AbstractTypeHierarchyTraversingFilter.java` | `type/filter/abstract_type_hierarchy_traversing_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.AnnotationTypeFilter` | `type/filter/AnnotationTypeFilter.java` | `type/filter/annotation_type_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.AspectJTypeFilter` | `type/filter/AspectJTypeFilter.java` | `type/filter/aspect_j_type_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.AssignableTypeFilter` | `type/filter/AssignableTypeFilter.java` | `type/filter/assignable_type_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.RegexPatternTypeFilter` | `type/filter/RegexPatternTypeFilter.java` | `type/filter/regex_pattern_type_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.type.filter.TypeFilter` | `type/filter/TypeFilter.java` | `type/filter/type_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-messaging 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 222 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 222 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 2 |
| `MISSING` | 219 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 1 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `message.rs`：`Message`、`GenericMessage`
- 单文件多个公开对象位于 `channel.rs`：`SendFuture`、`ReceiveFuture`、`MessageChannel`、`SubscribableChannel`、`SubscriptionId`、`MessageHandler`、`InMemoryChannel`、`MessageError`
- 单文件多个公开对象位于 `default_simp_user_registry.rs`：`SimpSubscription`、`SimpSession`、`SimpUser`、`DefaultSimpUserRegistry`、`SharedSimpUserRegistry`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.messaging.Message` | `Message.java` | `message.rs` | `message.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释、测试引用 |
| `org.springframework.messaging.MessageChannel` | `MessageChannel.java` | `message_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.MessageDeliveryException` | `MessageDeliveryException.java` | `message_delivery_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.MessageHandler` | `MessageHandler.java` | `message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.MessageHandlingException` | `MessageHandlingException.java` | `message_handling_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.MessageHeaders` | `MessageHeaders.java` | `message_headers.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.MessagingException` | `MessagingException.java` | `messaging_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.PollableChannel` | `PollableChannel.java` | `pollable_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.ReactiveMessageHandler` | `ReactiveMessageHandler.java` | `reactive_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.SubscribableChannel` | `SubscribableChannel.java` | `subscribable_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.AbstractJsonMessageConverter` | `converter/AbstractJsonMessageConverter.java` | `converter/abstract_json_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.AbstractMessageConverter` | `converter/AbstractMessageConverter.java` | `converter/abstract_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.ByteArrayMessageConverter` | `converter/ByteArrayMessageConverter.java` | `converter/byte_array_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.CompositeMessageConverter` | `converter/CompositeMessageConverter.java` | `converter/composite_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.ContentTypeResolver` | `converter/ContentTypeResolver.java` | `converter/content_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.DefaultContentTypeResolver` | `converter/DefaultContentTypeResolver.java` | `converter/default_content_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.GenericMessageConverter` | `converter/GenericMessageConverter.java` | `converter/generic_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.GsonMessageConverter` | `converter/GsonMessageConverter.java` | `converter/gson_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.JacksonJsonMessageConverter` | `converter/JacksonJsonMessageConverter.java` | `converter/jackson_json_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.JsonbMessageConverter` | `converter/JsonbMessageConverter.java` | `converter/jsonb_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.KotlinSerializationJsonMessageConverter` | `converter/KotlinSerializationJsonMessageConverter.java` | `converter/kotlin_serialization_json_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.MappingJackson2MessageConverter` | `converter/MappingJackson2MessageConverter.java` | `converter/mapping_jackson2_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.MarshallingMessageConverter` | `converter/MarshallingMessageConverter.java` | `converter/marshalling_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.MessageConversionException` | `converter/MessageConversionException.java` | `converter/message_conversion_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.MessageConverter` | `converter/MessageConverter.java` | `converter/message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.ProtobufJsonFormatMessageConverter` | `converter/ProtobufJsonFormatMessageConverter.java` | `converter/protobuf_json_format_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.ProtobufMessageConverter` | `converter/ProtobufMessageConverter.java` | `converter/protobuf_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.SimpleMessageConverter` | `converter/SimpleMessageConverter.java` | `converter/simple_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.SmartMessageConverter` | `converter/SmartMessageConverter.java` | `converter/smart_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.converter.StringMessageConverter` | `converter/StringMessageConverter.java` | `converter/string_message_converter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.AbstractDestinationResolvingMessagingTemplate` | `core/AbstractDestinationResolvingMessagingTemplate.java` | `core/abstract_destination_resolving_messaging_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.AbstractMessageReceivingTemplate` | `core/AbstractMessageReceivingTemplate.java` | `core/abstract_message_receiving_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.AbstractMessageSendingTemplate` | `core/AbstractMessageSendingTemplate.java` | `core/abstract_message_sending_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.AbstractMessagingTemplate` | `core/AbstractMessagingTemplate.java` | `core/abstract_messaging_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.BeanFactoryMessageChannelDestinationResolver` | `core/BeanFactoryMessageChannelDestinationResolver.java` | `core/bean_factory_message_channel_destination_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.CachingDestinationResolverProxy` | `core/CachingDestinationResolverProxy.java` | `core/caching_destination_resolver_proxy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.CompositeMessagePostProcessor` | `core/CompositeMessagePostProcessor.java` | `core/composite_message_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.DestinationResolutionException` | `core/DestinationResolutionException.java` | `core/destination_resolution_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.DestinationResolver` | `core/DestinationResolver.java` | `core/destination_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.DestinationResolvingMessageReceivingOperations` | `core/DestinationResolvingMessageReceivingOperations.java` | `core/destination_resolving_message_receiving_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.DestinationResolvingMessageRequestReplyOperations` | `core/DestinationResolvingMessageRequestReplyOperations.java` | `core/destination_resolving_message_request_reply_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.DestinationResolvingMessageSendingOperations` | `core/DestinationResolvingMessageSendingOperations.java` | `core/destination_resolving_message_sending_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.GenericMessagingTemplate` | `core/GenericMessagingTemplate.java` | `core/generic_messaging_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.MessagePostProcessor` | `core/MessagePostProcessor.java` | `core/message_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.MessageReceivingOperations` | `core/MessageReceivingOperations.java` | `core/message_receiving_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.MessageRequestReplyOperations` | `core/MessageRequestReplyOperations.java` | `core/message_request_reply_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.core.MessageSendingOperations` | `core/MessageSendingOperations.java` | `core/message_sending_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.AbstractMessageCondition` | `handler/AbstractMessageCondition.java` | `handler/abstract_message_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.CompositeMessageCondition` | `handler/CompositeMessageCondition.java` | `handler/composite_message_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.DestinationPatternsMessageCondition` | `handler/DestinationPatternsMessageCondition.java` | `handler/destination_patterns_message_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.HandlerMethod` | `handler/HandlerMethod.java` | `handler/handler_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.MessageCondition` | `handler/MessageCondition.java` | `handler/message_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.MessagingAdviceBean` | `handler/MessagingAdviceBean.java` | `handler/messaging_advice_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.DestinationVariable` | `handler/annotation/DestinationVariable.java` | `handler/annotation/destination_variable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.Header` | `handler/annotation/Header.java` | `handler/annotation/header.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.Headers` | `handler/annotation/Headers.java` | `handler/annotation/headers.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.MessageExceptionHandler` | `handler/annotation/MessageExceptionHandler.java` | `handler/annotation/message_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.MessageMapping` | `handler/annotation/MessageMapping.java` | `handler/annotation/message_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.MessageMappingReflectiveProcessor` | `handler/annotation/MessageMappingReflectiveProcessor.java` | `handler/annotation/message_mapping_reflective_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.Payload` | `handler/annotation/Payload.java` | `handler/annotation/payload.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.SendTo` | `handler/annotation/SendTo.java` | `handler/annotation/send_to.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.ValueConstants` | `handler/annotation/ValueConstants.java` | `handler/annotation/value_constants.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.AbstractNamedValueMethodArgumentResolver` | `handler/annotation/reactive/AbstractNamedValueMethodArgumentResolver.java` | `annotation/reactive/abstract_named_value_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.ContinuationHandlerMethodArgumentResolver` | `handler/annotation/reactive/ContinuationHandlerMethodArgumentResolver.java` | `annotation/reactive/continuation_handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.DestinationVariableMethodArgumentResolver` | `handler/annotation/reactive/DestinationVariableMethodArgumentResolver.java` | `annotation/reactive/destination_variable_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.HeaderMethodArgumentResolver` | `handler/annotation/reactive/HeaderMethodArgumentResolver.java` | `annotation/reactive/header_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.HeadersMethodArgumentResolver` | `handler/annotation/reactive/HeadersMethodArgumentResolver.java` | `annotation/reactive/headers_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.MessageMappingMessageHandler` | `handler/annotation/reactive/MessageMappingMessageHandler.java` | `annotation/reactive/message_mapping_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.reactive.PayloadMethodArgumentResolver` | `handler/annotation/reactive/PayloadMethodArgumentResolver.java` | `annotation/reactive/payload_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.AbstractNamedValueMethodArgumentResolver` | `handler/annotation/support/AbstractNamedValueMethodArgumentResolver.java` | `annotation/support/abstract_named_value_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.AnnotationExceptionHandlerMethodResolver` | `handler/annotation/support/AnnotationExceptionHandlerMethodResolver.java` | `annotation/support/annotation_exception_handler_method_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.DefaultMessageHandlerMethodFactory` | `handler/annotation/support/DefaultMessageHandlerMethodFactory.java` | `annotation/support/default_message_handler_method_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.DestinationVariableMethodArgumentResolver` | `handler/annotation/support/DestinationVariableMethodArgumentResolver.java` | `annotation/support/destination_variable_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.HeaderMethodArgumentResolver` | `handler/annotation/support/HeaderMethodArgumentResolver.java` | `annotation/support/header_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.HeadersMethodArgumentResolver` | `handler/annotation/support/HeadersMethodArgumentResolver.java` | `annotation/support/headers_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.MessageHandlerMethodFactory` | `handler/annotation/support/MessageHandlerMethodFactory.java` | `annotation/support/message_handler_method_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.MessageMethodArgumentResolver` | `handler/annotation/support/MessageMethodArgumentResolver.java` | `annotation/support/message_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.MethodArgumentNotValidException` | `handler/annotation/support/MethodArgumentNotValidException.java` | `annotation/support/method_argument_not_valid_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.MethodArgumentTypeMismatchException` | `handler/annotation/support/MethodArgumentTypeMismatchException.java` | `annotation/support/method_argument_type_mismatch_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.annotation.support.PayloadMethodArgumentResolver` | `handler/annotation/support/PayloadMethodArgumentResolver.java` | `annotation/support/payload_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.AbstractAsyncReturnValueHandler` | `handler/invocation/AbstractAsyncReturnValueHandler.java` | `handler/invocation/abstract_async_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.AbstractExceptionHandlerMethodResolver` | `handler/invocation/AbstractExceptionHandlerMethodResolver.java` | `handler/invocation/abstract_exception_handler_method_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.AbstractMethodMessageHandler` | `handler/invocation/AbstractMethodMessageHandler.java` | `handler/invocation/abstract_method_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.AsyncHandlerMethodReturnValueHandler` | `handler/invocation/AsyncHandlerMethodReturnValueHandler.java` | `handler/invocation/async_handler_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.CompletableFutureReturnValueHandler` | `handler/invocation/CompletableFutureReturnValueHandler.java` | `handler/invocation/completable_future_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.HandlerMethodArgumentResolver` | `handler/invocation/HandlerMethodArgumentResolver.java` | `handler/invocation/handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.HandlerMethodArgumentResolverComposite` | `handler/invocation/HandlerMethodArgumentResolverComposite.java` | `handler/invocation/handler_method_argument_resolver_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.HandlerMethodReturnValueHandler` | `handler/invocation/HandlerMethodReturnValueHandler.java` | `handler/invocation/handler_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.HandlerMethodReturnValueHandlerComposite` | `handler/invocation/HandlerMethodReturnValueHandlerComposite.java` | `handler/invocation/handler_method_return_value_handler_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.InvocableHandlerMethod` | `handler/invocation/InvocableHandlerMethod.java` | `handler/invocation/invocable_handler_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.MethodArgumentResolutionException` | `handler/invocation/MethodArgumentResolutionException.java` | `handler/invocation/method_argument_resolution_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.ReactiveReturnValueHandler` | `handler/invocation/ReactiveReturnValueHandler.java` | `handler/invocation/reactive_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.AbstractEncoderMethodReturnValueHandler` | `handler/invocation/reactive/AbstractEncoderMethodReturnValueHandler.java` | `invocation/reactive/abstract_encoder_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.AbstractMethodMessageHandler` | `handler/invocation/reactive/AbstractMethodMessageHandler.java` | `invocation/reactive/abstract_method_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.ArgumentResolverConfigurer` | `handler/invocation/reactive/ArgumentResolverConfigurer.java` | `invocation/reactive/argument_resolver_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.ChannelSendOperator` | `handler/invocation/reactive/ChannelSendOperator.java` | `invocation/reactive/channel_send_operator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.HandlerMethodArgumentResolver` | `handler/invocation/reactive/HandlerMethodArgumentResolver.java` | `invocation/reactive/handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.HandlerMethodArgumentResolverComposite` | `handler/invocation/reactive/HandlerMethodArgumentResolverComposite.java` | `invocation/reactive/handler_method_argument_resolver_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.HandlerMethodReturnValueHandler` | `handler/invocation/reactive/HandlerMethodReturnValueHandler.java` | `invocation/reactive/handler_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.HandlerMethodReturnValueHandlerComposite` | `handler/invocation/reactive/HandlerMethodReturnValueHandlerComposite.java` | `invocation/reactive/handler_method_return_value_handler_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.InvocableHandlerMethod` | `handler/invocation/reactive/InvocableHandlerMethod.java` | `invocation/reactive/invocable_handler_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.InvocableHelper` | `handler/invocation/reactive/InvocableHelper.java` | `invocation/reactive/invocable_helper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.ReturnValueHandlerConfigurer` | `handler/invocation/reactive/ReturnValueHandlerConfigurer.java` | `invocation/reactive/return_value_handler_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.handler.invocation.reactive.SyncHandlerMethodArgumentResolver` | `handler/invocation/reactive/SyncHandlerMethodArgumentResolver.java` | `invocation/reactive/sync_handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.DefaultMetadataExtractor` | `rsocket/DefaultMetadataExtractor.java` | `rsocket/default_metadata_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.DefaultRSocketRequester` | `rsocket/DefaultRSocketRequester.java` | `rsocket/default_r_socket_requester.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.DefaultRSocketRequesterBuilder` | `rsocket/DefaultRSocketRequesterBuilder.java` | `rsocket/default_r_socket_requester_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.DefaultRSocketStrategies` | `rsocket/DefaultRSocketStrategies.java` | `rsocket/default_r_socket_strategies.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.MetadataEncoder` | `rsocket/MetadataEncoder.java` | `rsocket/metadata_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.MetadataExtractor` | `rsocket/MetadataExtractor.java` | `rsocket/metadata_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.MetadataExtractorRegistry` | `rsocket/MetadataExtractorRegistry.java` | `rsocket/metadata_extractor_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.PayloadUtils` | `rsocket/PayloadUtils.java` | `rsocket/payload_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.RSocketConnectorConfigurer` | `rsocket/RSocketConnectorConfigurer.java` | `rsocket/r_socket_connector_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.RSocketRequester` | `rsocket/RSocketRequester.java` | `rsocket/r_socket_requester.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.RSocketStrategies` | `rsocket/RSocketStrategies.java` | `rsocket/r_socket_strategies.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.annotation.ConnectMapping` | `rsocket/annotation/ConnectMapping.java` | `rsocket/annotation/connect_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.annotation.support.MessagingRSocket` | `rsocket/annotation/support/MessagingRSocket.java` | `annotation/support/messaging_r_socket.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.annotation.support.RSocketFrameTypeMessageCondition` | `rsocket/annotation/support/RSocketFrameTypeMessageCondition.java` | `annotation/support/r_socket_frame_type_message_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.annotation.support.RSocketMessageHandler` | `rsocket/annotation/support/RSocketMessageHandler.java` | `annotation/support/r_socket_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.annotation.support.RSocketPayloadReturnValueHandler` | `rsocket/annotation/support/RSocketPayloadReturnValueHandler.java` | `annotation/support/r_socket_payload_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.annotation.support.RSocketRequesterMethodArgumentResolver` | `rsocket/annotation/support/RSocketRequesterMethodArgumentResolver.java` | `annotation/support/r_socket_requester_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.DestinationVariableArgumentResolver` | `rsocket/service/DestinationVariableArgumentResolver.java` | `rsocket/service/destination_variable_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.MetadataArgumentResolver` | `rsocket/service/MetadataArgumentResolver.java` | `rsocket/service/metadata_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.PayloadArgumentResolver` | `rsocket/service/PayloadArgumentResolver.java` | `rsocket/service/payload_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketExchange` | `rsocket/service/RSocketExchange.java` | `rsocket/service/r_socket_exchange.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketExchangeBeanRegistrationAotProcessor` | `rsocket/service/RSocketExchangeBeanRegistrationAotProcessor.java` | `rsocket/service/r_socket_exchange_bean_registration_aot_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketExchangeReflectiveProcessor` | `rsocket/service/RSocketExchangeReflectiveProcessor.java` | `rsocket/service/r_socket_exchange_reflective_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketRequestValues` | `rsocket/service/RSocketRequestValues.java` | `rsocket/service/r_socket_request_values.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketServiceArgumentResolver` | `rsocket/service/RSocketServiceArgumentResolver.java` | `rsocket/service/r_socket_service_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketServiceMethod` | `rsocket/service/RSocketServiceMethod.java` | `rsocket/service/r_socket_service_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.rsocket.service.RSocketServiceProxyFactory` | `rsocket/service/RSocketServiceProxyFactory.java` | `rsocket/service/r_socket_service_proxy_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpAttributes` | `simp/SimpAttributes.java` | `simp/simp_attributes.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpAttributesContextHolder` | `simp/SimpAttributesContextHolder.java` | `simp/simp_attributes_context_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpLogging` | `simp/SimpLogging.java` | `simp/simp_logging.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpMessageHeaderAccessor` | `simp/SimpMessageHeaderAccessor.java` | `simp/simp_message_header_accessor.rs` | `simp_message_header_accessor.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.messaging.simp.SimpMessageMappingInfo` | `simp/SimpMessageMappingInfo.java` | `simp/simp_message_mapping_info.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpMessageSendingOperations` | `simp/SimpMessageSendingOperations.java` | `simp/simp_message_sending_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpMessageType` | `simp/SimpMessageType.java` | `simp/simp_message_type.rs` | `simp_message_type.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.messaging.simp.SimpMessageTypeMessageCondition` | `simp/SimpMessageTypeMessageCondition.java` | `simp/simp_message_type_message_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpMessagingTemplate` | `simp/SimpMessagingTemplate.java` | `simp/simp_messaging_template.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.SimpSessionScope` | `simp/SimpSessionScope.java` | `simp/simp_session_scope.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.SendToUser` | `simp/annotation/SendToUser.java` | `simp/annotation/send_to_user.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.SubscribeMapping` | `simp/annotation/SubscribeMapping.java` | `simp/annotation/subscribe_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.support.MissingSessionUserException` | `simp/annotation/support/MissingSessionUserException.java` | `annotation/support/missing_session_user_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.support.PrincipalMethodArgumentResolver` | `simp/annotation/support/PrincipalMethodArgumentResolver.java` | `annotation/support/principal_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.support.SendToMethodReturnValueHandler` | `simp/annotation/support/SendToMethodReturnValueHandler.java` | `annotation/support/send_to_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.support.SimpAnnotationMethodMessageHandler` | `simp/annotation/support/SimpAnnotationMethodMessageHandler.java` | `annotation/support/simp_annotation_method_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.annotation.support.SubscriptionMethodReturnValueHandler` | `simp/annotation/support/SubscriptionMethodReturnValueHandler.java` | `annotation/support/subscription_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.AbstractBrokerMessageHandler` | `simp/broker/AbstractBrokerMessageHandler.java` | `simp/broker/abstract_broker_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.AbstractSubscriptionRegistry` | `simp/broker/AbstractSubscriptionRegistry.java` | `simp/broker/abstract_subscription_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.BrokerAvailabilityEvent` | `simp/broker/BrokerAvailabilityEvent.java` | `simp/broker/broker_availability_event.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.DefaultSubscriptionRegistry` | `simp/broker/DefaultSubscriptionRegistry.java` | `simp/broker/default_subscription_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.OrderedMessageChannelDecorator` | `simp/broker/OrderedMessageChannelDecorator.java` | `simp/broker/ordered_message_channel_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.SimpleBrokerMessageHandler` | `simp/broker/SimpleBrokerMessageHandler.java` | `simp/broker/simple_broker_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.broker.SubscriptionRegistry` | `simp/broker/SubscriptionRegistry.java` | `simp/broker/subscription_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.AbstractBrokerRegistration` | `simp/config/AbstractBrokerRegistration.java` | `simp/config/abstract_broker_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.AbstractMessageBrokerConfiguration` | `simp/config/AbstractMessageBrokerConfiguration.java` | `simp/config/abstract_message_broker_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.ChannelRegistration` | `simp/config/ChannelRegistration.java` | `simp/config/channel_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.MessageBrokerRegistry` | `simp/config/MessageBrokerRegistry.java` | `simp/config/message_broker_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.SimpleBrokerRegistration` | `simp/config/SimpleBrokerRegistration.java` | `simp/config/simple_broker_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.StompBrokerRelayRegistration` | `simp/config/StompBrokerRelayRegistration.java` | `simp/config/stomp_broker_relay_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.config.TaskExecutorRegistration` | `simp/config/TaskExecutorRegistration.java` | `simp/config/task_executor_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.BufferingStompDecoder` | `simp/stomp/BufferingStompDecoder.java` | `simp/stomp/buffering_stomp_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.ConnectionHandlingStompSession` | `simp/stomp/ConnectionHandlingStompSession.java` | `simp/stomp/connection_handling_stomp_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.ConnectionLostException` | `simp/stomp/ConnectionLostException.java` | `simp/stomp/connection_lost_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.DefaultStompSession` | `simp/stomp/DefaultStompSession.java` | `simp/stomp/default_stomp_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.ReactorNettyTcpStompClient` | `simp/stomp/ReactorNettyTcpStompClient.java` | `simp/stomp/reactor_netty_tcp_stomp_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.SplittingStompEncoder` | `simp/stomp/SplittingStompEncoder.java` | `simp/stomp/splitting_stomp_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompBrokerRelayMessageHandler` | `simp/stomp/StompBrokerRelayMessageHandler.java` | `simp/stomp/stomp_broker_relay_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompClientSupport` | `simp/stomp/StompClientSupport.java` | `simp/stomp/stomp_client_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompCommand` | `simp/stomp/StompCommand.java` | `simp/stomp/stomp_command.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompConversionException` | `simp/stomp/StompConversionException.java` | `simp/stomp/stomp_conversion_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompDecoder` | `simp/stomp/StompDecoder.java` | `simp/stomp/stomp_decoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompEncoder` | `simp/stomp/StompEncoder.java` | `simp/stomp/stomp_encoder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompFrameHandler` | `simp/stomp/StompFrameHandler.java` | `simp/stomp/stomp_frame_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompHeaderAccessor` | `simp/stomp/StompHeaderAccessor.java` | `simp/stomp/stomp_header_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompHeaders` | `simp/stomp/StompHeaders.java` | `simp/stomp/stomp_headers.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompReactorNettyCodec` | `simp/stomp/StompReactorNettyCodec.java` | `simp/stomp/stomp_reactor_netty_codec.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompSession` | `simp/stomp/StompSession.java` | `simp/stomp/stomp_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompSessionHandler` | `simp/stomp/StompSessionHandler.java` | `simp/stomp/stomp_session_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompSessionHandlerAdapter` | `simp/stomp/StompSessionHandlerAdapter.java` | `simp/stomp/stomp_session_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompTcpConnectionHandler` | `simp/stomp/StompTcpConnectionHandler.java` | `simp/stomp/stomp_tcp_connection_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.stomp.StompTcpMessageCodec` | `simp/stomp/StompTcpMessageCodec.java` | `simp/stomp/stomp_tcp_message_codec.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.DefaultUserDestinationResolver` | `simp/user/DefaultUserDestinationResolver.java` | `simp/user/default_user_destination_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.DestinationUserNameProvider` | `simp/user/DestinationUserNameProvider.java` | `simp/user/destination_user_name_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.MultiServerUserRegistry` | `simp/user/MultiServerUserRegistry.java` | `simp/user/multi_server_user_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.SimpSession` | `simp/user/SimpSession.java` | `simp/user/simp_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.SimpSubscription` | `simp/user/SimpSubscription.java` | `simp/user/simp_subscription.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.SimpSubscriptionMatcher` | `simp/user/SimpSubscriptionMatcher.java` | `simp/user/simp_subscription_matcher.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.SimpUser` | `simp/user/SimpUser.java` | `simp/user/simp_user.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.SimpUserRegistry` | `simp/user/SimpUserRegistry.java` | `simp/user/simp_user_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.UserDestinationMessageHandler` | `simp/user/UserDestinationMessageHandler.java` | `simp/user/user_destination_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.UserDestinationResolver` | `simp/user/UserDestinationResolver.java` | `simp/user/user_destination_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.UserDestinationResult` | `simp/user/UserDestinationResult.java` | `simp/user/user_destination_result.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.simp.user.UserRegistryMessageHandler` | `simp/user/UserRegistryMessageHandler.java` | `simp/user/user_registry_message_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.AbstractHeaderMapper` | `support/AbstractHeaderMapper.java` | `support/abstract_header_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.AbstractMessageChannel` | `support/AbstractMessageChannel.java` | `support/abstract_message_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.AbstractSubscribableChannel` | `support/AbstractSubscribableChannel.java` | `support/abstract_subscribable_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.ChannelInterceptor` | `support/ChannelInterceptor.java` | `support/channel_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.ErrorMessage` | `support/ErrorMessage.java` | `support/error_message.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.ExecutorChannelInterceptor` | `support/ExecutorChannelInterceptor.java` | `support/executor_channel_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.ExecutorSubscribableChannel` | `support/ExecutorSubscribableChannel.java` | `support/executor_subscribable_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.GenericMessage` | `support/GenericMessage.java` | `support/generic_message.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.HeaderMapper` | `support/HeaderMapper.java` | `support/header_mapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.IdTimestampMessageHeaderInitializer` | `support/IdTimestampMessageHeaderInitializer.java` | `support/id_timestamp_message_header_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.ImmutableMessageChannelInterceptor` | `support/ImmutableMessageChannelInterceptor.java` | `support/immutable_message_channel_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.InterceptableChannel` | `support/InterceptableChannel.java` | `support/interceptable_channel.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.MessageBuilder` | `support/MessageBuilder.java` | `support/message_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.MessageHandlingRunnable` | `support/MessageHandlingRunnable.java` | `support/message_handling_runnable.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.MessageHeaderAccessor` | `support/MessageHeaderAccessor.java` | `support/message_header_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.MessageHeaderInitializer` | `support/MessageHeaderInitializer.java` | `support/message_header_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.support.NativeMessageHeaderAccessor` | `support/NativeMessageHeaderAccessor.java` | `support/native_message_header_accessor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.FixedIntervalReconnectStrategy` | `tcp/FixedIntervalReconnectStrategy.java` | `tcp/fixed_interval_reconnect_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.ReconnectStrategy` | `tcp/ReconnectStrategy.java` | `tcp/reconnect_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.TcpConnection` | `tcp/TcpConnection.java` | `tcp/tcp_connection.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.TcpConnectionHandler` | `tcp/TcpConnectionHandler.java` | `tcp/tcp_connection_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.TcpOperations` | `tcp/TcpOperations.java` | `tcp/tcp_operations.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.reactor.AbstractNioBufferReactorNettyCodec` | `tcp/reactor/AbstractNioBufferReactorNettyCodec.java` | `tcp/reactor/abstract_nio_buffer_reactor_netty_codec.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.reactor.ReactorNettyCodec` | `tcp/reactor/ReactorNettyCodec.java` | `tcp/reactor/reactor_netty_codec.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.reactor.ReactorNettyTcpClient` | `tcp/reactor/ReactorNettyTcpClient.java` | `tcp/reactor/reactor_netty_tcp_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.reactor.ReactorNettyTcpConnection` | `tcp/reactor/ReactorNettyTcpConnection.java` | `tcp/reactor/reactor_netty_tcp_connection.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.messaging.tcp.reactor.TcpMessageCodec` | `tcp/reactor/TcpMessageCodec.java` | `tcp/reactor/tcp_message_codec.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

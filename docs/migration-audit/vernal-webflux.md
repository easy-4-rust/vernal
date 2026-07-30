<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-webflux 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 252 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 252 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 0 |
| `MISSING` | 252 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

- 未发现 `lib.rs`/`mod.rs` 类型定义或生产 wildcard import。

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.web.reactive.BindingContext` | `web/reactive/BindingContext.java` | `web/reactive/binding_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.DispatchExceptionHandler` | `web/reactive/DispatchExceptionHandler.java` | `web/reactive/dispatch_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.DispatcherHandler` | `web/reactive/DispatcherHandler.java` | `web/reactive/dispatcher_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.HandlerAdapter` | `web/reactive/HandlerAdapter.java` | `web/reactive/handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.HandlerMapping` | `web/reactive/HandlerMapping.java` | `web/reactive/handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.HandlerResult` | `web/reactive/HandlerResult.java` | `web/reactive/handler_result.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.HandlerResultHandler` | `web/reactive/HandlerResultHandler.java` | `web/reactive/handler_result_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.ApiVersionDeprecationHandler` | `web/reactive/accept/ApiVersionDeprecationHandler.java` | `reactive/accept/api_version_deprecation_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.ApiVersionResolver` | `web/reactive/accept/ApiVersionResolver.java` | `reactive/accept/api_version_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.ApiVersionStrategy` | `web/reactive/accept/ApiVersionStrategy.java` | `reactive/accept/api_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.AsyncApiVersionResolver` | `web/reactive/accept/AsyncApiVersionResolver.java` | `reactive/accept/async_api_version_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.DefaultApiVersionStrategy` | `web/reactive/accept/DefaultApiVersionStrategy.java` | `reactive/accept/default_api_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.FixedContentTypeResolver` | `web/reactive/accept/FixedContentTypeResolver.java` | `reactive/accept/fixed_content_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.HeaderApiVersionResolver` | `web/reactive/accept/HeaderApiVersionResolver.java` | `reactive/accept/header_api_version_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.HeaderContentTypeResolver` | `web/reactive/accept/HeaderContentTypeResolver.java` | `reactive/accept/header_content_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.MediaTypeParamApiVersionResolver` | `web/reactive/accept/MediaTypeParamApiVersionResolver.java` | `reactive/accept/media_type_param_api_version_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.ParameterContentTypeResolver` | `web/reactive/accept/ParameterContentTypeResolver.java` | `reactive/accept/parameter_content_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.PathApiVersionResolver` | `web/reactive/accept/PathApiVersionResolver.java` | `reactive/accept/path_api_version_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.QueryApiVersionResolver` | `web/reactive/accept/QueryApiVersionResolver.java` | `reactive/accept/query_api_version_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.RequestedContentTypeResolver` | `web/reactive/accept/RequestedContentTypeResolver.java` | `reactive/accept/requested_content_type_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.RequestedContentTypeResolverBuilder` | `web/reactive/accept/RequestedContentTypeResolverBuilder.java` | `reactive/accept/requested_content_type_resolver_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.accept.StandardApiVersionDeprecationHandler` | `web/reactive/accept/StandardApiVersionDeprecationHandler.java` | `reactive/accept/standard_api_version_deprecation_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.ApiVersionConfigurer` | `web/reactive/config/ApiVersionConfigurer.java` | `reactive/config/api_version_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.BlockingExecutionConfigurer` | `web/reactive/config/BlockingExecutionConfigurer.java` | `reactive/config/blocking_execution_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.CorsRegistration` | `web/reactive/config/CorsRegistration.java` | `reactive/config/cors_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.CorsRegistry` | `web/reactive/config/CorsRegistry.java` | `reactive/config/cors_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.DelegatingWebFluxConfiguration` | `web/reactive/config/DelegatingWebFluxConfiguration.java` | `reactive/config/delegating_web_flux_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.EnableWebFlux` | `web/reactive/config/EnableWebFlux.java` | `reactive/config/enable_web_flux.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.PathMatchConfigurer` | `web/reactive/config/PathMatchConfigurer.java` | `reactive/config/path_match_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.ResourceChainRegistration` | `web/reactive/config/ResourceChainRegistration.java` | `reactive/config/resource_chain_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.ResourceHandlerRegistration` | `web/reactive/config/ResourceHandlerRegistration.java` | `reactive/config/resource_handler_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.ResourceHandlerRegistry` | `web/reactive/config/ResourceHandlerRegistry.java` | `reactive/config/resource_handler_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.UrlBasedViewResolverRegistration` | `web/reactive/config/UrlBasedViewResolverRegistration.java` | `reactive/config/url_based_view_resolver_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.ViewResolverRegistry` | `web/reactive/config/ViewResolverRegistry.java` | `reactive/config/view_resolver_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.WebFluxConfigurationSupport` | `web/reactive/config/WebFluxConfigurationSupport.java` | `reactive/config/web_flux_configuration_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.WebFluxConfigurer` | `web/reactive/config/WebFluxConfigurer.java` | `reactive/config/web_flux_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.config.WebFluxConfigurerComposite` | `web/reactive/config/WebFluxConfigurerComposite.java` | `reactive/config/web_flux_configurer_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.BodyExtractor` | `web/reactive/function/BodyExtractor.java` | `reactive/function/body_extractor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.BodyExtractors` | `web/reactive/function/BodyExtractors.java` | `reactive/function/body_extractors.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.BodyInserter` | `web/reactive/function/BodyInserter.java` | `reactive/function/body_inserter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.BodyInserters` | `web/reactive/function/BodyInserters.java` | `reactive/function/body_inserters.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.UnsupportedMediaTypeException` | `web/reactive/function/UnsupportedMediaTypeException.java` | `reactive/function/unsupported_media_type_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ClientHttpObservationDocumentation` | `web/reactive/function/client/ClientHttpObservationDocumentation.java` | `function/client/client_http_observation_documentation.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ClientRequest` | `web/reactive/function/client/ClientRequest.java` | `function/client/client_request.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ClientRequestObservationContext` | `web/reactive/function/client/ClientRequestObservationContext.java` | `function/client/client_request_observation_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ClientRequestObservationConvention` | `web/reactive/function/client/ClientRequestObservationConvention.java` | `function/client/client_request_observation_convention.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ClientResponse` | `web/reactive/function/client/ClientResponse.java` | `function/client/client_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultClientRequestBuilder` | `web/reactive/function/client/DefaultClientRequestBuilder.java` | `function/client/default_client_request_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultClientRequestObservationConvention` | `web/reactive/function/client/DefaultClientRequestObservationConvention.java` | `function/client/default_client_request_observation_convention.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultClientResponse` | `web/reactive/function/client/DefaultClientResponse.java` | `function/client/default_client_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultClientResponseBuilder` | `web/reactive/function/client/DefaultClientResponseBuilder.java` | `function/client/default_client_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultExchangeStrategiesBuilder` | `web/reactive/function/client/DefaultExchangeStrategiesBuilder.java` | `function/client/default_exchange_strategies_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultWebClient` | `web/reactive/function/client/DefaultWebClient.java` | `function/client/default_web_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.DefaultWebClientBuilder` | `web/reactive/function/client/DefaultWebClientBuilder.java` | `function/client/default_web_client_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ExchangeFilterFunction` | `web/reactive/function/client/ExchangeFilterFunction.java` | `function/client/exchange_filter_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ExchangeFilterFunctions` | `web/reactive/function/client/ExchangeFilterFunctions.java` | `function/client/exchange_filter_functions.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ExchangeFunction` | `web/reactive/function/client/ExchangeFunction.java` | `function/client/exchange_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ExchangeFunctions` | `web/reactive/function/client/ExchangeFunctions.java` | `function/client/exchange_functions.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.ExchangeStrategies` | `web/reactive/function/client/ExchangeStrategies.java` | `function/client/exchange_strategies.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.UnknownHttpStatusCodeException` | `web/reactive/function/client/UnknownHttpStatusCodeException.java` | `function/client/unknown_http_status_code_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.WebClient` | `web/reactive/function/client/WebClient.java` | `function/client/web_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.WebClientException` | `web/reactive/function/client/WebClientException.java` | `function/client/web_client_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.WebClientRequestException` | `web/reactive/function/client/WebClientRequestException.java` | `function/client/web_client_request_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.WebClientResponseException` | `web/reactive/function/client/WebClientResponseException.java` | `function/client/web_client_response_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.WebClientUtils` | `web/reactive/function/client/WebClientUtils.java` | `function/client/web_client_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.support.ClientResponseWrapper` | `web/reactive/function/client/support/ClientResponseWrapper.java` | `client/support/client_response_wrapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.support.NotFoundWebClientAdapterDecorator` | `web/reactive/function/client/support/NotFoundWebClientAdapterDecorator.java` | `client/support/not_found_web_client_adapter_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.support.WebClientAdapter` | `web/reactive/function/client/support/WebClientAdapter.java` | `client/support/web_client_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.support.WebClientHttpServiceGroupAdapter` | `web/reactive/function/client/support/WebClientHttpServiceGroupAdapter.java` | `client/support/web_client_http_service_group_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.client.support.WebClientHttpServiceGroupConfigurer` | `web/reactive/function/client/support/WebClientHttpServiceGroupConfigurer.java` | `client/support/web_client_http_service_group_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.ChangePathPatternParserVisitor` | `web/reactive/function/server/ChangePathPatternParserVisitor.java` | `function/server/change_path_pattern_parser_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.DefaultEntityResponseBuilder` | `web/reactive/function/server/DefaultEntityResponseBuilder.java` | `function/server/default_entity_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.DefaultHandlerStrategiesBuilder` | `web/reactive/function/server/DefaultHandlerStrategiesBuilder.java` | `function/server/default_handler_strategies_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.DefaultRenderingResponseBuilder` | `web/reactive/function/server/DefaultRenderingResponseBuilder.java` | `function/server/default_rendering_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.DefaultServerRequest` | `web/reactive/function/server/DefaultServerRequest.java` | `function/server/default_server_request.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.DefaultServerRequestBuilder` | `web/reactive/function/server/DefaultServerRequestBuilder.java` | `function/server/default_server_request_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.DefaultServerResponseBuilder` | `web/reactive/function/server/DefaultServerResponseBuilder.java` | `function/server/default_server_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.EntityResponse` | `web/reactive/function/server/EntityResponse.java` | `function/server/entity_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.HandlerFilterFunction` | `web/reactive/function/server/HandlerFilterFunction.java` | `function/server/handler_filter_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.HandlerFunction` | `web/reactive/function/server/HandlerFunction.java` | `function/server/handler_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.HandlerStrategies` | `web/reactive/function/server/HandlerStrategies.java` | `function/server/handler_strategies.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.PathResourceLookupFunction` | `web/reactive/function/server/PathResourceLookupFunction.java` | `function/server/path_resource_lookup_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.PredicateResourceLookupFunction` | `web/reactive/function/server/PredicateResourceLookupFunction.java` | `function/server/predicate_resource_lookup_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.RenderingResponse` | `web/reactive/function/server/RenderingResponse.java` | `function/server/rendering_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.RequestPredicate` | `web/reactive/function/server/RequestPredicate.java` | `function/server/request_predicate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.RequestPredicates` | `web/reactive/function/server/RequestPredicates.java` | `function/server/request_predicates.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.ResourceHandlerFunction` | `web/reactive/function/server/ResourceHandlerFunction.java` | `function/server/resource_handler_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.RouterFunction` | `web/reactive/function/server/RouterFunction.java` | `function/server/router_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.RouterFunctionBuilder` | `web/reactive/function/server/RouterFunctionBuilder.java` | `function/server/router_function_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.RouterFunctions` | `web/reactive/function/server/RouterFunctions.java` | `function/server/router_functions.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.ServerRequest` | `web/reactive/function/server/ServerRequest.java` | `function/server/server_request.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.ServerResponse` | `web/reactive/function/server/ServerResponse.java` | `function/server/server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.ToStringVisitor` | `web/reactive/function/server/ToStringVisitor.java` | `function/server/to_string_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.support.HandlerFunctionAdapter` | `web/reactive/function/server/support/HandlerFunctionAdapter.java` | `server/support/handler_function_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.support.RouterFunctionMapping` | `web/reactive/function/server/support/RouterFunctionMapping.java` | `server/support/router_function_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.support.ServerRequestWrapper` | `web/reactive/function/server/support/ServerRequestWrapper.java` | `server/support/server_request_wrapper.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.support.ServerResponseResultHandler` | `web/reactive/function/server/support/ServerResponseResultHandler.java` | `server/support/server_response_result_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.function.server.support.SupportedVersionVisitor` | `web/reactive/function/server/support/SupportedVersionVisitor.java` | `server/support/supported_version_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.handler.AbstractHandlerMapping` | `web/reactive/handler/AbstractHandlerMapping.java` | `reactive/handler/abstract_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.handler.AbstractUrlHandlerMapping` | `web/reactive/handler/AbstractUrlHandlerMapping.java` | `reactive/handler/abstract_url_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.handler.SimpleUrlHandlerMapping` | `web/reactive/handler/SimpleUrlHandlerMapping.java` | `reactive/handler/simple_url_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.handler.WebFluxResponseStatusExceptionHandler` | `web/reactive/handler/WebFluxResponseStatusExceptionHandler.java` | `reactive/handler/web_flux_response_status_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.AbstractFileNameVersionStrategy` | `web/reactive/resource/AbstractFileNameVersionStrategy.java` | `reactive/resource/abstract_file_name_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.AbstractPrefixVersionStrategy` | `web/reactive/resource/AbstractPrefixVersionStrategy.java` | `reactive/resource/abstract_prefix_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.AbstractResourceResolver` | `web/reactive/resource/AbstractResourceResolver.java` | `reactive/resource/abstract_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.CachingResourceResolver` | `web/reactive/resource/CachingResourceResolver.java` | `reactive/resource/caching_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.CachingResourceTransformer` | `web/reactive/resource/CachingResourceTransformer.java` | `reactive/resource/caching_resource_transformer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ContentVersionStrategy` | `web/reactive/resource/ContentVersionStrategy.java` | `reactive/resource/content_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.CssLinkResourceTransformer` | `web/reactive/resource/CssLinkResourceTransformer.java` | `reactive/resource/css_link_resource_transformer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.DefaultResourceResolverChain` | `web/reactive/resource/DefaultResourceResolverChain.java` | `reactive/resource/default_resource_resolver_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.DefaultResourceTransformerChain` | `web/reactive/resource/DefaultResourceTransformerChain.java` | `reactive/resource/default_resource_transformer_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.EncodedResourceResolver` | `web/reactive/resource/EncodedResourceResolver.java` | `reactive/resource/encoded_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.FixedVersionStrategy` | `web/reactive/resource/FixedVersionStrategy.java` | `reactive/resource/fixed_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.HttpResource` | `web/reactive/resource/HttpResource.java` | `reactive/resource/http_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.LiteWebJarsResourceResolver` | `web/reactive/resource/LiteWebJarsResourceResolver.java` | `reactive/resource/lite_web_jars_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.NoResourceFoundException` | `web/reactive/resource/NoResourceFoundException.java` | `reactive/resource/no_resource_found_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.PathResourceResolver` | `web/reactive/resource/PathResourceResolver.java` | `reactive/resource/path_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceHandlerUtils` | `web/reactive/resource/ResourceHandlerUtils.java` | `reactive/resource/resource_handler_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceResolver` | `web/reactive/resource/ResourceResolver.java` | `reactive/resource/resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceResolverChain` | `web/reactive/resource/ResourceResolverChain.java` | `reactive/resource/resource_resolver_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceTransformer` | `web/reactive/resource/ResourceTransformer.java` | `reactive/resource/resource_transformer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceTransformerChain` | `web/reactive/resource/ResourceTransformerChain.java` | `reactive/resource/resource_transformer_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceTransformerSupport` | `web/reactive/resource/ResourceTransformerSupport.java` | `reactive/resource/resource_transformer_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceUrlProvider` | `web/reactive/resource/ResourceUrlProvider.java` | `reactive/resource/resource_url_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.ResourceWebHandler` | `web/reactive/resource/ResourceWebHandler.java` | `reactive/resource/resource_web_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.TransformedResource` | `web/reactive/resource/TransformedResource.java` | `reactive/resource/transformed_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.VersionResourceResolver` | `web/reactive/resource/VersionResourceResolver.java` | `reactive/resource/version_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.resource.VersionStrategy` | `web/reactive/resource/VersionStrategy.java` | `reactive/resource/version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.ExtendedWebExchangeDataBinder` | `web/reactive/result/ExtendedWebExchangeDataBinder.java` | `reactive/result/extended_web_exchange_data_binder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.HandlerResultHandlerSupport` | `web/reactive/result/HandlerResultHandlerSupport.java` | `reactive/result/handler_result_handler_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.SimpleHandlerAdapter` | `web/reactive/result/SimpleHandlerAdapter.java` | `reactive/result/simple_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.AbstractMediaTypeExpression` | `web/reactive/result/condition/AbstractMediaTypeExpression.java` | `result/condition/abstract_media_type_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.AbstractNameValueExpression` | `web/reactive/result/condition/AbstractNameValueExpression.java` | `result/condition/abstract_name_value_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.AbstractRequestCondition` | `web/reactive/result/condition/AbstractRequestCondition.java` | `result/condition/abstract_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.CompositeRequestCondition` | `web/reactive/result/condition/CompositeRequestCondition.java` | `result/condition/composite_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.ConsumesRequestCondition` | `web/reactive/result/condition/ConsumesRequestCondition.java` | `result/condition/consumes_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.HeadersRequestCondition` | `web/reactive/result/condition/HeadersRequestCondition.java` | `result/condition/headers_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.MediaTypeExpression` | `web/reactive/result/condition/MediaTypeExpression.java` | `result/condition/media_type_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.NameValueExpression` | `web/reactive/result/condition/NameValueExpression.java` | `result/condition/name_value_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.ParamsRequestCondition` | `web/reactive/result/condition/ParamsRequestCondition.java` | `result/condition/params_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.PatternsRequestCondition` | `web/reactive/result/condition/PatternsRequestCondition.java` | `result/condition/patterns_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.ProducesRequestCondition` | `web/reactive/result/condition/ProducesRequestCondition.java` | `result/condition/produces_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.RequestCondition` | `web/reactive/result/condition/RequestCondition.java` | `result/condition/request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.RequestConditionHolder` | `web/reactive/result/condition/RequestConditionHolder.java` | `result/condition/request_condition_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.RequestMethodsRequestCondition` | `web/reactive/result/condition/RequestMethodsRequestCondition.java` | `result/condition/request_methods_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.condition.VersionRequestCondition` | `web/reactive/result/condition/VersionRequestCondition.java` | `result/condition/version_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.AbstractHandlerMethodMapping` | `web/reactive/result/method/AbstractHandlerMethodMapping.java` | `result/method/abstract_handler_method_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.HandlerMethodArgumentResolver` | `web/reactive/result/method/HandlerMethodArgumentResolver.java` | `result/method/handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.HandlerMethodArgumentResolverComposite` | `web/reactive/result/method/HandlerMethodArgumentResolverComposite.java` | `result/method/handler_method_argument_resolver_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.HandlerMethodArgumentResolverSupport` | `web/reactive/result/method/HandlerMethodArgumentResolverSupport.java` | `result/method/handler_method_argument_resolver_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.InvocableHandlerMethod` | `web/reactive/result/method/InvocableHandlerMethod.java` | `result/method/invocable_handler_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.RequestMappingInfo` | `web/reactive/result/method/RequestMappingInfo.java` | `result/method/request_mapping_info.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.RequestMappingInfoHandlerMapping` | `web/reactive/result/method/RequestMappingInfoHandlerMapping.java` | `result/method/request_mapping_info_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.SyncHandlerMethodArgumentResolver` | `web/reactive/result/method/SyncHandlerMethodArgumentResolver.java` | `result/method/sync_handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.SyncInvocableHandlerMethod` | `web/reactive/result/method/SyncInvocableHandlerMethod.java` | `result/method/sync_invocable_handler_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.AbstractMessageReaderArgumentResolver` | `web/reactive/result/method/annotation/AbstractMessageReaderArgumentResolver.java` | `method/annotation/abstract_message_reader_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.AbstractMessageWriterResultHandler` | `web/reactive/result/method/annotation/AbstractMessageWriterResultHandler.java` | `method/annotation/abstract_message_writer_result_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.AbstractNamedValueArgumentResolver` | `web/reactive/result/method/annotation/AbstractNamedValueArgumentResolver.java` | `method/annotation/abstract_named_value_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.AbstractNamedValueSyncArgumentResolver` | `web/reactive/result/method/annotation/AbstractNamedValueSyncArgumentResolver.java` | `method/annotation/abstract_named_value_sync_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ApiVersionMethodArgumentResolver` | `web/reactive/result/method/annotation/ApiVersionMethodArgumentResolver.java` | `method/annotation/api_version_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ArgumentResolverConfigurer` | `web/reactive/result/method/annotation/ArgumentResolverConfigurer.java` | `method/annotation/argument_resolver_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ContinuationHandlerMethodArgumentResolver` | `web/reactive/result/method/annotation/ContinuationHandlerMethodArgumentResolver.java` | `method/annotation/continuation_handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ControllerMethodResolver` | `web/reactive/result/method/annotation/ControllerMethodResolver.java` | `method/annotation/controller_method_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.CookieValueMethodArgumentResolver` | `web/reactive/result/method/annotation/CookieValueMethodArgumentResolver.java` | `method/annotation/cookie_value_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ErrorsMethodArgumentResolver` | `web/reactive/result/method/annotation/ErrorsMethodArgumentResolver.java` | `method/annotation/errors_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ExpressionValueMethodArgumentResolver` | `web/reactive/result/method/annotation/ExpressionValueMethodArgumentResolver.java` | `method/annotation/expression_value_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ExtendedWebExchangeDataBinder` | `web/reactive/result/method/annotation/ExtendedWebExchangeDataBinder.java` | `method/annotation/extended_web_exchange_data_binder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.HttpEntityMethodArgumentResolver` | `web/reactive/result/method/annotation/HttpEntityMethodArgumentResolver.java` | `method/annotation/http_entity_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.InitBinderBindingContext` | `web/reactive/result/method/annotation/InitBinderBindingContext.java` | `method/annotation/init_binder_binding_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.MatrixVariableMapMethodArgumentResolver` | `web/reactive/result/method/annotation/MatrixVariableMapMethodArgumentResolver.java` | `method/annotation/matrix_variable_map_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.MatrixVariableMethodArgumentResolver` | `web/reactive/result/method/annotation/MatrixVariableMethodArgumentResolver.java` | `method/annotation/matrix_variable_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ModelAttributeMethodArgumentResolver` | `web/reactive/result/method/annotation/ModelAttributeMethodArgumentResolver.java` | `method/annotation/model_attribute_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ModelInitializer` | `web/reactive/result/method/annotation/ModelInitializer.java` | `method/annotation/model_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ModelMethodArgumentResolver` | `web/reactive/result/method/annotation/ModelMethodArgumentResolver.java` | `method/annotation/model_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.PathVariableMapMethodArgumentResolver` | `web/reactive/result/method/annotation/PathVariableMapMethodArgumentResolver.java` | `method/annotation/path_variable_map_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.PathVariableMethodArgumentResolver` | `web/reactive/result/method/annotation/PathVariableMethodArgumentResolver.java` | `method/annotation/path_variable_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.PrincipalMethodArgumentResolver` | `web/reactive/result/method/annotation/PrincipalMethodArgumentResolver.java` | `method/annotation/principal_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestAttributeMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestAttributeMethodArgumentResolver.java` | `method/annotation/request_attribute_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestBodyMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestBodyMethodArgumentResolver.java` | `method/annotation/request_body_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestHeaderMapMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestHeaderMapMethodArgumentResolver.java` | `method/annotation/request_header_map_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestHeaderMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestHeaderMethodArgumentResolver.java` | `method/annotation/request_header_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestMappingHandlerAdapter` | `web/reactive/result/method/annotation/RequestMappingHandlerAdapter.java` | `method/annotation/request_mapping_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestMappingHandlerMapping` | `web/reactive/result/method/annotation/RequestMappingHandlerMapping.java` | `method/annotation/request_mapping_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestParamMapMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestParamMapMethodArgumentResolver.java` | `method/annotation/request_param_map_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestParamMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestParamMethodArgumentResolver.java` | `method/annotation/request_param_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.RequestPartMethodArgumentResolver` | `web/reactive/result/method/annotation/RequestPartMethodArgumentResolver.java` | `method/annotation/request_part_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ResponseBodyResultHandler` | `web/reactive/result/method/annotation/ResponseBodyResultHandler.java` | `method/annotation/response_body_result_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ResponseEntityExceptionHandler` | `web/reactive/result/method/annotation/ResponseEntityExceptionHandler.java` | `method/annotation/response_entity_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ResponseEntityResultHandler` | `web/reactive/result/method/annotation/ResponseEntityResultHandler.java` | `method/annotation/response_entity_result_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.ServerWebExchangeMethodArgumentResolver` | `web/reactive/result/method/annotation/ServerWebExchangeMethodArgumentResolver.java` | `method/annotation/server_web_exchange_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.SessionAttributeMethodArgumentResolver` | `web/reactive/result/method/annotation/SessionAttributeMethodArgumentResolver.java` | `method/annotation/session_attribute_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.SessionAttributesHandler` | `web/reactive/result/method/annotation/SessionAttributesHandler.java` | `method/annotation/session_attributes_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.SessionStatusMethodArgumentResolver` | `web/reactive/result/method/annotation/SessionStatusMethodArgumentResolver.java` | `method/annotation/session_status_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.method.annotation.WebSessionMethodArgumentResolver` | `web/reactive/result/method/annotation/WebSessionMethodArgumentResolver.java` | `method/annotation/web_session_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.AbstractUrlBasedView` | `web/reactive/result/view/AbstractUrlBasedView.java` | `result/view/abstract_url_based_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.AbstractView` | `web/reactive/result/view/AbstractView.java` | `result/view/abstract_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.BindStatus` | `web/reactive/result/view/BindStatus.java` | `result/view/bind_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.DefaultFragmentsRenderingBuilder` | `web/reactive/result/view/DefaultFragmentsRenderingBuilder.java` | `result/view/default_fragments_rendering_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.DefaultRendering` | `web/reactive/result/view/DefaultRendering.java` | `result/view/default_rendering.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.DefaultRenderingBuilder` | `web/reactive/result/view/DefaultRenderingBuilder.java` | `result/view/default_rendering_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.Fragment` | `web/reactive/result/view/Fragment.java` | `result/view/fragment.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.FragmentsRendering` | `web/reactive/result/view/FragmentsRendering.java` | `result/view/fragments_rendering.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.HttpMessageWriterView` | `web/reactive/result/view/HttpMessageWriterView.java` | `result/view/http_message_writer_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.RedirectView` | `web/reactive/result/view/RedirectView.java` | `result/view/redirect_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.Rendering` | `web/reactive/result/view/Rendering.java` | `result/view/rendering.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.RequestContext` | `web/reactive/result/view/RequestContext.java` | `result/view/request_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.RequestDataValueProcessor` | `web/reactive/result/view/RequestDataValueProcessor.java` | `result/view/request_data_value_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.UrlBasedViewResolver` | `web/reactive/result/view/UrlBasedViewResolver.java` | `result/view/url_based_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.View` | `web/reactive/result/view/View.java` | `result/view/view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.ViewResolutionResultHandler` | `web/reactive/result/view/ViewResolutionResultHandler.java` | `result/view/view_resolution_result_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.ViewResolver` | `web/reactive/result/view/ViewResolver.java` | `result/view/view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.ViewResolverSupport` | `web/reactive/result/view/ViewResolverSupport.java` | `result/view/view_resolver_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.freemarker.FreeMarkerConfig` | `web/reactive/result/view/freemarker/FreeMarkerConfig.java` | `view/freemarker/free_marker_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.freemarker.FreeMarkerConfigurer` | `web/reactive/result/view/freemarker/FreeMarkerConfigurer.java` | `view/freemarker/free_marker_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.freemarker.FreeMarkerView` | `web/reactive/result/view/freemarker/FreeMarkerView.java` | `view/freemarker/free_marker_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.freemarker.FreeMarkerViewResolver` | `web/reactive/result/view/freemarker/FreeMarkerViewResolver.java` | `view/freemarker/free_marker_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.script.RenderingContext` | `web/reactive/result/view/script/RenderingContext.java` | `view/script/rendering_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.script.ScriptTemplateConfig` | `web/reactive/result/view/script/ScriptTemplateConfig.java` | `view/script/script_template_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.script.ScriptTemplateConfigurer` | `web/reactive/result/view/script/ScriptTemplateConfigurer.java` | `view/script/script_template_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.script.ScriptTemplateView` | `web/reactive/result/view/script/ScriptTemplateView.java` | `view/script/script_template_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.result.view.script.ScriptTemplateViewResolver` | `web/reactive/result/view/script/ScriptTemplateViewResolver.java` | `view/script/script_template_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.CloseStatus` | `web/reactive/socket/CloseStatus.java` | `reactive/socket/close_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.HandshakeInfo` | `web/reactive/socket/HandshakeInfo.java` | `reactive/socket/handshake_info.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.WebSocketHandler` | `web/reactive/socket/WebSocketHandler.java` | `reactive/socket/web_socket_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.WebSocketMessage` | `web/reactive/socket/WebSocketMessage.java` | `reactive/socket/web_socket_message.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.WebSocketSession` | `web/reactive/socket/WebSocketSession.java` | `reactive/socket/web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.AbstractListenerWebSocketSession` | `web/reactive/socket/adapter/AbstractListenerWebSocketSession.java` | `socket/adapter/abstract_listener_web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.AbstractWebSocketSession` | `web/reactive/socket/adapter/AbstractWebSocketSession.java` | `socket/adapter/abstract_web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.ContextWebSocketHandler` | `web/reactive/socket/adapter/ContextWebSocketHandler.java` | `socket/adapter/context_web_socket_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.JettyWebSocketHandlerAdapter` | `web/reactive/socket/adapter/JettyWebSocketHandlerAdapter.java` | `socket/adapter/jetty_web_socket_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.JettyWebSocketSession` | `web/reactive/socket/adapter/JettyWebSocketSession.java` | `socket/adapter/jetty_web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.NettyWebSocketSessionSupport` | `web/reactive/socket/adapter/NettyWebSocketSessionSupport.java` | `socket/adapter/netty_web_socket_session_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.ReactorNettyWebSocketSession` | `web/reactive/socket/adapter/ReactorNettyWebSocketSession.java` | `socket/adapter/reactor_netty_web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.StandardWebSocketHandlerAdapter` | `web/reactive/socket/adapter/StandardWebSocketHandlerAdapter.java` | `socket/adapter/standard_web_socket_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.StandardWebSocketSession` | `web/reactive/socket/adapter/StandardWebSocketSession.java` | `socket/adapter/standard_web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.adapter.TomcatWebSocketSession` | `web/reactive/socket/adapter/TomcatWebSocketSession.java` | `socket/adapter/tomcat_web_socket_session.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.client.JettyWebSocketClient` | `web/reactive/socket/client/JettyWebSocketClient.java` | `socket/client/jetty_web_socket_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.client.ReactorNettyWebSocketClient` | `web/reactive/socket/client/ReactorNettyWebSocketClient.java` | `socket/client/reactor_netty_web_socket_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.client.StandardWebSocketClient` | `web/reactive/socket/client/StandardWebSocketClient.java` | `socket/client/standard_web_socket_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.client.TomcatWebSocketClient` | `web/reactive/socket/client/TomcatWebSocketClient.java` | `socket/client/tomcat_web_socket_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.client.WebSocketClient` | `web/reactive/socket/client/WebSocketClient.java` | `socket/client/web_socket_client.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.RequestUpgradeStrategy` | `web/reactive/socket/server/RequestUpgradeStrategy.java` | `socket/server/request_upgrade_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.WebSocketService` | `web/reactive/socket/server/WebSocketService.java` | `socket/server/web_socket_service.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.support.HandshakeWebSocketService` | `web/reactive/socket/server/support/HandshakeWebSocketService.java` | `server/support/handshake_web_socket_service.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.support.HandshakeWebSocketServiceRuntimeHints` | `web/reactive/socket/server/support/HandshakeWebSocketServiceRuntimeHints.java` | `server/support/handshake_web_socket_service_runtime_hints.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.support.WebSocketHandlerAdapter` | `web/reactive/socket/server/support/WebSocketHandlerAdapter.java` | `server/support/web_socket_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.support.WebSocketUpgradeHandlerPredicate` | `web/reactive/socket/server/support/WebSocketUpgradeHandlerPredicate.java` | `server/support/web_socket_upgrade_handler_predicate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.upgrade.DefaultServerEndpointConfig` | `web/reactive/socket/server/upgrade/DefaultServerEndpointConfig.java` | `server/upgrade/default_server_endpoint_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.upgrade.JettyCoreRequestUpgradeStrategy` | `web/reactive/socket/server/upgrade/JettyCoreRequestUpgradeStrategy.java` | `server/upgrade/jetty_core_request_upgrade_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.upgrade.JettyRequestUpgradeStrategy` | `web/reactive/socket/server/upgrade/JettyRequestUpgradeStrategy.java` | `server/upgrade/jetty_request_upgrade_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.upgrade.ReactorNettyRequestUpgradeStrategy` | `web/reactive/socket/server/upgrade/ReactorNettyRequestUpgradeStrategy.java` | `server/upgrade/reactor_netty_request_upgrade_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.reactive.socket.server.upgrade.StandardWebSocketUpgradeStrategy` | `web/reactive/socket/server/upgrade/StandardWebSocketUpgradeStrategy.java` | `server/upgrade/standard_web_socket_upgrade_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

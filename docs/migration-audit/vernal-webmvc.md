<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-webmvc 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 337 个 class/interface/enum/record；`package-info.java` 不计入 |
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
| Java 业务对象 | 337 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 0 |
| `MISSING` | 337 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

- 未发现 `lib.rs`/`mod.rs` 类型定义或生产 wildcard import。

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.web.servlet.AsyncHandlerInterceptor` | `web/servlet/AsyncHandlerInterceptor.java` | `web/servlet/async_handler_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.DispatcherServlet` | `web/servlet/DispatcherServlet.java` | `web/servlet/dispatcher_servlet.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.FlashMap` | `web/servlet/FlashMap.java` | `web/servlet/flash_map.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.FlashMapManager` | `web/servlet/FlashMapManager.java` | `web/servlet/flash_map_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.FrameworkServlet` | `web/servlet/FrameworkServlet.java` | `web/servlet/framework_servlet.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.HandlerAdapter` | `web/servlet/HandlerAdapter.java` | `web/servlet/handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.HandlerExceptionResolver` | `web/servlet/HandlerExceptionResolver.java` | `web/servlet/handler_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.HandlerExecutionChain` | `web/servlet/HandlerExecutionChain.java` | `web/servlet/handler_execution_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.HandlerInterceptor` | `web/servlet/HandlerInterceptor.java` | `web/servlet/handler_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.HandlerMapping` | `web/servlet/HandlerMapping.java` | `web/servlet/handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.HttpServletBean` | `web/servlet/HttpServletBean.java` | `web/servlet/http_servlet_bean.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.LocaleContextResolver` | `web/servlet/LocaleContextResolver.java` | `web/servlet/locale_context_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.LocaleResolver` | `web/servlet/LocaleResolver.java` | `web/servlet/locale_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.ModelAndView` | `web/servlet/ModelAndView.java` | `web/servlet/model_and_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.ModelAndViewDefiningException` | `web/servlet/ModelAndViewDefiningException.java` | `web/servlet/model_and_view_defining_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.NoHandlerFoundException` | `web/servlet/NoHandlerFoundException.java` | `web/servlet/no_handler_found_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.RequestToViewNameTranslator` | `web/servlet/RequestToViewNameTranslator.java` | `web/servlet/request_to_view_name_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.SmartView` | `web/servlet/SmartView.java` | `web/servlet/smart_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.View` | `web/servlet/View.java` | `web/servlet/view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.ViewResolver` | `web/servlet/ViewResolver.java` | `web/servlet/view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.AnnotationDrivenBeanDefinitionParser` | `web/servlet/config/AnnotationDrivenBeanDefinitionParser.java` | `servlet/config/annotation_driven_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.CorsBeanDefinitionParser` | `web/servlet/config/CorsBeanDefinitionParser.java` | `servlet/config/cors_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.DefaultServletHandlerBeanDefinitionParser` | `web/servlet/config/DefaultServletHandlerBeanDefinitionParser.java` | `servlet/config/default_servlet_handler_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.FreeMarkerConfigurerBeanDefinitionParser` | `web/servlet/config/FreeMarkerConfigurerBeanDefinitionParser.java` | `servlet/config/free_marker_configurer_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.GroovyMarkupConfigurerBeanDefinitionParser` | `web/servlet/config/GroovyMarkupConfigurerBeanDefinitionParser.java` | `servlet/config/groovy_markup_configurer_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.InterceptorsBeanDefinitionParser` | `web/servlet/config/InterceptorsBeanDefinitionParser.java` | `servlet/config/interceptors_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.MvcNamespaceHandler` | `web/servlet/config/MvcNamespaceHandler.java` | `servlet/config/mvc_namespace_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.MvcNamespaceUtils` | `web/servlet/config/MvcNamespaceUtils.java` | `servlet/config/mvc_namespace_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.ResourcesBeanDefinitionParser` | `web/servlet/config/ResourcesBeanDefinitionParser.java` | `servlet/config/resources_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.ScriptTemplateConfigurerBeanDefinitionParser` | `web/servlet/config/ScriptTemplateConfigurerBeanDefinitionParser.java` | `servlet/config/script_template_configurer_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.ViewControllerBeanDefinitionParser` | `web/servlet/config/ViewControllerBeanDefinitionParser.java` | `servlet/config/view_controller_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.ViewResolversBeanDefinitionParser` | `web/servlet/config/ViewResolversBeanDefinitionParser.java` | `servlet/config/view_resolvers_bean_definition_parser.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ApiVersionConfigurer` | `web/servlet/config/annotation/ApiVersionConfigurer.java` | `config/annotation/api_version_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.AsyncSupportConfigurer` | `web/servlet/config/annotation/AsyncSupportConfigurer.java` | `config/annotation/async_support_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ContentNegotiationConfigurer` | `web/servlet/config/annotation/ContentNegotiationConfigurer.java` | `config/annotation/content_negotiation_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.CorsRegistration` | `web/servlet/config/annotation/CorsRegistration.java` | `config/annotation/cors_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.CorsRegistry` | `web/servlet/config/annotation/CorsRegistry.java` | `config/annotation/cors_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.DefaultServletHandlerConfigurer` | `web/servlet/config/annotation/DefaultServletHandlerConfigurer.java` | `config/annotation/default_servlet_handler_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.DelegatingWebMvcConfiguration` | `web/servlet/config/annotation/DelegatingWebMvcConfiguration.java` | `config/annotation/delegating_web_mvc_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.EnableWebMvc` | `web/servlet/config/annotation/EnableWebMvc.java` | `config/annotation/enable_web_mvc.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.InterceptorRegistration` | `web/servlet/config/annotation/InterceptorRegistration.java` | `config/annotation/interceptor_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.InterceptorRegistry` | `web/servlet/config/annotation/InterceptorRegistry.java` | `config/annotation/interceptor_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.PathMatchConfigurer` | `web/servlet/config/annotation/PathMatchConfigurer.java` | `config/annotation/path_match_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.RedirectViewControllerRegistration` | `web/servlet/config/annotation/RedirectViewControllerRegistration.java` | `config/annotation/redirect_view_controller_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ResourceChainRegistration` | `web/servlet/config/annotation/ResourceChainRegistration.java` | `config/annotation/resource_chain_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ResourceHandlerRegistration` | `web/servlet/config/annotation/ResourceHandlerRegistration.java` | `config/annotation/resource_handler_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ResourceHandlerRegistry` | `web/servlet/config/annotation/ResourceHandlerRegistry.java` | `config/annotation/resource_handler_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.UrlBasedViewResolverRegistration` | `web/servlet/config/annotation/UrlBasedViewResolverRegistration.java` | `config/annotation/url_based_view_resolver_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ViewControllerRegistration` | `web/servlet/config/annotation/ViewControllerRegistration.java` | `config/annotation/view_controller_registration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ViewControllerRegistry` | `web/servlet/config/annotation/ViewControllerRegistry.java` | `config/annotation/view_controller_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.ViewResolverRegistry` | `web/servlet/config/annotation/ViewResolverRegistry.java` | `config/annotation/view_resolver_registry.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.WebMvcConfigurationSupport` | `web/servlet/config/annotation/WebMvcConfigurationSupport.java` | `config/annotation/web_mvc_configuration_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.WebMvcConfigurer` | `web/servlet/config/annotation/WebMvcConfigurer.java` | `config/annotation/web_mvc_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.config.annotation.WebMvcConfigurerComposite` | `web/servlet/config/annotation/WebMvcConfigurerComposite.java` | `config/annotation/web_mvc_configurer_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.AbstractServerResponse` | `web/servlet/function/AbstractServerResponse.java` | `servlet/function/abstract_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.AsyncServerResponse` | `web/servlet/function/AsyncServerResponse.java` | `servlet/function/async_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.ChangePathPatternParserVisitor` | `web/servlet/function/ChangePathPatternParserVisitor.java` | `servlet/function/change_path_pattern_parser_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.CompletedAsyncServerResponse` | `web/servlet/function/CompletedAsyncServerResponse.java` | `servlet/function/completed_async_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.DefaultAsyncServerResponse` | `web/servlet/function/DefaultAsyncServerResponse.java` | `servlet/function/default_async_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.DefaultEntityResponseBuilder` | `web/servlet/function/DefaultEntityResponseBuilder.java` | `servlet/function/default_entity_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.DefaultRenderingResponseBuilder` | `web/servlet/function/DefaultRenderingResponseBuilder.java` | `servlet/function/default_rendering_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.DefaultServerRequest` | `web/servlet/function/DefaultServerRequest.java` | `servlet/function/default_server_request.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.DefaultServerRequestBuilder` | `web/servlet/function/DefaultServerRequestBuilder.java` | `servlet/function/default_server_request_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.DefaultServerResponseBuilder` | `web/servlet/function/DefaultServerResponseBuilder.java` | `servlet/function/default_server_response_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.EntityResponse` | `web/servlet/function/EntityResponse.java` | `servlet/function/entity_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.ErrorHandlingServerResponse` | `web/servlet/function/ErrorHandlingServerResponse.java` | `servlet/function/error_handling_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.HandlerFilterFunction` | `web/servlet/function/HandlerFilterFunction.java` | `servlet/function/handler_filter_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.HandlerFunction` | `web/servlet/function/HandlerFunction.java` | `servlet/function/handler_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.PathResourceLookupFunction` | `web/servlet/function/PathResourceLookupFunction.java` | `servlet/function/path_resource_lookup_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.PredicateResourceLookupFunction` | `web/servlet/function/PredicateResourceLookupFunction.java` | `servlet/function/predicate_resource_lookup_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.RenderingResponse` | `web/servlet/function/RenderingResponse.java` | `servlet/function/rendering_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.RequestPredicate` | `web/servlet/function/RequestPredicate.java` | `servlet/function/request_predicate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.RequestPredicates` | `web/servlet/function/RequestPredicates.java` | `servlet/function/request_predicates.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.ResourceHandlerFunction` | `web/servlet/function/ResourceHandlerFunction.java` | `servlet/function/resource_handler_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.RouterFunction` | `web/servlet/function/RouterFunction.java` | `servlet/function/router_function.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.RouterFunctionBuilder` | `web/servlet/function/RouterFunctionBuilder.java` | `servlet/function/router_function_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.RouterFunctions` | `web/servlet/function/RouterFunctions.java` | `servlet/function/router_functions.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.ServerRequest` | `web/servlet/function/ServerRequest.java` | `servlet/function/server_request.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.ServerResponse` | `web/servlet/function/ServerResponse.java` | `servlet/function/server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.SseServerResponse` | `web/servlet/function/SseServerResponse.java` | `servlet/function/sse_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.StreamingServerResponse` | `web/servlet/function/StreamingServerResponse.java` | `servlet/function/streaming_server_response.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.ToStringVisitor` | `web/servlet/function/ToStringVisitor.java` | `servlet/function/to_string_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.support.HandlerFunctionAdapter` | `web/servlet/function/support/HandlerFunctionAdapter.java` | `function/support/handler_function_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.support.RouterFunctionMapping` | `web/servlet/function/support/RouterFunctionMapping.java` | `function/support/router_function_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.function.support.SupportedVersionVisitor` | `web/servlet/function/support/SupportedVersionVisitor.java` | `function/support/supported_version_visitor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.AbstractDetectingUrlHandlerMapping` | `web/servlet/handler/AbstractDetectingUrlHandlerMapping.java` | `servlet/handler/abstract_detecting_url_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.AbstractHandlerExceptionResolver` | `web/servlet/handler/AbstractHandlerExceptionResolver.java` | `servlet/handler/abstract_handler_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.AbstractHandlerMapping` | `web/servlet/handler/AbstractHandlerMapping.java` | `servlet/handler/abstract_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.AbstractHandlerMethodExceptionResolver` | `web/servlet/handler/AbstractHandlerMethodExceptionResolver.java` | `servlet/handler/abstract_handler_method_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.AbstractHandlerMethodMapping` | `web/servlet/handler/AbstractHandlerMethodMapping.java` | `servlet/handler/abstract_handler_method_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.AbstractUrlHandlerMapping` | `web/servlet/handler/AbstractUrlHandlerMapping.java` | `servlet/handler/abstract_url_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.BeanNameUrlHandlerMapping` | `web/servlet/handler/BeanNameUrlHandlerMapping.java` | `servlet/handler/bean_name_url_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.ConversionServiceExposingInterceptor` | `web/servlet/handler/ConversionServiceExposingInterceptor.java` | `servlet/handler/conversion_service_exposing_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.DispatcherServletWebRequest` | `web/servlet/handler/DispatcherServletWebRequest.java` | `servlet/handler/dispatcher_servlet_web_request.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.HandlerExceptionResolverComposite` | `web/servlet/handler/HandlerExceptionResolverComposite.java` | `servlet/handler/handler_exception_resolver_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.HandlerMappingIntrospector` | `web/servlet/handler/HandlerMappingIntrospector.java` | `servlet/handler/handler_mapping_introspector.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.HandlerMethodMappingNamingStrategy` | `web/servlet/handler/HandlerMethodMappingNamingStrategy.java` | `servlet/handler/handler_method_mapping_naming_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.MappedInterceptor` | `web/servlet/handler/MappedInterceptor.java` | `servlet/handler/mapped_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.MatchableHandlerMapping` | `web/servlet/handler/MatchableHandlerMapping.java` | `servlet/handler/matchable_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.PathPatternMatchableHandlerMapping` | `web/servlet/handler/PathPatternMatchableHandlerMapping.java` | `servlet/handler/path_pattern_matchable_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.RequestMatchResult` | `web/servlet/handler/RequestMatchResult.java` | `servlet/handler/request_match_result.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.SimpleMappingExceptionResolver` | `web/servlet/handler/SimpleMappingExceptionResolver.java` | `servlet/handler/simple_mapping_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.SimpleServletHandlerAdapter` | `web/servlet/handler/SimpleServletHandlerAdapter.java` | `servlet/handler/simple_servlet_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.SimpleServletPostProcessor` | `web/servlet/handler/SimpleServletPostProcessor.java` | `servlet/handler/simple_servlet_post_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.SimpleUrlHandlerMapping` | `web/servlet/handler/SimpleUrlHandlerMapping.java` | `servlet/handler/simple_url_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.UserRoleAuthorizationInterceptor` | `web/servlet/handler/UserRoleAuthorizationInterceptor.java` | `servlet/handler/user_role_authorization_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.handler.WebRequestHandlerInterceptorAdapter` | `web/servlet/handler/WebRequestHandlerInterceptorAdapter.java` | `servlet/handler/web_request_handler_interceptor_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.AbstractLocaleContextResolver` | `web/servlet/i18n/AbstractLocaleContextResolver.java` | `servlet/i18n/abstract_locale_context_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.AbstractLocaleResolver` | `web/servlet/i18n/AbstractLocaleResolver.java` | `servlet/i18n/abstract_locale_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.AcceptHeaderLocaleResolver` | `web/servlet/i18n/AcceptHeaderLocaleResolver.java` | `servlet/i18n/accept_header_locale_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.CookieLocaleResolver` | `web/servlet/i18n/CookieLocaleResolver.java` | `servlet/i18n/cookie_locale_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.FixedLocaleResolver` | `web/servlet/i18n/FixedLocaleResolver.java` | `servlet/i18n/fixed_locale_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.LocaleChangeInterceptor` | `web/servlet/i18n/LocaleChangeInterceptor.java` | `servlet/i18n/locale_change_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.i18n.SessionLocaleResolver` | `web/servlet/i18n/SessionLocaleResolver.java` | `servlet/i18n/session_locale_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.AbstractController` | `web/servlet/mvc/AbstractController.java` | `servlet/mvc/abstract_controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.AbstractUrlViewController` | `web/servlet/mvc/AbstractUrlViewController.java` | `servlet/mvc/abstract_url_view_controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.Controller` | `web/servlet/mvc/Controller.java` | `servlet/mvc/controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.HttpRequestHandlerAdapter` | `web/servlet/mvc/HttpRequestHandlerAdapter.java` | `servlet/mvc/http_request_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.ParameterizableViewController` | `web/servlet/mvc/ParameterizableViewController.java` | `servlet/mvc/parameterizable_view_controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.ServletForwardingController` | `web/servlet/mvc/ServletForwardingController.java` | `servlet/mvc/servlet_forwarding_controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.ServletWrappingController` | `web/servlet/mvc/ServletWrappingController.java` | `servlet/mvc/servlet_wrapping_controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.SimpleControllerHandlerAdapter` | `web/servlet/mvc/SimpleControllerHandlerAdapter.java` | `servlet/mvc/simple_controller_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.UrlFilenameViewController` | `web/servlet/mvc/UrlFilenameViewController.java` | `servlet/mvc/url_filename_view_controller.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.WebContentInterceptor` | `web/servlet/mvc/WebContentInterceptor.java` | `servlet/mvc/web_content_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.annotation.ModelAndViewResolver` | `web/servlet/mvc/annotation/ModelAndViewResolver.java` | `mvc/annotation/model_and_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.annotation.ResponseStatusExceptionResolver` | `web/servlet/mvc/annotation/ResponseStatusExceptionResolver.java` | `mvc/annotation/response_status_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.AbstractMediaTypeExpression` | `web/servlet/mvc/condition/AbstractMediaTypeExpression.java` | `mvc/condition/abstract_media_type_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.AbstractNameValueExpression` | `web/servlet/mvc/condition/AbstractNameValueExpression.java` | `mvc/condition/abstract_name_value_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.AbstractRequestCondition` | `web/servlet/mvc/condition/AbstractRequestCondition.java` | `mvc/condition/abstract_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.CompositeRequestCondition` | `web/servlet/mvc/condition/CompositeRequestCondition.java` | `mvc/condition/composite_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.ConsumesRequestCondition` | `web/servlet/mvc/condition/ConsumesRequestCondition.java` | `mvc/condition/consumes_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.HeadersRequestCondition` | `web/servlet/mvc/condition/HeadersRequestCondition.java` | `mvc/condition/headers_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.MediaTypeExpression` | `web/servlet/mvc/condition/MediaTypeExpression.java` | `mvc/condition/media_type_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.NameValueExpression` | `web/servlet/mvc/condition/NameValueExpression.java` | `mvc/condition/name_value_expression.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.ParamsRequestCondition` | `web/servlet/mvc/condition/ParamsRequestCondition.java` | `mvc/condition/params_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.PathPatternsRequestCondition` | `web/servlet/mvc/condition/PathPatternsRequestCondition.java` | `mvc/condition/path_patterns_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.PatternsRequestCondition` | `web/servlet/mvc/condition/PatternsRequestCondition.java` | `mvc/condition/patterns_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.ProducesRequestCondition` | `web/servlet/mvc/condition/ProducesRequestCondition.java` | `mvc/condition/produces_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.RequestCondition` | `web/servlet/mvc/condition/RequestCondition.java` | `mvc/condition/request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.RequestConditionHolder` | `web/servlet/mvc/condition/RequestConditionHolder.java` | `mvc/condition/request_condition_holder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.RequestMethodsRequestCondition` | `web/servlet/mvc/condition/RequestMethodsRequestCondition.java` | `mvc/condition/request_methods_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.condition.VersionRequestCondition` | `web/servlet/mvc/condition/VersionRequestCondition.java` | `mvc/condition/version_request_condition.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.AbstractHandlerMethodAdapter` | `web/servlet/mvc/method/AbstractHandlerMethodAdapter.java` | `mvc/method/abstract_handler_method_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.RequestMappingInfo` | `web/servlet/mvc/method/RequestMappingInfo.java` | `mvc/method/request_mapping_info.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.RequestMappingInfoHandlerMapping` | `web/servlet/mvc/method/RequestMappingInfoHandlerMapping.java` | `mvc/method/request_mapping_info_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.RequestMappingInfoHandlerMethodMappingNamingStrategy` | `web/servlet/mvc/method/RequestMappingInfoHandlerMethodMappingNamingStrategy.java` | `mvc/method/request_mapping_info_handler_method_mapping_naming_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.AbstractMappingJacksonResponseBodyAdvice` | `web/servlet/mvc/method/annotation/AbstractMappingJacksonResponseBodyAdvice.java` | `method/annotation/abstract_mapping_jackson_response_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.AbstractMessageConverterMethodArgumentResolver` | `web/servlet/mvc/method/annotation/AbstractMessageConverterMethodArgumentResolver.java` | `method/annotation/abstract_message_converter_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.AbstractMessageConverterMethodProcessor` | `web/servlet/mvc/method/annotation/AbstractMessageConverterMethodProcessor.java` | `method/annotation/abstract_message_converter_method_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ApiVersionMethodArgumentResolver` | `web/servlet/mvc/method/annotation/ApiVersionMethodArgumentResolver.java` | `method/annotation/api_version_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.AsyncTaskMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/AsyncTaskMethodReturnValueHandler.java` | `method/annotation/async_task_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.CallableMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/CallableMethodReturnValueHandler.java` | `method/annotation/callable_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ContextClassRequestBodyAdvice` | `web/servlet/mvc/method/annotation/ContextClassRequestBodyAdvice.java` | `method/annotation/context_class_request_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ContinuationHandlerMethodArgumentResolver` | `web/servlet/mvc/method/annotation/ContinuationHandlerMethodArgumentResolver.java` | `method/annotation/continuation_handler_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.DeferredResultMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/DeferredResultMethodReturnValueHandler.java` | `method/annotation/deferred_result_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ExceptionHandlerExceptionResolver` | `web/servlet/mvc/method/annotation/ExceptionHandlerExceptionResolver.java` | `method/annotation/exception_handler_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ExtendedServletRequestDataBinder` | `web/servlet/mvc/method/annotation/ExtendedServletRequestDataBinder.java` | `method/annotation/extended_servlet_request_data_binder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.HttpEntityMethodProcessor` | `web/servlet/mvc/method/annotation/HttpEntityMethodProcessor.java` | `method/annotation/http_entity_method_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.HttpHeadersReturnValueHandler` | `web/servlet/mvc/method/annotation/HttpHeadersReturnValueHandler.java` | `method/annotation/http_headers_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.JsonViewRequestBodyAdvice` | `web/servlet/mvc/method/annotation/JsonViewRequestBodyAdvice.java` | `method/annotation/json_view_request_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.JsonViewResponseBodyAdvice` | `web/servlet/mvc/method/annotation/JsonViewResponseBodyAdvice.java` | `method/annotation/json_view_response_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.KotlinRequestBodyAdvice` | `web/servlet/mvc/method/annotation/KotlinRequestBodyAdvice.java` | `method/annotation/kotlin_request_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.KotlinResponseBodyAdvice` | `web/servlet/mvc/method/annotation/KotlinResponseBodyAdvice.java` | `method/annotation/kotlin_response_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.MatrixVariableMapMethodArgumentResolver` | `web/servlet/mvc/method/annotation/MatrixVariableMapMethodArgumentResolver.java` | `method/annotation/matrix_variable_map_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.MatrixVariableMethodArgumentResolver` | `web/servlet/mvc/method/annotation/MatrixVariableMethodArgumentResolver.java` | `method/annotation/matrix_variable_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ModelAndViewMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/ModelAndViewMethodReturnValueHandler.java` | `method/annotation/model_and_view_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ModelAndViewResolverMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/ModelAndViewResolverMethodReturnValueHandler.java` | `method/annotation/model_and_view_resolver_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.MvcUriComponentsBuilder` | `web/servlet/mvc/method/annotation/MvcUriComponentsBuilder.java` | `method/annotation/mvc_uri_components_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.PathVariableMapMethodArgumentResolver` | `web/servlet/mvc/method/annotation/PathVariableMapMethodArgumentResolver.java` | `method/annotation/path_variable_map_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.PathVariableMethodArgumentResolver` | `web/servlet/mvc/method/annotation/PathVariableMethodArgumentResolver.java` | `method/annotation/path_variable_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.PrincipalMethodArgumentResolver` | `web/servlet/mvc/method/annotation/PrincipalMethodArgumentResolver.java` | `method/annotation/principal_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ReactiveTypeHandler` | `web/servlet/mvc/method/annotation/ReactiveTypeHandler.java` | `method/annotation/reactive_type_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RedirectAttributesMethodArgumentResolver` | `web/servlet/mvc/method/annotation/RedirectAttributesMethodArgumentResolver.java` | `method/annotation/redirect_attributes_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestAttributeMethodArgumentResolver` | `web/servlet/mvc/method/annotation/RequestAttributeMethodArgumentResolver.java` | `method/annotation/request_attribute_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestBodyAdvice` | `web/servlet/mvc/method/annotation/RequestBodyAdvice.java` | `method/annotation/request_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestBodyAdviceAdapter` | `web/servlet/mvc/method/annotation/RequestBodyAdviceAdapter.java` | `method/annotation/request_body_advice_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestMappingHandlerAdapter` | `web/servlet/mvc/method/annotation/RequestMappingHandlerAdapter.java` | `method/annotation/request_mapping_handler_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestMappingHandlerMapping` | `web/servlet/mvc/method/annotation/RequestMappingHandlerMapping.java` | `method/annotation/request_mapping_handler_mapping.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestPartMethodArgumentResolver` | `web/servlet/mvc/method/annotation/RequestPartMethodArgumentResolver.java` | `method/annotation/request_part_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestResponseBodyAdviceChain` | `web/servlet/mvc/method/annotation/RequestResponseBodyAdviceChain.java` | `method/annotation/request_response_body_advice_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.RequestResponseBodyMethodProcessor` | `web/servlet/mvc/method/annotation/RequestResponseBodyMethodProcessor.java` | `method/annotation/request_response_body_method_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ResponseBodyAdvice` | `web/servlet/mvc/method/annotation/ResponseBodyAdvice.java` | `method/annotation/response_body_advice.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ResponseBodyEmitter` | `web/servlet/mvc/method/annotation/ResponseBodyEmitter.java` | `method/annotation/response_body_emitter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ResponseBodyEmitterReturnValueHandler` | `web/servlet/mvc/method/annotation/ResponseBodyEmitterReturnValueHandler.java` | `method/annotation/response_body_emitter_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ResponseEntityExceptionHandler` | `web/servlet/mvc/method/annotation/ResponseEntityExceptionHandler.java` | `method/annotation/response_entity_exception_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ResponseEntityReturnValueHandler` | `web/servlet/mvc/method/annotation/ResponseEntityReturnValueHandler.java` | `method/annotation/response_entity_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletCookieValueMethodArgumentResolver` | `web/servlet/mvc/method/annotation/ServletCookieValueMethodArgumentResolver.java` | `method/annotation/servlet_cookie_value_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletInvocableHandlerMethod` | `web/servlet/mvc/method/annotation/ServletInvocableHandlerMethod.java` | `method/annotation/servlet_invocable_handler_method.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletModelAttributeMethodProcessor` | `web/servlet/mvc/method/annotation/ServletModelAttributeMethodProcessor.java` | `method/annotation/servlet_model_attribute_method_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletRequestDataBinderFactory` | `web/servlet/mvc/method/annotation/ServletRequestDataBinderFactory.java` | `method/annotation/servlet_request_data_binder_factory.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletRequestMethodArgumentResolver` | `web/servlet/mvc/method/annotation/ServletRequestMethodArgumentResolver.java` | `method/annotation/servlet_request_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletResponseMethodArgumentResolver` | `web/servlet/mvc/method/annotation/ServletResponseMethodArgumentResolver.java` | `method/annotation/servlet_response_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ServletWebArgumentResolverAdapter` | `web/servlet/mvc/method/annotation/ServletWebArgumentResolverAdapter.java` | `method/annotation/servlet_web_argument_resolver_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.SessionAttributeMethodArgumentResolver` | `web/servlet/mvc/method/annotation/SessionAttributeMethodArgumentResolver.java` | `method/annotation/session_attribute_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.SseEmitter` | `web/servlet/mvc/method/annotation/SseEmitter.java` | `method/annotation/sse_emitter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.StreamingResponseBody` | `web/servlet/mvc/method/annotation/StreamingResponseBody.java` | `method/annotation/streaming_response_body.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.StreamingResponseBodyReturnValueHandler` | `web/servlet/mvc/method/annotation/StreamingResponseBodyReturnValueHandler.java` | `method/annotation/streaming_response_body_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.UriComponentsBuilderMethodArgumentResolver` | `web/servlet/mvc/method/annotation/UriComponentsBuilderMethodArgumentResolver.java` | `method/annotation/uri_components_builder_method_argument_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ViewMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/ViewMethodReturnValueHandler.java` | `method/annotation/view_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.method.annotation.ViewNameMethodReturnValueHandler` | `web/servlet/mvc/method/annotation/ViewNameMethodReturnValueHandler.java` | `method/annotation/view_name_method_return_value_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.support.DefaultHandlerExceptionResolver` | `web/servlet/mvc/support/DefaultHandlerExceptionResolver.java` | `mvc/support/default_handler_exception_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.support.RedirectAttributes` | `web/servlet/mvc/support/RedirectAttributes.java` | `mvc/support/redirect_attributes.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.mvc.support.RedirectAttributesModelMap` | `web/servlet/mvc/support/RedirectAttributesModelMap.java` | `mvc/support/redirect_attributes_model_map.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.AbstractResourceResolver` | `web/servlet/resource/AbstractResourceResolver.java` | `servlet/resource/abstract_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.AbstractVersionStrategy` | `web/servlet/resource/AbstractVersionStrategy.java` | `servlet/resource/abstract_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.CachingResourceResolver` | `web/servlet/resource/CachingResourceResolver.java` | `servlet/resource/caching_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.CachingResourceTransformer` | `web/servlet/resource/CachingResourceTransformer.java` | `servlet/resource/caching_resource_transformer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ContentVersionStrategy` | `web/servlet/resource/ContentVersionStrategy.java` | `servlet/resource/content_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.CssLinkResourceTransformer` | `web/servlet/resource/CssLinkResourceTransformer.java` | `servlet/resource/css_link_resource_transformer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.DefaultResourceResolverChain` | `web/servlet/resource/DefaultResourceResolverChain.java` | `servlet/resource/default_resource_resolver_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.DefaultResourceTransformerChain` | `web/servlet/resource/DefaultResourceTransformerChain.java` | `servlet/resource/default_resource_transformer_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.DefaultServletHttpRequestHandler` | `web/servlet/resource/DefaultServletHttpRequestHandler.java` | `servlet/resource/default_servlet_http_request_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.EncodedResourceResolver` | `web/servlet/resource/EncodedResourceResolver.java` | `servlet/resource/encoded_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.FixedVersionStrategy` | `web/servlet/resource/FixedVersionStrategy.java` | `servlet/resource/fixed_version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.HttpResource` | `web/servlet/resource/HttpResource.java` | `servlet/resource/http_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.LiteWebJarsResourceResolver` | `web/servlet/resource/LiteWebJarsResourceResolver.java` | `servlet/resource/lite_web_jars_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.NoResourceFoundException` | `web/servlet/resource/NoResourceFoundException.java` | `servlet/resource/no_resource_found_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.PathResourceResolver` | `web/servlet/resource/PathResourceResolver.java` | `servlet/resource/path_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceHandlerUtils` | `web/servlet/resource/ResourceHandlerUtils.java` | `servlet/resource/resource_handler_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceHttpRequestHandler` | `web/servlet/resource/ResourceHttpRequestHandler.java` | `servlet/resource/resource_http_request_handler.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceResolver` | `web/servlet/resource/ResourceResolver.java` | `servlet/resource/resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceResolverChain` | `web/servlet/resource/ResourceResolverChain.java` | `servlet/resource/resource_resolver_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceTransformer` | `web/servlet/resource/ResourceTransformer.java` | `servlet/resource/resource_transformer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceTransformerChain` | `web/servlet/resource/ResourceTransformerChain.java` | `servlet/resource/resource_transformer_chain.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceTransformerSupport` | `web/servlet/resource/ResourceTransformerSupport.java` | `servlet/resource/resource_transformer_support.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceUrlEncodingFilter` | `web/servlet/resource/ResourceUrlEncodingFilter.java` | `servlet/resource/resource_url_encoding_filter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceUrlProvider` | `web/servlet/resource/ResourceUrlProvider.java` | `servlet/resource/resource_url_provider.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.ResourceUrlProviderExposingInterceptor` | `web/servlet/resource/ResourceUrlProviderExposingInterceptor.java` | `servlet/resource/resource_url_provider_exposing_interceptor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.TransformedResource` | `web/servlet/resource/TransformedResource.java` | `servlet/resource/transformed_resource.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.VersionPathStrategy` | `web/servlet/resource/VersionPathStrategy.java` | `servlet/resource/version_path_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.VersionResourceResolver` | `web/servlet/resource/VersionResourceResolver.java` | `servlet/resource/version_resource_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.resource.VersionStrategy` | `web/servlet/resource/VersionStrategy.java` | `servlet/resource/version_strategy.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.AbstractAnnotationConfigDispatcherServletInitializer` | `web/servlet/support/AbstractAnnotationConfigDispatcherServletInitializer.java` | `servlet/support/abstract_annotation_config_dispatcher_servlet_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.AbstractDispatcherServletInitializer` | `web/servlet/support/AbstractDispatcherServletInitializer.java` | `servlet/support/abstract_dispatcher_servlet_initializer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.AbstractFlashMapManager` | `web/servlet/support/AbstractFlashMapManager.java` | `servlet/support/abstract_flash_map_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.BindStatus` | `web/servlet/support/BindStatus.java` | `servlet/support/bind_status.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.ExtendedServletRequestDataBinder` | `web/servlet/support/ExtendedServletRequestDataBinder.java` | `servlet/support/extended_servlet_request_data_binder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.JspAwareRequestContext` | `web/servlet/support/JspAwareRequestContext.java` | `servlet/support/jsp_aware_request_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.JstlUtils` | `web/servlet/support/JstlUtils.java` | `servlet/support/jstl_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.RequestContext` | `web/servlet/support/RequestContext.java` | `servlet/support/request_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.RequestContextUtils` | `web/servlet/support/RequestContextUtils.java` | `servlet/support/request_context_utils.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.RequestDataValueProcessor` | `web/servlet/support/RequestDataValueProcessor.java` | `servlet/support/request_data_value_processor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.ServletUriComponentsBuilder` | `web/servlet/support/ServletUriComponentsBuilder.java` | `servlet/support/servlet_uri_components_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.SessionFlashMapManager` | `web/servlet/support/SessionFlashMapManager.java` | `servlet/support/session_flash_map_manager.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.support.WebContentGenerator` | `web/servlet/support/WebContentGenerator.java` | `servlet/support/web_content_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.ArgumentAware` | `web/servlet/tags/ArgumentAware.java` | `servlet/tags/argument_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.ArgumentTag` | `web/servlet/tags/ArgumentTag.java` | `servlet/tags/argument_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.BindErrorsTag` | `web/servlet/tags/BindErrorsTag.java` | `servlet/tags/bind_errors_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.BindTag` | `web/servlet/tags/BindTag.java` | `servlet/tags/bind_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.EditorAwareTag` | `web/servlet/tags/EditorAwareTag.java` | `servlet/tags/editor_aware_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.EscapeBodyTag` | `web/servlet/tags/EscapeBodyTag.java` | `servlet/tags/escape_body_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.EvalTag` | `web/servlet/tags/EvalTag.java` | `servlet/tags/eval_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.HtmlEscapeTag` | `web/servlet/tags/HtmlEscapeTag.java` | `servlet/tags/html_escape_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.HtmlEscapingAwareTag` | `web/servlet/tags/HtmlEscapingAwareTag.java` | `servlet/tags/html_escaping_aware_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.MessageTag` | `web/servlet/tags/MessageTag.java` | `servlet/tags/message_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.NestedPathTag` | `web/servlet/tags/NestedPathTag.java` | `servlet/tags/nested_path_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.Param` | `web/servlet/tags/Param.java` | `servlet/tags/param.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.ParamAware` | `web/servlet/tags/ParamAware.java` | `servlet/tags/param_aware.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.ParamTag` | `web/servlet/tags/ParamTag.java` | `servlet/tags/param_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.RequestContextAwareTag` | `web/servlet/tags/RequestContextAwareTag.java` | `servlet/tags/request_context_aware_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.TransformTag` | `web/servlet/tags/TransformTag.java` | `servlet/tags/transform_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.UrlTag` | `web/servlet/tags/UrlTag.java` | `servlet/tags/url_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractCheckedElementTag` | `web/servlet/tags/form/AbstractCheckedElementTag.java` | `tags/form/abstract_checked_element_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractDataBoundFormElementTag` | `web/servlet/tags/form/AbstractDataBoundFormElementTag.java` | `tags/form/abstract_data_bound_form_element_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractFormTag` | `web/servlet/tags/form/AbstractFormTag.java` | `tags/form/abstract_form_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractHtmlElementBodyTag` | `web/servlet/tags/form/AbstractHtmlElementBodyTag.java` | `tags/form/abstract_html_element_body_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractHtmlElementTag` | `web/servlet/tags/form/AbstractHtmlElementTag.java` | `tags/form/abstract_html_element_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractHtmlInputElementTag` | `web/servlet/tags/form/AbstractHtmlInputElementTag.java` | `tags/form/abstract_html_input_element_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractMultiCheckedElementTag` | `web/servlet/tags/form/AbstractMultiCheckedElementTag.java` | `tags/form/abstract_multi_checked_element_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.AbstractSingleCheckedElementTag` | `web/servlet/tags/form/AbstractSingleCheckedElementTag.java` | `tags/form/abstract_single_checked_element_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.ButtonTag` | `web/servlet/tags/form/ButtonTag.java` | `tags/form/button_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.CheckboxTag` | `web/servlet/tags/form/CheckboxTag.java` | `tags/form/checkbox_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.CheckboxesTag` | `web/servlet/tags/form/CheckboxesTag.java` | `tags/form/checkboxes_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.ErrorsTag` | `web/servlet/tags/form/ErrorsTag.java` | `tags/form/errors_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.FormTag` | `web/servlet/tags/form/FormTag.java` | `tags/form/form_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.HiddenInputTag` | `web/servlet/tags/form/HiddenInputTag.java` | `tags/form/hidden_input_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.InputTag` | `web/servlet/tags/form/InputTag.java` | `tags/form/input_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.LabelTag` | `web/servlet/tags/form/LabelTag.java` | `tags/form/label_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.OptionTag` | `web/servlet/tags/form/OptionTag.java` | `tags/form/option_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.OptionWriter` | `web/servlet/tags/form/OptionWriter.java` | `tags/form/option_writer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.OptionsTag` | `web/servlet/tags/form/OptionsTag.java` | `tags/form/options_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.PasswordInputTag` | `web/servlet/tags/form/PasswordInputTag.java` | `tags/form/password_input_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.RadioButtonTag` | `web/servlet/tags/form/RadioButtonTag.java` | `tags/form/radio_button_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.RadioButtonsTag` | `web/servlet/tags/form/RadioButtonsTag.java` | `tags/form/radio_buttons_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.SelectTag` | `web/servlet/tags/form/SelectTag.java` | `tags/form/select_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.SelectedValueComparator` | `web/servlet/tags/form/SelectedValueComparator.java` | `tags/form/selected_value_comparator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.TagIdGenerator` | `web/servlet/tags/form/TagIdGenerator.java` | `tags/form/tag_id_generator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.TagWriter` | `web/servlet/tags/form/TagWriter.java` | `tags/form/tag_writer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.TextareaTag` | `web/servlet/tags/form/TextareaTag.java` | `tags/form/textarea_tag.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.tags.form.ValueFormatter` | `web/servlet/tags/form/ValueFormatter.java` | `tags/form/value_formatter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.AbstractCachingViewResolver` | `web/servlet/view/AbstractCachingViewResolver.java` | `servlet/view/abstract_caching_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.AbstractJacksonView` | `web/servlet/view/AbstractJacksonView.java` | `servlet/view/abstract_jackson_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.AbstractTemplateView` | `web/servlet/view/AbstractTemplateView.java` | `servlet/view/abstract_template_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.AbstractTemplateViewResolver` | `web/servlet/view/AbstractTemplateViewResolver.java` | `servlet/view/abstract_template_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.AbstractUrlBasedView` | `web/servlet/view/AbstractUrlBasedView.java` | `servlet/view/abstract_url_based_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.AbstractView` | `web/servlet/view/AbstractView.java` | `servlet/view/abstract_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.BeanNameViewResolver` | `web/servlet/view/BeanNameViewResolver.java` | `servlet/view/bean_name_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.ContentNegotiatingViewResolver` | `web/servlet/view/ContentNegotiatingViewResolver.java` | `servlet/view/content_negotiating_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.DefaultFragmentsRendering` | `web/servlet/view/DefaultFragmentsRendering.java` | `servlet/view/default_fragments_rendering.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.DefaultFragmentsRenderingBuilder` | `web/servlet/view/DefaultFragmentsRenderingBuilder.java` | `servlet/view/default_fragments_rendering_builder.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.DefaultRequestToViewNameTranslator` | `web/servlet/view/DefaultRequestToViewNameTranslator.java` | `servlet/view/default_request_to_view_name_translator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.FragmentsRendering` | `web/servlet/view/FragmentsRendering.java` | `servlet/view/fragments_rendering.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.InternalResourceView` | `web/servlet/view/InternalResourceView.java` | `servlet/view/internal_resource_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.InternalResourceViewResolver` | `web/servlet/view/InternalResourceViewResolver.java` | `servlet/view/internal_resource_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.JstlView` | `web/servlet/view/JstlView.java` | `servlet/view/jstl_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.RedirectView` | `web/servlet/view/RedirectView.java` | `servlet/view/redirect_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.UrlBasedViewResolver` | `web/servlet/view/UrlBasedViewResolver.java` | `servlet/view/url_based_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.ViewResolverComposite` | `web/servlet/view/ViewResolverComposite.java` | `servlet/view/view_resolver_composite.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.document.AbstractPdfStamperView` | `web/servlet/view/document/AbstractPdfStamperView.java` | `view/document/abstract_pdf_stamper_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.document.AbstractPdfView` | `web/servlet/view/document/AbstractPdfView.java` | `view/document/abstract_pdf_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.document.AbstractXlsView` | `web/servlet/view/document/AbstractXlsView.java` | `view/document/abstract_xls_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.document.AbstractXlsxStreamingView` | `web/servlet/view/document/AbstractXlsxStreamingView.java` | `view/document/abstract_xlsx_streaming_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.document.AbstractXlsxView` | `web/servlet/view/document/AbstractXlsxView.java` | `view/document/abstract_xlsx_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.feed.AbstractAtomFeedView` | `web/servlet/view/feed/AbstractAtomFeedView.java` | `view/feed/abstract_atom_feed_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.feed.AbstractFeedView` | `web/servlet/view/feed/AbstractFeedView.java` | `view/feed/abstract_feed_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.feed.AbstractRssFeedView` | `web/servlet/view/feed/AbstractRssFeedView.java` | `view/feed/abstract_rss_feed_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.freemarker.FreeMarkerConfig` | `web/servlet/view/freemarker/FreeMarkerConfig.java` | `view/freemarker/free_marker_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.freemarker.FreeMarkerConfigurer` | `web/servlet/view/freemarker/FreeMarkerConfigurer.java` | `view/freemarker/free_marker_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.freemarker.FreeMarkerView` | `web/servlet/view/freemarker/FreeMarkerView.java` | `view/freemarker/free_marker_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.freemarker.FreeMarkerViewResolver` | `web/servlet/view/freemarker/FreeMarkerViewResolver.java` | `view/freemarker/free_marker_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.groovy.GroovyMarkupConfig` | `web/servlet/view/groovy/GroovyMarkupConfig.java` | `view/groovy/groovy_markup_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.groovy.GroovyMarkupConfigurer` | `web/servlet/view/groovy/GroovyMarkupConfigurer.java` | `view/groovy/groovy_markup_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.groovy.GroovyMarkupView` | `web/servlet/view/groovy/GroovyMarkupView.java` | `view/groovy/groovy_markup_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.groovy.GroovyMarkupViewResolver` | `web/servlet/view/groovy/GroovyMarkupViewResolver.java` | `view/groovy/groovy_markup_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.json.AbstractJackson2View` | `web/servlet/view/json/AbstractJackson2View.java` | `view/json/abstract_jackson2_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.json.JacksonJsonView` | `web/servlet/view/json/JacksonJsonView.java` | `view/json/jackson_json_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.json.MappingJackson2JsonView` | `web/servlet/view/json/MappingJackson2JsonView.java` | `view/json/mapping_jackson2_json_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.script.RenderingContext` | `web/servlet/view/script/RenderingContext.java` | `view/script/rendering_context.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.script.ScriptTemplateConfig` | `web/servlet/view/script/ScriptTemplateConfig.java` | `view/script/script_template_config.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.script.ScriptTemplateConfigurer` | `web/servlet/view/script/ScriptTemplateConfigurer.java` | `view/script/script_template_configurer.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.script.ScriptTemplateView` | `web/servlet/view/script/ScriptTemplateView.java` | `view/script/script_template_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.script.ScriptTemplateViewResolver` | `web/servlet/view/script/ScriptTemplateViewResolver.java` | `view/script/script_template_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.xml.JacksonXmlView` | `web/servlet/view/xml/JacksonXmlView.java` | `view/xml/jackson_xml_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.xml.MappingJackson2XmlView` | `web/servlet/view/xml/MappingJackson2XmlView.java` | `view/xml/mapping_jackson2_xml_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.xml.MarshallingView` | `web/servlet/view/xml/MarshallingView.java` | `view/xml/marshalling_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.xslt.XsltView` | `web/servlet/view/xslt/XsltView.java` | `view/xslt/xslt_view.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.web.servlet.view.xslt.XsltViewResolver` | `web/servlet/view/xslt/XsltViewResolver.java` | `view/xslt/xslt_view_resolver.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

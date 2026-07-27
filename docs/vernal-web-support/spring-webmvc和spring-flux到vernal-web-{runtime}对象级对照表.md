# spring-webmvc + spring-flux → vernal-web-{runtime} 对象级对照表

> 基线：Spring Framework 7.0.8 `spring-webmvc`（363 个 Java 文件）+ `spring-webflux`（275 个 Java 文件）
> 对照目标：vernal-framework 已存在的 12 个 runtime crate（actix-web、axum、gotham、hyper、ntex、poem、rocket、salvo、tide、warp、tonic、tower）。契约层是 `vernal-web` + `vernal-http` + `vernal-tower`。
> 本表回答"每个 Java 对象落在哪个 Rust 文件"。一个对象 ≤ 一个 Rust 类型；同语义对象可合并但必须标注。

状态图例：✅ 已存在；🔶 部分存在；⬜ 待补；🚫 Java 专有/不迁移；🆕 Vernal 新增/对应 Java 范畴外的概念。

## 〇、Crate 与目录基线

| Crate | 目录 | Rust 文件数 |
|---|---|---:|
| `vernal-web` | `crates/vernal-web/` | 15（含 `lib.rs`） |
| `vernal-web-testkit` | `crates/vernal-web-testkit/` | 9 |
| `vernal-http` | `crates/vernal-http/` | 7 |
| `vernal-tower` | `crates/vernal-tower/` | 22 |
| `vernal-hyper` | `crates/vernal-hyper/` | 2 |
| `vernal-actix-web` | `crates/vernal-actix-web/` | 14 |
| `vernal-axum` | `crates/vernal-axum/` | 12 |
| `vernal-gotham` | `crates/vernal-gotham/` | 13 |
| `vernal-ntex` | `crates/vernal-ntex/` | 14 |
| `vernal-poem` | `crates/vernal-poem/` | 11 |
| `vernal-rocket` | `crates/vernal-rocket/` | 14 |
| `vernal-salvo` | `crates/vernal-salvo/` | 10 |
| `vernal-tide` | `crates/vernal-tide/` | 10 |
| `vernal-warp` | `crates/vernal-warp/` | 11 |
| `vernal-tonic` | `crates/vernal-tonic/` | 9 |
| 合计 runtime + 契约 | | **173** |

## 一、Spring WebMVC 顶层（`org.springframework.web.servlet.*`）

| Java 源 | 全限定名 | 目标位置 | 类型 | 状态 |
|---|---|---|---|---|
| `DispatcherServlet` | `…web.servlet.DispatcherServlet` | `vernal-tower::dispatcher` 与各 runtime 的 `<runtime>_service.rs` | `Dispatcher` | 🔶 |
| `FrameworkServlet` | `…web.servlet.FrameworkServlet` | `vernal-http::http_request` + `http_response` | 抽象 trait | ⬜ |
| `HttpServletBean` | `…web.servlet.HttpServletBean` | — | Servlet 容器 trait | 🚫 |
| `HandlerMapping` | `…web.servlet.HandlerMapping` | `vernal-web::contract::handler_mapping` | `HandlerMapping` | ⬜ |
| `HandlerAdapter` | `…web.servlet.HandlerAdapter` | `vernal-web::contract::handler_adapter` | `HandlerAdapter` | ⬜ |
| `HandlerInterceptor` | `…web.servlet.HandlerInterceptor` | `vernal-web::contract::handler_interceptor` | `HandlerInterceptor` | ⬜ |
| `AsyncHandlerInterceptor` | `…web.servlet.AsyncHandlerInterceptor` | 同上文件，多一个 `post_*_async` 方法 | 同 | ⬜ |
| `HandlerExecutionChain` | `…web.servlet.HandlerExecutionChain` | `vernal-web::contract::handler_invocation` 内部 | `HandlerExecutionChain` | ⬜ |
| `HandlerExceptionResolver` | `…web.servlet.HandlerExceptionResolver` | `vernal-web::contract::handler_exception_handler` | `HandlerExceptionHandler` | ⬜ |
| `LocaleResolver` | `…web.servlet.LocaleResolver` | `vernal-web::contract::locale_context_resolver` | `LocaleContextResolver` | ⬜ |
| `LocaleContextResolver` | `…web.servlet.LocaleContextResolver` | 同上文件 | `LocaleContextResolver` | ⬜ |
| `FlashMap` | `…web.servlet.FlashMap` | `vernal-web::contract::flash_map_manager` | `FlashMap` | ⬜ |
| `FlashMapManager` | `…web.servlet.FlashMapManager` | 同上文件 | `FlashMapManager` | ⬜ |
| `View` | `…web.servlet.View` | `vernal-web::contract::view_resolver` | `View` trait | ⬜ |
| `SmartView` | `…web.servlet.SmartView` | 同上 | `SmartView` trait | ⬜ |
| `ViewResolver` | `…web.servlet.ViewResolver` | 同上 | `ViewResolver` trait | ⬜ |
| `RequestToViewNameTranslator` | `…web.servlet.RequestToViewNameTranslator` | 同上 | trait | ⬜ |
| `ModelAndView` | `…web.servlet.ModelAndView` | `vernal-web::contract::model_and_view` | `ModelAndView` | ⬜ |
| `NoHandlerFoundException` | `…web.servlet.NoHandlerFoundException` | 嵌入 `vernal-web::web_failure` | `WebFailure` 变体 | 🔶 |
| `ModelAndViewDefiningException` | `…web.servlet.ModelAndViewDefiningException` | 同上 | 异常 variant | ⬜ |

## 二、Spring WebMVC 子包（重点）

### 2.1 `config`（XML Bean 定义解析）

| Java 源 | 目标 | 状态 |
|---|---|---|
| `MvcNamespaceHandler` | 不迁移到 `vernal-web-{runtime}`，仅保留在 macro/DSL | 🚫/🆕 |
| `AnnotationDrivenBeanDefinitionParser` | 不迁移（XML 解析） | 🚫 |
| `MvcNamespaceUtils` | 不迁移（XML） | 🚫 |
| `InterceptorsBeanDefinitionParser` | 不迁移（XML） | 🚫 |
| `ViewResolversBeanDefinitionParser` | 不迁移（XML） | 🚫 |
| `ResourcesBeanDefinitionParser` | 不迁移（XML） | 🚫 |
| `ViewControllerBeanDefinitionParser` | 不迁移（XML） | 🚫 |
| `CorsBeanDefinitionParser` | 不迁移（XML） | 🚫 |
| `DefaultServletHandlerBeanDefinitionParser` | 不迁移（XML） | 🚫 |
| `FreeMarkerConfigurerBeanDefinitionParser` | 后续 `vernal-templating` feature | 🚫 |
| `ScriptTemplateConfigurerBeanDefinitionParser` | 同上 | 🚫 |
| `GroovyMarkupConfigurerBeanDefinitionParser` | 同上 | 🚫 |

### 2.2 `function`（Router DSL）

| Java 源 | 目标 | 状态 |
|---|---|---|
| `RouterFunction` | `vernal-web::contract::router_function` trait | ⬜ |
| `RouterFunctions` | `vernal-web::contract::router_functions` | ⬜ |
| `RequestPredicate` | `vernal-web::contract::request_predicate` | ⬜ |
| `HandlerFunction` | `vernal-web::contract::handler_function` | ⬜ |
| `RequestPredicates` | `vernal-web::contract::request_predicates` | ⬜ |
| `ServerResponse` | `vernal-web::contract::server_response` | ⬜ |
| `EntityResponseConverterFunction` | `vernal-web::contract::entity_response_converter` | ⬜ |

### 2.3 `handler`（基础 handler）

| Java 源 | 目标 | 状态 |
|---|---|---|
| `Controller` | `vernal-web::contract::controller` | ⬜ |
| `HttpRequestHandler` | `vernal-web::contract::http_request_handler` | ⬜ |
| `WebRequestHandler` | 同上 | ⬜ |
| `SimpleServletHandlerAdapter` | runtime crate 的 `<runtime>_adapter` | ⬜ |
| `HttpRequestHandlerAdapter` | 同上 | ⬜ |
| `ServletForwardingController` | runtime crate | 🆕/⬜ |
| `ServletWrappingController` | runtime crate | 🆕/⬜ |
| `SimpleControllerHandlerAdapter` | runtime crate | ⬜ |
| `WebContentInterceptor` | runtime crate 或 `vernal-web::contract` | ⬜ |
| `AbstractController` | runtime crate | ⬜ |
| `AbstractUrlViewController` | runtime crate | ⬜ |
| `UrlFilenameViewController` | runtime crate | ⬜ |
| `ParameterizableViewController` | runtime crate | ⬜ |
| `ConditionalDelegatingController` | runtime crate | ⬜ |

### 2.4 `mvc.method`（基于注解）

| Java 源 | 目标 | 状态 |
|---|---|---|
| `RequestMappingInfo` | `vernal-web::contract::request_mapping_info` | ⬜ |
| `RequestMappingInfoHandlerMapping` | `vernal-web::contract::request_mapping_info_handler_mapping` | ⬜ |
| `RequestMappingInfoHandlerMethodMappingNamingStrategy` | 同上文件 | ⬜ |
| `AbstractHandlerMethodAdapter` | `vernal-web::contract::abstract_handler_method_adapter` | ⬜ |
| 各种 `RequestMapping*Adapter` | `vernal-web::contract::request_mapping_handler_adapter`（按运行时切实现） | ⬜ |
| `annotation.RequestMappingHandlerMapping` | `vernal-web::contract::request_mapping_handler_mapping` | ⬜ |
| `annotation.RequestMappingHandlerAdapter` | `vernal-web::contract::request_mapping_handler_adapter` | ⬜ |
| `annotation.RequestMapping` (annotation) | 过程宏 `#[vernal::route]`（已存在于 `vernal-macros`，待暴露） | 🔶 |
| `annotation.GetMapping/PostMapping/...` | 派生宏 | 🔶 |
| `annotation.Controller` | `#[vernal::route(controller)]` | 🔶 |
| `annotation.RestController` | `#[vernal::route(rest)]` | 🔶 |

### 2.5 `support`

| Java 源 | 目标 | 状态 |
|---|---|---|
| `AbstractDispatcherServletInitializer` | `vernal-tower::dispatcher_initializer` trait | ⬜ |
| `AbstractAnnotationConfigDispatcherServletInitializer` | 同上 trait | ⬜ |
| `WebContentGenerator` | `vernal-web::contract::web_content_generator` | ⬜ |
| `RequestContext` | `vernal-web::request_context`（升级） | 🔶 |
| `RequestContextUtils` | `vernal-web::contract::request_context_utils` | ⬜ |
| `RequestDataValueProcessor` | `vernal-web::contract::request_data_value_processor` | ⬜ |
| `SessionFlashMapManager` | `vernal-web::contract::flash_map_manager` 中 | ⬜ |
| `BindStatus` | 暂归 `vernal-templating` feature | 🆕/🚫 |
| `JspAwareRequestContext` | 同上 | 🚫 |
| `JstlUtils` | 同上 | 🚫 |
| `ExtendedServletRequestDataBinder` | `vernal-web::contract::data_binder` | ⬜ |
| `ServletUriComponentsBuilder` | `vernal-web::contract::uri_builder` | ⬜ |

### 2.6 `view`、`tags`、`resource`、`i18n`

| Java 源 | 目标 | 状态 |
|---|---|---|
| `View*`、`InternalResourceView*`、`JstlView` | `vernal-templating`（feature crate） | 🚫 |
| `FormTag`、`InputTag` | 同上 | 🚫 |
| `ResourceHttpRequestHandler` / `Resource` | `vernal-web::contract::resource_handler` | ⬜ |
| `LocaleChangeInterceptor` / `SessionLocaleResolver` | `vernal-web::contract::locale_*` | ⬜ |

## 三、Spring WebFlux 顶层（`org.springframework.web.reactive.*`）

| Java 源 | 目标 | 类型 | 状态 |
|---|---|---|---|
| `DispatcherHandler` | `vernal-tower::dispatcher_handler` | `DispatcherHandler` | ⬜ |
| `HandlerMapping` | 同 WebMVC | `HandlerMapping` | ⬜ |
| `HandlerAdapter` | 同 WebMVC | `HandlerAdapter` | ⬜ |
| `HandlerResult` | `vernal-web::contract::handler_result` | `HandlerResult` | ⬜ |
| `HandlerResultHandler` | `vernal-web::contract::handler_result_handler` | trait | ⬜ |
| `DispatchExceptionHandler` | `vernal-web::contract::dispatch_exception_handler` | trait | ⬜ |
| `BindingContext` | `vernal-web::contract::binding_context` | `BindingContext` | ⬜ |
| 各种 DispatcherHandler 内部策略 | 与 servlet 共用 `contract` trait | ⬜ |

### WebFlux 子包

| Java 源 | 子包 | 目标 | 状态 |
|---|---|---|---|
| `WebFluxConfigurer` | `config` | `vernal-web::contract::webflux_configurer` trait | ⬜ |
| `RouterFunction`（Flux 版） | `function` | 与 servlet 共用 `contract` trait | ⬜ |
| `SimpleHandlerAdapter` | `result` | `vernal-web::contract::simple_handler_adapter` | ⬜ |
| `HandlerResultHandlerSupport` | `result` | `vernal-web::contract::handler_result_handler_support` | ⬜ |
| `ViewResolutionResultHandler` | `result.view` | `vernal-web::contract::view_resolution_result_handler` | ⬜ |
| `HttpEntityResultHandler` | `result.method` | `vernal-web::contract::http_entity_result_handler` | ⬜ |
| `ServerResponseResultHandler` | `result.method` | `vernal-web::contract::server_response_result_handler` | ⬜ |
| `ResponseBodyResultHandler` | `result.method` | `vernal-web::contract::response_body_result_handler` | ⬜ |
| `ResponseEntityResultHandler` | `result.method` | 同上 | ⬜ |
| `RequestMappingHandlerAdapter`（Flux 版） | `result.method` | 同 servlet | ⬜ |
| `ExtendedWebExchangeDataBinder` | `result` | `vernal-web::contract::data_binder` | ⬜ |
| `WebFluxResponseStatusExceptionHandler` | `handler` | `vernal-web::contract::response_status_exception_handler` | ⬜ |
| `AbstractHandlerMapping`、`AbstractUrlHandlerMapping`、`SimpleUrlHandlerMapping` | `handler` | 同 servlet | ⬜ |
| `resource.ResourceHttpRequestHandler`、`resource.ResourceWebHandler` | `resource` | 同 servlet | ⬜ |
| `socket.*`（WebFlux WebSocket） | `socket` | 暂并入 `vernal-websocket` crate | 🆕 |

## 四、12 个 runtime crate 的当前对象 → Spring 来源映射

> 行模板：`<runtime>_*.rs` 文件 → 当前已存在的 Rust 类型 → Spring 来源（WebMVC + WebFlux）→ 状态 → 差异说明

### 4.1 `vernal-actix-web`（14 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 | 说明 |
|---|---|---|---|---|
| `actix_body_error.rs` | `BodyError` | `HttpMessageConversionException` | ✅ | 与 `HyperRequestBody` 解耦 |
| `actix_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ | runtime 拦截与 AOP 错误 |
| `actix_rejection.rs` | `Rejection` | `HandlerExceptionResolver` 输出 | ✅ | actix 的失败泛型 |
| `actix_scoped_body.rs` | `ScopedBody` | `HttpInputMessage` | ✅ | body 范围资源 |
| `actix_request_snapshot.rs` | `RequestSnapshot` | `WebRequestContextHolder` | ✅ | 请求上下文快照 |
| `actix_request_snapshot_error.rs` | `RequestSnapshotError` | `RequestContextException` | ✅ | 快照错误 |
| `vernal_actix_component.rs` | `VernalActixComponent` | `DispatcherServlet` 初始化 | ✅ | 把 actix 注册成 vernal component |
| `vernal_actix_context.rs` | `VernalActixContext` | `ApplicationContext` 关联 | ✅ | 与 `vernal-context` 桥接 |
| `vernal_actix_middleware.rs` | `Middleware` | `HandlerInterceptor` + `WebFilter` | ✅ | actix 中间件 trait |
| `vernal_actix_request_context.rs` | `ActixRequestContext` | `HttpServletRequest` 抽象 | ✅ | actix→vernal 请求 |
| `vernal_actix_request_scope.rs` | `ActixRequestScope` | `RequestContextHolder` | ✅ | 请求作用域 |
| `vernal_actix_service.rs` | `Service` | `DispatcherServlet.doDispatch` | ✅ | 派发契约 |
| `lib.rs` | module | `FrameworkServlet` | ✅ | runtime 门面 |

### 4.2 `vernal-axum`（12 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 | 说明 |
|---|---|---|---|---|
| `axum_request_scope.rs` | `RequestScope` | `RequestContextHolder` | ✅ | 请求级状态 |
| `axum_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ | 错误归一 |
| `axum_aop_error_mapper.rs` | `IntoResponse for AopError` | `ResponseEntityExceptionHandler` | ✅ | 把错误映射成 axum 响应 |
| `axum_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ | 拒绝 |
| `axum_route_resolver.rs` | `RouteResolver` | `HandlerMapping` | ✅ | axum 路由解析 |
| `vernal_request_context.rs` | `AxumRequestContext` | `HttpServletRequest` | ✅ | 请求上下文 |
| `vernal_request_scope.rs` | `RequestScopeOwner` | `RequestContextHolder` | ✅ | 拥有者语义 |
| `vernal_router_ext.rs` | `RouterExt` | `MvcEndpoint` 注册 | ✅ | `Router` 扩展 |
| `vernal_component.rs` | `VernalAxumComponent` | `DispatcherServlet` | ✅ | vernal 注册 |
| `vernal_context.rs` | `VernalAxumContext` | `ApplicationContext` | ✅ | vernal 桥接 |
| `lib.rs` | module | `FrameworkServlet` | ✅ | 门面 |

### 4.3 `vernal-gotham`（13 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `gotham_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `gotham_body_error.rs` | `BodyError` | `HttpMessageConversionException` | ✅ |
| `gotham_borrowed_target.rs` | `BorrowedTarget` | `StreamingHttpOutputMessage.Body` | ✅ |
| `gotham_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `gotham_response.rs` | `Response` | `HttpOutputMessage` | ✅ |
| `gotham_scoped_body.rs` | `ScopedBody` | `HttpInputMessage` | ✅ |
| `gotham_upstream_error.rs` | `UpstreamError` | `WebExceptionHandler` | ✅ |
| `vernal_gotham_context.rs` | `VernalGothamContext` | `ApplicationContext` | ✅ |
| `vernal_gotham_state_ext.rs` | `StateExt` | `RequestContextUtils` | ✅ |
| `vernal_gotham_middleware.rs` | `Middleware` | `WebFilter` | ✅ |
| `vernal_gotham_request_context.rs` | `GothamRequestContext` | `ServerHttpRequest` | ✅ |
| `vernal_gotham_request_scope.rs` | `RequestScope` | `RequestContextHolder` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.4 `vernal-hyper`（2 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `hyper_bridge.rs` | `HyperBridge` | `HttpHandler` 抽象 | ✅ |
| `lib.rs` | module | `ServerHttp*` | ✅ |

> hyper 是 transport 基底；在 `vernal-tower` 之上面向 axum/tonic 复用。

### 4.5 `vernal-ntex`（14 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `ntex_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `ntex_body_error.rs` | `BodyError` | `HttpMessageConversionException` | ✅ |
| `ntex_borrowed_target.rs` | `BorrowedTarget` | `StreamingHttpOutputMessage.Body` | ✅ |
| `ntex_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `ntex_request_snapshot.rs` | `RequestSnapshot` | `RequestContextHolder` | ✅ |
| `ntex_response_envelope.rs` | `ResponseEnvelope` | `HttpEntity` | ✅ |
| `ntex_scoped_body.rs` | `ScopedBody` | `HttpInputMessage` | ✅ |
| `ntex_service_error.rs` | `ServiceError` | `HandlerExecutionException` | ✅ |
| `vernal_ntex_component.rs` | `VernalNtexComponent` | `DispatcherServlet` | ✅ |
| `vernal_ntex_context.rs` | `VernalNtexContext` | `ApplicationContext` | ✅ |
| `vernal_ntex_middleware.rs` | `Middleware` | `WebFilter` | ✅ |
| `vernal_ntex_request_context.rs` | `NtexRequestContext` | `ServerHttpRequest` | ✅ |
| `vernal_ntex_request_scope.rs` | `RequestScope` | `RequestContextHolder` | ✅ |
| `vernal_ntex_service.rs` | `Service` | `DispatcherServlet.doDispatch` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.6 `vernal-poem`（11 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `poem_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `poem_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `poem_response.rs` | `Response` | `HttpOutputMessage` | ✅ |
| `poem_scoped_stream.rs` | `ScopedStream` | `ReactiveHttpInputMessage` | ✅ |
| `vernal_poem_component.rs` | `VernalPoemComponent` | `DispatcherServlet` | ✅ |
| `vernal_poem_context.rs` | `VernalPoemContext` | `ApplicationContext` | ✅ |
| `vernal_poem_endpoint.rs` | `Endpoint` | `RouterFunction`（选） | ✅ |
| `vernal_poem_middleware.rs` | `Middleware` | `WebFilter` | ✅ |
| `vernal_poem_request_context.rs` | `PoemRequestContext` | `ServerHttpRequest` | ✅ |
| `vernal_poem_request_scope.rs` | `RequestScope` | `RequestContextHolder` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.7 `vernal-rocket`（14 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `rocket_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `rocket_borrowed_target.rs` | `BorrowedTarget` | `StreamingHttpOutputMessage.Body` | ✅ |
| `rocket_outcome_marker.rs` | `OutcomeMarker` | `HandlerExecutionException` | ✅ |
| `rocket_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `rocket_request_snapshot.rs` | `RequestSnapshot` | `WebRequest` | ✅ |
| `rocket_request_state.rs` | `RequestState` | `RequestContext` | ✅ |
| `rocket_scoped_reader.rs` | `ScopedReader` | `HttpInputMessage` | ✅ |
| `vernal_rocket_component.rs` | `VernalRocketComponent` | `DispatcherServlet` | ✅ |
| `vernal_rocket_context.rs` | `VernalRocketContext` | `ApplicationContext` | ✅ |
| `vernal_rocket_fairing.rs` | `Fairing` | `WebFilter` / `HandlerInterceptor` | ✅ |
| `vernal_rocket_handler.rs` | `Handler` | `DispatcherServlet.doDispatch` | ✅ |
| `vernal_rocket_request_context.rs` | `RocketRequestContext` | `ServerHttpRequest` | ✅ |
| `vernal_rocket_request_scope.rs` | `RequestScope` | `RequestContextHolder` | ✅ |
| `vernal_rocket_routes_ext.rs` | `RoutesExt` | `RouterFunction` 注册 | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.8 `vernal-salvo`（10 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `salvo_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `salvo_borrowed_target.rs` | `BorrowedTarget` | `StreamingHttpOutputMessage.Body` | ✅ |
| `salvo_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `salvo_request_snapshot.rs` | `RequestSnapshot` | `WebRequest` | ✅ |
| `salvo_response.rs` | `Response` | `HttpOutputMessage` | ✅ |
| `salvo_route_metadata.rs` | `RouteMetadata` | `HandlerMapping` | ✅ |
| `salvo_scoped_body.rs` | `ScopedBody` | `HttpInputMessage` | ✅ |
| `vernal_salvo_depot_ext.rs` | `DepotExt` | `RequestContextUtils` | ✅ |
| `vernal_salvo_hoop.rs` | `Hoop` | `WebFilter` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.9 `vernal-tide`（10 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `tide_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `tide_body_error.rs` | `BodyError` | `HttpMessageConversionException` | ✅ |
| `tide_borrowed_target.rs` | `BorrowedTarget` | `StreamingHttpOutputMessage.Body` | ✅ |
| `tide_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `tide_request_snapshot.rs` | `RequestSnapshot` | `WebRequest` | ✅ |
| `tide_response.rs` | `Response` | `HttpOutputMessage` | ✅ |
| `tide_scoped_reader.rs` | `ScopedReader` | `HttpInputMessage` | ✅ |
| `vernal_tide_middleware.rs` | `Middleware` | `WebFilter` | ✅ |
| `vernal_tide_request_ext.rs` | `RequestExt` | `RequestContextUtils` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.10 `vernal-warp`（11 文件）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `warp_aop_error.rs` | `AopError` | `AopInvocationException` | ✅ |
| `warp_rejection.rs` | `Rejection` | `HandlerExceptionResolver` | ✅ |
| `warp_route_resolver.rs` | `RouteResolver` | `HandlerMapping` | ✅ |
| `vernal_warp_aop_layer.rs` | `AopLayer` | `HandlerInterceptor` | ✅ |
| `vernal_warp_aop_service.rs` | `AopService` | `HandlerAdapter` | ✅ |
| `vernal_warp_component.rs` | `VernalWarpComponent` | `DispatcherServlet` | ✅ |
| `vernal_warp_context.rs` | `VernalWarpContext` | `ApplicationContext` | ✅ |
| `vernal_warp_layer.rs` | `Layer` | `WebFilter` | ✅ |
| `vernal_warp_request_context.rs` | `WarpRequestContext` | `ServerHttpRequest` | ✅ |
| `vernal_warp_request_scope.rs` | `RequestScope` | `RequestContextHolder` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.11 `vernal-tower`（22 文件，作为 Service/Layer 抽象）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `missing_plan_policy.rs` | `MissingPlanPolicy` | `AopException` | ✅ |
| `aop_service.rs` | `AopService` | `AopProxyFactory` | ✅ |
| `aop_layer.rs` | `AopLayer` | `HandlerInterceptor` | ✅ |
| `aop_service_error.rs` | `AopServiceError` | `AopInvocationException` | ✅ |
| `tower_upstream_error.rs` | `UpstreamError` | `WebExceptionHandler` | ✅ |
| `tower_response.rs` | `Response` | `HttpEntity` | ✅ |
| `tower_error.rs` | `Error` | `HandlerExceptionResolver` | ✅ |
| `tower_body_error.rs` | `BodyError` | `HttpMessageConversionException` | ✅ |
| `tower_route_resolver.rs` | `RouteResolver` | `HandlerMapping` | ✅ |
| `tower_error_mapper.rs` | `ErrorMapper` | `ResponseEntityExceptionHandler` | ✅ |
| `vernal_service.rs` | `VernalService` | `DispatcherServlet.doDispatch` | ✅ |
| `vernal_layer.rs` | `VernalLayer` | `WebFilter` | ✅ |
| `scoped_body.rs` | `ScopedBody` | `HttpInputMessage` | ✅ |
| `extension_route_resolver.rs` | `ExtensionRouteResolver` | `HandlerMapping` 扩展 | ✅ |
| `request_scope_layer.rs` | `RequestScopeLayer` | `RequestContextHolder` | ✅ |
| `request_scope_service.rs` | `RequestScopeService` | 同上 | ✅ |
| `error_mapping_layer.rs` | `ErrorMappingLayer` | `HandlerExceptionResolver` | ✅ |
| `error_mapping_service.rs` | `ErrorMappingService` | 同上 | ✅ |
| `context_propagation_layer.rs` | `ContextPropagationLayer` | `ContextSnapshot` | ✅ |
| `context_propagation_service.rs` | `ContextPropagationService` | 同上 | ✅ |
| `context_propagation_error.rs` | `ContextPropagationError` | `ContextException` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

### 4.12 `vernal-tonic`（9 文件，gRPC）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `tonic_aop_layer.rs` | `AopLayer` | `HandlerInterceptor` | ✅ |
| `tonic_aop_service.rs` | `AopService` | `AopProxyFactory` | ✅ |
| `tonic_aop_error_mapper.rs` | `IntoStatus for WebFailure` | `ResponseEntityExceptionHandler` 的双栈版 | ✅ |
| `tonic_context_interceptor.rs` | `ContextInterceptor` | `HandlerInterceptor` | ✅ |
| `tonic_request_ext.rs` | `RequestExt` | `RequestContextUtils` | ✅ |
| `tonic_request_error.rs` | `RequestError` | `HttpMessageConversionException` | ✅ |
| `tonic_route_resolver.rs` | `RouteResolver` | `HandlerMapping` | ✅ |
| `tonic_status_mapper.rs` | `StatusMapper` | `ResponseEntityExceptionHandler` | ✅ |
| `lib.rs` | module | `FrameworkServlet` | ✅ |

## 五、`vernal-web` 契约层（统一抽象、跨 runtime 共享）

| Rust 文件 | 类型 | Spring 来源 | 状态 |
|---|---|---|---|
| `request_context.rs` | `RequestContext` | `RequestContext` | 🔶 |
| `route_metadata.rs` | `RouteMetadata` | `RequestMappingInfo` | 🔶 |
| `handler_invocation.rs` | `HandlerInvocation` | `HandlerExecutionChain` | 🔶 |
| `web_failure.rs` | `WebFailure` | `HandlerExceptionResolver` + `NoHandlerFoundException` | 🔶 |
| `problem_kind.rs` | `ProblemKind` | `ErrorResponse` + `ProblemDetail` 类型 | 🔶 |
| `problem_details.rs` | `ProblemDetails` | `ProblemDetail` (Spring 6.0+) | 🔶 |
| `request_id.rs` | `RequestId` | `Observation` / MDC | ✅ |
| `security_principal.rs` | `SecurityPrincipal` | `Authentication` + `Principal` | ✅ |
| `web_request_scope.rs` | `WebRequestScope` | `RequestContextHolder` | ✅ |
| `web_request_scope_owner.rs` | `WebRequestScopeOwner` | 同上 | ✅ |
| `context_carrier.rs` | `ContextCarrier` | `ContextSnapshot` | ✅ |
| `transport_kind.rs` | `TransportKind` | HTTP/WebSocket | ✅ |
| `integration_descriptor.rs` | `IntegrationDescriptor` | runtime 适配描述 | ✅ |
| `integration_role.rs` | `IntegrationRole` | runtime 适配角色 | ✅ |

## 六、`vernal-web-testkit` 跨 runtime 验证基线

| Rust 文件 | 类型 | 用途 | 状态 |
|---|---|---|---|
| `web_adapter_contract.rs` | `WebAdapterContract` | 跨 runtime contract test | ✅ 存在 |
| `security_contract_interceptor.rs` | security fixture | 测试安全主体 | ✅ |
| `scope_cleanup_timeout_fixture.rs` | scope fixture | scope 清理超时 | ✅ |
| `scope_close_probe.rs` | scope fixture | scope 关闭探测 | ✅ |
| `scope_rejecting_interceptor.rs` | scope fixture | scope 拒绝拦截器 | ✅ |
| `failing_http_body.rs` / `failing_byte_stream.rs` / `failing_tokio_reader.rs` / `failing_futures_reader.rs` | 失败 fixture | body/stream 错误传播 | ✅ |
| `lib.rs` | module | 入口 | ✅ |
| `tests/security_contract.rs` | integration tests | 安全合同 | ✅ |
| `tests/web_adapter_contract.rs` | integration tests | adapter contract | ✅ |

## 七、对象拆分与命名规则（不变，沿用 vernal-web 4 份文档的规则）

1. 每个公开对象一个 Rust 文件；marker trait 与只读视图例外。
2. `<runtime>` 前缀只在 runtime crate 内部使用；契约 trait 放在 `vernal-web::contract` 且不带前缀。
3. 每加一个新 runtime，按 `<runtime>_<concept>.rs` 命名并补 `tests/<runtime>_adapter.rs`。
4. Java 反射/Servlet/XML 配置不进入 vernal；annotation 通过 `vernal-macros` 派生宏暴露。
5. 每个 runtime 在 `Cargo.toml` 中固定 description：`"<Runtime> integration for Vernal."`（与现有命名一致）。

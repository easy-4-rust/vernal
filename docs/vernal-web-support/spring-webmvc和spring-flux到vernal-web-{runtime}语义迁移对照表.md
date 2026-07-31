<!-- migration-doc: authority=historical canonical=语义迁移对照表.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [语义迁移对照表.md](语义迁移对照表.md) 为准。

> **历史文档，非当前验收依据。** 当前版本见[语义迁移对照表](语义迁移对照表.md)。

# spring-webmvc + spring-flux → vernal-web-{runtime} 功能语义迁移对照表

> 目标不是复制 Java 类型层级，而是保持可观察行为、生命周期、错误、扩展点和默认策略一致。
> 状态：✅ 已具备 / 🔶 部分具备 / ⬜ 待补 / 🚫 不迁移（运行时/Java 专有）。

## 一、请求生命周期（Dispatcher）

| Spring 语义 | Vernal 设计 | 状态 | 备注 |
|---|---|---|---|
| `DispatcherServlet` init/destroy | `vernal-tower::dispatcher` + 每个 runtime 的 `<runtime>_service.rs` | ⬜ | 单实例、可配置 |
| `FrameworkServlet.processRequest` | `vernal-http::http_request`/`http_response` 处理循环 | ⬜ | |
| `DispatcherServlet.doDispatch` | `vernal-web::contract::handler_invocation::invoke` | ⬜ | |
| `HandlerMapping` getHandler | `vernal-web::contract::handler_mapping::resolve` | ⬜ | |
| `HandlerAdapter.handle` | `vernal-web::contract::handler_adapter::handle` | ⬜ | |
| `HandlerInterceptor.preHandle/postHandle/afterCompletion` | `vernal-web::contract::handler_interceptor::pre/post/completion` | ⬜ | |
| `WebFilter` (Flux) | `vernal-web::contract::web_filter` | ⬜ | 与 interceptor 共存 |
| `HandlerExceptionResolver` | `vernal-web::contract::handler_exception_handler` | ⬜ | |
| `LocaleResolver`/`LocaleContextResolver` | `vernal-web::contract::locale_context_resolver` | ⬜ | |
| `FlashMap` / `FlashMapManager` | `vernal-web::contract::flash_map_manager` | ⬜ | |
| `ThemeResolver` (Spring 6.0 前) | 不进 vernal（用 cookie/middleware 替代） | 🚫 | |

## 二、WebFlux reactive 栈

| Spring Flux 语义 | Vernal 设计 | 状态 | 备注 |
|---|---|---|---|
| `DispatcherHandler.handler`(…) | `vernal-tower::dispatcher_handler` | ⬜ | |
| `HandlerResult` | `vernal-web::contract::handler_result` | ⬜ | |
| `HandlerResultHandler` | `vernal-web::contract::handler_result_handler` | ⬜ | response/body/entity/handler 各 trait |
| `BindingContext` | `vernal-web::contract::binding_context` | ⬜ | |
| `DispatchExceptionHandler` | `vernal-web::contract::dispatch_exception_handler` | ⬜ | |
| `reactor.core.publisher.Mono` | `Future<Output=T>` | 🆕/✅ | |
| `Flux` | `Stream<Item=T>` | 🆕/✅ | |
| `WebExceptionHandler` | `vernal-web::contract::web_exception_handler` | ⬜ | |
| `WebSocket*` (Flux socket) | 暂归 `vernal-websocket`，与 runtime 解耦 | 🆕/⬜ | |

## 三、Handler Mapping 与参数解析

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `@RequestMapping` / `@GetMapping`/… | `#[vernal::route]`, `#[vernal::route(get)]`/…派生宏 | 🔶 |
| `@PathVariable` | `Path<T>` extractor (`vernal-web::contract::argument_resolver`) | ⬜ |
| `@RequestParam` | `Query<T>` extractor | ⬜ |
| `@RequestHeader` | `Header<T>` extractor | ⬜ |
| `@CookieValue` | `Cookie<T>` extractor | ⬜ |
| `@RequestBody`/`@RequestPart` | `JsonBody<T>` / `Bytes` / `Form` / `Multipart` | ⬜ |
| `@ModelAttribute` (servlet) | trait 占位 `ModelAttribute<T>`（与 templating feature 隔离） | ⬜ |
| `HandlerMethodArgumentResolver` | `argument_resolver::Resolver` | ⬜ |
| `HandlerMethodReturnValueHandler` | `return_value_handler::Handler` | ⬜ |
| `@RequestMapping(method=…)` | `route(http::Method)` | ⬜ |
| `@RequestMapping(produces=…)` | `route_content(produces = [...])` | ⬜ |
| `@RequestMapping(consumes=…)` | `route_content(consumes = [...])` | ⬜ |
| `RequestCondition`（复合条件） | `request_predicate::and/or/not` | ⬜ |
| `RequestMappingInfo.matches` | `RouteMetadata::matches` | 🔶 |
| `RequestMappingInfoHandlerMapping` | `vernal-web::contract::request_mapping_handler_mapping` | ⬜ |

## 四、Handler Interceptor / Filter / AOP

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `HandlerInterceptor` | `vernal-web::contract::handler_interceptor` | ⬜ |
| `AsyncHandlerInterceptor` | 同接口 + `post_*_async` | ⬜ |
| `WebFilter` | `vernal-web::contract::web_filter` | ⬜ |
| `AspectJPointcutAdvisor` | 通过 `vernal-aop` + `vernal-aspects` + `#[intercept]` | ✅ |
| `MethodInterceptor` | `vernal-aop::Operation` + `vernal-aop::AopComponent` | ✅ |
| `ProxyFactory`/`ProxyFactoryBean` | 编译期 derive + `vernal-beans` 运行时 | ✅ |
| `WebAsyncManager` | runtime async 模型直接负责 | 🆕/⬜ |

## 五、消息读写与转换（HttpMessageConverter）

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `HttpMessageConverter<T>` | `vernal-web::contract::http_message_codec` (reader/writer) | ⬜ |
| `StringHttpMessageConverter` | `codec::text` | ⬜ |
| `ByteArrayHttpMessageConverter` | `codec::bytes` | ⬜ |
| `ResourceHttpMessageConverter` | `codec::resource` | ⬜ |
| `ResourceRegionHttpMessageConverter`（Range） | `codec::resource_region` | ⬜ |
| `AllEncompassingFormHttpMessageConverter` | `codec::form_urlencoded` | ⬜ |
| `MappingJackson2HttpMessageConverter` | `codec::json`（serde feature） | ⬜ |
| `MappingJackson2XmlHttpMessageConverter` | `codec::xml` | ⬜ |
| `Jaxb2RootElementHttpMessageConverter` | 同上 | ⬜ |
| `ProtobufHttpMessageConverter` | `codec::protobuf` (feature) | ⬜ |
| `ServerSentEventHttpMessageConverter` | `codec::sse` | ⬜ |
| `FormHttpMessageConverter` | `codec::form`（multipart） | ⬜ |
| `RssChannelHttpMessageConverter` / `AtomFeedHttpMessageConverter` | 不进核心，feature crate | 🆕/⬜ |
| `HttpRequestHandler` / `HttpRequestHandlerAdapter` | `vernal-web::contract::http_request_handler` | ⬜ |

## 六、协商 (Content Negotiation) 与 Locale

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `ContentNegotiationManager` | `vernal-web::contract::content_negotiation_manager` | ⬜ |
| `ContentNegotiationStrategy` | `ContentNegotiationStrategy` trait | ⬜ |
| `HeaderContentNegotiationStrategy` | `Header` 策略实现 | ⬜ |
| `ParameterContentNegotiationStrategy` | `Parameter` 策略 | ⬜ |
| `PathExtensionContentNegotiationStrategy` | `Path` 策略 | ⬜ |
| `MediaTypeFileExtensionResolver` | trait 占位 | ⬜ |
| `LocaleResolver` / `LocaleContextResolver` | 同上契约 | ⬜ |
| `AcceptHeaderLocaleResolver` | 实现 | ⬜ |
| `SessionLocaleResolver` | 实现 | ⬜ |
| `CookieLocaleResolver` | 实现 | ⬜ |
| `FixedLocaleResolver` | 实现 | ⬜ |

## 七、FlashMap

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `FlashMap` | `vernal-web::contract::flash_map` | ⬜ |
| `FlashMapManager` | trait | ⬜ |
| `AbstractFlashMapManager` | trait 抽象基 | ⬜ |
| `SessionFlashMapManager` | session-backed 实现 | ⬜ |

## 八、View 与模板（暂留 trait，未来 `vernal-templating` 实现）

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `View` | `View` trait | ⬜ |
| `SmartView` | `SmartView` trait | ⬜ |
| `ViewResolver` | `ViewResolver` trait | ⬜ |
| `BeanNameViewResolver` | 派生实现 | ⬜ |
| `UrlBasedViewResolver` | 派生实现 | ⬜ |
| `InternalResourceViewResolver` | 待与 templating | ⬜ |
| `ThymeleafViewResolver` | `vernal-templating` feature | 🆕/⬜ |
| `RequestToViewNameTranslator` | trait 占位 | ⬜ |
| `DefaultRequestToViewNameTranslator` | 派生实现 | ⬜ |
| JSP / JSTL / Tiles | 不进 vernal | 🚫 |

## 九、WebFlux 专属

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| `ServerWebExchange` (Flux) | `vernal-web::contract::server_web_exchange` | ⬜ |
| `ServerHttpRequest` / `ServerHttpResponse` | `vernal-http::http_request/response` + `body` | 🔶 |
| `ReactiveAdapterRegistry` | runtime 自带 async adapter | 🆕 |
| `WebSession` | vernal-runtime 提供 | ⬜ |
| `WebHandler` | `vernal-web::contract::web_handler` | ⬜ |
| `ViewResolutionResultHandler` | `ViewResolutionResultHandler` trait | ⬜ |
| `ServerResponse` (Flux function) | `vernal-web::contract::server_response` | ⬜ |
| `RouterFunction` (Flux) | trait | ⬜ |
| `ResourceWebHandler` | `vernal-web::contract::resource_handler` | ⬜ |
| `ConditionalDelegatingWebHandler` | trait | ⬜ |
| `ReactorResourceFactory` | runtime 自带 | 🆕 |
| `WebFluxSecurityConfiguration` | 委托 `vernal-aspects` 或 `vernal-async` | ⬜ |

## 十、跨 runtime 行为一致性

**vernal-web-testkit/web_adapter_contract.rs 必须验证**：

- 同一段 handler 函数（用 `vernal-macros` 标记）在 12 个 runtime 上：
  - 状态码一致（成功 / 4xx / 5xx / 业务失败）
  - Header 一致（Content-Type、Content-Length、CORS、Vary）
  - Body 一致（JSON、Text、Bytes、Stream、SSE）
  - 超时/取消 行为一致
  - 重定向/forward 一致
- 同一段 `#[intercept]`（来自 `vernal-aop`）在 12 个 runtime 上：
  - pre/post/completion 时机一致
  - 短路响应一致
- 同一段 `RequestFailure` 在 12 个 runtime 上：
  - 异常 → ProblemDetail 映射一致
  - 状态码一致
- AOP + 异常处理：
  - AOP 拦截器不能绕过异常处理
  - 异常处理不能绕过 AOP 的 afterCompletion

## 十一、错误与边界语义（保留 per-request 透明度）

| 错误 | 默认状态 |
|---|---:|
| 路由不存在 | 404 |
| 方法不允许 | 405 |
| 参数绑定失败 | 400 |
| 内容协商失败 | 406 |
| Content-Type 不支持 | 415 |
| 请求体过大 | 413 |
| handler panic/错误 | 500（异常被映射为 ProblemDetail） |
| 客户端取消 / 断开 | 不强行写响应；scope 应正确清理 |

## 十二、不迁移项

| Java 语义 | 原因 |
|---|---|
| `DispatcherServlet` 内部的多组件 bean wiring（initStrategies） | 由 `vernal-context` 替代 |
| `DispatcherServlet.properties` 体系 | 由 `vernal-config` 替代 |
| `@Controller`/`@Service` 等 stereotype annotation | 用 `vernal-beans` + `#[component]`/`#[route]` 替代 |
| `ServletConfig`/`ServletContext`/`ServletRequest`/`ServletResponse` 等 Servlet API | 不存在对应物，由 `vernal-http` + `tower::Service<Request>` 替代 |
| `WebApplicationInitializer` | 由 `vernal-config` 启动入口替代 |
| `BeanFactoryPostProcessor` | 由 `vernal-context` 替代 |
| XML Bean 定义（`config/*.BeanDefinitionParser` 全部） | 不进 vernal，使用 DSL/派生宏 |
| Spring Web 老的 `WebMvcConfigurationSupport` 内部大量方法 | 由 `vernal-config` + 派生宏替代 |
| `DefaultStrategies`（properties 文件） | 不进 vernal |
| `TilesConfigurer`、`PortletConfig` | 不进 vernal |

## 十三、设计原则（不变）

1. **可观察语义优先**：状态码、Header、Body、错误与生命周期必须稳定；Java 的类继承不是兼容目标。
2. **类型安全替代反射**：trait、泛型、derive 宏和显式 resolver 代替运行时反射。
3. **Stream 替代 Publisher**：背压、取消、结束、错误传播保持一致。
4. **核心与适配器隔离**：12 个 runtime 绝不能改变 `vernal-web` 的核心合同。
5. **默认行为可配置但有确定默认**：codec 顺序、body 限制、日志脱敏、超时、CORS 都必须显式记录。
6. **跨 runtime contract test 必须严格一致**：每个 runtime 的 contract test 跑同一份断言。
7. **gRPC 双栈异常映射**：HTTP gRPC 与 HTTP HTTP 之间通过 `vernal-tonic::StatusMapper` 双向翻译；通过 `vernal-web::WebFailure` 中间层。

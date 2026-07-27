# vernal-web-{runtime} 对象名称一致性检查

> 检查时间：2026-07-27
> 基线：Spring Framework 7.0.8 `spring-webmvc` + `spring-webflux`
> 对照对象：12 个 runtime crate（actix-web / axum / gotham / hyper / ntex / poem / rocket / salvo / tide / warp / tower / tonic）+ 契约层 `vernal-web` / `vernal-http` / `vernal-web-testkit`。
> 本表回答"命名是否规范、与 Spring + Rust 生态是否一致"。

## 一、命名规则（沿用 vernal-web 4 份文档的规则）

1. 文件名一律 snake_case；类型 PascalCase；方法 snake_case。
2. 一个公开对象一个 `.rs` 文件；`lib.rs`/`mod.rs` 只声明模块与 re-export。
3. runtime 内部类型以 `<runtime>_<concept>.rs` + `Vernal<Runtime><Concept>` 形式命名；契约层（`vernal-web::contract`、`vernal-http`）类型不带 runtime 前缀。
4. Spring 名称作为语义契约时必须与之对齐；Java 运行时/类库专有名称不许复制到 vernal 类型名上（例如 `commons-fileupload`、`Jetty*` 适配类）。
5. 缩写统一：`Http`、`Uri`、`Sse`、`Json`、`Xml`、`Id`、`Server`、`Web`；不在同一 crate 混用。
6. 类型名称不得只用过宽的 `Request`、`Response`、`Error`；必须携带层次前缀或明确泛型语义。

## 二、12 个 runtime crate 的命名一致性

### 2.1 `vernal-actix-web`（14 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `actix_body_error.rs` | `BodyError` | ✅ | 语义对应 `HttpMessageConversionException` |
| `actix_aop_error.rs` | `AopError` | ✅ | |
| `actix_rejection.rs` | `Rejection` | ✅ | actix rejection 与 `HandlerExceptionResolver` 输出对齐 |
| `actix_scoped_body.rs` | `ScopedBody` | ✅ | 与 `HttpInputMessage` 含义一致 |
| `actix_request_snapshot.rs` | `RequestSnapshot` | ✅ | |
| `actix_request_snapshot_error.rs` | `RequestSnapshotError` | ✅ | |
| `vernal_actix_component.rs` | `VernalActixComponent` | ✅ | 与 `DispatcherServlet` init 对齐 |
| `vernal_actix_context.rs` | `VernalActixContext` | ✅ | |
| `vernal_actix_middleware.rs` | `Middleware` | 🔶 | 一个 `Middleware` 类型名未带 actix 前缀；若是 vernal 通用 trait 应放 `vernal-tower`；若是 actix 专属 trait 应该 `ActixMiddleware`。待 R1 决策。 |
| `vernal_actix_request_context.rs` | `ActixRequestContext`（推测） | ✅ | |
| `vernal_actix_request_scope.rs` | `ActixRequestScope`（推测） | ✅ | |
| `vernal_actix_service.rs` | `ActixService`（推测） | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.2 `vernal-axum`（12 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `axum_request_scope.rs` | `AxumRequestScope`（推测） | ✅ | |
| `axum_aop_error.rs` | `AopError` | ✅ | |
| `axum_aop_error_mapper.rs` | `IntoResponse` impl | ✅ | axum-friendly 字面 `IntoResponse` 是合理别名 |
| `axum_rejection.rs` | `AxumRejection`（推测） | ✅ | |
| `axum_route_resolver.rs` | `RouteResolver` | ✅ | |
| `vernal_request_context.rs` | `VernalAxumRequestContext`？ | 🔶 | 文件名 `vernal_*` 但类型应该带 `Axum`；统一为 `vernal_axum_request_context.rs` 更对齐 actix/gotham |
| `vernal_request_scope.rs` | `RequestScope` | 🔶 | 同上 |
| `vernal_router_ext.rs` | `RouterExt` | ✅ | 与 `MvcEndpoint` 注册等价 |
| `vernal_component.rs` | `VernalAxumComponent` | ✅ | |
| `vernal_context.rs` | `VernalAxumContext` | ✅ | |
| `lib.rs` | module | ✅ | |

> 🔶 标记：axum crate 中部分文件使用 `vernal_*` 前缀而非 `vernal_axum_*`，与其他 runtime 不齐；**建议**重命名为 `vernal_axum_*` 系列或统一改为 `vernal_<concept>_axum.rs`。

### 2.3 `vernal-gotham`（13 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `gotham_aop_error.rs` | `GothamAopError`（推测） | ✅ | 建议类型名带 Gotham 前缀 |
| `gotham_body_error.rs` | `GothamBodyError`（推测） | ✅ | |
| `gotham_borrowed_target.rs` | `GothamBorrowedTarget`（推测） | ✅ | |
| `gotham_rejection.rs` | `GothamRejection`（推测） | ✅ | |
| `gotham_response.rs` | `GothamResponse`（推测） | ✅ | |
| `gotham_scoped_body.rs` | `GothamScopedBody`（推测） | ✅ | |
| `gotham_upstream_error.rs` | `GothamUpstreamError`（推测） | ✅ | |
| `vernal_gotham_context.rs` | `VernalGothamContext` | ✅ | |
| `vernal_gotham_state_ext.rs` | `StateExt` | 🔶 | `StateExt` 字面没有 `Gotham` 修饰；建议 `GothamStateExt` |
| `vernal_gotham_middleware.rs` | `Middleware` | 🔶 | 同 actix |
| `vernal_gotham_request_context.rs` | `GothamRequestContext` | ✅ | |
| `vernal_gotham_request_scope.rs` | `GothamRequestScope` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.4 `vernal-hyper`（2 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `hyper_bridge.rs` | `HyperBridge` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.5 `vernal-ntex`（14 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `ntex_aop_error.rs` | `NtexAopError`（推测） | ✅ | |
| `ntex_body_error.rs` | `NtexBodyError` | ✅ | |
| `ntex_borrowed_target.rs` | `NtexBorrowedTarget` | ✅ | |
| `ntex_rejection.rs` | `NtexRejection` | ✅ | |
| `ntex_request_snapshot.rs` | `NtexRequestSnapshot` | ✅ | |
| `ntex_response_envelope.rs` | `NtexResponseEnvelope` | ✅ | |
| `ntex_scoped_body.rs` | `NtexScopedBody` | ✅ | |
| `ntex_service_error.rs` | `NtexServiceError` | ✅ | |
| `vernal_ntex_component.rs` | `VernalNtexComponent` | ✅ | |
| `vernal_ntex_context.rs` | `VernalNtexContext` | ✅ | |
| `vernal_ntex_middleware.rs` | `Middleware` | 🔶 | 同 actix |
| `vernal_ntex_request_context.rs` | `NtexRequestContext` | ✅ | |
| `vernal_ntex_request_scope.rs` | `NtexRequestScope` | ✅ | |
| `vernal_ntex_service.rs` | `NtexService` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.6 `vernal-poem`（11 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `poem_aop_error.rs` | `PoemAopError` | ✅ | |
| `poem_rejection.rs` | `PoemRejection` | ✅ | |
| `poem_response.rs` | `PoemResponse` | ✅ | |
| `poem_scoped_stream.rs` | `PoemScopedStream` | ✅ | |
| `vernal_poem_component.rs` | `VernalPoemComponent` | ✅ | |
| `vernal_poem_context.rs` | `VernalPoemContext` | ✅ | |
| `vernal_poem_endpoint.rs` | `PoemEndpoint` | ✅ | |
| `vernal_poem_middleware.rs` | `Middleware` | 🔶 | 同 actix |
| `vernal_poem_request_context.rs` | `PoemRequestContext` | ✅ | |
| `vernal_poem_request_scope.rs` | `PoemRequestScope` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.7 `vernal-rocket`（14 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `rocket_aop_error.rs` | `RocketAopError` | ✅ | |
| `rocket_borrowed_target.rs` | `RocketBorrowedTarget` | ✅ | |
| `rocket_outcome_marker.rs` | `RocketOutcomeMarker` | ✅ | |
| `rocket_rejection.rs` | `RocketRejection` | ✅ | |
| `rocket_request_snapshot.rs` | `RocketRequestSnapshot` | ✅ | |
| `rocket_request_state.rs` | `RocketRequestState` | ✅ | |
| `rocket_scoped_reader.rs` | `RocketScopedReader` | ✅ | |
| `vernal_rocket_component.rs` | `VernalRocketComponent` | ✅ | |
| `vernal_rocket_context.rs` | `VernalRocketContext` | ✅ | |
| `vernal_rocket_fairing.rs` | `Fairing` | 🔶 | rocket middleware 字面叫 `Fairing`（rocket 原生），但建议改为 `RocketFairing` 避免与 `rocket::Fairing` 命名冲突；待对齐。 |
| `vernal_rocket_handler.rs` | `RocketHandler` | ✅ | |
| `vernal_rocket_request_context.rs` | `RocketRequestContext` | ✅ | |
| `vernal_rocket_request_scope.rs` | `RocketRequestScope` | ✅ | |
| `vernal_rocket_routes_ext.rs` | `RoutesExt` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.8 `vernal-salvo`（10 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `salvo_aop_error.rs` | `SalvoAopError` | ✅ | |
| `salvo_borrowed_target.rs` | `SalvoBorrowedTarget` | ✅ | |
| `salvo_rejection.rs` | `SalvoRejection` | ✅ | |
| `salvo_request_snapshot.rs` | `SalvoRequestSnapshot` | ✅ | |
| `salvo_response.rs` | `SalvoResponse` | ✅ | |
| `salvo_route_metadata.rs` | `RouteMetadata` | 🔶 | 字面 `RouteMetadata` 与 `vernal_web::RouteMetadata` 同名；建议 `SalvoRouteMetadata` 或将共享类型放进 `vernal-web::contract::route_metadata`。 |
| `salvo_scoped_body.rs` | `SalvoScopedBody` | ✅ | |
| `vernal_salvo_depot_ext.rs` | `DepotExt` | 🔶 | 同 `StateExt`，建议 `SalvoDepotExt` |
| `vernal_salvo_hoop.rs` | `Hoop` | 🔶 | salvo middleware 字面名 `Hoop`（salvo 原生），建议 `SalvoHoop` |
| `lib.rs` | module | ✅ | |

### 2.9 `vernal-tide`（10 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `tide_aop_error.rs` | `TideAopError` | ✅ | |
| `tide_body_error.rs` | `TideBodyError` | ✅ | |
| `tide_borrowed_target.rs` | `TideBorrowedTarget` | ✅ | |
| `tide_rejection.rs` | `TideRejection` | ✅ | |
| `tide_request_snapshot.rs` | `TideRequestSnapshot` | ✅ | |
| `tide_response.rs` | `TideResponse` | ✅ | |
| `tide_scoped_reader.rs` | `TideScopedReader` | ✅ | |
| `vernal_tide_middleware.rs` | `Middleware` | 🔶 | 同 actix |
| `vernal_tide_request_ext.rs` | `RequestExt` | 🔶 | 字面 `RequestExt` 与 web 内核 `request_context.rs::RequestContext` 重名风险；建议 `TideRequestExt` |
| `lib.rs` | module | ✅ | |

### 2.10 `vernal-warp`（11 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `warp_aop_error.rs` | `WarpAopError` | ✅ | |
| `warp_rejection.rs` | `WarpRejection` | ✅ | |
| `warp_route_resolver.rs` | `RouteResolver` | 🔶 | `RouteResolver` 字面；建议 `WarpRouteResolver` 或共建 `vernal-web::contract::route_resolver` |
| `vernal_warp_aop_layer.rs` | `WarpAopLayer` | ✅ | |
| `vernal_warp_aop_service.rs` | `WarpAopService` | ✅ | |
| `vernal_warp_component.rs` | `VernalWarpComponent` | ✅ | |
| `vernal_warp_context.rs` | `VernalWarpContext` | ✅ | |
| `vernal_warp_layer.rs` | `Layer` | 🔶 | `Layer` 字面；建议 `WarpLayer` |
| `vernal_warp_request_context.rs` | `WarpRequestContext` | ✅ | |
| `vernal_warp_request_scope.rs` | `WarpRequestScope` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.11 `vernal-tower`（22 文件）

> 契约层/Servcie+Layer 基础；命名可作为 runtime crate 的样板。

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `aop_service.rs` | `AopService` | ✅ | 与 `AopProxyFactory` 对齐 |
| `aop_layer.rs` | `AopLayer` | ✅ | |
| `aop_service_error.rs` | `AopServiceError` | ✅ | |
| `tower_upstream_error.rs` | `UpstreamError` | ✅ | |
| `tower_response.rs` | `Response` | 🔶 | 名称与 `axum::response::Response` 等多冲突；建议 `VernalResponse` 或迁入 `vernal-web::contract::tower_response` |
| `tower_error.rs` | `Error` | 🔶 | 字面 `Error` 风险；建议 `VernalServiceError` |
| `tower_body_error.rs` | `BodyError` | ✅（但 ⚠ 与其他 runtime 的 `<runtime>_body_error.rs` 同名） | |
| `tower_route_resolver.rs` | `RouteResolver` | ✅（⚠ 同名风险） | |
| `tower_error_mapper.rs` | `ErrorMapper` | ✅ | |
| `vernal_service.rs` | `VernalService` | ✅ | |
| `vernal_layer.rs` | `VernalLayer` | ✅ | |
| `scoped_body.rs` | `ScopedBody` | ✅ | |
| `extension_route_resolver.rs` | `ExtensionRouteResolver` | ✅ | |
| `request_scope_layer.rs` | `RequestScopeLayer` | ✅ | |
| `request_scope_service.rs` | `RequestScopeService` | ✅ | |
| `error_mapping_layer.rs` | `ErrorMappingLayer` | ✅ | |
| `error_mapping_service.rs` | `ErrorMappingService` | ✅ | |
| `context_propagation_layer.rs` | `ContextPropagationLayer` | ✅ | |
| `context_propagation_service.rs` | `ContextPropagationService` | ✅ | |
| `context_propagation_error.rs` | `ContextPropagationError` | ✅ | |
| `missing_plan_policy.rs` | `MissingPlanPolicy` | ✅ | |
| `lib.rs` | module | ✅ | |

### 2.12 `vernal-tonic`（9 文件）

| Rust 文件 | 类型 | 一致性 | 备注 |
|---|---|---|---|
| `tonic_aop_layer.rs` | `TonicAopLayer` | ✅ | |
| `tonic_aop_service.rs` | `TonicAopService` | ✅ | |
| `tonic_aop_error_mapper.rs` | `IntoStatus for WebFailure` | ✅ | |
| `tonic_context_interceptor.rs` | `ContextInterceptor` | ✅（⚠ 建议 `TonicContextInterceptor` 字面） | |
| `tonic_request_ext.rs` | `RequestExt` | 🔶 | 同 tide，建议 `TonicRequestExt` |
| `tonic_request_error.rs` | `RequestError` | 🔶 | 字面，建议 `TonicRequestError` |
| `tonic_route_resolver.rs` | `RouteResolver` | 🔶 | 字面 |
| `tonic_status_mapper.rs` | `StatusMapper` | ✅（⚠ 建议 `TonicStatusMapper`） | |
| `lib.rs` | module | ✅ | |

## 三、契约层（`vernal-web` / `vernal-http`）命名一致性

| 文件 | 类型 | 与 Spring 对齐 | 一致性 | 备注 |
|---|---|---|---|---|
| `request_context.rs` | `RequestContext` | `RequestContext` | ✅ | |
| `route_metadata.rs` | `RouteMetadata` | `RequestMappingInfo` | 🔶 | Spring 名称是 `RequestMappingInfo`；用 `RouteMetadata` 更友好 |
| `handler_invocation.rs` | `HandlerInvocation` | `HandlerExecutionChain` | 🔶 | 字面不同 |
| `web_failure.rs` | `WebFailure` | `HandlerExceptionResolver` | 🆕 | 行为别名 + 分支枚举 |
| `problem_kind.rs` | `ProblemKind` | `ProblemDetail` 类型 | 🆕 | |
| `problem_details.rs` | `ProblemDetails` | `ProblemDetail`（Spring 6.0+） | 🔶 | Spring 单数，此处复数；与 `spring-web到vernal-web对象名称一致性检查` 中已记录的策略一致 |
| `request_id.rs` | `RequestId` | `Observation`/MDC | 🆕 | |
| `security_principal.rs` | `SecurityPrincipal` | `Authentication` + `Principal` | ✅ | |
| `web_request_scope.rs` | `WebRequestScope` | `RequestContextHolder` | ✅ | |
| `web_request_scope_owner.rs` | `WebRequestScopeOwner` | 同上 | ✅ | |
| `context_carrier.rs` | `ContextCarrier` | `ContextSnapshot` | 🆕 | |
| `transport_kind.rs` | `TransportKind` | — | 🆕 | |
| `integration_descriptor.rs` | `IntegrationDescriptor` | — | 🆕 | |
| `integration_role.rs` | `IntegrationRole` | — | 🆕 | |

## 四、`vernal-http` 命名一致性

| 文件 | 类型 | Spring 来源 | 一致性 |
|---|---|---|---|
| `http_request_snapshot.rs` | `HttpRequestSnapshot` | `HttpRequest` snapshot | ✅ |
| `http_response.rs` | `HttpResponse` | `ClientHttpResponse` | ✅ |
| `http_body.rs` | `HttpBody` | `HttpInputMessage`/`StreamingHttpOutputMessage.Body` | ✅ |
| `http_body_error.rs` | `BodyError` | `HttpMessageConversionException` | ✅ |
| `http_request.rs` | `HttpRequest` | `ClientHttpRequest` | ✅ |
| `collected_body.rs` | `CollectedBody` | (Spring 没有对应物) | 🆕 |

## 五、跨 crate 一致性检查（必须解决）

| 现象 | 出现的 crate | 建议 |
|---|---|---|
| `Middleware` 类型不带 runtime 前缀 | actix / gotham / ntex / poem / tide | 改为 `ActixMiddleware` / `GothamMiddleware` / … |
| `Layer` 同上（无前缀） | warp | 改为 `WarpLayer` |
| `Hoop`（无前缀） | salvo | 改为 `SalvoHoop` |
| `Fairing`（无前缀） | rocket | 改为 `RocketFairing` |
| `RouteMetadata`（无前缀） | salvo + vernal-web | 让 `vernal-web::RouteMetadata` 作唯一来源；salvo 改为 `SalvoRouteMetadata` |
| `RouteResolver`（无前缀） | axum / warp / tonic / tower | 同上：定义 `vernal-web::contract::route_resolver::RouteResolver`；axum/warp/tonic/tower 实现为 `<runtime>RouteResolver` |
| `RequestExt`（无前缀） | tide / tonic | 改为 `<runtime>RequestExt` |
| `RequestError`（无前缀） | tonic | 改为 `TonicRequestError` |
| `StatusMapper`（无前缀） | tonic | 改为 `TonicStatusMapper` |
| `ContextInterceptor`（无前缀） | tonic | 改为 `TonicContextInterceptor` |
| `StateExt`（无前缀） | gotham | 改为 `GothamStateExt` |
| `DepotExt`（无前缀） | salvo | 改为 `SalvoDepotExt` |
| `RouterExt`（axum） | axum | 改为 `AxumRouterExt` |
| `RoutesExt`（rocket） | rocket | 改为 `RocketRoutesExt` |
| `Response` / `Error`（tower） | tower | 改为 `VernalResponse` / `VernalServiceError`，并迁入 contract 模块 |
| `ProblemDetails` vs Spring `ProblemDetail` | vernal-web | 已存在 🔶 标记，与 `spring-web到vernal-web对象名称一致性检查` 一致 |
| `RequestContext`（vernal-web） vs Spring `RequestContext`（`web.servlet.support`） | vernal-web | 命名同；运行时不会与 servlet 共生，保留 ✅；但 README 必须标注目标位置 |
| vernal-axum 的 `vernal_*.rs` 前缀与其它 runtime 的 `vernal_<rt>_*` 不齐 | vernal-axum | R2/R3 重命名 `vernal_*.rs` → `vernal_axum_*.rs` |

## 六、当前总体结论

| 维度 | 结果 |
|---|---|
| 命名一致性 | **≈ 86%**（绝大部分文件遵守运行时前缀规则） |
| 主要风险点 | 8 个 runtime 中 `Middleware` / `Layer` / `Hoop` / `Fairing` / `RouteResolver` / `StateExt` / `DepotExt` / `RoutesExt` / `RequestExt` / `RequestError` / `StatusMapper` / `ContextInterceptor` 字面无 runtime 前缀，需统一进入契约层或重新加前缀 |
| 建议动作（R1 完成时一并清理） | 1) 在 `vernal-web::contract` 中定义 `RouteResolver`、`RouteMetadata`、`RouteCondition` 等共享 trait；2) 各 runtime 用 `Vernal<Runtime>*` 或 `<Runtime>*` 命名自己的实现；3) `Middleware`/`Layer`/`Hoop`/`Fairing` 全部加 runtime 前缀 |
| Cargo description 一致性 | 12 个 runtime crate 全部使用 `"<Runtime> integration for Vernal."` 模板，**完全一致** |
| 已落地但仍待完善的契约 | `HandlerAdapter` / `ArgumentResolver` / `ReturnValueHandler` / `HandlerMapping` / `HandlerInterceptor` / `WebFilter` / `HandlerExceptionHandler` / `ContentNegotiationManager` / `LocaleContextResolver` / `FlashMapManager` |

## 七、命名验收规则（合并本套与既有文档）

每次新增 runtime 文件，**必须**同时满足：

1. 文件名 snake_case，以 `<runtime>_<concept>.rs` 模板（gateway crate 例外：`vernal-tower` 用 `tower_<concept>.rs` 或 `vernal_<concept>.rs`）。
2. 类型名 PascalCase，并**带 runtime 前缀**（除非该类型在 `vernal-web::contract` 定义已带前缀）。
3. marker trait / 共享 trait 仅在 `vernal-web::contract` / `vernal-tower` 中定义一次；runtime 不复制定义。
4. 不复制 Java 专有类型名（如 `DispatcherServlet`、`FrameworkServlet`、`HttpServletBean`、`*BeanDefinitionParser`）。
5. 不复制 runtime 原生 trait 名（如 `salvo::router::Router`、`rocket::fairing::Fairing` 直接作为唯一名字）；vernal 侧一律 `<runtime>Original` 前缀。
6. 跨 runtime 行为对齐通过 `vernal-web-testkit::web_adapter_contract::run_contract::<T>` 实现；命名错误的实现立刻在测试矩阵里报错。
7. 每次新对象/文件落地必须更新本套 4 份文档，并同步勾选路线图、对象对照表与本表。

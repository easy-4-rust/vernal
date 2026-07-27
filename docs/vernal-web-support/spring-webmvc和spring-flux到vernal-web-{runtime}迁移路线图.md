# spring-webmvc + spring-flux → vernal-web-{runtime} 全量迁移路线图

> 版本：v1.0（2026-07-27）｜基线：Spring Framework 7.0.8 `spring-webmvc` + `spring-webflux`
> 目标：以 `vernal-web` 为契约核心，把 servlet 与 reactive 双栈语义完整迁移到 vernal 现有的 12 个 runtime crate（actix-web、axum、gotham、hyper、ntex、poem、rocket、salvo、tide、warp、tower、tonic）。
> 名称约定：**每个 runtime 一个独立 crate，平级**（已落地），不建 `vernal-web-support` 聚合。

## 一、CodeGraph 与源码基线

| 项目 | 事实 |
|---|---|
| `vernal-framework` | 4,606 节点 / 34,731 边 / 754 文件；契约层 32 communities（含 `src-web`、HTTP、Codec、tower） |
| `spring-framework` | 115,037 节点 / 1,029,168 边 / 9,680 文件；27 communities；`web-context` 13,042 节点、`support-request` 14,449 节点、`annotation-handler` 9,413 节点、`annotation-handle` 6,957 节点 |
| `spring-webmvc` | 363 个 Java 文件；`org.springframework.web.servlet` 含 21 个一级 + config/function/handler/i18n/mvc/resource/support/tags/view 子包 |
| `spring-webflux` | 275 个 Java 文件；`org.springframework.web.reactive` 含 DispatcherHandler、HandlerResult、accept/config/function/handler/resource/result/socket 子包 |
| `vernal-web` | 15 文件；`request_context.rs`、`route_metadata.rs`、`handler_invocation.rs`、`web_failure.rs`、`problem_kind.rs` 与 `ProblemDetails`、request scope 等 |
| `vernal-actix-web` | 14 文件（component、request_context、middleware、service、rejection、scoped_body、snapshot、aop_error、scope、context 等） |
| `vernal-axum` | 12 文件（route_resolver、router_ext、context、scope、component、rejection、aop_error_mapper 等） |
| `vernal-gotham` | 13 文件（state_ext、middleware、context、request_context、scope、rejection、scoped_body、aop_error、borrowed_target、body_error、response、upstream_error） |
| `vernal-hyper` | 2 文件（lib + hyper_bridge；纯 HTTP transport 基础） |
| `vernal-ntex` | 14 文件（与 actix 一致同形态） |
| `vernal-poem` | 11 文件（endpoint、middleware、context、scope、component、rejection、scoped_stream、response、aop_error） |
| `vernal-rocket` | 14 文件（handler、routes_ext、fairing、scope、component、request_context、request_state、rejection、borrowed_target、snapshot、request_scope、aop_error） |
| `vernal-salvo` | 10 文件（hoop、depot_ext、route_metadata、request_snapshot、scoped_body、borrowed_target、rejection、response、aop_error） |
| `vernal-tide` | 10 文件（middleware、request_ext、scope、rejection、snapshot、borrowed_target、response、body_error、aop_error） |
| `vernal-warp` | 11 文件（route_resolver、layer、aop_service、aop_layer、context、request_context、scope、component、rejection、aop_error） |
| `vernal-tonic` | 9 文件（request_ext、request_error、aop_service、aop_layer、aop_error_mapper、route_resolver、status_mapper、context_interceptor） |
| `vernal-tower` | 22 文件（service、layer、error_mapping_service/layer、context_propagation_service/layer、request_scope_service/layer、extension_route_resolver、tower_route_resolver、aop_service_error、tower_body_error、missing_plan_policy 等） |
| `vernal-web-testkit` | 已存在；tests 下含 `security_contract.rs` 与 `web_adapter_contract.rs` — 跨 runtime contract test 入口 |

## 二、总目标与边界

### 总目标

1. **以 `vernal-web` 为契约层**：定义所有 runtime 共用的 `HandlerAdapter`、`HandlerMapping`、`ArgumentResolver`、`ReturnValueHandler`、`HandlerInterceptor`、`WebFilter`、`HandlerExceptionHandler`、`ServerWebExchange`、`ServerRequest`、`ServerResponse`、`ContentNegotiationManager`、`LocaleContextResolver`、`FlashMapManager` 等。
2. **runtime crate 实现统一接口**：每个 `vernal-{runtime}` crate 把该 runtime 的 middleware/handler/extractor/response 映射到 `vernal-web` 的抽象；同一段业务代码可以零修改切换 runtime。
3. **同步覆盖 servlet 与 reactive 双栈**：
   - servlet → `vernal-actix-web`、`vernal-rocket`、`vernal-warp`、`vernal-salvo` 等同步派发 crate；
   - reactive → `vernal-axum`、`vernal-tide`、`vernal-ntex`、`vernal-poem`、`vernal-gotham`、`vernal-hyper`（hyper 同时为 servlet/reactive 基底）等异步派发 crate。
   - `vernal-tower` + `vernal-tonic` 共用 `Service` / `Layer` 抽象，连接 tower 与 gRPC。
4. **抽象与实现测试同源**：所有 12 个 runtime 共享 `vernal-web-testkit` 的 `web_adapter_contract.rs`。

### 不直接迁移的 Java 专有机制

| Java 语义 | Rust 化替代 |
|---|---|
| Servlet 容器 API、`DispatcherServlet` 生命周期 | Tower `Service` + Hyper/Axum 的 Router/Middleware；通过 `vernal-tower` 抽象 |
| `HttpServletRequest`/`HttpServletResponse` | `vernal-http::{HttpRequest, HttpResponse, HttpBody}`；`vernal-web::{ServerRequest, ServerResponse, ServerWebExchange}` |
| `ModelAndView` + JSP/Thymeleaf 渲染 | 不直接进入 web 层；通过 `vernal-templating`（未来）提供 JSON 序列化；ViewResolver 留作 trait |
| `Publisher`/`Mono`/`Flux` | Rust `Future`/`Stream`；`BoxStream`、`Pin<Box<dyn Future>>`；使用 `tokio::sync::*`、`futures::*` |
| `TaskExecutor` / 线程切换 | 直接由运行时执行器负责，扩展点抽 `Executor`/`reactor` 抽象 |
| `HandlerMethodArgumentResolver` 反射 | 显式 `FromRequestParts` + `FromRequest`；与 axum 的 extractor 形态对齐 |
| `@Controller/@RestController` 注解 | `#[vernal::route]`、`#[intercept]`、`#[component]` 派生宏；annotation 名称后改 |
| `MultipartResolver` + `Commons FileUpload` | `multipart` feature + `mul-types` crate；按运行时切片 |

## 三、Crate 拓扑与依赖矩阵

```text
vernal-core        ──┐
vernal-beans       ──┤
vernal-context     ──┼──> vernal-web (contract)
vernal-aop         ──┤
vernal-http        ──┘
                       │
                       ├── vernal-tower  (Service+Layer foundation)
                       │       ├── vernal-actix-web
                       │       ├── vernal-axum
                       │       ├── vernal-tide
                       │       ├── vernal-warp
                       │       ├── vernal-rocket   (间接通过 Service trait)
                       │       ├── vernal-salvo
                       │       └── vernal-tonic
                       ├── vernal-hyper  (HTTP transport base)
                       ├── vernal-gotham (stateful router)
                       ├── vernal-ntex
                       └── vernal-poem
```

`vernal-web-testkit`：单测跨 runtime contract 测试入口；要求每个 `vernal-{runtime}` 把 contract test 挂在 `tests/{runtime}_adapter.rs`。

## 四、阶段总览

| 阶段 | 主题 | 状态 | 产物 | 验收 |
|---|---|---|---|---|
| R0 | 本套 4 份文档 | ✅ | 本目录文档 | 同步仓库现状 |
| R1 | 在 `vernal-web` 暴露完整抽象 | ⬜ | HandlerAdapter/ArgumentResolver/ReturnValueHandler/HandlerMapping/HandlerInterceptor/WebFilter/HandlerExceptionHandler/ContentNegotiationManager/LocaleContextResolver/FlashMapManager 等 | trait 编译通过 + 编译期 mock |
| R2 | actix-web 完整迁移（样板） | ⬜ | 用现有 `vernal-actix-web/*` 文件对齐 R1 抽象 | `tests/actix_adapter.rs` 过 contract |
| R3 | axum 完整迁移（样板） | ⬜ | 用现有 `vernal-axum/*` 文件对齐 R1 抽象 | `tests/axum_adapter.rs` 过 contract |
| R4 | Rocket + Salvo + Warp 迁移 | ⬜ | 三个 crate `tests/*_adapter.rs` | contract |
| R5 | Tide + Poem + Gotham + Ntex 迁移 | ⬜ | 四个 crate `tests/*_adapter.rs` | contract |
| R6 | Hyper + Tower 落地 Service/Layer | ⬜ | `vernal-hyper::hyper_bridge` 扩展 + `vernal-tower` 单元补全 | `tests/hyper_transport.rs`、`tests/tower_contract.rs` |
| R7 | Tonic/gRPC + 双栈异常映射 | ⬜ | `vernal-tonic` 异常 → HTTP/Status 映射 | `tests/tonic_adapter.rs` |
| R8 | 测试矩阵与文档收尾 | ⬜ | `vernal-web-testkit/web_adapter_contract.rs` + CI matrix | 中文注释、覆盖率 |

## 五、详细路线

### R1：在 `vernal-web` 暴露完整抽象（P0）

增加模块（按已有命名风格）：

```text
crates/vernal-web/src/
├── contract/                       # 新建 contract 子模块
│   ├── mod.rs
│   ├── handler_adapter.rs          # HandlerAdapter trait
│   ├── handler_mapping.rs          # HandlerMapping trait
│   ├── handler_invocation.rs       # HandlerInvocation（已存在 → 升级）
│   ├── handler_interceptor.rs      # pre/post/completion
│   ├── handler_exception_handler.rs# 异常 → ServerResponse 映射
│   ├── argument_resolver.rs        # ServerRequest → dyn trait → typed argument
│   ├── return_value_handler.rs     # handler 返回值 → IntoResponse
│   ├── content_negotiation_manager.rs
│   ├── locale_context_resolver.rs
│   ├── flash_map_manager.rs
│   ├── request_context.rs（升级）  # 已有 → 增加 Uri、Method、Header、Body 视图
│   ├── server_request.rs
│   ├── server_response.rs
│   ├── server_web_exchange.rs
│   └── view_resolver.rs            # trait 占位，未来 templating feature 用
```

要求：每个 trait 独立文件；`mod.rs` 仅做模块声明与 re-export。

### R2-R5：按 runtime 实施迁移（每个 runtime 一个阶段，独立打勾）

每个 runtime crate 必须具备的文件形态（参考 `vernal-actix-web`）：

| 文件名 | 用途 | Spring 对应 |
|---|---|---|
| `<runtime>_request_context.rs` | 把 runtime request 提升为 `ServerRequest` | `HttpServletRequest`/`ServerHttpRequest` |
| `<runtime>_service.rs` | runtime handler → vernal Service | `DispatcherServlet.doDispatch` |
| `<runtime>_middleware.rs` 或 `<runtime>_layer.rs` 或 `<runtime>_fairing.rs` | runtime 拦截器 | `HandlerInterceptor` + `WebFilter` |
| `<runtime>_component.rs` | 把 runtime 注册成 vernal component | `DispatcherServlet` 中的 `initStrategies` |
| `<runtime>_scope.rs` 或 `<runtime>_request_scope.rs` | runtime scope ↔ `WebRequestScope` | `RequestContextHolder` |
| `<runtime>_context.rs` 或 `<runtime>_state_ext.rs` 或 `<runtime>_depot_ext.rs` 或 `<runtime>_request_ext.rs` | runtime 上下文 → vernal context | `RequestContextUtils` |
| `<runtime>_rejection.rs` | runtime 拒绝 → vernal exception | `HandlerExceptionResolver` |
| `<runtime>_aop_error.rs` 或 `<runtime>_aop_error_mapper.rs` | runtime AOP 错误归一 | `ResponseStatusExceptionResolver` |
| `<runtime>_borrowed_target.rs` 或 `<runtime>_scoped_body.rs` 或 `<runtime>_snapshot.rs` 或 `<runtime>_scoped_stream.rs` 或 `<runtime>_scoped_reader.rs` | runtime body ↔ vernal `Body` | `HttpInputMessage`/`HttpOutputMessage` |
| `<runtime>_response.rs` 或 `<runtime>_response_envelope.rs` 或 `<runtime>_route_resolver.rs` 等 | runtime response 辅助 | `HttpMessageConverter` 适配 |
| `lib.rs` | 门面 + re-export | `FrameworkServlet` |

> 一个 runtime crate 中“功能合一”的文件只允许在以下情况下合并：①真正的 marker trait；②同一值对象的只读视图；③runtime 自身的设计（如 axum 的 `Router/Handler`）。每个 runtime 都需要在自己的 `tests/<runtime>_adapter.rs` 跑 `vernal_web_testkit::web_adapter_contract` 的同等断言。

### R6：Hyper/Tower 抽象落地（P0）

- `vernal-hyper` 维持纯 transport 定位；在 `hyper_bridge.rs` 暴露 `HyperRequest → ServerRequest`、`HyperResponse → ServerResponse` 双向转换。
- `vernal-tower` 已经 22 文件结构完整；新增/补：`request_scope_service/layer`、`context_propagation_service/layer`、`error_mapping_service/layer`、`aop_layer`、`extension_route_resolver`、`tower_route_resolver`、`aop_service_error`、`tower_body_error`。这是 `vernal-tower` 给所有 runtime 提供 Tower 抽象的核心。
- 单元测试：`tests/aop_contract.rs`、`tests/error_mapping_contract.rs`、`tests/context_propagation_contract.rs`、`tests/tower_contract.rs`。

### R7：Tonic/gRPC 双栈异常映射（P1）

- `vernal-tonic` 已存在 9 文件；以 `tonic_status_mapper.rs` 把 `tonic::Status` ↔ `ProblemDetail` ↔ `HandlerExceptionHandler`。
- 同步：`tests/tonic_adapter.rs` 跑 `vernal-web-testkit` 异常映射 contract。

### R8：测试与文档收尾（P1）

- `crates/vernal-web-testkit/src/web_adapter_contract.rs` 增加 `run_contract::<Runtime>` async 函数。
- CI matrix：
  ```bash
  cargo test -p vernal-web -p vernal-web-testkit -p vernal-tower -p vernal-hyper
  for r in actix-web axum gotham ntex poem rocket salvo tide warp tonic; do
    cargo test -p vernal-$r
  done
  ```
- 文档：每个 runtime 增 `README.md`、`CHANGELOG.md`、`examples/minimal.rs`。
- 中文注释必须 100% 覆盖 `pub` 接口（继承现有 lints：`missing_docs = "warn"`）。

## 六、验收门槛

| 维度 | 目标 |
|---|---|
| 单元测试 | `vernal-web`/每个 runtime crate ≥ 已有 + 新增 contract 测试 |
| 契约覆盖 | R1 trait 必须有 mock 实现，并被 `vernal-web-testkit` 用 |
| 跨 runtime 一致性 | 同一段 handler 在 actix/axum/warp/tide/poem/gotham/ntex/rocket/salvo 上行为一致 |
| HTTP 消息兼容 | 与 `vernal-http` 的 `HttpRequest/Response/Body` 互通，且能直接桥接 `hyper`/`axum` |
| AOP 兼容 | 业务方法 `@intercept` 在 12 个 runtime 都能触发；`vernal-aop` plan 自动发现 |
| Context 兼容 | 12 个 runtime 都能从 `vernal-context::ApplicationEnvironment` 读取环境与 Bean |
| 表达式兼容 | `vernal-expression` 路由谓词、参数绑定、SpEL 属性在 12 个 runtime 都生效 |
| CI | `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` |

## 七、风险与回退策略

| 风险 | 应对 |
|---|---|
| 不同 runtime 的 cancel/timeout 语义不一致 | `vernal-web-testkit` 增加 async cancel/timeout 矩阵断言 |
| Body 在 axum/actix/ntex 间的双消费规范差异 | 统一在 `vernal-http::HttpBody` 之上做 BoxBody/StreamBody 包装；runtime crate 不直接 `read()` |
| AOP plan 与 dispatch 流耦合造成差异 | AOP 必须在 `HandlerAdapter` 之上做，runtime 不能修改 plan |
| tower/Service 抽象被忽略 | 所有 runtime 测试必须挂上 `vernal-tower` 共享断言 |
| gRPC 与 HTTP 语义漂移 | `vernal-tonic` 的 status mapper 必须经过 `vernal-web::WebFailure` 翻译 |
| gotham/rocket 的同步模型差异 | 把它们封装为独立 sync adapter；统一在 `vernal-web::ServerRequest` 中保留 `block_in_place` 入口 |

## 八、交付节奏

- R1：先抽 `vernal-web/contract` 抽象；同步给所有 runtime 跑通最小 `tests/<runtime>_adapter.rs`。
- R2-R5：每完成一个 runtime，更新一次文档（本套 4 份）。
- R6-R7：hyper/tower 收尾；tonic 完成异常映射。
- R8：CI matrix 与文档稳定。

## 九、文档同步规则

每次合并 runtime adapter PR，必须一并更新本目录四份文档：

1. 路线图：勾选对应的 R 阶段、记录新增/拆分文件。
2. 对象级对照表：标注 `<runtime>_*` 对象名与 Spring 来源的全限定名。
3. 语义迁移对照表：标注每个能力在 `<runtime>` 的实现方式或“不支持”。
4. 对象名称一致性检查：列出文件命名是否符合 `<runtime>_<concept>.rs` 规则；列出公开 trait 名称是否与 `vernal-web::contract` 完全一致。

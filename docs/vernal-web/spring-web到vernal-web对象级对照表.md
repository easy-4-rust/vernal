<!-- migration-doc: authority=historical canonical=对象级对照表.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [对象级对照表.md](对象级对照表.md) 为准。

> **历史文档，非当前验收依据。** 当前版本见[对象级对照表](对象级对照表.md)。

# spring-web → vernal-web 对象级对照表

> 基线：Spring Framework 7.0.8。本文是结构验收清单：每个迁移对象必须有明确 Rust 类型/文件、状态和语义差异说明。Spring `web` 下的 Servlet、WebFlux、WebSocket、client、codec 等实现按层次归类；同一语义只保留一个 Vernal 核心对象，运行时差异放在 adapter。

状态：✅ 已有；🔶 需要扩展；⬜ 待实现；🚫 Java/运行时专有；🆕 Vernal 新增。

## 一、当前 Vernal Web 对象

| 当前文件 | Rust 对象 | Spring 对偶 | 状态 | 说明 |
|---|---|---|---|---|
| `request_context.rs` | `RequestContext` | `ServerWebExchange`/request context | 🔶 | 已有请求上下文，需补 URI、方法、Header、Body、属性、响应句柄 |
| `web_request_scope.rs` | `WebRequestScope` | request scope | ✅ | 请求级生命周期与作用域 |
| `web_request_scope_owner.rs` | `WebRequestScopeOwner` | request scope owner | ✅ | 所有权/关闭语义 |
| `route_metadata.rs` | `RouteMetadata` | `HandlerMapping` metadata | 🔶 | 需补路径变量、媒体类型、名称、顺序 |
| `handler_invocation.rs` | `HandlerInvocation` | `HandlerAdapter` invocation | 🔶 | 需补参数解析、返回值处理、错误转换 |
| `problem_details.rs` | `ProblemDetails` | `ProblemDetail` | 🔶 | 需对齐 RFC 9457 字段及扩展属性 |
| `problem_kind.rs` | `ProblemKind` | Web exception categories | 🔶 | 需覆盖 400/404/405/406/415/413/500 等 |
| `web_failure.rs` | `WebFailure` | `ResponseStatusException`/Web error | 🔶 | 统一错误分类与响应映射 |
| `request_id.rs` | `RequestId` | observation/request correlation | ✅ | 请求关联 ID |
| `security_principal.rs` | `SecurityPrincipal` | `Principal` | ✅ | 安全主体抽象 |
| `context_carrier.rs` | `ContextCarrier` | request attributes/context | 🔶 | 需支持 typed extensions 与上下文传播 |
| `transport_kind.rs` | `TransportKind` | HTTP/WebSocket transport | ✅ | 传输类型 |
| `integration_descriptor.rs` | `IntegrationDescriptor` | framework integration metadata | ✅ | 集成描述 |
| `integration_role.rs` | `IntegrationRole` | adapter role | ✅ | 适配器角色 |
| `handler_invocation.rs` | `HandlerInvocation` | handler call | 🔶 | 需成为服务端核心入口 |

## 二、HTTP 核心值对象（`org.springframework.http`）

| Spring 类 | 目标 Rust 文件 | 目标对象 | 状态 | 语义要求 |
|---|---|---|---|---|
| `HttpMethod` | `http_method.rs` | `HttpMethod` | ⬜ | 标准方法、扩展方法、大小写规范 |
| `HttpStatusCode` | `http_status_code.rs` | `HttpStatusCode` | ⬜ | 任意 3 位码、系列判断 |
| `HttpStatus` | `http_status.rs` | `HttpStatus` | ⬜ | 标准状态枚举与 reason |
| `MediaType` | `media_type.rs` | `MediaType` | ⬜ | type/subtype/参数/通配符/兼容性 |
| `HttpHeaders` | `http_headers.rs` | `HttpHeaders` | ⬜ | 大小写不敏感、多值、常用 typed getter |
| `HttpCookie` | `http_cookie.rs` | `HttpCookie` | ⬜ | 解析、属性和安全标志 |
| `ResponseCookie` | `response_cookie.rs` | `ResponseCookie` | ⬜ | Set-Cookie 构建 |
| `HttpMessage` | `http_message.rs` | `HttpMessage` | ⬜ | headers 合同 |
| `HttpInputMessage` | `http_input_message.rs` | `HttpInputMessage` | ⬜ | headers + body stream |
| `HttpOutputMessage` | `http_output_message.rs` | `HttpOutputMessage` | ⬜ | headers + async writer |
| `HttpEntity` | `http_entity.rs` | `HttpEntity<T>` | ⬜ | body 可选、headers 不可变快照 |
| `RequestEntity` | `request_entity.rs` | `RequestEntity<T>` | ⬜ | method + URL + headers + body |
| `ResponseEntity` | `response_entity.rs` | `ResponseEntity<T>` | ⬜ | status + headers + body |
| `ProblemDetail` | `problem_detail.rs` | `ProblemDetail` | 🔶 | 扩展现有 `ProblemDetails` |
| `ContentDisposition` | `content_disposition.rs` | `ContentDisposition` | ⬜ | form-data、filename 编码 |
| `HttpRange` | `http_range.rs` | `HttpRange` | ⬜ | bytes range 解析/合并 |
| `ETag` | `etag.rs` | `ETag` | ⬜ | 强/弱比较 |
| `CacheControl` | `cache_control.rs` | `CacheControl` | ⬜ | Cache-Control 指令 |
| `ReadOnlyHttpHeaders` | `http_headers.rs` | `ReadOnlyHttpHeaders` | 🔶 | Rust 借用/只读视图 |
| `StreamingHttpOutputMessage` | `streaming_http_output.rs` | `StreamingHttpOutputMessage` | ⬜ | Stream body |
| `ZeroCopyHttpOutputMessage` | `zero_copy_output.rs` | `ZeroCopyHttpOutputMessage` | ⬜ | 平台能力可选 |

## 三、服务端 Web 对象

| Spring 语义/类族 | 目标 Rust 文件 | 目标对象 | 状态 |
|---|---|---|---|
| `HttpRequest` | `server_request.rs` | `ServerRequest` | ⬜ |
| `HttpResponse` | `server_response.rs` | `ServerResponse` | ⬜ |
| `ServerWebExchange` | `server_web_exchange.rs` | `ServerWebExchange` | ⬜ |
| `WebHandler` | `web_handler.rs` | `WebHandler` | ⬜ |
| `WebFilter` | `web_filter.rs` | `WebFilter` | ⬜ |
| `HandlerMapping` | `handler_mapping.rs` | `HandlerMapping` | ⬜ |
| `HandlerAdapter` | `handler_adapter.rs` | `HandlerAdapter` | 🔶 |
| `HandlerResult` | `handler_result.rs` | `HandlerResult` | ⬜ |
| `HandlerMethod` | `handler_method.rs` | `HandlerMethod` | ⬜ |
| `HandlerMethodArgumentResolver` | `argument_resolver.rs` | `ArgumentResolver` | ⬜ |
| `HandlerMethodReturnValueHandler` | `return_value_handler.rs` | `ReturnValueHandler` | ⬜ |
| `WebExceptionHandler` | `web_exception_handler.rs` | `WebExceptionHandler` | ⬜ |
| `ResponseStatusException` | `response_status_exception.rs` | `ResponseStatusException` | ⬜ |
| `CorsConfiguration` | `cors_configuration.rs` | `CorsConfiguration` | ⬜ |
| `CorsProcessor` | `cors_processor.rs` | `CorsProcessor` | ⬜ |
| `LocaleContext` | `locale_context.rs` | `LocaleContext` | ⬜ |
| `RequestPath` | `request_path.rs` | `RequestPath` | ⬜ |
| `PathContainer` | `path_container.rs` | `PathContainer` | ⬜ |
| `WebUtils` | `web_utils.rs` | `WebUtils` | ⬜ |

## 四、Codec 与消息转换器

| Spring 类族 | 目标 Rust 文件 | 目标对象 | 状态 |
|---|---|---|---|
| `HttpMessageReader` | `http_message_reader.rs` | `HttpMessageReader<T>` | ⬜ |
| `HttpMessageWriter` | `http_message_writer.rs` | `HttpMessageWriter<T>` | ⬜ |
| `Decoder` | `decoder.rs` | `Decoder<T>` | ⬜ |
| `Encoder` | `encoder.rs` | `Encoder<T>` | ⬜ |
| `DecoderHttpMessageReader` | `decoder_http_message_reader.rs` | `DecoderHttpMessageReader` | ⬜ |
| `EncoderHttpMessageWriter` | `encoder_http_message_writer.rs` | `EncoderHttpMessageWriter` | ⬜ |
| `FormHttpMessageReader/Writer` | `form_codec.rs` | `FormCodec` | ⬜ |
| `MultipartHttpMessageReader/Writer` | `multipart_codec.rs` | `MultipartCodec` | ⬜ |
| `Jackson/Gson JSON` | `json_codec.rs` | `JsonDecoder/JsonEncoder` | ⬜ |
| `ResourceHttpMessageReader/Writer` | `resource_codec.rs` | `ResourceCodec` | ⬜ |
| `ServerSentEvent` | `server_sent_event.rs` | `ServerSentEvent<T>` | ⬜ |
| `ServerSentEventHttpMessageReader/Writer` | `sse_codec.rs` | `SseCodec` | ⬜ |
| `Protobuf*` | `protobuf_codec.rs` | `ProtobufCodec` | ⬜ |
| `Xml/Jaxb2*` | `xml_codec.rs` | `XmlCodec` | ⬜ |
| `CodecConfigurer` | `codec_configurer.rs` | `CodecConfigurer` | ⬜ |
| `DataBuffer` | `data_buffer.rs` | `DataBuffer` | ⬜ |
| `DataBufferFactory` | `data_buffer_factory.rs` | `DataBufferFactory` | ⬜ |

## 五、客户端

| Spring 类族 | 目标 Rust 文件 | 目标对象 | 状态 |
|---|---|---|---|
| `ClientHttpRequest` | `client_http_request.rs` | `ClientHttpRequest` | ⬜ |
| `ClientHttpResponse` | `client_http_response.rs` | `ClientHttpResponse` | ⬜ |
| `ClientHttpRequestFactory` | `client_http_request_factory.rs` | `ClientHttpRequestFactory` | ⬜ |
| `ClientHttpRequestExecution` | `client_http_request_execution.rs` | `ClientHttpRequestExecution` | ⬜ |
| `ClientHttpRequestInterceptor` | `client_http_request_interceptor.rs` | `ClientHttpRequestInterceptor` | ⬜ |
| `HttpAccessor` | `http_accessor.rs` | `HttpAccessor` | ⬜ |
| `RestClient`/`WebClient` 语义 | `client.rs` | `WebClient` | ⬜ |
| `RequestBodySpec`/`ResponseSpec` | `client_spec.rs` | typed request/response specs | ⬜ |
| JDK/Reactor/Jetty/HttpComponents implementations | `adapters/*` | transport adapters | 🚫/⬜ | 不在核心 crate 复制，分别作为 adapter crate |

## 六、Web 扩展对象

| 语义 | 目标 Rust 文件 | 目标对象 | 状态 |
|---|---|---|---|
| WebSocket handler/session | `websocket.rs` | `WebSocketSession`/`WebSocketHandler` | ⬜ |
| WebSocket handshake | `websocket_handshake.rs` | `WebSocketHandshake` | ⬜ |
| URI builder | `uri_builder.rs` | `UriBuilder` | ⬜ |
| URL/resource resolver | `resource.rs` | `Resource`/`ResourceResolver` | ⬜ |
| observation/logging | `web_observation.rs` | `WebObservationContext` | ⬜ |
| request attributes | `request_attributes.rs` | typed `RequestAttributes` | 🔶 |
| Multipart part | `part.rs` | `Part`/`FilePart`/`FormFieldPart` | ⬜ |

## 七、对象拆分规则

1. 每个公开 Spring 语义对象对应独立 `.rs` 文件；只有明确的 marker trait、错误 enum 或同一值对象的只读视图可合并。
2. `mod.rs`/`lib.rs` 只声明模块与 re-export，不承载业务逻辑。
3. 目录 snake_case，类型 PascalCase，方法 snake_case；Spring 缩写按既有 Rust 约定保留（`Http`、`URI`、`SSE`）。
4. Java 运行时实现（Servlet、Reactor、JDK client、Jackson）不得污染核心对象命名；使用 `adapters/` 与 feature crate。
5. 每个对象记录：Spring 全限定名、Rust 文件、功能语义、形态差异、测试状态和依赖。

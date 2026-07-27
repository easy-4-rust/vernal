# spring-web → vernal-web 功能语义迁移对照表

> 目标不是复制 Spring 的 Java 类型层级，而是保持外部可观察行为、生命周期、错误、扩展点和默认策略一致。状态：✅ 已有；🔶 部分已有；⬜ 待实现；🚫 不直接迁移。

## 一、HTTP 消息模型

| Spring 语义 | Vernal 设计 | 状态 | 对齐要求 |
|---|---|---|---|
| 方法与状态码 | `HttpMethod`、`HttpStatusCode`、`HttpStatus` | ⬜ | 扩展方法/任意 3 位状态码、系列判断、reason |
| MediaType | `MediaType` | ⬜ | 参数、质量因子、通配符、`includes`/兼容性 |
| Header | `HttpHeaders` | ⬜ | key 大小写不敏感、多值顺序、typed 访问器 |
| Cookie | `HttpCookie`、`ResponseCookie` | ⬜ | 请求 Cookie 与 Set-Cookie 分离、属性编码 |
| Entity | `HttpEntity<T>`、`RequestEntity<T>`、`ResponseEntity<T>` | ⬜ | body 可选、头快照、构建器语义 |
| Problem Detail | `ProblemDetail` 扩展 `ProblemDetails` | 🔶 | RFC 9457 type/title/status/detail/instance + extensions |
| Range/ETag/Cache | `HttpRange`、`ETag`、`CacheControl` | ⬜ | 条件请求、范围响应和缓存指令 |

## 二、请求生命周期

| Spring 语义 | Vernal 设计 | 状态 | 语义迁移要求 |
|---|---|---|---|
| 请求接收 | `ServerWebExchange` + `RequestContext` | 🔶 | method/URI/headers/body/attributes 一体化、异步可取消 |
| 路由匹配 | `HandlerMapping` + `RouteMetadata` | 🔶 | path/template、method、host、Accept/Content-Type、优先级 |
| Handler 调用 | `HandlerAdapter` + `HandlerInvocation` | 🔶 | typed handler、参数解析、返回值处理、错误边界 |
| 请求参数 | `ArgumentResolver`/`FromRequest` | ⬜ | path/query/header/cookie/body/form/multipart/exension |
| 返回值 | `ReturnValueHandler`/`IntoResponse` | ⬜ | status/headers/body/stream/empty response |
| 过滤器 | `WebFilter` | ⬜ | 顺序、短路、前后置、异常传播、上下文修改 |
| 异常处理 | `WebExceptionHandler` + `WebFailure` | 🔶 | 错误分类、状态码、ProblemDetail、日志策略 |
| 请求作用域 | `WebRequestScope` | ✅ | 创建、注入、取消、关闭、资源清理 |
| 请求 ID | `RequestId` | ✅ | 生成/透传/响应回写/观测关联 |

## 三、内容协商与消息转换

| Spring 语义 | Vernal 设计 | 状态 | 语义迁移要求 |
|---|---|---|---|
| Reader/Writer | `HttpMessageReader<T>`/`HttpMessageWriter<T>` | ⬜ | `can_read`/`can_write`、媒体类型选择、异步流 |
| Decoder/Encoder | `Decoder<T>`/`Encoder<T>` | ⬜ | 类型、媒体类型、流式 frame、编码错误 |
| JSON | `JsonCodec`（serde feature） | ⬜ | JSON body、未知字段策略、错误位置 |
| Text/Bytes | `TextCodec`/`BytesCodec` | ⬜ | charset、空体、长度和流式输出 |
| Form | `FormCodec` | ⬜ | URL encoded、重复 key、字符集 |
| Multipart | `MultipartCodec`/`Part` | ⬜ | boundary、文件流、大小限制、临时文件策略 |
| SSE | `SseCodec`/`ServerSentEvent<T>` | ⬜ | event/id/retry/comment/data、多行 data、flush |
| Protobuf/XML | 可选 feature codecs | ⬜ | 不进入核心默认依赖，行为由合同测试约束 |
| 默认 codecs | `CodecConfigurer` | ⬜ | 默认顺序、最大内存、日志敏感信息配置 |

## 四、客户端语义

| Spring 语义 | Vernal 设计 | 状态 | 语义迁移要求 |
|---|---|---|---|
| 创建请求 | `ClientHttpRequestFactory`/`WebClient` | ⬜ | method/URI/header/body builder |
| 执行链 | `ClientHttpRequestExecution` | ⬜ | 拦截器顺序、一次执行、取消 |
| 响应 | `ClientHttpResponse`/`ResponseEntity<T>` | ⬜ | status/header/body/错误状态 |
| body 编解码 | 共用 Reader/Writer/Codec | ⬜ | 客户端和服务端一致的协商规则 |
| 超时/重试 | client policy | ⬜ | connect/read/write/overall timeout；重试必须幂等可配置 |
| Transport | Hyper/Reqwest/平台 adapter | 🚫/⬜ | 核心不绑定具体 HTTP 客户端 |

## 五、协议与 Web 扩展

| Spring 语义 | Vernal 设计 | 状态 |
|---|---|---|
| WebSocket | `WebSocketSession`/`WebSocketHandler` | ⬜ |
| CORS | `CorsConfiguration`/`CorsProcessor` | ⬜ |
| URI | `UriBuilder`/`RequestPath` | ⬜ |
| 静态资源 | `Resource`/`ResourceResolver` | ⬜ |
| Range 下载 | `HttpRange` + resource writer | ⬜ |
| Observation | `WebObservationContext` | ⬜ |
| Servlet/WebFlux 专用生命周期 | runtime adapter | 🚫 |

## 六、与 vernal-expression 的集成语义

| 表达式能力 | Web 桥接 | 状态 | 说明 |
|---|---|---|---|
| 根对象 | `RequestContext` 作为 root | ⬜ | `#root` 可访问请求、路由和安全主体 |
| 属性访问 | `RequestPropertyAccessor` | ⬜ | `path/query/header/cookie/attribute/body` |
| Bean 引用 | `VernalBeanResolver` | ⬜ | 从 `vernal-context`/容器解析 Bean |
| 环境类型 | `EnvironmentTypeLocator` | ⬜ | 读取应用环境和 feature 配置 |
| handler 条件 | `ExpressionRoutePredicate` | ⬜ | 路由前置条件、权限和媒体类型判断 |
| 绑定错误 | `EvaluationError` → `WebFailure` | ⬜ | 不泄露内部类型/敏感值 |

## 七、错误与边界语义

必须覆盖：空 body、重复读取、非法 Header、非法 URI、Content-Type 缺失、不支持媒体类型、Accept 不匹配、body 超限、Multipart boundary 错误、handler panic/错误、客户端断开、取消、超时、连接重置、响应已提交后错误。

状态码默认映射：

| 错误 | 默认状态 |
|---|---:|
| 路由不存在 | 404 |
| 方法不允许 | 405 |
| 参数/格式绑定失败 | 400 |
| Accept 不可满足 | 406 |
| Content-Type 不支持 | 415 |
| 请求体过大 | 413 |
| 业务显式状态错误 | 指定状态 |
| 未处理内部错误 | 500 |
| 客户端取消/断开 | 不强行写响应，记录取消 |

## 八、迁移原则

1. **可观察语义优先**：状态码、Header、body、错误和生命周期必须稳定；Java 的类继承不是兼容目标。
2. **类型安全替代反射**：trait、泛型、derive 宏和显式 resolver 代替运行时反射。
3. **Stream 替代 Publisher**：背压、取消、结束和错误传播保持一致。
4. **核心与适配器隔离**：Hyper/Reqwest/Axum/Actix/Salvo 等不能改变核心合同。
5. **默认行为可配置但有确定默认值**：codec 顺序、body 限制、日志脱敏、超时和 CORS 都必须显式记录。

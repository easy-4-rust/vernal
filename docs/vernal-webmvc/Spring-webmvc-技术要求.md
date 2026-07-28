# vernal-webmvc 技术要求（对标 spring-webmvc）

> **版本**：v1.0（2026-07-28）
> **对标**：`spring-webmvc` 6.1 / Spring Framework 6.1
> **Rust 基线**：edition 2024 / rustc 1.88
> **crate 现状**：待建（规划中）
> **选型**：Topcoat MVC 模式

---

## 一、概述与定位

### 1.1 crate 职责

`vernal-webmvc` 是 Vernal Framework 的 **MVC 模式实现层**，对标 Spring Framework
中的 `spring-webmvc` 模块。它在 `vernal-web` 的框架中立合同之上，提供基于
Topcoat 的 MVC 编程模型，包括路由声明、请求映射、视图解析、模型管理和异常处理。

在 Vernal 双轨架构中，`vernal-webmvc` 属于**轨道一 Topcoat 全栈主线**：

```
Spring Boot 体验
  ┌─────────────────────────────────────────────────┐
  │  vernal-webmvc (本 crate)                        │
  │    ├─ DispatcherServlet  → Topcoat Router        │
  │    ├─ @RequestMapping    → Topcoat 路由 DSL       │
  │    ├─ ModelAndView       → Topcoat View           │
  │    ├─ ViewResolver       → Topcoat ViewEngine     │
  │    └─ HandlerExceptionResolver → 错误映射         │
  ├─────────────────────────────────────────────────┤
  │  vernal-web (框架中立合同)                        │
  │    ├─ RequestContext / WebRequestScope / ...       │
  ├─────────────────────────────────────────────────┤
  │  vernal-beans + vernal-context (IoC/AOP 内核)     │
  └─────────────────────────────────────────────────┘
```

### 1.2 与 spring-webmvc 的对齐边界

| spring-webmvc 概念 | vernal-webmvc 对应 | 说明 |
|:---|:---|:---|
| `DispatcherServlet` | `TopcoatRouter` + 中间件链 | 请求分发入口 |
| `@RequestMapping` | Topcoat 路由 DSL + `#[handler]` 宏 | 路由声明 |
| `HandlerMapping` | Topcoat 内置路由匹配 | 路由到 Handler 的映射 |
| `HandlerAdapter` | Topcoat Handler trait 适配 | 调用约定适配 |
| `ModelAndView` | `ViewContext` + `ViewEngine` | 模型数据 + 视图选择 |
| `ViewResolver` | `ViewResolver` trait | 逻辑视图名 → 物理视图 |
| `HandlerExceptionResolver` | `ExceptionHandler` trait | 异常 → 错误视图/响应 |
| `MultipartResolver` | Topcoat 原生 multipart | 不单独抽象 |
| `LocaleResolver` | `Accept-Language` 解析 | 国际化支持 |
| `ThemeResolver` | 不实现 | Rust Web 应用通常不需要 |

### 1.3 设计约束

1. **`#![forbid(unsafe_code)]`**：与 vernal-web 保持一致。
2. **Topcoat-first**：直接使用 Topcoat 的 Router、Middleware、Handler API。
3. **IoC 桥接**：Handler 通过 vernal-beans `Container` 解析，支持依赖注入。
4. **AOP 集成**：Handler 调用经过 vernal-aop 拦截链，支持 `@Transactional` 等。
5. **Send + Sync**：所有公共类型满足跨线程边界。

---

## 二、待建 crate 规划

### 2.1 目录结构

```
crates/vernal-webmvc/
├── Cargo.toml
└── src/
    ├── lib.rs                    # 入口
    ├── dispatcher.rs             # Topcoat Router 装配
    ├── handler_macro.rs          # #[handler] 过程宏（vernal-macros 中实现）
    ├── request_mapping.rs        # 路由映射元数据
    ├── view_context.rs           # 模型数据容器
    ├── view_engine.rs            # 视图引擎 trait
    ├── view_resolver.rs          # 视图解析器 trait
    ├── exception_handler.rs      # 异常处理器 trait
    ├── content_negotiation.rs    # 内容协商
    ├── model_attribute.rs        # 模型属性绑定
    ├── session_attribute.rs      # 会话属性
    ├── response_status.rs        # 响应状态注解
    ├── cross_origin.rs           # CORS 配置
    ├── async_handler.rs          # 异步 Handler 支持
    └── reactive_handler.rs       # 响应式 Handler（委托给 vernal-webflux）
```

### 2.2 Cargo.toml 依赖规划

```toml
[package]
name = "vernal-webmvc"
edition = "2024"
rust-version = "1.88"

[dependencies]
vernal-web = { path = "../vernal-web" }
vernal-aop = { path = "../vernal-aop" }
vernal-beans = { path = "../vernal-beans" }
vernal-context = { path = "../vernal-context" }
vernal-macros = { path = "../vernal-macros" }
http = "1.4.0"
http-body = "1.0.1"
http-body-util = "0.1.3"
tokio = { version = "1.52.4", features = ["rt-multi-thread", "sync"] }
serde = { version = "1.0.228", features = ["derive"] }
thiserror = "2.0"
tracing = "0.1.41"
```

### 2.3 实施阶段

| 阶段 | 内容 | 前置条件 |
|:---|:---|:---|
| P0 | trait 定义（ViewResolver、ExceptionHandler） | vernal-web 稳定 |
| P1 | Topcoat Router 装配 + `#[handler]` 宏 | Topcoat 0.5 API 稳定 |
| P2 | 视图解析 + 模型管理 | 视图引擎选型确认 |
| P3 | 内容协商 + CORS + 会话 | 基础 Handler 可运行 |
| P4 | 合同测试 + 文档 | 所有 trait 实现完成 |

---

## 三、选型与依赖

### 3.1 核心选型

引用 [Spring 组件替换约定](../Spring-组件替换约定.md) 第 4.1 节"双轨架构"。

| 用途 | crate / 技术 | 说明 |
|:---|:---|:---|
| MVC 框架 | Topcoat 0.5 | tokio-rs 官方全栈框架，MVC 模式 |
| 路由 DSL | Topcoat Router | 声明式路由注册 |
| 请求映射 | Topcoat 路由 + `#[handler]` 宏 | 类似 `@RequestMapping` |
| 视图引擎 | Topcoat ViewEngine | 模板渲染 |
| HTTP 类型 | `http` 1.4 | 框架无关 HTTP 类型 |
| IoC 容器 | vernal-beans | 组件解析与依赖注入 |
| AOP | vernal-aop | Handler 拦截链 |
| 过程宏 | vernal-macros | `#[handler]`、`#[RequestMapping]` 等 |

### 3.2 Topcoat MVC 模式映射

Topcoat 提供类似 Spring MVC 的编程模型：

| Spring MVC | Topcoat MVC | 说明 |
|:---|:---|:---|
| `@Controller` | `#[derive(Handler)]` 宏 | 标记为 Handler 组件 |
| `@RequestMapping("/api")` | `Router::new().route("/api", ...)` | 路由前缀 |
| `@GetMapping("/{id}")` | `Router::get("/:id", handler)` | HTTP 方法绑定 |
| `@PostMapping` | `Router::post("", handler)` | POST 路由 |
| `@RequestParam` | Handler 函数参数提取器 | 请求参数绑定 |
| `@RequestBody` | `Json<T>` 提取器 | 请求体反序列化 |
| `@ResponseBody` | `Json<T>` 响应 | 响应体序列化 |
| `@PathVariable` | 路径参数提取器 | 路径变量绑定 |
| `Model` | `ViewContext` | 模型数据传递 |
| `ModelAndView` | `ViewContext` + `ViewEngine` | 模型 + 视图选择 |

### 3.3 过程宏设计

`vernal-macros` 将提供以下宏，对标 Spring MVC 注解：

```rust
// 对标 @Controller + @RequestMapping
#[handler(prefix = "/api/users")]
impl UserController {
    // 对标 @GetMapping("/{id}")
    #[get("/{id}")]
    async fn get_user(&self, id: Path<i64>) -> JsonResult<User> {
        // ...
    }

    // 对标 @PostMapping
    #[post("")]
    async fn create_user(&self, body: Json<CreateUserRequest>) -> JsonResult<User> {
        // ...
    }

    // 对标 @PutMapping("/{id}")
    #[put("/{id}")]
    async fn update_user(
        &self,
        id: Path<i64>,
        body: Json<UpdateUserRequest>,
    ) -> JsonResult<User> {
        // ...
    }

    // 对标 @DeleteMapping("/{id}")
    #[delete("/{id}")]
    async fn delete_user(&self, id: Path<i64>) -> StatusCode {
        // ...
    }
}
```

### 3.4 不引入的方案

| 方案 | 原因 |
|:---|:---|
| 直接使用 Axum Router | 与 Topcoat 全栈主线冲突 |
| 自建 Servlet 容器 | 非目标，Rust 生态不需要 Servlet 模型 |
| 复制 Spring 的 `HandlerAdapter` | Topcoat 已有 Handler trait，无需再抽象一层 |

---

## 四、核心组件设计

### 4.1 DispatcherServlet → Topcoat Router

Spring MVC 的 `DispatcherServlet` 是请求分发的核心。在 Vernal 中，这个角色由
Topcoat Router + 中间件链承担：

```rust
/// MVC 路由装配器，对标 DispatcherServlet 的路由注册职责。
pub struct MvcRouter {
    router: topcoat::Router,
    app_context: Arc<ApplicationContext>,
}

impl MvcRouter {
    /// 创建 MVC 路由器。
    pub fn new(app_context: Arc<ApplicationContext>) -> Self {
        Self {
            router: topcoat::Router::new(),
            app_context,
        }
    }

    /// 注册 Handler 组件。
    ///
    /// 从 IoC 容器解析 Handler，将其路由信息注册到 Topcoat Router。
    pub fn register_handler<H>(&mut self, prefix: &str)
    where
        H: Handler + 'static,
    {
        let handler = self.app_context.container().resolve::<H>().unwrap();
        self.router.route(prefix, handler.into_topcoat());
    }

    /// 构建最终的 Topcoat 应用。
    pub fn build(self) -> topcoat::App {
        topcoat::App::new()
            .router(self.router)
            .middleware(VernalMiddleware::new(self.app_context))
    }
}
```

### 4.2 @RequestMapping → Topcoat 路由

Spring MVC 的 `@RequestMapping` 在 Vernal 中通过 Topcoat 路由 DSL 实现：

| Spring 注解 | Topcoat 路由 | 宏展开 |
|:---|:---|:---|
| `@RequestMapping("/api")` | `Router::new().nest("/api", sub_router)` | 路由前缀嵌套 |
| `@GetMapping("/{id}")` | `Router::get("/:id", handler)` | GET 路由 |
| `@PostMapping("")` | `Router::post("", handler)` | POST 路由 |
| `@PutMapping("/{id}")` | `Router::put("/:id", handler)` | PUT 路由 |
| `@DeleteMapping("/{id}")` | `Router::delete("/:id", handler)` | DELETE 路由 |
| `@PatchMapping("/{id}")` | `Router::patch("/:id", handler)` | PATCH 路由 |
| `consumes = "application/json"` | 内容协商中间件 | 请求体类型限制 |
| `produces = "application/json"` | `Accept` 头检查 | 响应类型协商 |

### 4.3 ModelAndView → ViewContext + ViewEngine

Spring MVC 的 `ModelAndView` 在 Vernal 中拆分为两个概念：

```rust
/// 视图上下文，对标 Spring MVC 的 Model。
///
/// 持有模板渲染所需的键值对数据。
pub struct ViewContext {
    view_name: Option<String>,
    model: HashMap<String, Box<dyn Any + Send + Sync>>,
    status: http::StatusCode,
    headers: http::HeaderMap,
}

impl ViewContext {
    /// 创建空的视图上下文。
    pub fn new() -> Self { /* ... */ }

    /// 设置逻辑视图名。
    pub fn view_name(mut self, name: impl Into<String>) -> Self {
        self.view_name = Some(name.into());
        self
    }

    /// 添加模型属性。
    pub fn attribute(mut self, key: impl Into<String>, value: impl Any + Send + Sync) -> Self {
        self.model.insert(key.into(), Box::new(value));
        self
    }

    /// 设置响应状态码。
    pub fn status(mut self, status: http::StatusCode) -> Self {
        self.status = status;
        self
    }
}

/// 视图引擎，对标 Spring MVC 的 View 接口。
///
/// 负责将 ViewContext 渲染为 HTTP 响应 Body。
#[async_trait]
pub trait ViewEngine: Send + Sync {
    /// 渲染视图。
    async fn render(&self, context: &ViewContext) -> Result<Vec<u8>, ViewError>;

    /// 返回支持的内容类型。
    fn content_type(&self) -> &str;
}
```

### 4.4 ViewResolver

```rust
/// 视图解析器，对标 Spring MVC 的 ViewResolver。
///
/// 将逻辑视图名解析为 ViewEngine 实例。
#[async_trait]
pub trait ViewResolver: Send + Sync {
    /// 解析逻辑视图名。
    async fn resolve_view(
        &self,
        view_name: &str,
        locale: &str,
    ) -> Result<Arc<dyn ViewEngine>, ViewResolutionError>;
}
```

内置实现：

| 实现 | 说明 |
|:---|:---|
| `TeraViewResolver` | 基于 `tera` 模板引擎 |
| `StaticViewResolver` | 静态 HTML 文件 |
| `JsonViewResolver` | JSON 响应（`serde_json`） |

### 4.5 HandlerExceptionResolver

```rust
/// 异常处理器，对标 Spring MVC 的 HandlerExceptionResolver。
///
/// 将 Handler 返回的错误转换为 HTTP 响应。
#[async_trait]
pub trait ExceptionHandler: Send + Sync {
    /// 处理异常。
    async fn handle_exception(
        &self,
        request: &RequestContext,
        error: &dyn std::error::Error,
    ) -> Result<http::Response<BoxBody>, ExceptionError>;
}
```

内置实现：

| 实现 | 对标 Spring | 说明 |
|:---|:---|:---|
| `ProblemDetailsHandler` | `ResponseEntityExceptionHandler` | RFC 9457 错误响应 |
| `ResponseStatusHandler` | `@ResponseStatus` 注解 | 根据注解返回状态码 |
| `ValidationExceptionHandler` | `MethodArgumentNotValidException` | 参数校验错误 |
| `NotFoundHandler` | `NoHandlerFoundException` | 404 处理 |

### 4.6 内容协商

```rust
/// 内容协商器，对标 Spring MVC 的 ContentNegotiationManager。
pub struct ContentNegotiation {
    /// 默认内容类型。
    default_content_type: mime::Mime,
    /// 支持的内容类型列表。
    supported_types: Vec<mime::Mime>,
    /// 是否使用 URL 后缀协商。
    favor_path_extension: bool,
    /// 是否使用 Accept 头协商。
    favor_parameter: bool,
}
```

---

## 五、IoC/AOP 集成

### 5.1 Handler 组件注册

`vernal-webmvc` 的 Handler 通过 IoC 容器注册和解析，对标 Spring MVC 的
`@Controller` 组件扫描：

```rust
// 1. 定义 Handler 组件
#[derive(Component)]
#[component(scope = Singleton)]
struct UserController {
    user_service: Arc<UserService>, // IoC 注入
}

// 2. 实现 MVC Handler
#[handler(prefix = "/api/users")]
impl UserController {
    #[get("/{id}")]
    async fn get_user(
        &self,
        id: Path<i64>,
        scope: &WebRequestScope,
    ) -> JsonResult<User> {
        let user = self.user_service.find_by_id(id.0).await?;
        JsonResult::ok(user)
    }
}

// 3. 注册到 MVC Router
let mut mvc = MvcRouter::new(app_context);
mvc.register_handler::<UserController>("/api/users");
```

### 5.2 AOP 拦截链

每个 Handler 方法的调用都经过 vernal-aop 拦截链，对标 Spring MVC 的
`HandlerInterceptor`：

```
请求到达
  → Topcoat Router 匹配路由
  → Vernal 中间件创建 RequestContext + WebRequestScope
  → AOP 拦截链（安全 → 事务 → 审计 → 可观测性）
    → before: 安全检查（Sa-Token-Rust）
    → before: 事务开始
    → around: Handler 方法执行
    → after: 事务提交/回滚
    → after: 审计记录
  → 视图解析（如果返回 ViewContext）
  → 响应发送
  → WebRequestScope 关闭
```

### 5.3 路由元数据填充

Topcoat 路由匹配后，vernal-webmvc 中间件填充 `RouteMetadata`：

```rust
// Topcoat 路由匹配后的元数据
let path_template = matched_route.path_template(); // "/api/users/:id"
let http_method = request.method();                 // GET
let handler_name = "UserController::get_user";      // Handler 名称

// 构建 RouteMetadata
let route = RouteMetadata::new(
    handler_name,
    format!("{} {}", http_method, path_template),
    path_template,
);

// 创建 RequestContext
let ctx = RequestContext::new(route, cancellation_token);
```

### 5.4 异步 Handler 支持

Spring MVC 5+ 支持 `Callable`、`DeferredResult` 和 `WebAsyncTask` 异步返回。
vernal-webmvc 通过 Rust 原生 async/await 实现：

```rust
// 同步 Handler（Rust 中实际是 async）
#[get("/{id}")]
async fn get_user(&self, id: Path<i64>) -> JsonResult<User> {
    let user = self.service.find(id.0).await?;
    JsonResult::ok(user)
}

// 并发 Handler（对标 Spring 的 @Async）
#[get("/dashboard")]
async fn dashboard(&self) -> JsonResult<Dashboard> {
    let (user, orders, notifications) = tokio::join!(
        self.user_service.current_user(),
        self.order_service.recent_orders(),
        self.notification_service.unread(),
    );
    JsonResult::ok(Dashboard { user?, orders?, notifications? })
}

// 流式 Handler（委托给 vernal-webflux）
#[get("/events")]
async fn events(&self) -> SseStream<Event> {
    // 由 vernal-webflux 的流式支持处理
    self.event_service.subscribe().into_sse()
}
```

---

## 六、集成规范与引用约定

### 6.1 引用约定

本文档引用 [Spring 组件替换约定](../Spring-组件替换约定.md) 第 4.1 节"双轨架构"。

具体引用条目：

- **4.1 双轨架构**：vernal-webmvc 属于轨道一 Topcoat 全栈主线。
- **4.2 轨道一 Topcoat 全栈主线**：vernal-webmvc 对标 `spring-webmvc`，提供
  Topcoat MVC 集成。
- **4.4 传输层基础 crate**：`http` 1.4、`tower` 0.5。

### 6.2 与兄弟 crate 的关系

```
vernal-webmvc
  ├── 依赖 vernal-web    (框架中立合同)
  ├── 依赖 vernal-aop    (AOP 拦截链)
  ├── 依赖 vernal-beans  (IoC 容器)
  ├── 依赖 vernal-context (ApplicationContext)
  ├── 依赖 vernal-macros (#[handler] 等过程宏)
  ├── 被 vernal-webflux  继承响应式扩展
  └── 被 vernal-web-testkit 测试
```

### 6.3 与轨道二适配器的关系

vernal-webmvc **不替代**轨道二的 10 个 API 后端适配器。两者的关系是：

| 维度 | 轨道一 vernal-webmvc | 轨道二适配器 |
|:---|:---|:---|
| 目标用户 | 新项目，选择 Topcoat 全栈 | 已有项目，保留现有框架 |
| 编程模型 | MVC（Handler + View + Model） | API 后端（JSON 响应） |
| 视图支持 | 完整视图解析 | 不涉及 |
| 路由声明 | `#[handler]` 宏 + Topcoat DSL | 框架原生路由 |
| IoC 集成 | 深度集成，Handler 是 IoC 组件 | 适配层集成 |
| 选型依据 | 约定 4.2 | 约定 4.3 |

### 6.4 测试策略

| 测试类别 | 覆盖内容 |
|:---|:---|
| 单元测试 | ViewContext、ContentNegotiation 等独立组件 |
| 集成测试 | 完整 MVC 请求链：路由 → Handler → 视图 → 响应 |
| 合同测试 | 复用 `vernal-web-testkit` 的公共合同套件 |
| 编译测试 | `#![forbid(unsafe_code)]` + 宏展开正确性 |
| 文档测试 | 公共 API 示例可编译运行 |

### 6.5 视图引擎选型待决

| 候选 | crate | 状态 | 说明 |
|:---|:---|:---|:---|
| Tera | `tera` | `[待集成]` | Jinja2 风格模板，vernal-context-support 已使用 |
| Askama | `askama` | `[待验证]` | 编译期模板，类型安全 |
| Maud | `maud` | `[待验证]` | Rust 原生 HTML 模板 |
| 直接 JSON | `serde_json` | `[已确认]` | API 后端默认响应格式 |

### 6.6 后续演进

| 阶段 | 内容 | 前置条件 |
|:---|:---|:---|
| Phase 5 | trait 定义 + 骨架 crate | vernal-web 稳定 |
| Phase 6 | Topcoat Router 装配 + `#[handler]` 宏 | Topcoat 0.5 API 稳定 |
| Phase 6 | 视图解析 + 模型管理 | 视图引擎选型确认 |
| Phase 7 | 内容协商 + CORS + 会话 | 基础 Handler 可运行 |
| Phase 8 | 合同测试 + 性能基准 | 所有 trait 实现完成 |

---

> **文档结束** — vernal-webmvc 是 Vernal Web 层的 MVC 模式实现，基于 Topcoat
> 全栈框架提供 Spring MVC 风格的编程模型。选型依据参见
> [Spring 组件替换约定](../Spring-组件替换约定.md) 第 4.1 节。

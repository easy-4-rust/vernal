# vernal-test 技术要求（对标 spring-test）

> **版本**：v1.0（2026-07-28）
> **定位**：vernal-test crate 技术交接文档，对标 Spring Framework 7.0.8 spring-test。
> **主线**：testcontainers 0.27 为容器化集成测试主线。
> **现状**：2 文件 / 30 行骨架，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》8.6 节。

---

## 一、总览

### 1.1 定位与边界

vernal-test 是 Vernal Framework 的 **测试基础设施层**，
对标 spring-test 模块，提供集成测试、Mock 测试和应用上下文测试能力。

| 维度 | spring-test（语义参考） | vernal-test（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（注解 + 反射） | Rust（属性宏 + trait） | 编译期宏替代反射 |
| 容器测试 | `@SpringBootTest` | `#[spring_test]` 宏 | 属性宏启动完整 Context |
| Mock 测试 | `MockMvc` | `axum::Router::oneshot` | Rust 原生 HTTP 测试 |
| 属性覆盖 | `@TestPropertySource` | `#[test_property_source]` 宏 | 编译期属性注入 |
| 集成容器 | 无（依赖外部 DB） | testcontainers 0.27 | 容器化依赖服务 |
| 测试上下文 | `TestContext` / `TestContextManager` | `TestContext` | 简化版 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  用户层：#[spring_test] 属性宏                         │
│  → 自动启动 ApplicationContext → 注入依赖 → 执行测试   │
├─────────────────────────────────────────────────────┤
│  Mock 层：axum::Router::oneshot                      │
│  → 无需启动真实 HTTP 服务器即可测试路由                 │
├─────────────────────────────────────────────────────┤
│  容器层：testcontainers 0.27                          │
│  → PostgreSQL / Redis / Kafka 等容器化依赖            │
├─────────────────────────────────────────────────────┤
│  上下文层：TestContext                                │
│  → 测试名称 + ApplicationContext 生命周期管理         │
├─────────────────────────────────────────────────────┤
│  属性层：TestPropertySource                           │
│  → 覆盖 application.yml 中的配置项                    │
└─────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 容器化测试 | testcontainers 0.27 | Rust 生态最成熟的容器化测试库 |
| Mock HTTP | axum::Router::oneshot | axum 原生支持，无需额外依赖 |
| 测试宏 | `#[spring_test]` 属性宏 | 对标 `@SpringBootTest`，编译期织入 |
| 属性覆盖 | `#[test_property_source]` | 编译期注入，零运行时开销 |
| 测试框架 | tokio::test 为底层 | Rust 异步测试标准 |

### 1.4 命名映射

| Spring 原名 | vernal-test 移植名 | 说明 |
|:---|:---|:---|
| `@SpringBootTest` | `#[spring_test]` | 属性宏 |
| `MockMvc` | `axum::Router::oneshot` | axum 原生 |
| `@TestPropertySource` | `#[test_property_source]` | 属性宏 |
| `TestContext` | `TestContext` | 直接对标 |
| `TestContextManager` | 不迁移 | Rust 无等价需求 |
| `@Sql` | 不迁移 | 由 testcontainers 承载 |
| `@Transactional` | vernal-tx 提供 | 跨 crate 复用 |

---

## 二、核心 Trait 体系

### 2.1 TestContext —— 测试上下文

**来源**：vernal-test 现有实现。
**语义参照**：spring-test `TestContext`。

#### Spring API（Java）

```java
// spring-test 测试上下文
public interface TestContext {
    ApplicationContext getApplicationContext();
    Class<?> getTestClass();
    Method getTestMethod();
    Throwable getTestException();
}
```

#### Rust 实现

```rust
/// 测试上下文。对标 Spring `TestContext`。
/// 提供测试环境下的 ApplicationContext 管理。
pub struct TestContext {
    /// 测试名称。
    name: String,
}

impl TestContext {
    /// 创建测试上下文。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self;

    /// 获取测试名称。
    #[must_use]
    pub fn name(&self) -> &str;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| 生命周期 | 独立于 ApplicationContext | 测试上下文是轻量容器 |
| 名称 | 必须提供 | 用于日志和诊断 |

#### 待补齐

- [x] `TestContext` 基本实现
- [ ] `TestContext::application_context()` 方法（P0）
- [ ] `TestContext::test_class()` 方法（P1）
- [ ] `TestContext::test_method()` 方法（P1）

---

### 2.2 ApplicationContext 测试启动（待建）

**语义参照**：spring-test `@SpringBootTest`。

#### Spring API（Java）

```java
// spring-test 应用上下文测试
@SpringBootTest
class UserServiceTest {
    @Autowired
    private UserService userService;

    @Test
    void testSave() { ... }
}
```

#### Rust 宏设计稿

```rust
/// 启动完整 ApplicationContext 并注入依赖。
/// 对标 @SpringBootTest。
#[spring_test]
async fn test_save_user(#[inject] user_service: Arc<UserService>) {
    let result = user_service.save(user).await;
    assert!(result.is_ok());
}
```

#### 宏展开伪代码

```rust
// #[spring_test] 展开为：
#[tokio::test]
async fn test_save_user() {
    let ctx = ApplicationContext::builder()
        .scan_base_packages(&["my_app"])
        .build()
        .await
        .expect("failed to build ApplicationContext");

    let user_service: Arc<UserService> = ctx.get::<UserService>();
    test_save_user_inner(user_service).await;
}

async fn test_save_user_inner(#[inject] user_service: Arc<UserService>) {
    // 用户原始测试逻辑
}
```

#### 待补齐

- [ ] `#[spring_test]` 属性宏（P0）
- [ ] `#[inject]` 参数宏（P0）
- [ ] trybuild 负例测试（P0）

---

## 三、Mock 测试

### 3.1 MockMvc → axum::Router::oneshot

**语义参照**：spring-test `MockMvc`。

#### Spring API（Java）

```java
// spring-test MockMvc 测试
@SpringBootTest
class UserControllerTest {
    @Autowired
    private MockMvc mockMvc;

    @Test
    void testGetUser() throws Exception {
        mockMvc.perform(get("/users/1"))
            .andExpect(status().isOk())
            .andExpect(jsonPath("$.name").value("Alice"));
    }
}
```

#### Rust 实现（axum 原生）

```rust
/// 使用 axum::Router::oneshot 测试 HTTP 路由。
/// 无需启动真实 HTTP 服务器。
#[tokio::test]
async fn test_get_user() {
    let app = Router::new()
        .route("/users/{id}", get(get_user));

    let request = Request::builder()
        .uri("/users/1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user: User = serde_json::from_slice(&body).unwrap();
    assert_eq!(user.name, "Alice");
}
```

#### 对比表

| 特性 | Spring MockMvc | axum::Router::oneshot |
|:---|:---|:---|
| 启动方式 | 自动启动 DispatcherServlet | 直接调用 Router |
| 请求构造 | `MockHttpServletRequestBuilder` | `Request::builder()` |
| 响应断言 | `ResultActions.andExpect()` | 直接 `assert_eq!` |
| JSON 断言 | `jsonPath()` | `serde_json::from_slice()` |
| 异步支持 | 需要 `AsyncMockMvc` | 原生 async |
| 依赖 | Spring MVC | axum（已依赖） |

#### 待补齐

- [ ] `MockRequestBuilder` 辅助工具（P1）
- [ ] `assert_json_eq!` 宏（P2）
- [ ] `assert_status!` 宏（P2）

---

### 3.2 MockMvc 辅助工具设计（待建）

```rust
/// HTTP 测试辅助构建器。简化 axum::Router::oneshot 的使用。
pub struct MockRequestBuilder {
    method: Method,
    uri: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl MockRequestBuilder {
    pub fn get(uri: &str) -> Self;
    pub fn post(uri: &str) -> Self;
    pub fn put(uri: &str) -> Self;
    pub fn delete(uri: &str) -> Self;
    pub fn header(mut self, name: &str, value: &str) -> Self;
    pub fn json_body(mut self, body: &impl Serialize) -> Self;
    pub fn build(self) -> Request<Body>;
}

/// HTTP 响应断言辅助。
pub struct MockResponseAssert {
    response: Response<Body>,
}

impl MockResponseAssert {
    pub fn assert_status(self, expected: StatusCode) -> Self;
    pub async fn assert_json<T: DeserializeOwned>(self) -> T;
    pub fn assert_header(self, name: &str, value: &str) -> Self;
}
```

---

## 四、容器化集成测试

### 4.1 testcontainers 0.27 选型

**语义参照**：spring-test `@Sql` + 外部数据库。

#### 选型依据

| 候选 | 版本 | 优势 | 劣势 | 决策 |
|:---|:---|:---|:---|:---|
| testcontainers-rs | 0.27 | Rust 生态最成熟，Docker 原生 | 需要 Docker | ✅ 选定 |
| docker-rs | 0.7 | 轻量 | 功能少，社区小 | ❌ |
| 手动 Docker | - | 完全控制 | 维护成本高 | ❌ |

#### Cargo 依赖

```toml
[dev-dependencies]
testcontainers = "0.27"
testcontainers-modules = "0.11"  # 预定义容器模块
```

### 4.2 PostgreSQL 容器化测试

```rust
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn test_user_repository() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@localhost:{port}/postgres");

    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let repo = UserRepository::new(pool);
    let user = repo.find_by_id(1).await.unwrap();
    assert!(user.is_some());
}
```

### 4.3 Redis 容器化测试

```rust
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::Redis;

#[tokio::test]
async fn test_cache_service() {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let url = format!("redis://localhost:{port}");

    let client = redis::Client::open(url).unwrap();
    let cache = CacheService::new(client);

    cache.set("key", "value").await.unwrap();
    let result = cache.get("key").await.unwrap();
    assert_eq!(result, Some("value".to_string()));
}
```

### 4.4 Kafka 容器化测试

```rust
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::kafka::Kafka;

#[tokio::test]
async fn test_message_producer() {
    let container = Kafka::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(9093).await.unwrap();
    let bootstrap = format!("localhost:{port}");

    let producer = MessageProducer::new(&bootstrap);
    producer.send("topic", b"hello").await.unwrap();
}
```

### 4.5 容器化测试辅助 trait

```rust
/// 容器化测试辅助 trait。
/// 封装 testcontainers 的通用模式。
pub trait ContainerTest: Send + Sync {
    /// 容器类型。
    type Container;

    /// 启动容器。
    async fn start_container(&self) -> Self::Container;

    /// 获取连接 URL。
    fn connection_url(container: &Self::Container) -> String;

    /// 执行测试。
    async fn run_test<F, Fut>(&self, test: F)
    where
        F: FnOnce(Self::Container) -> Fut + Send,
        Fut: Future<Output = ()> + Send;
}
```

---

## 五、属性覆盖与配置

### 5.1 TestPropertySource —— 测试属性源

**语义参照**：spring-test `@TestPropertySource`。

#### Spring API（Java）

```java
// spring-test 属性覆盖
@SpringBootTest
@TestPropertySource(properties = {
    "spring.datasource.url=jdbc:h2:mem:test",
    "spring.datasource.driver-class-name=org.h2.Driver"
})
class UserServiceTest { ... }
```

#### Rust 宏设计稿

```rust
/// 覆盖测试环境的配置属性。
/// 对标 @TestPropertySource。
#[spring_test]
#[test_property_source(
    database_url = "postgres://localhost:5432/test",
    redis_url = "redis://localhost:6379"
)]
async fn test_with_custom_config(#[inject] user_service: Arc<UserService>) {
    // 测试逻辑使用覆盖后的配置
}
```

#### 实现机制

```rust
// #[test_property_source] 宏展开伪代码：
// 1. 生成环境变量设置代码
// 2. 在测试前设置，测试后恢复
// 3. 与 #[spring_test] 协作

#[tokio::test]
async fn test_with_custom_config() {
    let _guard = EnvGuard::set(&[
        ("DATABASE_URL", "postgres://localhost:5432/test"),
        ("REDIS_URL", "redis://localhost:6379"),
    ]);

    let ctx = ApplicationContext::builder()
        .build()
        .await
        .unwrap();

    let user_service = ctx.get::<UserService>();
    test_with_custom_config_inner(user_service).await;
}
```

#### 待补齐

- [ ] `#[test_property_source]` 属性宏（P1）
- [ ] `EnvGuard` RAII 环境变量守卫（P1）
- [ ] trybuild 负例测试（P1）

---

### 5.2 配置文件覆盖

| 场景 | Spring 方式 | vernal-test 方式 |
|:---|:---|:---|
| 单属性覆盖 | `@TestPropertySource(properties=...)` | `#[test_property_source(...)]` |
| 文件覆盖 | `@TestPropertySource(locations=...)` | `#[test_property_source(file = "...")]` |
| 环境变量 | `@DynamicPropertySource` | `EnvGuard::set()` |
| Profile | `@ActiveProfiles("test")` | `#[active_profile("test")]`（待建） |

---

## 六、运行时与集成约束

### 6.1 运行时核心对象

| 对象 | 说明 | spring-test 对应 |
|:---|:---|:---|
| `TestContext` | 测试上下文（名称 + Context 生命周期） | `TestContext` |
| `#[spring_test]` | 应用上下文测试宏 | `@SpringBootTest` |
| `#[test_property_source]` | 属性覆盖宏 | `@TestPropertySource` |
| `MockRequestBuilder` | HTTP 测试请求构建器（待建） | `MockHttpServletRequestBuilder` |
| `MockResponseAssert` | HTTP 响应断言（待建） | `ResultActions` |
| `EnvGuard` | RAII 环境变量守卫（待建） | `@DynamicPropertySource` |
| `ContainerTest` | 容器化测试辅助 trait（待建） | `@Sql` + 外部 DB |

### 6.2 与 spring-test 概念的不移植项

| spring-test 概念 | 不移植原因 | vernal-test 替代 |
|:---|:---|:---|
| `TestContextManager` | Rust 无 JUnit 生命周期 | `#[spring_test]` 宏编译期处理 |
| `@BeforeTransaction` / `@AfterTransaction` | vernal-tx 提供 | 跨 crate 复用 |
| `@Sql` / `@SqlConfig` | testcontainers 更优 | 容器化数据库 |
| `@Repeat` / `@Timed` | Rust 测试框架内置 | `#[tokio::test]` + 循环 |
| `WebDriver` | Rust 生态有 headless-chrome | 不在 vernal 范围 |
| `HtmlUnit` | Rust 生态无等价物 | 不迁移 |
| `MockRestServiceServer` | axum::Router::oneshot 替代 | 原生 HTTP 测试 |

### 6.3 引用约定文档

crate 选型依据、集成模式和硬约束见《Spring 组件替换约定》：

- **8.6 节（测试）**：vernal-test 对标 spring-test，testcontainers 0.27 对标
  外部数据库依赖，axum::Router::oneshot 对标 MockMvc。

### 6.4 待补齐工作路线图

| 阶段 | 内容 | 优先级 | 预估工作量 |
|:---|:---|:---|:---|
| S1 | `#[spring_test]` 宏 + `#[inject]` | P0 | 3 天 |
| S1 | `TestContext` 扩展（application_context 方法） | P0 | 1 天 |
| S2 | `#[test_property_source]` 宏 + `EnvGuard` | P1 | 2 天 |
| S2 | `MockRequestBuilder` + `MockResponseAssert` | P1 | 2 天 |
| S3 | `ContainerTest` trait + PostgreSQL/Redis 示例 | P1 | 2 天 |
| S3 | `#[active_profile]` 宏 | P2 | 1 天 |

#### S1 #[spring_test] 宏

```rust
// vernal-macros 中实现
#[proc_macro_attribute]
pub fn spring_test(args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. 解析参数（base_packages, profile 等）
    // 2. 解析函数签名，提取 #[inject] 参数
    // 3. 生成 #[tokio::test] 包装
    // 4. 生成 ApplicationContext 构建代码
    // 5. 生成依赖注入代码
    // 6. 调用用户原始测试函数
}
```

### 6.5 测试基线

#### 已完成

| 测试内容 | 数量 | 说明 |
|:---|:---|:---|
| `TestContext` 基本功能 | 2 | 创建、名称获取 |
| 总计 | 2 | |

#### 待做

| 测试内容 | 预估数量 |
|:---|:---|
| `#[spring_test]` 宏 + trybuild | 10+ |
| `#[test_property_source]` 宏 + trybuild | 6+ |
| `MockRequestBuilder` | 5+ |
| `ContainerTest` PostgreSQL | 3+ |
| `ContainerTest` Redis | 3+ |

### 6.6 成熟度状态

| 维度 | 当前 | 目标 |
|:---|:---|:---|
| 文件数 | 2 | 12+ |
| 行数 | 30 | 600+ |
| 测试宏 | 0 | 2（`#[spring_test]` / `#[test_property_source]`） |
| Mock 工具 | 0 | 2（`MockRequestBuilder` / `MockResponseAssert`） |
| 容器化支持 | 0 | 3+（PostgreSQL / Redis / Kafka） |
| 与 spring-test 语义对标度 | ~10% | 70%+ |
| 测试数 | 2 | 40+ |

---

## 附录：spring-test 语义覆盖全景

| spring-test 包 | 类数 | vernal-test 状态 | 说明 |
|:---|:---|:---|:---|
| 根包（TestContext/Bootstrap） | 20 | ⬜ 部分 | TestContext 已有骨架 |
| context（上下文测试） | 15 | ⬜ S1 待建 | `#[spring_test]` 宏 |
| web（Web 测试） | 25 | ⬜ S2 待建 | axum::Router::oneshot |
| annotation（注解） | 10 | ⬜ S2 待建 | `#[test_property_source]` |
| jdbc（SQL 测试） | 8 | 🚫 testcontainers 替代 | 容器化数据库 |
| web.servlet（MockMvc） | 15 | ⬜ S2 待建 | axum 原生替代 |
| web.reactive（WebTestClient） | 10 | 🚫 不迁移 | axum 原生替代 |
| junit（JUnit 集成） | 8 | 🚫 不迁移 | Rust 测试框架内置 |

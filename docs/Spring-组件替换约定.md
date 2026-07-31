<!-- migration-doc: authority=support canonical=迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [迁移验收规范.md](迁移验收规范.md) 和自动审计报告为准。

# Spring 组件替换约定

> **版本**：v1.5（2026-07-28）  
> **定位**：Vernal Framework 项目的**单一权威选型字典**。所有技术要求文档、架构设计、  
> 实施计划和 crate README 均引用本文档作为 Rust 生态 crate 选型的最终依据。  
> **适用范围**：vernal-framework workspace + ddd4r 业务框架 + hutool-rust 工具库 +  
> sa-token-rust 鉴权框架。

---

## 一、定位与使用方式

### 1.0 组件替换不是对象豁免

本文只决定 Rust 生态组件选型。Spring 对象是否可以不建立本地文件，必须另行满足
[迁移验收规范](迁移验收规范.md)中的 `DEPENDENCY_REUSED`：

- 记录 Java FQN、依赖 crate、固定版本/提交和精确源码符号；
- `Cargo.toml` 中存在真实依赖；
- 集成测试证明接口、顺序、错误与生命周期语义；
- “生态已有”“功能相似”或仅有同名类型均不能作为豁免证据。

`aspect-rs` 特别遵循此规则：它为 `vernal-aop` 提供可复用基础切面能力，
但 `spring-aop` 仍是 Advice、Interceptor、Advisor 与自动代理的结构和语义主线。

### 1.1 与现有文档的关系

| 文档 | 关系 |
|:---|:---|
| `Vernal-Architecture.md` | 架构设计文档，定义系统边界和分层规则。本文档为其提供 crate 选型依据。 |
| `Vernal-Web-Architecture.md` | Web 层架构文档，定义双轨架构和适配器矩阵。本文档第四节与其对齐。 |
| `Vernal-Complete-Integration-Plan.md` | 集成计划文档，按阶段实施。本文档为其每个阶段的 crate 选型负责。 |
| `Vernal-Target-Architecture.md` | 目标架构文档，定义最终态。本文档定义达到最终态所需的全部 crate。 |
| `web-integration-manifest.toml` | Web 适配器清单，记录 10 个 HTTP 框架的集成状态。本文档第四节与其同步。 |
| 各 crate `Cargo.toml` | 代码级依赖声明。本文档是其选型决策的上游权威。 |

**使用方式**：需要选型决策时，先查本文档对应章节；本文档未覆盖的场景，标记为"未决项"  
并提交到第十节。

### 1.2 状态图例

| 标签 | 含义 |
|:---|:---|
| `[已确认]` | 已在 vernal-framework workspace 的 `Cargo.toml` 或运行时验证 |
| `[已验证]` | 已在 ddd4r / hutool-rust / sa-token-rust 中实际使用并验证 |
| `[待集成]` | crate 已选定，尚未集成到 vernal 或下游项目 |
| `[待验证]` | crate 已选定，需要 POC 或基准验证 |
| `[暂缓]` | 明确推迟，记录原因和恢复条件 |
| `[不采用]` | 评估后明确不采用，记录原因 |

---

## 二、选型原则

### 2.1 优先级排序

选型时按以下优先级从高到低依次评估：

1. **生态成熟度**：crate 下载量、维护活跃度、生产环境使用案例、文档质量
2. **异步原生**：优先选择 Tokio-first 的 crate，避免阻塞运行时桥接
3. **Spring 语义对齐**：优先选择与 Spring 对应组件语义一致的 crate，降低迁移成本
4. **`forbid unsafe` 友好**：vernal workspace 声明 `unsafe_code = "forbid"`，优先选择  
   不暴露 unsafe 接口或提供 safe wrapper 的 crate
5. **已用优先**：已在 vernal / ddd4r / hutool-rust / sa-token-rust 中使用的 crate  
   优先于同等条件的替代品

### 2.2 硬约束

- vernal workspace 最低 Rust 版本：**edition 2024 / rustc 1.88**
- 所有 vernal crate 设置 `publish = false`，不向 crates.io 发布
- vernal 内核不得反向依赖具体 Web、ORM、鉴权或配置实现
- 消费方（ddd4r / hutool-rust / sa-token-rust）拥有自己的 Bridge/Starter crate

---

## 三、集成模式

Vernal 生态采用四种集成模式，每种模式对应不同的 crate 组织方式：

### 模式 A：Trait + 多实现

vernal 定义抽象 trait，多个下游 crate 各自提供实现。

**适用场景**：Web 框架适配、消息 broker 适配、缓存后端  
**示例**：

```
vernal-web (trait: Handler, Filter, ...)
  ├── vernal-axum      (Axum 实现)
  ├── vernal-actix-web (Actix Web 实现)
  ├── vernal-salvo     (Salvo 实现)
  └── ... (共 10 个适配器)
```

### 模式 B：直接封装

vernal 直接封装一个 Rust crate，提供 Spring 语义的 API。

**适用场景**：只有一个主流实现的领域  
**示例**：

```
vernal-cache  → moka（封装 moka::sync::Cache 提供 Spring Cache 语义）
vernal-rbdc   → 通用数据库抽象层（对标 spring-jdbc JdbcTemplate，基于 sqlx）
vernal-rbatis → rbatis + rbdc 4.9.10 独立整合（rbatis 专属，不作为 vernal-rbdc 底层）
vernal-db     → sqlx（底层连接池实现，vernal-rbdc 的底层）
vernal-log    → tracing（封装 tracing 提供 SLF4J 语义）
```

### 模式 C：SPI 插件

vernal 定义 Service Provider Interface，通过 `inventory` 或 `linkme` 实现分布式注册。

**适用场景**：组件发现、表达式函数注册、AOP 切面注册  
**示例**：

```
vernal-context-indexer → linkme 分布式 slice 注册
vernal-expression      → inventory 函数注册
```

### 模式 D：全栈框架整合

vernal 提供全栈 Web 框架整合层，将 IoC + AOP + Web + 模板引擎打包。

**适用场景**：对标 Spring Boot 的全栈开发体验  
**示例**：

```
vernal-web     → Topcoat 全栈主线（tokio-rs 官方全栈框架）
vernal-webmvc  → Topcoat MVC 模式
vernal-webflux → Topcoat 响应式模式
```

---

## 四、Web/HTTP 层

### 4.1 双轨架构

Vernal Web 层采用**双轨架构**：

- **轨道一：Topcoat 全栈主线** — vernal-web / vernal-webmvc / vernal-webflux，对标  
  Spring Boot 的一站式开发体验
- **轨道二：10 个 API 后端适配器** — 为已有项目提供 IoC + AOP 集成，不要求切换框架

### 4.2 轨道一：Topcoat 全栈主线

| 对标 Spring | vernal crate | 说明 |
|:---|:---|:---|
| `spring-web` | `vernal-web` | 框架无关的 Web 合约（请求上下文、Scope、Handler 抽象） |
| `spring-webmvc` | `vernal-webmvc` | Topcoat MVC 集成（同步 + 异步 Handler、视图解析） |
| `spring-webflux` | `vernal-webflux` | Topcoat 响应式集成（Streaming、SSE、WebSocket） |

**选型依据**：Topcoat 是 tokio-rs 官方全栈 Web 框架，2026-07-22 发布，原生 Tokio、  
Tower 生态兼容、异步优先。vernal-web 已有的 trait 抽象可直接桥接到 Topcoat。

### 4.3 轨道二：API 后端适配器矩阵

| 优先级 | 框架 | vernal crate | 上游 crate | 版本 | 协议 |
|:---:|:---|:---|:---|:---|:---|
| 1 | Axum | `vernal-axum` | `axum` | 0.8.9 | HTTP |
| 2 | Actix Web | `vernal-actix-web` | `actix-web` | 4.11.0 | HTTP |
| 3 | Rocket | `vernal-rocket` | `rocket` | 0.5.1 | HTTP |
| 4 | Warp | `vernal-warp` | `warp` | 0.4.3 | HTTP |
| 5 | Salvo | `vernal-salvo` | `salvo` | 0.85.0 | HTTP |
| 6 | Poem | `vernal-poem` | `poem` | 3.1.12 | HTTP |
| 7 | Ntex | `vernal-ntex` | `ntex` | 2.18.0 | HTTP |
| 8 | Gotham | `vernal-gotham` | `gotham` | 0.8.0 | HTTP |
| 9 | Tide | `vernal-tide` | `tide` | 0.17.0-beta.1 | HTTP |
| 10 | Tonic | `vernal-tonic` | `tonic` | 0.12.3 | gRPC |

### 4.4 传输层基础 crate

| 用途 | crate | 说明 |
|:---|:---|:---|
| HTTP 类型 | `http` 1.4.0 | `Request`/`Response`/`StatusCode` 等框架无关类型 |
| HTTP Body | `http-body` 1.0.1 / `http-body-util` 0.1.3 | Body trait 与工具 |
| HTTP 传输 | `hyper` 1.9.0 | `vernal-hyper` 的底层传输 |
| 中间件 | `tower` 0.5.3 | `vernal-tower` 的底层 Service/Layer 抽象 |
| 异步运行时 | `tokio` 1.52.4 | 全局异步运行时 |
| MIME 检测 | `mime` 0.3.17（hyperium 官方）| vernal-core 已使用；**`mime-type` 和 `mimetype-detector` 不推荐**（570 格式但零依赖文档少）|

### 4.5 WebSocket

| 对标 Spring | crate | 说明 |
|:---|:---|:---|
| `spring-websocket` | `vernal-websocket` | WebSocket 抽象层 |
| WebSocket 实现 | `tokio-websockets` 0.12.0 | Tokio-native WebSocket 库 |

### 4.6 gRPC

| 对标 Spring | crate | 说明 |
|:---|:---|:---|
| `spring-grpc` | `vernal-tonic` | gRPC 集成 |
| gRPC 实现 | `tonic` 0.12.3 | Tokio-native gRPC 框架 |
| Protobuf | `prost`（ddd4r 层） | Protobuf 编解码 |

---

## 五、消息层

### 5.1 设计原则

vernal 只提供消息抽象 trait（`vernal-messaging`），broker adapter **全部在 ddd4r  
业务框架实现**。MQ 选型表使用 `ddd4r-mq-*` 命名。

JMS 不单独建 crate，JMS 语义融入 `vernal-messaging` 2.11 节（点对点 / 发布订阅  
抽象模型）。

### 5.2 消息抽象

| 对标 Spring | vernal crate | 说明 |
|:---|:---|:---|
| `spring-messaging` | `vernal-messaging` | 消息通道抽象（Message、MessageChannel、MessageHandler） |
| `spring-jms` | 融入 `vernal-messaging` | JMS 语义（点对点 Queue、发布订阅 Topic）通过 trait 抽象 |

### 5.3 Broker 适配器（ddd4r 层）

| 对标 Spring Boot Starter | ddd4r crate | 上游 crate | 状态 |
|:---|:---|:---|:---|
| `spring-boot-starter-amqp` | `ddd4r-mq-rabbitmq` | `lapin` | `[待集成]` |
| `spring-boot-starter-activemq` | `ddd4r-mq-activemq` | `fe2o3-amqp` | `[待集成]` |
| `spring-boot-starter-kafka` | `ddd4r-mq-kafka` | `rdkafka` | `[待集成]` |
| `spring-boot-starter-rocketmq` | `ddd4r-mq-rocketmq` | 待选型 | `[待验证]` |
| `spring-boot-starter-pulsar` | `ddd4r-mq-pulsar` | `pulsar-rs` | `[待验证]` |
| `spring-boot-starter-stream-redis` | `ddd4r-mq-redis-stream` | `redis-rs` Streams | `[待集成]` |

### 5.4 消息序列化

| 用途 | crate | 说明 |
|:---|:---|:---|
| JSON | `serde_json` | 消息体 JSON 序列化 |
| Avro | `apache-avro`（ddd4r 层） | Avro Schema 序列化 |
| Protobuf | `prost`（ddd4r 层） | Protobuf 序列化 |

---

## 六、数据层

### 6.1 ORM 主线：Toasty

**vernal-orm 锁定 Toasty** 作为 ORM 主线。Toasty 是 tokio-rs 官方 ORM 框架，  
原生异步、Tokio 生态集成。

| 对标 Spring | vernal crate | 上游 crate | 说明 |
|:---|:---|:---|:---|
| `spring-orm` / `spring-data-jpa` | `vernal-orm` | `toasty` | ORM 抽象层 + Toasty 集成 |

### 6.2 缓存：moka

**moka 替代 Caffeine/Guava Cache**，已通过 hutool-cache 验证。

| 对标 Spring / Java | crate | 说明 |
|:---|:---|:---|
| `spring-cache` | `vernal-cache` | 缓存抽象（Cache、CacheManager） |
| Caffeine / Guava Cache | `moka` 0.12 | 高性能并发缓存，vernal-cache 和 vernal-expression 已使用 |
| `hutool-cache` | moka | hutool-cache 全部缓存策略在 moka 上验证通过 |

### 6.3 数据库访问

| 对标 Spring / Java | vernal crate | 上游 crate | 说明 |
|:---|:---|:---|:---|
| `spring-jdbc` / `JdbcTemplate` | `vernal-rbdc` | `sqlx` | **通用数据库抽象层**，sqlx 对标 JdbcTemplate 语义（rbdc 4.9.10 仅作 vernal-rbatis 独立整合）|
| `MyBatis` / `MyBatis-Plus` | `vernal-rbatis`（已有） | `rbatis` | ORM 替代方案，rbatis 对标 MyBatis |
| 数据库驱动 | — | `sqlx` | 数据库连接驱动抽象（vernal-rbdc 底层）；`rbdc` 仅用于 vernal-rbatis |

### 6.4 数据库驱动（通过 sqlx；rbdc 仅限 vernal-rbatis）

| 数据库 | 驱动 crate | 说明 |
|:---|:---|:---|
| PostgreSQL | `sqlx-postgres` / `rbdc-pg` | 异步 PostgreSQL 驱动 |
| MySQL | `sqlx-mysql` / `rbdc-mysql` | 异步 MySQL 驱动 |
| SQLite | `sqlx-sqlite` / `rbdc-sqlite` | 异步 SQLite 驱动 |
| Redis | `redis-rs` | Redis 客户端（ddd4r 层） |

### 6.5 事务管理

| 对标 Spring | crate | 说明 |
|:---|:---|:---|
| `spring-tx` | `vernal-tx` | 事务抽象层 |

---

## 七、文档处理层

### 7.1 设计原则

对标 easydoc / easyexcel / easyofd / easypdf，在 Rust 生态中逐一验证对应 crate  
的可行性。

### 7.2 文档处理 crate 选型

| 对标 Java 项目 | Rust crate | 状态 | 说明 |
|:---|:---|:---|:---|
| easydoc（Word） | `docx-rs` | `[待验证]` | DOCX 文件读写 |
| easydoc（PDF 生成） | `printpdf` | `[待验证]` | PDF 文档生成 |
| easyexcel（Excel） | `rust_xlsxwriter` | `[待验证]` | Excel 文件写入 |
| easyexcel（Excel 读取） | `calamine` | `[待验证]` | Excel 文件读取 |
| easyofd（OFD） | `quick-xml` + 自定义 | `[待验证]` | OFD 是中国国标格式，需基于 XML 解析自建 |
| easypdf（PDF 处理） | `lopdf` | `[待验证]` | PDF 文件读写 |
| easypdf（PDF 渲染） | `printpdf` | `[待验证]` | PDF 渲染输出 |

### 7.3 XML 处理

| 用途 | crate | 说明 |
|:---|:---|:---|
| XML 解析/序列化 | `quick-xml` 0.41 | vernal-core 已集成，对标 woodstox-core |
| XML Schema | `xsd-parser` | `[待验证]` |

---

## 八、基础能力层

### 8.1 序列化

| 对标 Java | crate | 说明 |
|:---|:---|:---|
| Jackson / Gson | `serde` 1.0.228 + `serde_json` 1.0.150 | JSON 序列化/反序列化，vernal-core 已集成 |
| Jackson XML | `quick-xml` 0.41 | XML 序列化，vernal-core 已集成 |
| Jackson YAML | `serde_yaml_ng` | YAML 序列化/反序列化 `[待集成]` |
| Protobuf | `prost` | Protobuf 编解码（ddd4r 层） |
| MessagePack | `rmp-serde` | MessagePack 序列化 `[待集成]` |

### 8.2 配置文件解析

| 对标 Java / Spring | crate | vernal crate | 说明 |
|:---|:---|:---|:---|
| `.properties` 文件 | `java-properties` | `vernal-core` | Java Properties 格式解析 |
| Properties 工具 | `props-util` | `vernal-context` | Properties 高级操作工具 |
| `application.yml` | `serde_yaml_ng` | `vernal-context` | YAML 配置文件解析 |
| `application.toml` | `toml` | `vernal-context` | TOML 配置文件解析（Rust 原生） |
| 配置绑定 | 自研 `#[derive(ConfigurationProperties)]` | `vernal-context` | 已实现，从 Environment 绑定字段 |
| 配置中心 | `etcd-client` | ddd4r 层 | etcd 配置中心客户端 |
| 配置中心 | `zookeeper-client` | ddd4r 层 | ZooKeeper 配置中心客户端 |
| 配置中心 | `nacos-sdk` | ddd4r 层 | Nacos 配置中心客户端 |

### 8.3 表达式引擎

| 对标 Spring | crate | 说明 |
|:---|:---|:---|
| SpEL（Spring Expression Language） | `vernal-expression`（自研） | 100% 语义迁移 SpEL 子集 |
| 任意精度数字 | `bigdecimal` 0.4.7 + `num-bigint` 0.4.6 | 表达式数值计算 |
| 正则表达式 | `regex` 1.11 | 表达式中的正则操作 |

### 8.4 校验

| 对标 Java | crate | 说明 |
|:---|:---|:---|
| `javax.validation` / Hibernate Validator | `validator` | 结构体字段校验 `[待集成]` |
| 自定义校验 | `vernal-web` 内置 | 请求参数校验抽象 |
| JSON Schema 校验 | `jsonschema` 0.30 | vernal-core 可选 feature |

### 8.5 AOP

| 对标 Spring | crate | 说明 |
|:---|:---|:---|
| `spring-aop` | `vernal-aop` | 已实现，Tokio-first 切面内核 |
| 切面定义 | `vernal-aspects` | 内建切面：@Transactional / @Cacheable / @Async / @Scheduled |
| 过程宏 | `vernal-macros` | `#[derive(Component)]`、`#[intercept]` 等 |
| 组件发现 | `vernal-context-indexer` | linkme 分布式 slice 注册 |

### 8.6 脚本引擎

| 对标 Java | crate | 说明 |
|:---|:---|:---|
| JavaScript（Nashorn/GraalJS） | `boa_engine` | JavaScript 脚本引擎 `[待集成]` |
| Groovy | `rhai` | Rust 原生脚本语言 `[待集成]` |
| Lua | `mlua` | Lua 脚本引擎 `[待集成]` |

### 8.7 工具库

| 对标 Java / hutool | crate | 说明 |
|:---|:---|:---|
| `commons-lang` | `vernal-core` 内置 | 类型转换、ID 生成、MIME 等 |
| `commons-io` | `tokio::fs` | 异步文件 IO |
| `commons-collections` | `im` / `dashmap` 6.1.0 | 不可变集合 / 并发 HashMap |
| `hutool-crypto` | 见第九节加密部分 | 27 个 RustCrypto crate |
| `hutool-captcha` | 见第九节图像部分 | image + qrcode + font8x8 |
| UUID | `uuid` 1.24 | vernal-core 可选 feature |
| ULID | `ulid` 3.0 | vernal-core 可选 feature |
| NanoID | `nanoid` 0.5 | vernal-core 可选 feature |
| 日期时间 | `chrono` 0.4.45 / `time` 0.3 | vernal-core 可选 feature |
| URL | `url` 2.5 | vernal-core 可选 feature |
| 字节操作 | `bytes` 1.11.1 | vernal-core 可选 feature |
| 正则表达式 | `regex` 1.11 | vernal-core / vernal-expression 已使用 |
| 高级正则 | `fancy-regex` | 支持回溯等高级特性 `[待集成]` |
| 多模式匹配 | `aho-corasick` | 高效多模式字符串匹配 `[待集成]` |
| 布隆过滤器 | `bloomfilter` | 概率数据结构 `[待集成]` |
| 精确十进制 | `rust_decimal` | 精确十进制运算 `[待集成]` |
| 并发 HashMap | `dashmap` 6.1.0 | vernal-expression 已使用 |
| 同步原语 | `parking_lot` | 高性能锁 `[待集成]` |
| 单例 | `once_cell` 1.21 | vernal-core 可选 feature |
| 分布式注册 | `inventory` 0.3 | vernal-core 可选 feature |
| 链接期注入 | `linkme` 0.3.37 | vernal-core / vernal-context-indexer 已使用 |

### 8.8 异步调度

| 对标 Spring | crate | 说明 |
|:---|:---|:---|
| `@Async` | `vernal-async` | 异步执行抽象 |
| `@Scheduled` | `vernal-aspects` 内置 | 定时任务调度 |
| 异步运行时 | `tokio` 1.52.4 | 全局异步运行时 |
| 异步工具 | `tokio-util` 0.7.16 | CancellationToken、TaskTracker 等 |
| 异步流 | `tokio-stream` | 异步流处理 |
| 异步 trait | `async-trait` 0.1 | 异步 trait 支持 |
| Futures 工具 | `futures-util` 0.3.32 | Future 组合器 |

### 8.9 日志

| 对标 Java | crate | 说明 |
|:---|:---|:---|
| SLF4J / commons-logging | `tracing` 0.1.41 | 日志门面，vernal-core / vernal-expression 已使用 |
| Logback | `tracing-subscriber` | tracing 订阅者实现 |
| 结构化日志 | `tracing` | 原生结构化日志支持 |

### 8.10 监控

| 对标 Java / Spring | crate | 说明 |
|:---|:---|:---|
| `spring-actuator` | `vernal-actuator` | 健康检查与指标端点 |
| Micrometer | `metrics` | 指标收集 `[待集成]` |
| 系统信息 | `sysinfo` | 系统资源监控 `[待集成]` |
| Prometheus | `prometheus` | Prometheus 指标导出 `[待集成]` |

---

## 九、工具/安全层

### 9.1 AI 集成

> **[暂缓]** AI 集成节点（9.8 节）暂缓实施，后续**全量对齐 Spring AI 2.0**。  
> 恢复条件：Spring AI 2.0 GA 发布后，评估 Rust 生态对应 crate 的成熟度。

### 9.1.1 跨语言 FFI（UniFFI，新增）

| Java/Swift/Python 组件 | Rust crate | 状态 | 选型理由 |
|---|---|---|---|
| JNI / GraalVM 互操作 | **`uniffi`** | 🔵 | Mozilla 官方，Kotlin/Java/Swift/Python ↔ Rust 跨语言 FFI |

**适用场景**：
- vernal 的 Rust 内核被 Kotlin/Java 业务层调用
- Android/iOS/macOS 客户端通过 UniFFI 调用 vernal 业务逻辑
- 现有 Java 工具（hutool 等）通过 UniFFI bridge 复用 vernal 服务

**集成方式**：
- 模式 B（直接封装）：`vernal-uniffi` crate 暴露 vernal 核心 API 给外部语言
- 不影响 vernal 内部架构，作为独立可选 crate

### 9.2 加密

对标 `hutool-crypto`，基于 RustCrypto 生态。以下 crate 已验证：

| 算法类别 | crate | 说明 |
|:---|:---|:---|
| AES | `aes` + `cbc` + `ctr` + `gcm` | AES 对称加密 |
| DES/3DES | `des` | DES 对称加密 |
| RSA | `rsa` | RSA 非对称加密 |
| ECC | `p256` / `p384` / `k256` | 椭圆曲线加密 |
| Ed25519 | `ed25519-dalek` | EdDSA 签名 |
| SHA | `sha2` / `sha3` | SHA 哈希族 |
| MD5 | `md5` | MD5 哈希（仅用于校验，不用于安全场景） |
| BLAKE | `blake2` / `blake3` | BLAKE 哈希族 |
| HMAC | `hmac` | HMAC 消息认证 |
| PBKDF2 | `pbkdf2` | 密码派生 |
| Argon2 | `argon2` | 密码哈希（推荐） |
| Scrypt | `scrypt` | 密码哈希 |
| ChaCha20 | `chacha20` + `chacha20poly1305` | ChaCha20 流加密 |
| XChaCha20 | `xchacha20poly1305` | XChaCha20 流加密 |
| Base64 | `base64` | Base64 编解码 |
| Hex | `hex` | 十六进制编解码 |
| PKCS | `pkcs8` / `pkcs1` / `spki` | PKCS 标准格式 |
| X.509 | `x509-cert` | X.509 证书处理 |
| JWT 签名 | 见 9.3 节 | JWT 相关 |
| 随机数 | `rand` | 密码学安全随机数 |
| 常量时间比较 | `constant-time-eq` | 防时序攻击比较 |

### 9.3 JWT

| 对标 Java | crate | 说明 |
|:---|:---|:---|
| `jjwt` / `java-jwt` | `jsonwebtoken` | JWT 编解码与验证 |

### 9.4 HTTP 客户端

| 对标 Java | crate | 说明 |
|:---|:---|:---|
| `HttpClient` / OkHttp | `reqwest` | 异步 HTTP 客户端 |
| 底层 | `hyper-util` 0.1.20 | Hyper 客户端工具 |

### 9.5 邮件

| 对标 Java / Spring | crate | 说明 |
|:---|:---|:---|
| `spring-boot-starter-mail` / JavaMail | `lettre` 0.11 | 异步邮件发送，vernal-context-support 已集成 |

### 9.6 图像/验证码

| 对标 Java / hutool | crate | 说明 |
|:---|:---|:---|
| `hutool-captcha` | `image` | 图像处理 |
| 二维码 | `qrcode` | 二维码生成 |
| 验证码字体 | `font8x8` | 8x8 像素字体，用于验证码 |

### 9.7 配置中心客户端

| 对标 Java / Spring | crate | 说明 |
|:---|:---|:---|
| etcd | `etcd-client` | etcd v3 客户端（ddd4r 层） |
| ZooKeeper | `zookeeper-client` | ZooKeeper 客户端（ddd4r 层） |
| Nacos | `nacos-sdk` | Nacos 客户端（ddd4r 层） |

### 9.8 AI 集成（暂缓）

> **[暂缓]** 本节占位。后续全量对齐 Spring AI 2.0，包括：
> - LLM 客户端抽象（对标 `spring-ai-core`）
> - 向量存储（对标 `spring-ai-vector-store`）
> - RAG 管道（对标 `spring-ai-rag`）
> - Function Calling（对标 `spring-ai-function-calling`）
>
> 恢复条件：Spring AI 2.0 GA 发布 + Rust 生态 LLM SDK 成熟度评估完成。

---

## 十、未决项与待验证

### 10.1 未决选型

| ID | 领域 | 待选型内容 | 候选 crate | 阻塞条件 |
|:---|:---|:---|:---|:---|
| P-001 | 消息 | RocketMQ Rust SDK | 待调研 | ddd4r-mq-rocketmq 需求确认 |
| P-002 | 文档 | OFD 文件生成 | `quick-xml` + 自建 | OFD 国标解析库调研 |
| P-003 | 数据库 | 分库分表 | `sharding-sphere` (Rust) | 等待 Rust 生态成熟 |
| P-004 | 监控 | OpenTelemetry Rust SDK | `opentelemetry` | 与 tracing 集成方案确认 |
| P-005 | 安全 | OAuth2 Server | `oxide-auth` | sa-token-rust OAuth2 需求 |
| P-006 | 序列化 | FlatBuffers | `flatbuffers` | ddd4r 高性能序列化需求 |
| P-007 | 脚本 | WebAssembly 运行时 | `wasmtime` / `wasmer` | 插件化架构需求 |

### 10.2 待验证 crate

| ID | crate | 验证内容 | 预期完成 |
|:---|:---|:---|:---|
| V-001 | `docx-rs` | DOCX 文件读写完整性 | Phase 7 |
| V-002 | `printpdf` | PDF 生成质量 | Phase 7 |
| V-003 | `rust_xlsxwriter` | Excel 文件兼容性 | Phase 7 |
| V-004 | `calamine` | Excel 读取性能 | Phase 7 |
| V-005 | `lopdf` | PDF 编辑能力 | Phase 7 |
| V-006 | `boa_engine` | JavaScript 引擎性能 | Phase 9 |
| V-007 | `rhai` | 脚本语言集成 | Phase 9 |
| V-008 | `fancy-regex` | 高级正则性能 | Phase 8 |

---

## 十一、选型总览（按 vernal crate 汇总表）

| vernal crate | 对标 Spring | 核心依赖 | 状态 |
|:---|:---|:---|:---|
| `vernal` | spring-boot-starter | 组合 facade | `[已确认]` |
| `vernal-core` | spring-core | linkme, thiserror, uuid, chrono, serde, quick-xml, sha2 | `[已确认]` |
| `vernal-beans` | spring-beans | serde, tokio, dashmap | `[已确认]` |
| `vernal-aop` | spring-aop | tokio, tokio-util | `[已确认]` |
| `vernal-context` | spring-context | serde, tokio, vernal-aop, vernal-beans, vernal-expression | `[已确认]` |
| `vernal-context-indexer` | spring-context-indexer | linkme, vernal-beans, vernal-core | `[已确认]` |
| `vernal-context-support` | spring-context-support | moka, lettre, tera, tokio | `[已确认]` |
| `vernal-expression` | spring-expression (SpEL) | bigdecimal, chrono, dashmap, moka, regex | `[已确认]` |
| `vernal-macros` | （Java 无直接对标） | proc-macro2, quote, syn | `[已确认]` |
| `vernal-web` | spring-web | tokio, vernal-aop, vernal-context, vernal-beans | `[已确认]` |
| `vernal-http` | spring-http | bytes, http, http-body, vernal-core, vernal-web | `[已确认]` |
| `vernal-tower` | （Spring 无直接对标） | tower, http, vernal-aop, vernal-context, vernal-web | `[已确认]` |
| `vernal-hyper` | （Spring 无直接对标） | hyper, http-body-util, vernal-http, vernal-web | `[已确认]` |
| `vernal-axum` | spring-webmvc (Axum) | axum 0.8.9, tower, vernal-http, vernal-web | `[已确认]` |
| `vernal-actix-web` | spring-webmvc (Actix) | actix-web 4.11.0, vernal-http, vernal-web | `[已确认]` |
| `vernal-rocket` | spring-webmvc (Rocket) | rocket 0.5.1, vernal-http, vernal-web | `[已确认]` |
| `vernal-warp` | spring-webmvc (Warp) | warp 0.4.3, tower, vernal-tower, vernal-web | `[已确认]` |
| `vernal-salvo` | spring-webmvc (Salvo) | salvo 0.85.0, vernal-http, vernal-web | `[已确认]` |
| `vernal-poem` | spring-webmvc (Poem) | poem 3.1.12, vernal-http, vernal-web | `[已确认]` |
| `vernal-ntex` | spring-webmvc (Ntex) | ntex 2.18.0, vernal-http, vernal-web | `[已确认]` |
| `vernal-gotham` | spring-webmvc (Gotham) | gotham 0.8.0, vernal-http, vernal-web | `[已确认]` |
| `vernal-tide` | spring-webmvc (Tide) | tide 0.17.0-beta.1, vernal-http, vernal-web | `[已确认]` |
| `vernal-tonic` | spring-grpc | tonic 0.12.3, tower, vernal-tower, vernal-web | `[已确认]` |
| `vernal-web-testkit` | spring-test (Web) | bytes, tokio, vernal-aop, vernal-context, vernal-web | `[已确认]` |
| `vernal-messaging` | spring-messaging / spring-jms | tokio | `[已确认]` |
| `vernal-websocket` | spring-websocket | tokio-websockets 0.12.0, vernal-messaging | `[已确认]` |
| `vernal-cache` | spring-cache | thiserror | `[已确认]` |
| `vernal-rbdc` | spring-jdbc | vernal-core | `[骨架]` **通用数据库抽象层**（基于 sqlx，非 rbatis 专属）|
| `vernal-db` | （通用数据库抽象）| vernal-core | `[骨架]` sqlx 连接池 |
| `vernal-tx` | spring-tx | vernal-core | `[骨架]` |
| `vernal-orm` | spring-orm / spring-data-jpa | toasty（待集成） | `[待集成]` |
| `vernal-log` | spring-jcl | vernal-core | `[骨架]` |
| `vernal-async` | spring-async | vernal-core | `[骨架]` |
| `vernal-actuator` | spring-actuator | vernal-core | `[骨架]` |
| `vernal-test` | spring-test | vernal-core | `[骨架]` |
| `vernal-aspects` | spring-aspects | （暂时禁用） | `[骨架]` |

---

## 十二、维护规则

### 12.1 更新流程

1. **新增选型**：在对应章节添加条目，标注状态为 `[待验证]`，记录候选 crate 和评估依据
2. **状态变更**：更新状态标签，记录变更原因和验证证据
3. **废弃选型**：移动到附录 A 纠错记录，保留原始决策和废弃原因
4. **版本更新**：更新 crate 版本号，记录兼容性评估结果

### 12.2 版本号规则

- 文档版本格式：`vX.Y`
- X：结构性变更（新增/删除章节、重大选型变更）
- Y：条目级变更（新增/更新/废弃条目）

### 12.3 引用规范

其他文档引用本文档时，使用以下格式：

```
参见 [Spring 组件替换约定](./Spring-组件替换约定.md) 第 N 节"xxx"
```

### 12.4 同步检查

每次 vernal-framework workspace 的 `Cargo.toml` 发生依赖变更时，必须同步更新  
本文档对应章节，确保文档与代码一致。

---

## 附录 A：清单纠错记录

本附录记录选型过程中的重要修正，保留原始决策和修正原因。

### A.1 Topcoat：UI 框架 → 全栈 Web 框架

| 项目 | 内容 |
|:---|:---|
| 原始判断 | Topcoat 是一个 UI 组件框架 |
| 修正判断 | Topcoat 是 tokio-rs 官方全栈 Web 框架（2026-07-22 发布） |
| 修正原因 | 深入调研 Topcoat 仓库后确认其定位为全栈 Web 框架，对标 Spring Boot |
| 影响 | vernal-web / vernal-webmvc / vernal-webflux 整合 Topcoat 作为轨道一主线 |

### A.2 Toasty：pre-1.0 → 锁定主线

| 项目 | 内容 |
|:---|:---|
| 原始判断 | Toasty 尚未发布 1.0，可能不稳定 |
| 修正判断 | 锁定 Toasty 为 ORM 主线，接受 pre-1.0 状态 |
| 修正原因 | Toasty 是 tokio-rs 官方 ORM，生态位明确，pre-1.0 不影响架构决策 |
| 影响 | vernal-orm 整合 Toasty，不引入其他 ORM 作为主线替代 |

### A.3 RabbitMQ：amiquor → lapin

| 项目 | 内容 |
|:---|:---|
| 原始判断 | 使用 `amiquor` 作为 RabbitMQ Rust 客户端 |
| 修正判断 | 使用 `lapin` 作为 RabbitMQ Rust 客户端 |
| 修正原因 | `lapin` 维护更活跃、Tokio-native、社区使用更广泛 |
| 影响 | `ddd4r-mq-rabbitmq` 依赖 `lapin` |

### A.4 ActiveMQ：activemq-rust → fe2o3-amqp

| 项目 | 内容 |
|:---|:---|
| 原始判断 | 使用 `activemq-rust` 作为 ActiveMQ Rust 客户端 |
| 修正判断 | 使用 `fe2o3-amqp` 作为 AMQP 1.0 Rust 客户端 |
| 修正原因 | `fe2o3-amqp` 是 AMQP 1.0 协议实现，ActiveMQ 支持 AMQP 1.0 |
| 影响 | `ddd4r-mq-activemq` 依赖 `fe2o3-amqp` |

### A.5 JavaScript 引擎：V8 → boa_engine

| 项目 | 内容 |
|:---|:---|
| 原始判断 | 使用 V8 绑定（`v8` crate）作为 JavaScript 引擎 |
| 修正判断 | 使用 `boa_engine` 作为 JavaScript 引擎 |
| 修正原因 | `boa_engine` 是纯 Rust 实现，无 C++ 依赖，编译更简单，`forbid unsafe` 友好 |
| 影响 | 脚本引擎选型表更新 |

### A.6 Groovy 替代：无 → rhai

| 项目 | 内容 |
|:---|:---|
| 原始判断 | Groovy 无 Rust 直接替代 |
| 修正判断 | 使用 `rhai` 作为 Rust 原生脚本语言替代 Groovy |
| 修正原因 | `rhai` 语法简洁、Rust 原生、嵌入式友好、性能优秀 |
| 影响 | 脚本引擎选型表新增 rhai |

---

## 附录 B：参考资料

| 序号 | 资料 | 说明 |
|:---|:---|:---|
| 1 | [Spring Boot Reference Documentation](https://docs.spring.io/spring-boot/docs/current/reference/htmlsingle/) | Spring Boot 官方文档 |
| 2 | [Spring Framework Documentation](https://docs.spring.io/spring-framework/reference/) | Spring Framework 官方文档 |
| 3 | [Tokio Documentation](https://tokio.rs/) | Tokio 异步运行时文档 |
| 4 | [Axum Documentation](https://docs.rs/axum/) | Axum Web 框架文档 |
| 5 | [Actix Web Documentation](https://actix.rs/) | Actix Web 框架文档 |
| 6 | [Toasty Documentation](https://github.com/tokio-rs/toasty) | Toasty ORM 文档 |
| 7 | [Topcoat Documentation](https://github.com/tokio-rs/topcoat) | Topcoat 全栈框架文档 |
| 8 | [Moka Documentation](https://docs.rs/moka/) | Moka 缓存库文档 |
| 9 | [Serde Documentation](https://serde.rs/) | Serde 序列化框架文档 |
| 10 | [Tracing Documentation](https://docs.rs/tracing/) | Tracing 日志框架文档 |
| 11 | [RustCrypto Documentation](https://docs.rs/) | RustCrypto 密码学库文档 |
| 12 | [hutool Documentation](https://hutool.cn/) | hutool Java 工具库文档（对标参考） |

---

## 附录 C：crate 地址索引

本附录按章节分组，列出本文档涉及的全部 crate 的 crates.io / docs.rs / GitHub 地址。

### C.1 Web/HTTP 层（第四节）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `axum` | [crates.io/crates/axum](https://crates.io/crates/axum) | [docs.rs/axum](https://docs.rs/axum) | [tokio-rs/axum](https://github.com/tokio-rs/axum) |
| `actix-web` | [crates.io/crates/actix-web](https://crates.io/crates/actix-web) | [docs.rs/actix-web](https://docs.rs/actix-web) | [actix/actix-web](https://github.com/actix/actix-web) |
| `rocket` | [crates.io/crates/rocket](https://crates.io/crates/rocket) | [docs.rs/rocket](https://docs.rs/rocket) | [rwf2/Rocket](https://github.com/rwf2/Rocket) |
| `warp` | [crates.io/crates/warp](https://crates.io/crates/warp) | [docs.rs/warp](https://docs.rs/warp) | [seanmonstar/warp](https://github.com/seanmonstar/warp) |
| `salvo` | [crates.io/crates/salvo](https://crates.io/crates/salvo) | [docs.rs/salvo](https://docs.rs/salvo) | [salvo-rs/salvo](https://github.com/salvo-rs/salvo) |
| `poem` | [crates.io/crates/poem](https://crates.io/crates/poem) | [docs.rs/poem](https://docs.rs/poem) | [poem-web/poem](https://github.com/poem-web/poem) |
| `ntex` | [crates.io/crates/ntex](https://crates.io/crates/ntex) | [docs.rs/ntex](https://docs.rs/ntex) | [ntex-rs/ntex](https://github.com/ntex-rs/ntex) |
| `gotham` | [crates.io/crates/gotham](https://crates.io/crates/gotham) | [docs.rs/gotham](https://docs.rs/gotham) | [gotham-rs/gotham](https://github.com/gotham-rs/gotham) |
| `tide` | [crates.io/crates/tide](https://crates.io/crates/tide) | [docs.rs/tide](https://docs.rs/tide) | [http-rs/tide](https://github.com/http-rs/tide) |
| `tonic` | [crates.io/crates/tonic](https://crates.io/crates/tonic) | [docs.rs/tonic](https://docs.rs/tonic) | [hyperium/tonic](https://github.com/hyperium/tonic) |
| `hyper` | [crates.io/crates/hyper](https://crates.io/crates/hyper) | [docs.rs/hyper](https://docs.rs/hyper) | [hyperium/hyper](https://github.com/hyperium/hyper) |
| `tower` | [crates.io/crates/tower](https://crates.io/crates/tower) | [docs.rs/tower](https://docs.rs/tower) | [tokio-rs/tower](https://github.com/tokio-rs/tower) |
| `http` | [crates.io/crates/http](https://crates.io/crates/http) | [docs.rs/http](https://docs.rs/http) | [hyperium/http](https://github.com/hyperium/http) |
| `http-body` | [crates.io/crates/http-body](https://crates.io/crates/http-body) | [docs.rs/http-body](https://docs.rs/http-body) | [hyperium/http-body](https://github.com/hyperium/http-body) |
| `tokio-websockets` | [crates.io/crates/tokio-websockets](https://crates.io/crates/tokio-websockets) | [docs.rs/tokio-websockets](https://docs.rs/tokio-websockets) | [nickel-org/tokio-websockets](https://github.com/nickel-org/tokio-websockets) |

### C.2 消息层（第五节）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `lapin` | [crates.io/crates/lapin](https://crates.io/crates/lapin) | [docs.rs/lapin](https://docs.rs/lapin) | [CleverCloud/lapin](https://github.com/CleverCloud/lapin) |
| `fe2o3-amqp` | [crates.io/crates/fe2o3-amqp](https://crates.io/crates/fe2o3-amqp) | [docs.rs/fe2o3-amqp](https://docs.rs/fe2o3-amqp) | [minghuaw/fe2o3-amqp](https://github.com/minghuaw/fe2o3-amqp) |
| `rdkafka` | [crates.io/crates/rdkafka](https://crates.io/crates/rdkafka) | [docs.rs/rdkafka](https://docs.rs/rdkafka) | [fede1024/rust-rdkafka](https://github.com/fede1024/rust-rdkafka) |
| `redis` | [crates.io/crates/redis](https://crates.io/crates/redis) | [docs.rs/redis](https://docs.rs/redis) | [redis-rs/redis-rs](https://github.com/redis-rs/redis-rs) |

### C.3 数据层（第六节）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `toasty` | [crates.io/crates/toasty](https://crates.io/crates/toasty) | [docs.rs/toasty](https://docs.rs/toasty) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| `moka` | [crates.io/crates/moka](https://crates.io/crates/moka) | [docs.rs/moka](https://docs.rs/moka) | [moka-rs/moka](https://github.com/moka-rs/moka) |
| `sqlx` | [crates.io/crates/sqlx](https://crates.io/crates/sqlx) | [docs.rs/sqlx](https://docs.rs/sqlx) | [launchbadge/sqlx](https://github.com/launchbadge/sqlx) |
| `rbatis` | [crates.io/crates/rbatis](https://crates.io/crates/rbatis) | [docs.rs/rbatis](https://docs.rs/rbatis) | [rbatis/rbatis](https://github.com/rbatis/rbatis) |
| `rbdc` | [crates.io/crates/rbdc](https://crates.io/crates/rbdc) | [docs.rs/rbdc](https://docs.rs/rbdc) | [rbatis/rbdc](https://github.com/rbatis/rbdc) |

### C.4 文档处理层（第七节）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `docx-rs` | [crates.io/crates/docx-rs](https://crates.io/crates/docx-rs) | [docs.rs/docx-rs](https://docs.rs/docx-rs) | [bokuweb/docx-rs](https://github.com/bokuweb/docx-rs) |
| `printpdf` | [crates.io/crates/printpdf](https://crates.io/crates/printpdf) | [docs.rs/printpdf](https://docs.rs/printpdf) | [fschutt/printpdf](https://github.com/fschutt/printpdf) |
| `rust_xlsxwriter` | [crates.io/crates/rust_xlsxwriter](https://crates.io/crates/rust_xlsxwriter) | [docs.rs/rust_xlsxwriter](https://docs.rs/rust_xlsxwriter) | [jmcnamara/rust_xlsxwriter](https://github.com/jmcnamara/rust_xlsxwriter) |
| `calamine` | [crates.io/crates/calamine](https://crates.io/crates/calamine) | [docs.rs/calamine](https://docs.rs/calamine) | [tafia/calamine](https://github.com/tafia/calamine) |
| `lopdf` | [crates.io/crates/lopdf](https://crates.io/crates/lopdf) | [docs.rs/lopdf](https://docs.rs/lopdf) | [J-F-Liu/lopdf](https://github.com/J-F-Liu/lopdf) |
| `quick-xml` | [crates.io/crates/quick-xml](https://crates.io/crates/quick-xml) | [docs.rs/quick-xml](https://docs.rs/quick-xml) | [tafia/quick-xml](https://github.com/tafia/quick-xml) |

### C.5 基础能力层（第八节）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `serde` | [crates.io/crates/serde](https://crates.io/crates/serde) | [docs.rs/serde](https://docs.rs/serde) | [serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_json` | [crates.io/crates/serde_json](https://crates.io/crates/serde_json) | [docs.rs/serde_json](https://docs.rs/serde_json) | [serde-rs/json](https://github.com/serde-rs/json) |
| `serde_yaml_ng` | [crates.io/crates/serde_yaml_ng](https://crates.io/crates/serde_yaml_ng) | [docs.rs/serde_yaml_ng](https://docs.rs/serde_yaml_ng) | [serde-rs/serde-yaml-ng](https://github.com/serde-rs/serde-yaml-ng) |
| `toml` | [crates.io/crates/toml](https://crates.io/crates/toml) | [docs.rs/toml](https://docs.rs/toml) | [toml-rs/toml](https://github.com/toml-rs/toml) |
| `java-properties` | [crates.io/crates/java-properties](https://crates.io/crates/java-properties) | [docs.rs/java-properties](https://docs.rs/java-properties) | [srijs/rust-java-properties](https://github.com/srijs/rust-java-properties) |
| `tokio` | [crates.io/crates/tokio](https://crates.io/crates/tokio) | [docs.rs/tokio](https://docs.rs/tokio) | [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `tokio-util` | [crates.io/crates/tokio-util](https://crates.io/crates/tokio-util) | [docs.rs/tokio-util](https://docs.rs/tokio-util) | [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `tokio-stream` | [crates.io/crates/tokio-stream](https://crates.io/crates/tokio-stream) | [docs.rs/tokio-stream](https://docs.rs/tokio-stream) | [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `async-trait` | [crates.io/crates/async-trait](https://crates.io/crates/async-trait) | [docs.rs/async-trait](https://docs.rs/async-trait) | [dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| `futures-util` | [crates.io/crates/futures-util](https://crates.io/crates/futures-util) | [docs.rs/futures-util](https://docs.rs/futures-util) | [rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `tracing` | [crates.io/crates/tracing](https://crates.io/crates/tracing) | [docs.rs/tracing](https://docs.rs/tracing) | [tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-subscriber` | [crates.io/crates/tracing-subscriber](https://crates.io/crates/tracing-subscriber) | [docs.rs/tracing-subscriber](https://docs.rs/tracing-subscriber) | [tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `metrics` | [crates.io/crates/metrics](https://crates.io/crates/metrics) | [docs.rs/metrics](https://docs.rs/metrics) | [metrics-rs/metrics](https://github.com/metrics-rs/metrics) |
| `sysinfo` | [crates.io/crates/sysinfo](https://crates.io/crates/sysinfo) | [docs.rs/sysinfo](https://docs.rs/sysinfo) | [GuillaumeGomez/sysinfo](https://github.com/GuillaumeGomez/sysinfo) |
| `prometheus` | [crates.io/crates/prometheus](https://crates.io/crates/prometheus) | [docs.rs/prometheus](https://docs.rs/prometheus) | [tikv/rust-prometheus](https://github.com/tikv/rust-prometheus) |
| `bigdecimal` | [crates.io/crates/bigdecimal](https://crates.io/crates/bigdecimal) | [docs.rs/bigdecimal](https://docs.rs/bigdecimal) | [akubera/bigdecimal-rs](https://github.com/akubera/bigdecimal-rs) |
| `num-bigint` | [crates.io/crates/num-bigint](https://crates.io/crates/num-bigint) | [docs.rs/num-bigint](https://docs.rs/num-bigint) | [rust-num/num](https://github.com/rust-num/num) |
| `chrono` | [crates.io/crates/chrono](https://crates.io/crates/chrono) | [docs.rs/chrono](https://docs.rs/chrono) | [chronotope/chrono](https://github.com/chronotope/chrono) |
| `time` | [crates.io/crates/time](https://crates.io/crates/time) | [docs.rs/time](https://docs.rs/time) | [time-rs/time](https://github.com/time-rs/time) |
| `uuid` | [crates.io/crates/uuid](https://crates.io/crates/uuid) | [docs.rs/uuid](https://docs.rs/uuid) | [uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| `ulid` | [crates.io/crates/ulid](https://crates.io/crates/ulid) | [docs.rs/ulid](https://docs.rs/ulid) | [dylanhart/ulid-rs](https://github.com/dylanhart/ulid-rs) |
| `nanoid` | [crates.io/crates/nanoid](https://crates.io/crates/nanoid) | [docs.rs/nanoid](https://docs.rs/nanoid) | [mrdimidium/nanoid](https://github.com/mrdimidium/nanoid) |
| `url` | [crates.io/crates/url](https://crates.io/crates/url) | [docs.rs/url](https://docs.rs/url) | [servo/rust-url](https://github.com/servo/rust-url) |
| `bytes` | [crates.io/crates/bytes](https://crates.io/crates/bytes) | [docs.rs/bytes](https://docs.rs/bytes) | [tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| `regex` | [crates.io/crates/regex](https://crates.io/crates/regex) | [docs.rs/regex](https://docs.rs/regex) | [rust-lang/regex](https://github.com/rust-lang/regex) |
| `fancy-regex` | [crates.io/crates/fancy-regex](https://crates.io/crates/fancy-regex) | [docs.rs/fancy-regex](https://docs.rs/fancy-regex) | [fancy-regex/fancy-regex](https://github.com/fancy-regex/fancy-regex) |
| `aho-corasick` | [crates.io/crates/aho-corasick](https://crates.io/crates/aho-corasick) | [docs.rs/aho-corasick](https://docs.rs/aho-corasick) | [BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| `bloomfilter` | [crates.io/crates/bloomfilter](https://crates.io/crates/bloomfilter) | [docs.rs/bloomfilter](https://docs.rs/bloomfilter) | [jedisct1/rust-bloom-filter](https://github.com/jedisct1/rust-bloom-filter) |
| `rust_decimal` | [crates.io/crates/rust_decimal](https://crates.io/crates/rust_decimal) | [docs.rs/rust_decimal](https://docs.rs/rust_decimal) | [paupino/rust-decimal](https://github.com/paupino/rust-decimal) |
| `dashmap` | [crates.io/crates/dashmap](https://crates.io/crates/dashmap) | [docs.rs/dashmap](https://docs.rs/dashmap) | [xacrimon/dashmap](https://github.com/xacrimon/dashmap) |
| `parking_lot` | [crates.io/crates/parking_lot](https://crates.io/crates/parking_lot) | [docs.rs/parking_lot](https://docs.rs/parking_lot) | [Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `once_cell` | [crates.io/crates/once_cell](https://crates.io/crates/once_cell) | [docs.rs/once_cell](https://docs.rs/once_cell) | [matklad/once_cell](https://github.com/matklad/once_cell) |
| `inventory` | [crates.io/crates/inventory](https://crates.io/crates/inventory) | [docs.rs/inventory](https://docs.rs/inventory) | [dtolnay/inventory](https://github.com/dtolnay/inventory) |
| `linkme` | [crates.io/crates/linkme](https://crates.io/crates/linkme) | [docs.rs/linkme](https://docs.rs/linkme) | [dtolnay/linkme](https://github.com/dtolnay/linkme) |
| `thiserror` | [crates.io/crates/thiserror](https://crates.io/crates/thiserror) | [docs.rs/thiserror](https://docs.rs/thiserror) | [dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `clap` | [crates.io/crates/clap](https://crates.io/crates/clap) | [docs.rs/clap](https://docs.rs/clap) | [clap-rs/clap](https://github.com/clap-rs/clap) |
| `validator` | [crates.io/crates/validator](https://crates.io/crates/validator) | [docs.rs/validator](https://docs.rs/validator) | [Keats/validator](https://github.com/Keats/validator) |
| `jsonschema` | [crates.io/crates/jsonschema](https://crates.io/crates/jsonschema) | [docs.rs/jsonschema](https://docs.rs/jsonschema) | [Stranger6667/jsonschema-rs](https://github.com/Stranger6667/jsonschema-rs) |
| `mockall` | [crates.io/crates/mockall](https://crates.io/crates/mockall) | [docs.rs/mockall](https://docs.rs/mockall) | [asomers/mockall](https://github.com/asomers/mockall) |
| `proptest` | [crates.io/crates/proptest](https://crates.io/crates/proptest) | [docs.rs/proptest](https://docs.rs/proptest) | [proptest-rs/proptest](https://github.com/proptest-rs/proptest) |
| `lettre` | [crates.io/crates/lettre](https://crates.io/crates/lettre) | [docs.rs/lettre](https://docs.rs/lettre) | [lettre/lettre](https://github.com/lettre/lettre) |
| `tera` | [crates.io/crates/tera](https://crates.io/crates/tera) | [docs.rs/tera](https://docs.rs/tera) | [Keats/tera](https://github.com/Keats/tera) |
| `prost` | [crates.io/crates/prost](https://crates.io/crates/prost) | [docs.rs/prost](https://docs.rs/prost) | [tokio-rs/prost](https://github.com/tokio-rs/prost) |

### C.6 脚本引擎（第八节 8.6）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `boa_engine` | [crates.io/crates/boa_engine](https://crates.io/crates/boa_engine) | [docs.rs/boa_engine](https://docs.rs/boa_engine) | [boa-dev/boa](https://github.com/boa-dev/boa) |
| `rhai` | [crates.io/crates/rhai](https://crates.io/crates/rhai) | [docs.rs/rhai](https://docs.rs/rhai) | [rhaiscript/rhai](https://github.com/rhaiscript/rhai) |
| `mlua` | [crates.io/crates/mlua](https://crates.io/crates/mlua) | [docs.rs/mlua](https://docs.rs/mlua) | [khvzak/mlua](https://github.com/khvzak/mlua) |

### C.7 工具/安全层（第九节）

| crate | crates.io | docs.rs | GitHub |
|:---|:---|:---|:---|
| `jsonwebtoken` | [crates.io/crates/jsonwebtoken](https://crates.io/crates/jsonwebtoken) | [docs.rs/jsonwebtoken](https://docs.rs/jsonwebtoken) | [Keats/jsonwebtoken](https://github.com/Keats/jsonwebtoken) |
| `reqwest` | [crates.io/crates/reqwest](https://crates.io/crates/reqwest) | [docs.rs/reqwest](https://docs.rs/reqwest) | [seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) |
| `image` | [crates.io/crates/image](https://crates.io/crates/image) | [docs.rs/image](https://docs.rs/image) | [image-rs/image](https://github.com/image-rs/image) |
| `qrcode` | [crates.io/crates/qrcode](https://crates.io/crates/qrcode) | [docs.rs/qrcode](https://docs.rs/qrcode) | [kennytm/qrcode-rs](https://github.com/kennytm/qrcode-rs) |
| `font8x8` | [crates.io/crates/font8x8](https://crates.io/crates/font8x8) | [docs.rs/font8x8](https://docs.rs/font8x8) | [phlosion/font8x8](https://github.com/phlosion/font8x8) |
| `etcd-client` | [crates.io/crates/etcd-client](https://crates.io/crates/etcd-client) | [docs.rs/etcd-client](https://docs.rs/etcd-client) | [etcdv3/etcd-client](https://github.com/etcdv3/etcd-client) |
| `nacos-sdk` | [crates.io/crates/nacos-sdk](https://crates.io/crates/nacos-sdk) | [docs.rs/nacos-sdk](https://docs.rs/nacos-sdk) | [nacos-group/nacos-sdk-rust](https://github.com/nacos-group/nacos-sdk-rust) |
| `aes` | [crates.io/crates/aes](https://crates.io/crates/aes) | [docs.rs/aes](https://docs.rs/aes) | [RustCrypto/block-ciphers](https://github.com/RustCrypto/block-ciphers) |
| `rsa` | [crates.io/crates/rsa](https://crates.io/crates/rsa) | [docs.rs/rsa](https://docs.rs/rsa) | [RustCrypto/RSA](https://github.com/RustCrypto/RSA) |
| `ed25519-dalek` | [crates.io/crates/ed25519-dalek](https://crates.io/crates/ed25519-dalek) | [docs.rs/ed25519-dalek](https://docs.rs/ed25519-dalek) | [dalek-cryptography/curve25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek) |
| `sha2` | [crates.io/crates/sha2](https://crates.io/crates/sha2) | [docs.rs/sha2](https://docs.rs/sha2) | [RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| **`uniffi`** | [crates.io/crates/uniffi](https://crates.io/crates/uniffi) | [Mozilla 文档](https://mozilla.github.io/uniffi-rs/latest/) | [mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `sha3` | [crates.io/crates/sha3](https://crates.io/crates/sha3) | [docs.rs/sha3](https://docs.rs/sha3) | [RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `blake2` | [crates.io/crates/blake2](https://crates.io/crates/blake2) | [docs.rs/blake2](https://docs.rs/blake2) | [RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `blake3` | [crates.io/crates/blake3](https://crates.io/crates/blake3) | [docs.rs/blake3](https://docs.rs/blake3) | [BLAKE3-team/BLAKE3](https://github.com/BLAKE3-team/BLAKE3) |
| `hmac` | [crates.io/crates/hmac](https://crates.io/crates/hmac) | [docs.rs/hmac](https://docs.rs/hmac) | [RustCrypto/MACs](https://github.com/RustCrypto/MACs) |
| `argon2` | [crates.io/crates/argon2](https://crates.io/crates/argon2) | [docs.rs/argon2](https://docs.rs/argon2) | [RustCrypto/password-hashes](https://github.com/RustCrypto/password-hashes) |
| `base64` | [crates.io/crates/base64](https://crates.io/crates/base64) | [docs.rs/base64](https://docs.rs/base64) | [marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `hex` | [crates.io/crates/hex](https://crates.io/crates/hex) | [docs.rs/hex](https://docs.rs/hex) | [KokaKiwi/rust-hex](https://github.com/KokaKiwi/rust-hex) |
| `rand` | [crates.io/crates/rand](https://crates.io/crates/rand) | [docs.rs/rand](https://docs.rs/rand) | [rust-random/rand](https://github.com/rust-random/rand) |
| `constant-time-eq` | [crates.io/crates/constant-time-eq](https://crates.io/crates/constant-time-eq) | [docs.rs/constant-time-eq](https://docs.rs/constant-time-eq) | [cesarb/constant-time-eq](https://github.com/cesarb/constant-time-eq) |

---

> **文档结束** — 本文档是 Vernal Framework 项目的单一权威选型字典。  
> 如有选型疑问，请查阅对应章节；如遇未覆盖场景，请提交到第十节"未决项与待验证"。

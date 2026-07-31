<!-- migration-doc: authority=support canonical=迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [迁移验收规范.md](迁移验收规范.md) 和自动审计报告为准。

# vernal-architecture-overview（中间层架构总览）

> 版本：v1.0（2026-07-29）
> 定位：vernal-framework 是建立在成熟底层组件基础上的**中间层框架**，对标 Spring Framework
> 核心价值：**数据库 + 底层组件 → vernal → Web 框架** 的中间层抽象
> edition 2024 / rustc 1.88

## 一、核心定位

```
┌─────────────────────────────────────────────────────────────┐
│                    应用层（业务代码）                          │
│           controller / service / repository                   │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                                                              │
│   ██ vernal-framework（中间层框架，对标 spring-framework）██ │
│                                                              │
│   IoC / Beans / AOP / Context / Expression / TX / Cache       │
│   Config / Messaging / Data / Web / Test / Actuator           │
│                                                              │
└─────────┬─────────────────────────────────────────┬─────────┘
          │                                         │
          ▼                                         ▼
┌──────────────────────┐                ┌────────────────────────┐
│   Web 框架层（10 选 1）│                │   数据/组件层         │
│   Axum/Actix/Rocket  │                │   Toasty/sqlx/redis/...│
└──────────────────────┘                └────────────────────────┘
          │                                         │
          └────────────────────┬────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    基础运行时（成熟底层）                        │
│   Tokio / Hyper / Tower / tracing / serde / moka / sqlx     │
│   Topcoat（Web 全栈）/ tokio-websockets / sqlparser         │
└─────────────────────────────────────────────────────────────┘
```

## 二、与 Spring 框架的对应

| Spring 概念 | vernal 对应 | vernal crate |
|---|---|---|
| `spring-core` | 基础合同层 | `vernal-core` |
| `spring-beans` | IoC 容器 | `vernal-beans` |
| `spring-context` | 应用上下文 | `vernal-context` |
| `spring-aop` + `aspectj` | 切面编程 | `vernal-aop` + `vernal-aspects` |
| `spring-expression` | SpEL 表达式 | `vernal-expression` |
| `spring-jdbc` | 数据访问抽象 | **`vernal-rbdc`**（通用，基于 sqlx）|
| `spring-jdbc` (rbatis 实现) | rbatis 整合 | `vernal-rbatis`（独立，基于 rbdc 4.9.10）|
| `spring-orm` | ORM | `vernal-orm`（Toasty 锁定）|
| `spring-tx` | 事务管理 | `vernal-tx` |
| `spring-messaging` + JMS | 消息抽象 | `vernal-messaging`（含 spring-jms 融入 2.11）|
| `spring-cache` | 缓存 | `vernal-cache`（moka）|
| `spring-web/webmvc/webflux/websocket` | Web 全栈 | `vernal-web/webmvc/webflux/websocket` |
| `spring-context-support` | 上下文支持 | `vernal-context-support` |
| `spring-context-indexer` | 链接期索引 | `vernal-context-indexer` |
| `spring-test` | 测试 | `vernal-test` |
| `spring-core-test` | 测试工具 | `vernal-core-test` |
| `spring-boot-actuator` | 监控 | `vernal-actuator` |
| `spring-instrument` | JVM agent（不迁移）| 🚫 |
| `spring-data-jpa` | Repository 自动实现 | `vernal-orm`（Toasty 自动生成）|
| `spring-oxm` | XML 映射 | `vernal-oxm` |
| `spring-r2dbc` | 响应式 DB | `vernal-r2dbc` |

## 三、三层依赖拓扑（模块完成顺序）

```
L0  零依赖
    └── vernal-core                          ✅ 已完成（94文件）

L1  仅依赖 L0（可并行）
    ├── vernal-expression  (113文件)        ✅ 已完成
    ├── vernal-aop         (以自动审计为准) ⚠️ 对象迁移未完成
    ├── vernal-async       (2文件骨架)
    ├── vernal-cache       (3文件骨架)       ✅ 技术要求 587 行
    ├── vernal-db          (2文件骨架)       🔜 通用数据库抽象层
    ├── vernal-tx          (4文件骨架)       ✅ 技术要求 663 行
    ├── vernal-log         (2文件骨架)       ✅ 技术要求 635 行
    ├── vernal-test        (2文件骨架)
    └── vernal-actuator    (2文件骨架)

L2  依赖 L1（可并行）
    ├── vernal-beans       (以自动审计为准) ⚠️ 对象迁移未完成
    └── vernal-macros      (12文件)         🔜 编译期过程宏（最关键）

L3  依赖 L2
    └── vernal-context     (87文件)         ✅ 已完成（应用上下文）

L4  依赖 L3（可并行）
    ├── vernal-web         (26文件)         ✅ 已完成（整合 Topcoat）
    ├── vernal-http        (7文件)
    ├── vernal-context-indexer (11文件)     ✅ 已完成
    ├── vernal-context-support (68文件)     ✅ 已完成
    ├── vernal-messaging   (6文件骨架)     🔜 含 JMS 语义融入
    ├── vernal-orm         (800行)          ✅ Toasty 锁定
    └── vernal-rbdc        🔜 通用数据库抽象层（基于 sqlx）

L5  依赖 L4
    ├── vernal-web-testkit (10文件)
    ├── vernal-tower       (22文件)
    ├── vernal-hyper       (2文件)
    ├── vernal-websocket   (99文件)         ✅ 已完成
    ├── vernal-oxm         (608行)          ✅ 已完成
    └── vernal-r2dbc       (727行)          ✅ 已完成

L6  Web 框架适配器（10 个，已完成 ✅）
    ├── vernal-axum / actix-web / poem / salvo / rocket / warp / ntex / gotham / tide
    └── vernal-tonic（gRPC）

L7  顶层门面
    └── vernal              (23行)          🔜 重新启用
```

### 3.1 推进优先级（按"价值/依赖度"排序）

1. **L0 补全**：`vernal-db`（通用数据库抽象层，基于 sqlx）—— 解锁 L4 的 vernal-rbdc/orm
2. **L1 骨架代码补全**：`vernal-async`/`log`/`cache`/`tx`/`test`/`actuator` —— 技术要求已就位，骨架代码待写
3. **L2 的 macros** —— 解锁所有 `#[derive(Component)]` 等过程宏，是 vernal 的关键
4. **L4 的 messaging** —— 解锁 14 broker adapter 与 ddd4r-mq 集成
5. **L5 hyper/tower 骨架** —— 补全 Web 适配器底层
6. **L7 vernal 顶层门面** —— 集成所有内部 crate

### 3.2 各层完成度（2026-07-29）

| 层 | 完成情况 | 说明 |
|---|---|---|
| **L0** | 100% | vernal-core 94 文件，是最稳定的底层 |
| **L1** | 100%（技术要求） | 9 个 crate，骨架代码待补 |
| **L2** | 50% | vernal-beans 完整，macros 骨架 |
| **L3** | 100% | vernal-context 87 文件 |
| **L4** | 60% | web/orm/r2dbc/oxm/context-indexer/context-support 完成 |
| **L5** | 80% | websocket 99 文件；tower/hyper 骨架 |
| **L6** | 100% | 10 个 Web 框架适配器全部完成 |
| **L7** | 5% | vernal 门面骨架，待集成 |
| **文档** | 100% | 23 份技术要求文档（约定+总览+22 模块）|



## 四、底层依赖清单（已选型完整）

### 4.1 核心运行时
| crate | 版本 | 用途 | vernal 集成位置 |
|---|---|---|---|
| `tokio` | 1.x | 异步运行时 | vernal 全栈 |
| `async-trait` | 0.1 | 异步 trait | vernal-aop, vernal-messaging |
| `futures` | 0.3 | Future/Stream | vernal-async |

### 4.2 Web/网络
| crate | 版本 | 用途 | vernal 集成位置 |
|---|---|---|---|
| `http` | 1.x | HTTP 类型 | vernal-http |
| `hyper` | 1.x | HTTP 底层 | vernal-hyper |
| `tower` | 0.5 | Service 抽象 | vernal-tower |
| `axum` | 0.8 | Web 框架 | vernal-axum |
| `actix-web` | 4.x | Web 框架 | vernal-actix-web |
| `tonic` | 0.12 | gRPC | vernal-tonic |
| `tokio-websockets` | 0.12 | WebSocket | vernal-websocket |

### 4.3 数据访问
| crate | 版本 | 用途 | vernal 集成位置 |
|---|---|---|---|
| `sqlx` | 0.9 | 数据库（vernal-rbdc 底层）| vernal-db, vernal-rbdc |
| `toasty` | 0.9 | ORM（vernal-orm 主线）| vernal-orm |
| `rbatis` + `rbdc` | 4.9 | rbatis 独立整合 | vernal-rbatis |
| `redis` | 1.4 | KV/缓存/MQ | vernal-cache, vernal-mq |
| `moka` | 0.12 | 本地缓存（替代 Caffeine/Guava）| vernal-cache |

### 4.4 消息（MQ broker）
| crate | 版本 | 用途 |
|---|---|---|
| `rdkafka` | 0.39 | Kafka |
| `pulsar` | 6.8 | Pulsar |
| `async-nats` | 0.50 | NATS |
| `lapin` | 4.x | RabbitMQ（AMQP 0-9-1）|
| `fe2o3-amqp` | 0.15 | ActiveMQ Artemis（AMQP 1.0）|
| `paho-mqtt` | 0.14 | MQTT |
| `disruptor` | 4.3 | 进程内高性能队列 |

### 4.5 序列化/配置
| crate | 版本 | 用途 |
|---|---|---|
| `serde` | 1.x | 序列化 |
| `serde_json` | 1.x | JSON |
| `quick-xml` | 0.41 | XML |
| `serde_yaml_ng` | 0.10 | YAML |
| `java-properties` | 1.4 | .properties 文件 |
| `props-util` | — | 强类型属性绑定 |

### 4.6 工具/可观测性
| crate | 版本 | 用途 |
|---|---|---|
| `tracing` | 0.1 | 日志 |
| `metrics` | 0.24 | 指标 |
| `tokio-metrics` | 0.5 | Tokio 运行时指标 |
| `sysinfo` | 0.39 | 系统信息 |
| `regex` + `fancy-regex` | 1.13 | 正则 |

### 4.7 安全/加密（RustCrypto 全家桶）
| crate | 用途 |
|---|---|
| `aes` + `cbc` + `ecb` | 对称加密 |
| `aes-gcm` | AEAD |
| `chacha20` + `chacha20poly1305` | 流密码 |
| `rsa` + `p256` + `p384` | 非对称 |
| `sha2` + `hmac` + `sha1` | 摘要 |
| `sm2` + `sm3` + `sm4` | 国密 |
| `argon2` + `pbkdf2` | 密码哈希 |
| `jsonwebtoken` | JWT |
| `x509-cert` | 证书 |

### 4.8 跨语言 FFI（新增）
| crate | 版本 | 用途 |
|---|---|---|
| `uniffi` | — | Kotlin/Java/Swift/Python ↔ Rust 跨语言 FFI |

### 4.9 文档处理
| crate | 版本 | 用途 |
|---|---|---|
| `docx-rs` | 0.4 | Word |
| `rust_xlsxwriter` | 0.96 | Excel 写 |
| `printpdf` | 0.12 | PDF |
| `quick-xml` | 0.41 | OFD/XML |

## 五、vernal 三大原则

### 5.1 不重复造轮子
所有底层选型来自 tokio-rs / hyperium / RustCrypto 等**成熟社区**，
vernal 只做**中间层抽象**，不替代底层实现。

### 5.2 多 Web 框架整合
业务按生态选 Web 框架（Axum/Actix/Rocket/Salvo/...），
vernal 提供统一 API，**框架可换，业务代码不动**。

### 5.3 完整生态
34+ crate 覆盖 IoC / AOP / TX / Web / Data / MQ / Cache / Test / Actuator 全部能力，
对标 Spring Framework 一站式中间层。

## 六、与 Spring 的核心差异

| 维度 | Spring | vernal |
|---|---|---|
| 运行时 | JVM | Tokio |
| 反射 | 运行时反射 | trait object + 过程宏编译期 |
| 类型擦除 | Java 类型擦除 | Rust 单态化（编译期）|
| 字节码增强 | CGLIB / AspectJ | 过程宏 + vernal-aop Interceptor |
| 并发模型 | 线程池 | tokio task |
| 序列化 | Jackson/Kryo | serde |
| Web 框架 | Servlet（单一）| 10 个 Web 框架（可选）|
| ORM | JPA/Hibernate | Toasty（编译期 codegen）|
| 事务 | JTA 分布式 | 进程内 + Outbox（未来分布式）|

## 七、当前完成度

| 类别 | 数量 | 完成度 |
|---|---|---|
| 核心层 | 4 crate | 100%（core/beans/context/expression 全部完成）|
| AOP 层 | 2 crate | 100%（aop/aspects 全部完成）|
| 上下文层 | 2 crate | 100% |
| 数据层 | 5 crate | 60%（orm/r2dbc/oxm 完成，rbdc/db/tx 待补）|
| Web 层 | 15 crate | 90%（10 适配器 + web/http/websocket 完成）|
| 消息层 | 1 crate | 30%（messaging 骨架）|
| 工具/测试 | 4 crate | 90% |
| **文档体系** | **23 份** | **100%（约定+总览+22 技术要求）** |

## 八、推进优先级

按依赖拓扑 + 商业价值：

1. **L1 补全**（cache/tx/async/log/db）—— 解锁上层 1
2. **L2 的 macros** —— 解锁 context-indexer 和所有派生宏 2
3. **L4 messaging 补全** —— 解锁 websocket/14 broker adapter 3
4. **L5 hyper/tower 补全** —— 完整 Web 适配器 4
5. **L7 vernal 门面** —— 集成所有内部 crate 5

每完成一层，立即推进下一层。

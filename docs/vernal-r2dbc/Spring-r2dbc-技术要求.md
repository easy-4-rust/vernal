<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-r2dbc 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> `spring-r2dbc` 的一对一目标模块；目标 crate 尚未建立。

固定提交中有 59 个业务对象，当前全部 `MISSING`。sqlx 的异步能力只是候选底座，不能整体豁免 Spring 的 client、connection、transaction、exception translation 与 database initialization 对象。

| Java 来源 | 目标 Rust |
|---|---|
| `core/DatabaseClient.java` | `core/database_client.rs` |
| `core/DefaultDatabaseClient.java` | `core/default_database_client.rs` |
| `connection/R2dbcTransactionManager.java` | `connection/r2dbc_transaction_manager.rs` |
| `connection/init/ScriptUtils.java` | `connection/init/script_utils.rs` |

结果流用 `Stream<Item = Result<T,E>>`，必须保持取消、错误、连接释放和事务上下文。遵循[迁移验收规范](../迁移验收规范.md)。

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-r2dbc 技术要求（对标 spring-r2dbc）

> **版本**：v1.0（2026-07-28）
> **对标**：`spring-r2dbc` 6.1 / Spring Framework 6.1
> **Rust 基线**：edition 2024 / rustc 1.88
> **crate 现状**：待建（规划中）
> **选型**：sqlx（Rust 无独立 R2DBC，sqlx 已是 async 原生）
> **引用约定**：《Spring 组件替换约定》第 6.2 节（缓存：moka）

---

## 一、概述与定位

### 1.1 crate 职责

`vernal-r2dbc` 是 Vernal Framework 的 **响应式关系型数据库连接抽象层**，对标
Spring Framework 中的 `spring-r2dbc` 模块。它在 `sqlx` 的异步数据库驱动之上，
提供 Spring R2DBC 语义的编程模型，包括 `ConnectionFactory`、`ConnectionPool`、
`R2dbcEntityTemplate` 和 `DatabaseClient` 四大核心抽象。

在 Vernal 数据层架构中，`vernal-r2dbc` 与 `vernal-db`（JdbcTemplate 语义）
互补：

```
数据层全景
  ┌─────────────────────────────────────────────────┐
  │  vernal-r2dbc (本 crate)                         │
  │    ├─ ConnectionFactory    → sqlx::Pool          │
  │    ├─ ConnectionPool       → sqlx::Pool 池化     │
  │    ├─ R2dbcEntityTemplate  → 响应式 CRUD          │
  │    └─ DatabaseClient       → 响应式 SQL 执行      │
  ├─────────────────────────────────────────────────┤
  │  vernal-db (JdbcTemplate 语义)                   │
  │    ├─ JdbcTemplate         → sqlx 同步封装        │
  │    └─ NamedParameterJdbcTemplate                 │
  ├─────────────────────────────────────────────────┤
  │  vernal-tx (事务抽象)                             │
  │    └─ ReactiveTransactionManager                 │
  ├─────────────────────────────────────────────────┤
  │  vernal-beans + vernal-context (IoC/AOP 内核)     │
  └─────────────────────────────────────────────────┘
```

### 1.2 与 spring-r2dbc 的对齐边界

| spring-r2dbc 概念 | vernal-r2dbc 对应 | 说明 |
|:---|:---|:---|
| `ConnectionFactory` | `ConnectionFactory` trait | 连接工厂抽象 |
| `ConnectionPool` / `PoolingConnectionFactory` | `SqlxConnectionPool` | 基于 sqlx::Pool 的连接池 |
| `Connection` | `ReactiveConnection` trait | 单连接操作抽象 |
| `Statement` | `ReactiveStatement` trait | 参数化语句抽象 |
| `Result` | `ReactiveResult` trait | 结果集流式读取 |
| `R2dbcEntityTemplate` | `R2dbcEntityTemplate` struct | 响应式实体 CRUD 模板 |
| `DatabaseClient` | `DatabaseClient` struct | 响应式 SQL 执行客户端 |
| `ReactiveTransactionManager` | 委托 `vernal-tx` | 响应式事务管理 |
| `@EnableR2dbcRepositories` | `vernal-context-indexer` SPI | 仓库自动发现 |
| `R2dbcException` | `R2dbcError` enum | 异常体系 |
| `RowsFetchSpec` | `FetchSpec<T>` | 结果拉取规格 |
| `GenericExecuteSpec` | `ExecuteSpec` | 通用 SQL 执行构建器 |

### 1.3 设计约束

1. **`#![forbid(unsafe_code)]`**：与 vernal-db 保持一致。
2. **sqlx-first**：直接使用 sqlx 的异步连接池和查询构建器。
3. **IoC 桥接**：ConnectionFactory / DatabaseClient 通过 vernal-beans `Container`
   解析，支持依赖注入。
4. **AOP 集成**：事务方法经过 vernal-aop 拦截链，支持 `@Transactional`。
5. **Send + Sync**：所有公共类型满足跨线程边界。
6. **Reactive Streams 语义**：结果集按需拉取（backpressure），不一次性加载全量数据。
7. **编译期校验**：支持 sqlx 的 `query!` / `query_as!` 编译期 SQL 校验（可选）。

### 1.4 为什么不用独立 R2DBC

R2DBC 是 JVM 生态的响应式数据库连接标准（Reactive Streams 驱动）。Rust 生态
不存在独立的 R2DBC 协议实现，原因如下：

- Rust 的 `async/await` 天然就是响应式的，不需要 Reactive Streams 抽象层。
- `sqlx` 已经是 Tokio-native 的异步数据库库，编译期查询校验、连接池、事务
  管理全部原生异步。
- 将 sqlx 包装为 Spring R2DBC 语义即可，无需引入额外的协议层。
- R2DBC 的核心价值（非阻塞、背压、流式结果集）在 Rust 中由 `async/await` +
  `Stream` trait 原生提供。

---

## 二、待建 crate 规划

### 2.1 目录结构

```
crates/vernal-r2dbc/
├── Cargo.toml
└── src/
    ├── lib.rs                          # 入口，重导出公共 API
    ├── connection_factory.rs           # ConnectionFactory trait
    ├── connection_pool.rs              # SqlxConnectionPool 实现
    ├── reactive_connection.rs          # ReactiveConnection trait
    ├── reactive_statement.rs           # ReactiveStatement trait
    ├── reactive_result.rs              # ReactiveResult trait
    ├── r2dbc_entity_template.rs        # R2dbcEntityTemplate
    ├── database_client.rs              # DatabaseClient
    ├── entity_mapping.rs               # 实体映射元数据（Entity trait）
    ├── row_mapper.rs                   # 行映射器 trait（RowMapper）
    ├── type_info.rs                    # R2DBC 类型信息
    ├── value.rs                        # R2dbcValue / FromR2dbcValue
    ├── exceptions.rs                   # R2DBC 异常体系
    └── dialect/
        ├── mod.rs                      # 方言抽象（SqlDialect trait）
        ├── postgres.rs                 # PostgreSQL 方言
        ├── mysql.rs                    # MySQL 方言
        └── sqlite.rs                   # SQLite 方言
```

### 2.2 Cargo.toml 草案

```toml
[package]
name = "vernal-r2dbc"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Vernal R2DBC 响应式数据库连接（对标 spring-r2dbc）"
publish = false

[features]
default = ["postgres"]
postgres = ["dep:sqlx", "sqlx/postgres"]
mysql = ["dep:sqlx", "sqlx/mysql"]
sqlite = ["dep:sqlx", "sqlx/sqlite"]

[dependencies]
bytes.workspace = true
tokio.workspace = true
tracing.workspace = true
vernal-core = { path = "../vernal-core" }
vernal-tx = { path = "../vernal-tx" }
vernal-cache = { path = "../vernal-cache", optional = true }
sqlx = { workspace = true, optional = true }
futures-util.workspace = true
pin-project-lite = "0.2"
thiserror.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
sqlx = { workspace = true, features = ["runtime-tokio"] }

[lints]
workspace = true
```

### 2.3 Feature 矩阵

| Feature | 数据库 | sqlx feature | 说明 |
|:---|:---|:---|:---|
| `postgres` | PostgreSQL | `sqlx/postgres` | 默认启用 |
| `mysql` | MySQL | `sqlx/mysql` | 可选 |
| `sqlite` | SQLite | `sqlx/sqlite` | 可选 |

---

## 三、核心 trait 设计

### 3.1 ConnectionFactory

连接工厂是所有数据库操作的入口，对标 Spring R2DBC 的 `ConnectionFactory`。

```rust
/// 连接工厂 trait，对标 spring-r2dbc ConnectionFactory。
pub trait ConnectionFactory: Send + Sync {
    /// 获取一个响应式连接。
    fn get_connection(
        &self,
    ) -> impl Future<Output = Result<Box<dyn ReactiveConnection>, R2dbcError>> + Send;

    /// 获取连接工厂的元数据。
    fn metadata(&self) -> &ConnectionFactoryMetadata;
}

/// 连接工厂元数据。
pub struct ConnectionFactoryMetadata {
    /// 数据库产品名称（如 "PostgreSQL"、"MySQL"）。
    pub database_product: String,
    /// 数据库版本。
    pub database_version: String,
    /// 驱动名称（如 "sqlx-postgres"）。
    pub driver_name: String,
    /// 驱动版本。
    pub driver_version: String,
}
```

### 3.2 ConnectionPool

连接池是对 `sqlx::Pool` 的封装，提供连接池统计和健康检查。

```rust
/// 连接池 trait，对标 spring-r2dbc ConnectionPool / PoolingConnectionFactory。
pub trait ConnectionPool: ConnectionFactory {
    /// 返回池中空闲连接数。
    fn idle_count(&self) -> usize;

    /// 返回池中总连接数（活跃 + 空闲）。
    fn total_count(&self) -> usize;

    /// 返回池的最大容量。
    fn max_size(&self) -> usize;

    /// 健康检查：尝试获取一个连接并执行 SELECT 1。
    fn is_healthy(&self) -> impl Future<Output = bool> + Send;

    /// 关闭连接池，释放所有连接。
    fn close(&self) -> impl Future<Output = ()> + Send;
}

/// 基于 sqlx::Pool 的连接池实现。
pub struct SqlxConnectionPool<DB: sqlx::Database> {
    pool: sqlx::Pool<DB>,
    metadata: ConnectionFactoryMetadata,
}

impl<DB: sqlx::Database> SqlxConnectionPool<DB> {
    /// 从连接 URL 创建连接池。
    pub async fn new(url: &str, max_connections: u32) -> Result<Self, R2dbcError> { ... }

    /// 从已有的 sqlx::Pool 创建。
    pub fn from_pool(pool: sqlx::Pool<DB>) -> Self { ... }

    /// 获取底层 sqlx::Pool 的引用（用于高级用法）。
    pub fn inner(&self) -> &sqlx::Pool<DB> { ... }
}
```

### 3.3 ReactiveConnection

单连接操作抽象，对标 Spring R2DBC 的 `Connection`。

```rust
/// 响应式连接 trait，对标 spring-r2dbc Connection。
pub trait ReactiveConnection: Send + Sync {
    /// 创建一个 Statement。
    fn create_statement(&self, sql: &str) -> Box<dyn ReactiveStatement>;

    /// 执行一条 SQL（无返回结果集），返回影响行数。
    fn execute(&self, sql: &str)
        -> impl Future<Output = Result<u64, R2dbcError>> + Send;

    /// 开始事务。
    fn begin(&self) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 提交事务。
    fn commit(&self) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 回滚事务。
    fn rollback(&self) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 设置自动提交。
    fn set_auto_commit(
        &self,
        auto_commit: bool,
    ) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 创建保存点。
    fn create_savepoint(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 回滚到保存点。
    fn rollback_to_savepoint(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 关闭连接。
    fn close(&self) -> impl Future<Output = Result<(), R2dbcError>> + Send;

    /// 是否已关闭。
    fn is_closed(&self) -> bool;
}
```

### 3.4 ReactiveStatement

参数化语句抽象，对标 Spring R2DBC 的 `Statement`。

```rust
/// 响应式语句 trait，对标 spring-r2dbc Statement。
pub trait ReactiveStatement: Send + Sync {
    /// 绑定参数（位置索引，从 0 开始）。
    fn bind(&mut self, index: usize, value: R2dbcValue) -> &mut Self;

    /// 绑定 NULL 参数（位置索引）。
    fn bind_null(&mut self, index: usize) -> &mut Self;

    /// 返回自增列。
    fn return_generated_values(&mut self, columns: &[&str]) -> &mut Self;

    /// 执行查询，返回结果集。
    fn execute(
        &self,
    ) -> impl Future<Output = Result<Box<dyn ReactiveResult>, R2dbcError>> + Send;
}

/// 命名参数语句 trait（扩展）。
pub trait NamedParameterStatement: ReactiveStatement {
    /// 绑定命名参数。
    fn bind_by_name(&mut self, name: &str, value: R2dbcValue) -> &mut Self;
}
```

### 3.5 ReactiveResult 与 Row

结果集流式读取抽象，对标 Spring R2DBC 的 `Result` 和 `Row`。

```rust
/// 响应式结果集 trait，对标 spring-r2dbc Result。
pub trait ReactiveResult: Send + Sync {
    /// 获取列元数据。
    fn column_metadata(&self) -> &[ColumnMetadata];

    /// 获取影响行数（DML 语句）。
    fn rows_updated(&self) -> u64;

    /// 按行映射读取所有行。
    fn map<T>(
        &mut self,
        row_mapper: impl FnMut(&dyn Row) -> Result<T, R2dbcError>,
    ) -> impl Future<Output = Result<Vec<T>, R2dbcError>> + Send;

    /// 读取下一行。
    fn fetch_next(
        &mut self,
    ) -> impl Future<Output = Result<Option<Box<dyn Row>>, R2dbcError>> + Send;
}

/// 行数据抽象，对标 spring-r2dbc Row。
pub trait Row: Send + Sync {
    /// 按索引获取列值。
    fn get<T: FromR2dbcValue>(&self, index: usize) -> Result<T, R2dbcError>;

    /// 按列名获取列值。
    fn get_by_name<T: FromR2dbcValue>(&self, name: &str) -> Result<T, R2dbcError>;
}

/// 列元数据。
pub struct ColumnMetadata {
    pub name: String,
    pub type_info: R2dbcTypeInfo,
    pub nullable: Option<bool>,
    pub ordinal: usize,
}
```

---

## 四、核心实现：R2dbcEntityTemplate 与 DatabaseClient

### 4.1 R2dbcEntityTemplate

`R2dbcEntityTemplate` 对标 Spring R2DBC 的 `R2dbcEntityTemplate`，提供基于实体
的响应式 CRUD 操作。

```rust
/// 响应式实体模板，对标 spring-r2dbc R2dbcEntityTemplate。
pub struct R2dbcEntityTemplate {
    connection_factory: Arc<dyn ConnectionFactory>,
    dialect: Box<dyn SqlDialect>,
}

impl R2dbcEntityTemplate {
    pub fn new(connection_factory: Arc<dyn ConnectionFactory>) -> Self { ... }

    pub fn with_dialect(
        connection_factory: Arc<dyn ConnectionFactory>,
        dialect: Box<dyn SqlDialect>,
    ) -> Self { ... }

    /// 插入一条记录，返回插入后的实体（含自增 ID）。
    pub async fn insert<T: Entity>(&self, entity: &T) -> Result<T, R2dbcError> { ... }

    /// 按 ID 查询单条记录。
    pub async fn select_one<T: Entity>(
        &self, id: &T::Id,
    ) -> Result<Option<T>, R2dbcError> { ... }

    /// 条件查询。
    pub async fn select<T: Entity>(
        &self, criteria: &Criteria,
    ) -> Result<Vec<T>, R2dbcError> { ... }

    /// 条件查询，带分页。
    pub async fn select_paged<T: Entity>(
        &self, criteria: &Criteria, page: PageRequest,
    ) -> Result<Page<T>, R2dbcError> { ... }

    /// 更新一条记录（乐观锁检查）。
    pub async fn update<T: Entity>(&self, entity: &T) -> Result<T, R2dbcError> { ... }

    /// 按 ID 删除。
    pub async fn delete<T: Entity>(&self, id: &T::Id) -> Result<u64, R2dbcError> { ... }

    /// 条件删除。
    pub async fn delete_by<T: Entity>(
        &self, criteria: &Criteria,
    ) -> Result<u64, R2dbcError> { ... }

    /// 条件计数。
    pub async fn count<T: Entity>(
        &self, criteria: &Criteria,
    ) -> Result<u64, R2dbcError> { ... }

    /// 条件判断是否存在。
    pub async fn exists<T: Entity>(
        &self, criteria: &Criteria,
    ) -> Result<bool, R2dbcError> { ... }
}
```

### 4.2 DatabaseClient

`DatabaseClient` 对标 Spring R2DBC 的 `DatabaseClient`，提供通用的 SQL 执行
能力，不依赖实体映射。

```rust
/// 响应式 SQL 客户端，对标 spring-r2dbc DatabaseClient。
pub struct DatabaseClient {
    connection_factory: Arc<dyn ConnectionFactory>,
}

impl DatabaseClient {
    pub fn new(connection_factory: Arc<dyn ConnectionFactory>) -> Self { ... }

    pub fn sql(&self, sql: impl Into<String>) -> ExecuteSpec { ... }
    pub fn insert_into(&self, table: &str) -> InsertSpec { ... }
    pub fn update(&self, table: &str) -> UpdateSpec { ... }
    pub fn delete_from(&self, table: &str) -> DeleteSpec { ... }
    pub fn select_from(&self, table: &str) -> SelectSpec { ... }
}

/// SQL 执行构建器，对标 spring-r2dbc GenericExecuteSpec。
pub struct ExecuteSpec {
    sql: String,
    bindings: Vec<R2dbcValue>,
    fetch_size: Option<usize>,
}

impl ExecuteSpec {
    pub fn bind(mut self, value: impl Into<R2dbcValue>) -> Self { ... }
    pub fn bind_null(mut self) -> Self { ... }
    pub fn fetch_size(mut self, size: usize) -> Self { ... }

    pub async fn map<T>(
        self,
        row_mapper: impl FnMut(&dyn Row) -> Result<T, R2dbcError>,
    ) -> Result<FetchSpec<T>, R2dbcError> { ... }

    pub async fn fetch(self) -> Result<RowsUpdatedSpec, R2dbcError> { ... }
    pub async fn fetch_one<T: FromR2dbcValue>(self) -> Result<T, R2dbcError> { ... }
    pub async fn first<T: FromR2dbcValue>(self) -> Result<Option<T>, R2dbcError> { ... }
}

/// 拉取结果规格，对标 spring-r2dbc RowsFetchSpec。
pub struct FetchSpec<T> {
    rows: Vec<T>,
    rows_updated: u64,
}

impl<T> FetchSpec<T> {
    pub fn all(self) -> Vec<T> { self.rows }
    pub fn first(self) -> Option<T> { self.rows.into_iter().next() }
    pub fn one(self) -> Result<Option<T>, R2dbcError> { ... }
    pub fn rows_updated(&self) -> u64 { self.rows_updated }
}
```

### 4.3 Entity trait

```rust
/// 实体 trait，定义映射元数据。
pub trait Entity: Send + Sync + Clone + 'static {
    type Id: Send + Sync + Clone + PartialEq + Eq + std::hash::Hash;

    fn table_name() -> &'static str;
    fn id_column() -> &'static str;
    fn id(&self) -> &Self::Id;
    fn from_row(row: &dyn Row) -> Result<Self, R2dbcError>;
    fn to_insert_params(&self) -> Vec<R2dbcValue>;
    fn to_update_params(&self) -> Vec<R2dbcValue>;
    fn columns() -> &'static [&'static str];
    fn version_column() -> Option<&'static str> { None }
    fn version(&self) -> Option<i64> { None }
}
```

---

## 五、与 vernal-tx / vernal-cache 的集成

### 5.1 响应式事务

响应式事务管理委托给 `vernal-tx`，`vernal-r2dbc` 提供事务上下文传播：

```rust
/// 响应式事务上下文。
pub struct ReactiveTransactionContext {
    connection: Arc<dyn ReactiveConnection>,
}

impl ReactiveTransactionContext {
    /// 在事务中执行闭包。
    pub async fn execute<T, F, Fut>(&self, f: F) -> Result<T, R2dbcError>
    where
        F: FnOnce(Arc<dyn ReactiveConnection>) -> Fut,
        Fut: Future<Output = Result<T, R2dbcError>>,
    {
        self.connection.begin().await?;
        match f(self.connection.clone()).await {
            Ok(result) => {
                self.connection.commit().await?;
                Ok(result)
            }
            Err(e) => {
                self.connection.rollback().await?;
                Err(e)
            }
        }
    }

    /// 在嵌套事务中执行（使用保存点）。
    pub async fn execute_nested<T, F, Fut>(
        &self, savepoint: &str, f: F,
    ) -> Result<T, R2dbcError>
    where
        F: FnOnce(Arc<dyn ReactiveConnection>) -> Fut,
        Fut: Future<Output = Result<T, R2dbcError>>,
    {
        self.connection.create_savepoint(savepoint).await?;
        match f(self.connection.clone()).await {
            Ok(result) => {
                self.connection
                    .execute(&format!("RELEASE SAVEPOINT {savepoint}"))
                    .await?;
                Ok(result)
            }
            Err(e) => {
                self.connection.rollback_to_savepoint(savepoint).await?;
                Err(e)
            }
        }
    }
}
```

### 5.2 @Transactional 支持

通过 vernal-aop 拦截链，响应式方法上的 `#[transactional]` 注解自动开启事务：

```rust
#[transactional]
async fn transfer(
    &self, from: i64, to: i64, amount: Decimal,
) -> Result<(), R2dbcError> {
    let from_account = self.template.select_one::<Account>(&from).await?.unwrap();
    let to_account = self.template.select_one::<Account>(&to).await?.unwrap();
    // ... 业务逻辑
    self.template.update(&from_account).await?;
    self.template.update(&to_account).await?;
    Ok(())
}
```

### 5.3 引用约定 6.2：缓存集成

参照《Spring 组件替换约定》第 6.2 节（缓存：moka），`R2dbcEntityTemplate` 的
查询结果可通过 `vernal-cache` 缓存。moka 是 Vernal 生态的统一缓存后端，
vernal-cache 和 vernal-expression 已使用。

```rust
/// 带缓存的查询。
pub async fn select_cached<T: Entity>(
    &self, id: &T::Id, cache: &vernal_cache::Cache<T::Id, T>,
) -> Result<Option<T>, R2dbcError> {
    if let Some(cached) = cache.get(id) {
        return Ok(Some(cached));
    }
    let result = self.select_one(id).await?;
    if let Some(ref entity) = result {
        cache.insert(id.clone(), entity.clone());
    }
    Ok(result)
}

/// 缓存失效（写操作后）。
pub async fn invalidate_cache<T: Entity>(
    &self, id: &T::Id, cache: &vernal_cache::Cache<T::Id, T>,
) {
    cache.invalidate(id);
}
```

### 5.4 与 vernal-db 的关系

| 场景 | 推荐使用 | 说明 |
|:---|:---|:---|
| 同步请求处理 | `vernal-db` (JdbcTemplate) | 适合简单 CRUD、管理后台 |
| 响应式流处理 | `vernal-r2dbc` (DatabaseClient) | 适合大数据量、流式读取 |
| 实体映射 CRUD | `vernal-r2dbc` (R2dbcEntityTemplate) | 适合实体驱动的领域模型 |
| 原生 SQL | `vernal-db` 或 `vernal-r2dbc` | 均支持，按同步/异步选择 |
| 事务管理 | 均通过 `vernal-tx` | 统一事务抽象 |

---

## 六、异常体系与测试策略

### 6.1 R2DBC 异常体系

对标 Spring R2DBC 的 `R2dbcException` 层次结构：

```rust
#[derive(Debug, thiserror::Error)]
pub enum R2dbcError {
    #[error("连接错误: {message}")]
    ConnectionError {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("SQL 语法错误: {message}")]
    BadSqlGrammar { message: String, sql_state: Option<String> },

    #[error("数据完整性违反: {message}")]
    DataIntegrityViolation { message: String, sql_state: Option<String> },

    #[error("数据访问资源错误: {message}")]
    DataAccessResourceFailure { message: String },

    #[error("瞬态资源错误: {message}")]
    TransientResourceError { message: String },

    #[error("乐观锁冲突: {message}")]
    OptimisticLockingFailure { message: String },

    #[error("结果集为空")]
    EmptyResult,

    #[error("非唯一结果: 期望 1 条，实际 {actual} 条")]
    NonUniqueResult { actual: usize },

    #[error("类型转换错误: {message}")]
    TypeMismatch { message: String },

    #[error("SQL 错误: {message}")]
    Uncategorized {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
```

### 6.2 异常映射规则

| sqlx 错误类型 | R2DBC 异常 | 说明 |
|:---|:---|:---|
| `sqlx::Error::PoolTimedOut` | `TransientResourceError` | 连接池超时，可重试 |
| `sqlx::Error::PoolClosed` | `ConnectionError` | 连接池已关闭 |
| `sqlx::Error::Io` | `ConnectionError` | 网络 IO 错误 |
| `sqlx::Error::Database(e)` | 按 SQL state 映射 | 数据库原生错误 |
| `sqlx::Error::RowNotFound` | `EmptyResult` | 行未找到 |
| `sqlx::Error::ColumnIndexOutOfBounds` | `TypeMismatch` | 列索引越界 |
| `sqlx::Error::Decode` | `TypeMismatch` | 解码失败 |
| `sqlx::Error::Protocol` | `BadSqlGrammar` | 协议错误 |
| `sqlx::Error::Configuration` | `ConnectionError` | 配置错误 |

### 6.3 SQL State 映射

| SQL State 前缀 | R2DBC 异常 | 说明 |
|:---|:---|:---|
| `23` | `DataIntegrityViolation` | 约束违反 |
| `42` | `BadSqlGrammar` | 语法错误 |
| `08` | `ConnectionError` | 连接异常 |
| `40` | `DataIntegrityViolation` | 事务回滚 |
| `57` | `DataAccessResourceFailure` | 操作符干预 |
| 其他 | `Uncategorized` | 未分类 |

### 6.4 测试策略

#### 单元测试

- `ConnectionFactory` trait mock 实现（`MockConnectionFactory`）
- `ReactiveConnection` trait mock 实现（`MockReactiveConnection`）
- `R2dbcEntityTemplate` CRUD 操作 mock 测试
- `DatabaseClient` SQL 构建测试（`ExecuteSpec` / `InsertSpec` 等）
- 异常映射测试（sqlx 错误 → R2dbcError）

#### 集成测试

- PostgreSQL 集成测试（sqlx-test 容器或 testcontainers）
- MySQL 集成测试（sqlx-test 容器或 testcontainers）
- SQLite 集成测试（内存数据库 `:memory:`）
- 连接池压力测试（并发获取/释放）
- 事务隔离级别测试（READ COMMITTED / REPEATABLE READ）
- 保存点嵌套测试
- 乐观锁冲突测试

#### 性能基准

- 连接池获取延迟（`criterion` 基准）
- 批量插入吞吐量（1000 / 10000 / 100000 行）
- 流式读取内存占用（vs 全量加载）
- 与 `vernal-db` 同步模式的性能对比

### 6.5 实施路线图

| 阶段 | 内容 | 依赖 | 预计工作量 |
|:---|:---|:---|:---|
| P0 | ConnectionFactory + ConnectionPool + ReactiveConnection | sqlx | 3 天 |
| P1 | ReactiveStatement + ReactiveResult + Row + R2dbcValue | sqlx | 3 天 |
| P2 | DatabaseClient（SQL 执行 + FetchSpec） | P0 + P1 | 2 天 |
| P3 | R2dbcEntityTemplate（实体 CRUD） | P2 | 3 天 |
| P4 | Entity trait + derive 宏 | P0 | 2 天 |
| P5 | vernal-tx 事务集成 + 保存点 | P0 | 2 天 |
| P6 | vernal-aop @Transactional 集成 | P5 | 1 天 |
| P7 | 方言适配（PostgreSQL / MySQL / SQLite） | P0 | 2 天 |
| P8 | 异常体系 + sqlx 错误映射 | P0 | 1 天 |
| P9 | vernal-cache 缓存集成 | P3 | 1 天 |
| P10 | 集成测试 + 性能基准 | P0-P9 | 3 天 |

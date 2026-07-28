# vernal-jdbc 技术要求（对标 spring-jdbc）

> **版本**：v2.0（2026-07-28）
> **定位**：vernal-jdbc crate 技术交接文档，对标 Spring Framework 7.0.8 spring-jdbc。
> **选型**：sqlx 0.9（Tokio-first async）为实现主线，spring-jdbc 仅作语义参考。
> **现状**：2 文件 / 19 行骨架，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》6.2 节（JDBC/R2DBC）。

---

## 一、总览

### 1.1 定位与边界

vernal-jdbc 是 Vernal Framework 的 **Tokio-first 异步数据库访问内核**，
对标 spring-jdbc 模块，提供统一的 JDBC 风格数据库操作抽象。

**核心范式转换**：Java 同步 JDBC → Rust 异步 sqlx。

| 维度 | spring-jdbc（语义参考） | vernal-jdbc（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（同步阻塞 JDBC） | Rust（async sqlx） | 同步 → 异步范式转换 |
| 驱动 | JDBC Driver（java.sql.*） | sqlx（原生 async 驱动） | 无 ODBC/JDBC 桥接 |
| 连接池 | HikariCP / DBCP | sqlx 内建连接池 | sqlx Pool<T> 自带 |
| SQL 校验 | 运行时 | **编译期**（`sqlx::query!`） | Rust 独有优势 |
| 参数绑定 | `?` 占位符（位置） | `$1, $2`（PostgreSQL）/ `?`（MySQL/SQLite） | 数据库方言差异 |
| Row 映射 | `RowMapper<T>` | `FromRow` derive / 手动 | 编译期静态分发 |
| 命名参数 | `NamedParameterJdbcTemplate` | sqlx 原生 `$name` | PostgreSQL 原生支持 |
| 事务管理 | `DataSourceTransactionManager` | 集成 vernal-tx | 跨 crate 协作 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  便捷层：SimpleJdbcInsert / SimpleJdbcCall           │
│  → 表名 + Map 参数 → 自动生成 INSERT/存储过程 SQL    │
├─────────────────────────────────────────────────────┤
│  模板层：JdbcTemplate / NamedParameterJdbcTemplate   │
│  → query / update / batch_update / execute           │
├─────────────────────────────────────────────────────┤
│  行映射层：RowMapper<T> trait / FromRow derive       │
│  → 行数据 → Rust 结构体                             │
├─────────────────────────────────────────────────────┤
│  数据源层：DataSource trait + sqlx::Pool              │
│  → 连接获取 / 连接池管理                             │
├─────────────────────────────────────────────────────┤
│  事务层：集成 vernal-tx PlatformTransactionManager    │
│  → DataSourceTransactionManager                      │
├─────────────────────────────────────────────────────┤
│  驱动层：sqlx（PostgreSQL / MySQL / SQLite）          │
│  → 原生 async TCP 连接                              │
└─────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 驱动选型 | sqlx 0.9 | Rust 生态最成熟的 async SQL 库，编译期校验 |
| 连接池 | sqlx 内建 `Pool<T>` | 无需外部连接池（HikariCP 对等物） |
| SQL 校验 | 编译期（`sqlx::query!`）+ 运行时（`sqlx::query()`） | 灵活选择 |
| 行映射 | `FromRow` derive + 手动 `RowMapper` trait | 编译期优先，运行时兜底 |
| 命名参数 | PostgreSQL 原生 `$name` / MySQL `?` 占位符 | 跟随数据库方言 |
| 批量操作 | `sqlx::QueryBuilder` | 高效批量 INSERT/UPDATE |

### 1.4 当前骨架文件

| 文件 | 行数 | 内容 |
|:---|:---|:---|
| `lib.rs` | 7 | 模块声明 + re-export |
| `datasource.rs` | 12 | `DataSource` trait（name + is_connected） |

---

## 二、核心 Trait 体系

### 2.1 DataSource —— 数据源抽象

**现状**：已定义 trait 骨架（2 个方法）。
**语义参照**：spring-jdbc `DataSource`（javax.sql.DataSource）。

#### Spring API（Java）

```java
// javax.sql.DataSource（Spring JDBC 依赖）
public interface DataSource extends CommonDataSource, Wrapper {
    Connection getConnection() throws SQLException;
    Connection getConnection(String username, String password) throws SQLException;
}
```

#### Rust trait（已有 + 待扩展）

```rust
/// 数据源 trait。
/// 对标 Spring DataSource，提供连接获取能力。
pub trait DataSource: Send + Sync {
    /// 获取数据源名称。
    fn name(&self) -> &str;

    /// 获取连接状态。
    fn is_connected(&self) -> bool;

    /// 获取数据库类型。
    fn database_type(&self) -> DatabaseType;
}

/// 数据库类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}
```

#### sqlx::Pool 适配

```rust
/// sqlx PostgreSQL 数据源适配器。
pub struct SqlxPgDataSource {
    pool: sqlx::PgPool,
    name: String,
}

impl SqlxPgDataSource {
    pub async fn new(url: &str, name: impl Into<String>) -> Result<Self, sqlx::Error>;
    pub fn pool(&self) -> &sqlx::PgPool;
}

impl DataSource for SqlxPgDataSource {
    fn name(&self) -> &str { &self.name }
    fn is_connected(&self) -> bool { !self.pool.is_closed() }
    fn database_type(&self) -> DatabaseType { DatabaseType::PostgreSQL }
}

/// sqlx MySQL 数据源适配器。
pub struct SqlxMySqlDataSource {
    pool: sqlx::MySqlPool,
    name: String,
}

/// sqlx SQLite 数据源适配器。
pub struct SqlxSqliteDataSource {
    pool: sqlx::SqlitePool,
    name: String,
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享 |
| 连接获取 | async | sqlx 连接池异步获取 |
| 连接池 | sqlx `Pool<T>` 内建 | 无需外部依赖 |
| 生命周期 | `'static` | 连接池可存入 Arc |

---

### 2.2 JdbcTemplate —— 核心模板

**现状**：⬜ 待实现。
**语义参照**：spring-jdbc `JdbcTemplate`。

#### Spring API（Java）

```java
// spring-jdbc 核心模板
public class JdbcTemplate extends JdbcAccessor implements JdbcOperations {
    public <T> T query(String sql, RowMapper<T> rm, Object... args);
    public int update(String sql, Object... args);
    public int[] batchUpdate(String sql, List<Object[]> batchArgs);
    public void execute(String sql);
    public <T> T queryForObject(String sql, RowMapper<T> rm, Object... args);
    public Map<String, Object> queryForMap(String sql, Object... args);
    public List<Map<String, Object>> queryForList(String sql, Object... args);
}
```

#### Rust 目标设计

```rust
/// JDBC 风格模板。
/// 对标 Spring JdbcTemplate，封装 sqlx 查询操作。
pub struct JdbcTemplate {
    pool: Arc<dyn AnyPool>,
}

impl JdbcTemplate {
    pub fn new(pool: Arc<dyn AnyPool>) -> Self;

    /// 查询多行。
    pub async fn query<F, T>(
        &self,
        sql: &str,
        mapper: F,
    ) -> Result<Vec<T>, DbError>
    where
        F: Fn(&dyn Row) -> Result<T, DbError>;

    /// 查询单行。
    pub async fn query_for_object<F, T>(
        &self,
        sql: &str,
        mapper: F,
    ) -> Result<T, DbError>
    where
        F: Fn(&dyn Row) -> Result<T, DbError>;

    /// 查询单值。
    pub async fn query_for_value<T: sqlx::Type<DB> + sqlx::Decode<'_, DB>>(
        &self,
        sql: &str,
    ) -> Result<T, DbError>;

    /// 更新（INSERT/UPDATE/DELETE）。
    pub async fn update(&self, sql: &str) -> Result<u64, DbError>;

    /// 带参数更新。
    pub async fn update_with(
        &self,
        sql: &str,
        params: &[&dyn sqlx::Encode<'_, DB>],
    ) -> Result<u64, DbError>;

    /// 批量更新。
    pub async fn batch_update(
        &self,
        sql: &str,
        batch_args: Vec<Vec<Box<dyn sqlx::Encode<'_, DB>>>>,
    ) -> Result<Vec<u64>, DbError>;

    /// 执行任意 SQL（DDL 等）。
    pub async fn execute(&self, sql: &str) -> Result<(), DbError>;

    /// 查询返回 Map 列表。
    pub async fn query_for_list(
        &self,
        sql: &str,
    ) -> Result<Vec<HashMap<String, SqlValue>>, DbError>;

    /// 查询返回单个 Map。
    pub async fn query_for_map(
        &self,
        sql: &str,
    ) -> Result<HashMap<String, SqlValue>, DbError>;
}
```

#### 与 Spring JdbcTemplate 的映射

| Spring 方法 | Rust 方法 | 说明 |
|:---|:---|:---|
| `query(sql, RowMapper, args)` | `query(sql, closure)` | 闭包替代 RowMapper 接口 |
| `queryForObject(sql, RowMapper, args)` | `query_for_object(sql, closure)` | 语义对齐 |
| `queryForMap(sql, args)` | `query_for_map(sql)` | 返回 HashMap |
| `queryForList(sql, args)` | `query_for_list(sql)` | 返回 Vec<HashMap> |
| `update(sql, args)` | `update(sql)` / `update_with(sql, params)` | 分离有参/无参 |
| `batchUpdate(sql, batchArgs)` | `batch_update(sql, batch_args)` | 批量操作 |
| `execute(sql)` | `execute(sql)` | DDL/任意 SQL |

---

## 三、行映射与参数绑定

### 3.1 RowMapper trait —— 行数据映射

**语义参照**：spring-jdbc `RowMapper<T>`。

#### Spring API（Java）

```java
// spring-jdbc 行映射器
@FunctionalInterface
public interface RowMapper<T> {
    T mapRow(ResultSet rs, int rowNum) throws SQLException;
}
```

#### Rust trait

```rust
/// 行映射器 trait。
/// 对标 Spring RowMapper<T>。
/// 推荐使用 sqlx FromRow derive 替代手动实现。
pub trait RowMapper<T>: Send + Sync {
    fn map_row(&self, row: &dyn Row) -> Result<T, DbError>;
}

/// 为闭包自动实现 RowMapper。
impl<T, F> RowMapper<T> for F
where
    F: Fn(&dyn Row) -> Result<T, DbError> + Send + Sync,
{
    fn map_row(&self, row: &dyn Row) -> Result<T, DbError> {
        (self)(row)
    }
}
```

#### sqlx FromRow derive（推荐）

```rust
/// 使用 sqlx FromRow derive 自动映射。
/// 编译期生成映射代码，零运行时开销。
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

// 使用方式
let users: Vec<User> = sqlx::query_as::<_, User>("SELECT * FROM users")
    .fetch_all(&pool)
    .await?;
```

#### RowMapper vs FromRow 对比

| 维度 | RowMapper trait | FromRow derive |
|:---|:---|:---|
| 绑定时机 | 运行时 | 编译期 |
| 灵活性 | 高（可动态映射） | 中（固定结构体） |
| 性能 | 闭包调用开销 | 零开销 |
| 推荐场景 | 动态查询 / 多表联合 | 固定表结构 |
| spring-jdbc 对应 | `RowMapper<T>` 接口 | `@Column` 注解 |

---

### 3.2 参数绑定

**语义参照**：spring-jdbc `PreparedStatementSetter` / `SqlParameterSource`。

#### Spring API（Java）

```java
// spring-jdbc 参数绑定
public interface PreparedStatementSetter {
    void setValues(PreparedStatement ps) throws SQLException;
}
public interface SqlParameterSource {
    Object getValue(String paramName) throws IllegalArgumentException;
    boolean hasValue(String paramName);
}
```

#### Rust 参数绑定方式

```rust
// 方式 1：位置参数（MySQL / SQLite）
sqlx::query("SELECT * FROM users WHERE id = ? AND name = ?")
    .bind(1i64)
    .bind("Alice")
    .fetch_one(&pool)
    .await?;

// 方式 2：命名参数（PostgreSQL 原生）
sqlx::query("SELECT * FROM users WHERE id = $1 AND name = $2")
    .bind(1i64)
    .bind("Alice")
    .fetch_one(&pool)
    .await?;

// 方式 3：结构体绑定
#[derive(sqlx::FromRow)]
struct UserQuery {
    id: i64,
    name: String,
}

sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE id = $1 AND name = $2"
)
.bind(1i64)
.bind("Alice")
.fetch_one(&pool)
.await?;
```

#### sqlx QueryBuilder（动态 SQL）

```rust
// 动态构建 SQL（对应 spring-jdbc SqlParameterSource）
let mut builder = sqlx::QueryBuilder::new(
    "INSERT INTO users (name, email) "
);
builder.push_values(
    users.iter(),
    |mut b, user| {
        b.push_bind(&user.name)
         .push_bind(&user.email);
    }
);
let result = builder.build().execute(&pool).await?;
```

---

### 3.3 错误处理

```rust
/// 数据库错误。
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("连接错误: {0}")]
    Connection(String),

    #[error("查询错误: {0}")]
    Query(String),

    #[error("映射错误: {0}")]
    Mapping(String),

    #[error("事务错误: {0}")]
    Transaction(String),

    #[error("无结果")]
    NotFound,

    #[error("多结果（期望单行）")]
    MultipleResults,
}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => DbError::NotFound,
            sqlx::Error::Database(d) => DbError::Query(d.message().to_string()),
            sqlx::Error::Io(io) => DbError::Connection(io.to_string()),
            other => DbError::Query(other.to_string()),
        }
    }
}
```

---

## 四、DataSourceTransactionManager —— 事务集成

### 4.1 事务管理器实现

**现状**：⬜ 待实现。
**语义参照**：spring-jdbc `DataSourceTransactionManager`。

#### Spring API（Java）

```java
// spring-jdbc 事务管理器
public class DataSourceTransactionManager extends AbstractPlatformTransactionManager
        implements ResourceTransactionManager {
    public DataSourceTransactionManager(DataSource dataSource) { ... }
    protected Object doGetTransaction() { ... }
    protected void doBegin(Object transaction, TransactionDefinition definition) { ... }
    protected void doCommit(DefaultTransactionStatus status) { ... }
    protected void doRollback(DefaultTransactionStatus status) { ... }
}
```

#### Rust 目标设计

```rust
/// 数据源事务管理器。
/// 对标 Spring DataSourceTransactionManager。
/// 实现 vernal-tx PlatformTransactionManager trait。
pub struct DataSourceTransactionManager {
    pool: Arc<dyn AnyPool>,
    name: String,
}

impl DataSourceTransactionManager {
    pub fn new(pool: Arc<dyn AnyPool>, name: impl Into<String>) -> Self;
}

/// 内部事务对象（对应 Spring DataSourceTransactionObject）。
struct DataSourceTransactionObject {
    connection: Option<Box<dyn AnyConnection>>,
    savepoint: Option<Box<dyn AnySavepoint>>,
    isolation_level: Option<Isolation>,
    read_only: bool,
}

impl PlatformTransactionManager for DataSourceTransactionManager {
    fn get_transaction(
        &self,
        definition: &TransactionDefinition,
    ) -> Result<TransactionStatus, TransactionError> {
        // 1. 根据 Propagation 决定行为
        // 2. 从连接池获取连接
        // 3. 设置隔离级别
        // 4. 开始事务
        todo!()
    }

    fn commit(&self, status: TransactionStatus) -> Result<(), TransactionError> {
        // 1. 检查 rollback_only
        // 2. 触发 before_commit 同步回调
        // 3. 提交事务
        // 4. 释放连接
        // 5. 触发 after_commit 同步回调
        todo!()
    }

    fn rollback(&self, status: TransactionStatus) -> Result<(), TransactionError> {
        // 1. 回滚事务（或回滚到 Savepoint）
        // 2. 释放连接
        // 3. 触发 after_completion 同步回调
        todo!()
    }
}
```

#### 事务生命周期

```
get_transaction()
  ├── 检查 Propagation
  ├── pool.acquire() → 获取连接
  ├── SET TRANSACTION ISOLATION LEVEL ...
  ├── connection.begin() → 开始事务
  └── 返回 TransactionStatus

commit(status)
  ├── 检查 status.rollback_only → 如果 true 则 rollback
  ├── trigger before_commit
  ├── connection.commit()
  ├── connection.release()
  └── trigger after_commit / after_completion(Committed)

rollback(status)
  ├── connection.rollback() 或 rollback_to_savepoint()
  ├── connection.release()
  └── trigger after_completion(RolledBack)
```

---

### 4.2 连接管理

```rust
/// 连接持有者（事务期间持有连接）。
/// 对应 spring-jdbc DataSourceUtils.getConnection()。
pub struct ConnectionHolder {
    connection: Box<dyn AnyConnection>,
    transaction_active: bool,
}

impl ConnectionHolder {
    pub fn new(connection: Box<dyn AnyConnection>) -> Self;
    pub fn is_transaction_active(&self) -> bool;
    pub fn set_transaction_active(&mut self, active: bool);
}

/// 连接工具类。
/// 对标 Spring DataSourceUtils。
pub struct DataSourceUtils;

impl DataSourceUtils {
    /// 获取当前事务绑定的连接（从 TransactionSynchronizationManager）。
    pub async fn get_connection(
        pool: &dyn AnyPool,
    ) -> Result<Box<dyn AnyConnection>, DbError>;

    /// 释放连接（如果不在事务中则归还连接池）。
    pub async fn release_connection(
        connection: Box<dyn AnyConnection>,
        pool: &dyn AnyPool,
    );
}
```

---

## 五、SimpleJdbcInsert —— 便捷插入

### 5.1 SimpleJdbcInsert

**现状**：⬜ 待实现。
**语义参照**：spring-jdbc `SimpleJdbcInsert`。

#### Spring API（Java）

```java
// spring-jdbc 便捷插入
public class SimpleJdbcInsert extends AbstractJdbcInsert {
    public SimpleJdbcInsert withTableName(String tableName);
    public SimpleJdbcInsert usingColumns(String... columnNames);
    public SimpleJdbcInsert usingGeneratedKeyColumns(String... columnNames);
    public Number execute(MapSqlParameterSource parameterSource);
    public Number execute(SqlParameterSource parameterSource);
}
```

#### Rust 目标设计

```rust
/// 简单 JDBC 插入。
/// 对标 Spring SimpleJdbcInsert。
pub struct SimpleJdbcInsert {
    table_name: String,
    columns: Vec<String>,
    generated_key_columns: Vec<String>,
    pool: Arc<dyn AnyPool>,
}

impl SimpleJdbcInsert {
    pub fn new(pool: Arc<dyn AnyPool>, table_name: impl Into<String>) -> Self;

    pub fn with_columns(mut self, columns: &[&str]) -> Self;
    pub fn with_generated_key_columns(mut self, columns: &[&str]) -> Self;

    /// 执行插入。
    pub async fn execute(
        &self,
        values: &HashMap<String, SqlValue>,
    ) -> Result<InsertResult, DbError>;

    /// 执行批量插入。
    pub async fn execute_batch(
        &self,
        batch: &[HashMap<String, SqlValue>],
    ) -> Result<Vec<InsertResult>, DbError>;

    /// 使用结构体执行插入。
    pub async fn execute_with_row<T: sqlx::Encode<'_, DB>>(
        &self,
        row: &T,
    ) -> Result<InsertResult, DbError>;
}

/// 插入结果。
pub struct InsertResult {
    pub rows_affected: u64,
    pub generated_keys: Vec<SqlValue>,
}
```

#### 使用示例

```rust
// 对标 Spring SimpleJdbcInsert 用法
let insert = SimpleJdbcInsert::new(pool.clone(), "users")
    .with_columns(&["name", "email"]);

let mut values = HashMap::new();
values.insert("name".to_string(), SqlValue::Text("Alice".to_string()));
values.insert("email".to_string(), SqlValue::Text("alice@example.com".to_string()));

let result = insert.execute(&values).await?;
println!("Generated ID: {:?}", result.generated_keys);
```

---

### 5.2 SimpleJdbcCall —— 存储过程调用

```rust
/// 简单 JDBC 调用（存储过程）。
/// 对标 Spring SimpleJdbcCall。
pub struct SimpleJdbcCall {
    function_name: String,
    catalog_name: Option<String>,
    schema_name: Option<String>,
    pool: Arc<dyn AnyPool>,
}

impl SimpleJdbcCall {
    pub fn new(pool: Arc<dyn AnyPool>, function_name: impl Into<String>) -> Self;

    pub fn with_catalog_name(mut self, catalog: impl Into<String>) -> Self;
    pub fn with_schema_name(mut self, schema: impl Into<String>) -> Self;

    /// 执行存储过程。
    pub async fn execute(
        &self,
        params: &HashMap<String, SqlValue>,
    ) -> Result<CallResult, DbError>;
}

/// 调用结果。
pub struct CallResult {
    pub out_params: HashMap<String, SqlValue>,
    pub result_sets: Vec<Vec<HashMap<String, SqlValue>>>,
}
```

---

### 5.3 Batch 操作

```rust
/// 批量更新工具。
/// 对标 spring-jdbc JdbcTemplate.batchUpdate()。
pub struct BatchUpdater {
    pool: Arc<dyn AnyPool>,
}

impl BatchUpdater {
    pub fn new(pool: Arc<dyn AnyPool>) -> Self;

    /// 批量执行相同 SQL（不同参数）。
    pub async fn batch_update(
        &self,
        sql: &str,
        batch: Vec<Vec<Box<dyn sqlx::Encode<'_, DB>>>>,
    ) -> Result<Vec<u64>, DbError>;

    /// 使用 QueryBuilder 批量插入。
    pub async fn batch_insert(
        &self,
        table: &str,
        columns: &[&str],
        rows: Vec<Vec<SqlValue>>,
    ) -> Result<u64, DbError>;
}
```

---

## 六、集成约束与路线图

### 6.1 sqlx 0.9 依赖约束

| 依赖 | 版本 | 说明 |
|:---|:---|:---|
| `sqlx` | `0.9` | 核心异步 SQL 库 |
| `sqlx-postgres` | `0.9` | PostgreSQL 驱动 |
| `sqlx-mysql` | `0.9` | MySQL 驱动 |
| `sqlx-sqlite` | `0.9` | SQLite 驱动 |
| `tokio` | `1.x` | async 运行时 |
| `thiserror` | `2.x` | 错误类型派生 |
| `chrono` | `0.4` | 日期时间类型 |

#### Cargo.toml 目标

```toml
[dependencies]
vernal-core = { path = "../vernal-core" }
vernal-tx = { path = "../vernal-tx" }
sqlx = { version = "0.9", features = ["runtime-tokio", "tls-rustls"] }
tokio = { version = "1", features = ["full"] }
thiserror = "2"
chrono = "0.4"
```

---

### 6.2 编译期 SQL 校验

```rust
// 方式 1：编译期校验（推荐，需要 DATABASE_URL 环境变量）
let user = sqlx::query_as!(User,
    "SELECT id, name, email FROM users WHERE id = $1",
    user_id
)
.fetch_one(&pool)
.await?;

// 方式 2：运行时查询（灵活，无编译期依赖）
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_one(&pool)
.await?;
```

#### 编译期校验约束

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| 环境变量 | `DATABASE_URL` | 编译时连接数据库校验 SQL |
| 数据库 | 必须可访问 | CI/CD 需要测试数据库 |
| 性能 | 编译变慢 | SQL 校验需要网络往返 |
| 备选 | `sqlx::query()` 运行时 | 无需编译期依赖 |

---

### 6.3 与 vernal-tx 集成

```rust
// 集成示例：vernal-jdbc 使用 vernal-tx 事务管理
use vernal_tx::{PlatformTransactionManager, TransactionDefinition, Propagation};

// 创建事务管理器
let manager = DataSourceTransactionManager::new(pool.clone(), "main");

// 编程式事务
let template = TransactionTemplate::new(Arc::new(manager));
let result = template
    .with_definition(TransactionDefinition {
        propagation: Propagation::Required,
        ..Default::default()
    })
    .execute(|_status| Box::pin(async {
        // 在事务中执行数据库操作
        let user = JdbcTemplate::new(pool.clone())
            .query_for_object("SELECT * FROM users WHERE id = $1", |row| {
                Ok(User::from_row(row))
            })
            .await?;
        Ok(user)
    }))
    .await?;
```

---

### 6.4 测试策略

| 测试类型 | 内容 | 工具 |
|:---|:---|:---|
| 单元测试 | RowMapper / DbError / 参数绑定 | `#[test]` |
| 集成测试 | 真实数据库 CRUD | `sqlx::test` + 测试容器 |
| 事务测试 | DataSourceTransactionManager | `vernal-test` + 内存 SQLite |
| 编译期测试 | `query!` 宏 SQL 校验 | `trybuild` |

#### sqlx::test 宏

```rust
/// sqlx 内建测试宏，自动创建/回滚事务。
#[sqlx::test]
async fn test_insert_user(pool: sqlx::PgPool) {
    let insert = SimpleJdbcInsert::new(Arc::new(pool), "users")
        .with_columns(&["name", "email"]);

    let mut values = HashMap::new();
    values.insert("name".into(), SqlValue::Text("Alice".into()));
    values.insert("email".into(), SqlValue::Text("alice@test.com".into()));

    let result = insert.execute(&values).await.unwrap();
    assert!(result.rows_affected > 0);
}
```

---

### 6.5 待补齐工作路线图

| 阶段 | 内容 | 优先级 | 预估工作量 |
|:---|:---|:---|:---|
| S1 | `DataSource` trait 扩展 + sqlx Pool 适配 | P0 | 1 天 |
| S2 | `JdbcTemplate` 核心（query / update / batch） | P0 | 2 天 |
| S3 | `RowMapper` trait + `FromRow` 集成 | P1 | 1 天 |
| S4 | `DataSourceTransactionManager` | P0 | 1.5 天 |
| S5 | `SimpleJdbcInsert` / `SimpleJdbcCall` | P1 | 1.5 天 |
| S6 | `NamedParameterJdbcTemplate` | P2 | 1 天 |
| S7 | PostgreSQL / MySQL / SQLite 三驱动适配 | P1 | 2 天 |

---

### 6.6 成熟度状态

| 维度 | 当前 | 目标 |
|:---|:---|:---|
| 文件数 | 2 | 15+ |
| 行数 | 19 | 1200+ |
| 核心 trait | 1（DataSource） | 4+（+ JdbcTemplate / RowMapper / Batch） |
| 事务管理器 | 0 | 1（DataSourceTransactionManager） |
| 便捷工具 | 0 | 3（SimpleJdbcInsert / Call / Batch） |
| 数据库支持 | 0 | 3（PostgreSQL / MySQL / SQLite） |
| 与 spring-jdbc 语义对标度 | ~5% | 80%+ |

---

## 附录：spring-jdbc 语义覆盖全景

| spring-jdbc 包 | 类数 | vernal-jdbc 状态 | 说明 |
|:---|:---|:---|:---|
| 核心（JdbcTemplate） | 20 | ⬜ 待实现 | sqlx 封装 |
| datasource（DataSource） | 12 | ✅ 已定义 trait | 骨架完成 |
| 支持（RowMapper / ResultSetExtractor） | 15 | ⬜ 待实现 | FromRow derive 替代 |
| 参数（SqlParameterSource） | 8 | ⬜ 待实现 | sqlx bind 替代 |
| 便捷（SimpleJdbcInsert/Call） | 6 | ⬜ 待实现 | 动态 SQL 生成 |
| 序列（DataFieldMaxValueIncrementer） | 4 | 🚫 不支持 | Rust 无序列概念 |

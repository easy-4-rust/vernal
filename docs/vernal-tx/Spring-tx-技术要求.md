# vernal-tx 技术要求（对标 spring-tx）

> **版本**：v2.0（2026-07-28）
> **定位**：vernal-tx crate 技术交接文档，对标 Spring Framework 7.0.8 spring-tx。
> **现状**：4 文件 / 128 行骨架，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》6.5 节（事务）。

---

## 一、总览

### 1.1 定位与边界

vernal-tx 是 Vernal Framework 的 **Tokio-first 异步事务抽象内核**，
对标 spring-tx 模块，提供统一的编程式 / 声明式事务管理能力。

| 维度 | spring-tx（语义参考） | vernal-tx（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（ThreadLocal + 反射） | Rust（task_local + trait） | 无反射，编译期静态分发 |
| 异步 | 同步 JDBC 事务 | Tokio-first async | 原生异步，无阻塞线程池 |
| 线程模型 | ThreadLocal 绑定 Connection | tokio::task::task_local 绑定 | 1:1 对应，语义等价 |
| 声明式 | `@Transactional` 注解 + AOP 代理 | `#[transactional]` 过程宏 + vernal-aop | 编译期织入 |
| JTA | 支持（javax.transaction） | **不支持**（🚫） | Rust 生态无 JTA 等价物 |
| 隔离级别 | 5 级（DEFAULT + 4 标准） | 5 级（同） | 完全对齐 |
| 传播行为 | 7 种 | 7 种（同） | 完全对齐 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  声明式层：#[transactional] 过程宏                    │
│  → 解析属性 → 生成 TransactionTemplate 调用           │
├─────────────────────────────────────────────────────┤
│  模板层：TransactionTemplate                          │
│  → execute(callback) → 自动 begin/commit/rollback    │
├─────────────────────────────────────────────────────┤
│  同步层：TransactionSynchronizationManager            │
│  → task_local 存储当前事务资源 + 同步回调              │
├─────────────────────────────────────────────────────┤
│  管理器层：PlatformTransactionManager trait            │
│  → get_transaction / commit / rollback               │
├─────────────────────────────────────────────────────┤
│  定义层：TransactionDefinition + Isolation + Propagation │
│  → 事务元数据（隔离级别/传播行为/超时/只读）           │
├─────────────────────────────────────────────────────┤
│  状态层：TransactionStatus                            │
│  → 事务运行时状态（rollback_only / completed / savepoint）│
└─────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 线程模型 | `tokio::task::task_local!` | Tokio 1:N 调度，task_local 等价 ThreadLocal |
| JTA | 🚫 不支持 | Rust 生态无 JTA/XA 标准，分布式事务用 Saga 模式替代 |
| 事务同步 | `TransactionSynchronization` trait | 与 Spring 对齐，支持 before_commit / after_commit 回调 |
| 声明式 | `#[transactional]` 过程宏 | 编译期织入，无运行时代理开销 |
| Savepoint | 支持（嵌套事务） | `Nested` 传播行为依赖 Savepoint |

### 1.4 当前骨架文件

| 文件 | 行数 | 内容 |
|:---|:---|:---|
| `lib.rs` | 10 | 模块声明 + re-export |
| `definition.rs` | 43 | `Propagation`(7) + `Isolation`(4) + `TransactionDefinition` |
| `manager.rs` | 34 | `PlatformTransactionManager` trait + `TransactionError` |
| `status.rs` | 41 | `TransactionStatus`（read_only / completed / rollback_only） |

---

## 二、核心 Trait 体系

### 2.1 PlatformTransactionManager —— 事务管理器契约

**现状**：已定义 trait 骨架（3 个方法）。
**语义参照**：spring-tx `PlatformTransactionManager`。

#### Spring API（Java）

```java
// spring-tx 核心事务管理器接口
public interface PlatformTransactionManager {
    TransactionStatus getTransaction(TransactionDefinition definition)
        throws TransactionException;
    void commit(TransactionStatus status) throws TransactionException;
    void rollback(TransactionStatus status) throws TransactionException;
}
```

#### Rust trait（已有）

```rust
/// 平台事务管理器 trait。
/// 对标 Spring 的 PlatformTransactionManager。
pub trait PlatformTransactionManager: Send + Sync {
    fn get_transaction(
        &self,
        definition: &TransactionDefinition,
    ) -> Result<TransactionStatus, TransactionError>;
    fn commit(&self, status: TransactionStatus) -> Result<(), TransactionError>;
    fn rollback(&self, status: TransactionStatus) -> Result<(), TransactionError>;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享，可存入 Arc |
| `get_transaction` | 语义传播 | 根据 Propagation 决定新建/加入/挂起 |
| `commit` | 消费 `TransactionStatus` | 按值移动，防止重复提交 |
| `rollback` | 消费 `TransactionStatus` | 按值移动，防止重复回滚 |

#### 待补齐

- [x] `PlatformTransactionManager` trait 定义
- [ ] `DataSourceTransactionManager`（vernal-jdbc 实现）
- [ ] `JpaTransactionManager`（vernal-orm 实现）

---

### 2.2 TransactionDefinition —— 事务元数据

**现状**：已定义完整结构体。
**语义参照**：spring-tx `TransactionDefinition`。

#### Spring API（Java）

```java
public interface TransactionDefinition {
    int getPropagationBehavior();
    int getIsolationLevel();
    int getTimeout();
    boolean isReadOnly();
}
```

#### Rust 结构体（已有）

```rust
/// 事务定义。
#[derive(Debug, Clone)]
pub struct TransactionDefinition {
    pub propagation: Propagation,
    pub isolation: Isolation,
    pub read_only: bool,
    pub timeout_secs: u64,
}

impl Default for TransactionDefinition {
    fn default() -> Self {
        Self {
            propagation: Propagation::Required,
            isolation: Isolation::Default,
            read_only: false,
            timeout_secs: 0,  // 0 = 不超时
        }
    }
}
```

#### 构建器模式（待实现）

```rust
/// 事务定义构建器。
pub struct TransactionDefinitionBuilder {
    inner: TransactionDefinition,
}

impl TransactionDefinitionBuilder {
    pub fn new() -> Self;
    pub fn propagation(mut self, p: Propagation) -> Self;
    pub fn isolation(mut self, i: Isolation) -> Self;
    pub fn read_only(mut self, ro: bool) -> Self;
    pub fn timeout(mut self, secs: u64) -> Self;
    pub fn build(self) -> TransactionDefinition;
}
```

---

### 2.3 TransactionStatus —— 事务运行时状态

**现状**：已定义基本结构体（3 字段）。
**语义参照**：spring-tx `TransactionStatus`。

#### Spring API（Java）

```java
public interface TransactionStatus extends SavepointManager {
    boolean isNewTransaction();
    boolean hasSavepoint();
    void setRollbackOnly();
    boolean isRollbackOnly();
    boolean isCompleted();
}
```

#### Rust 结构体（已有 + 待扩展）

```rust
/// 事务状态。
#[derive(Debug)]
pub struct TransactionStatus {
    pub read_only: bool,
    pub completed: bool,
    pub rollback_only: bool,
    // --- 待扩展字段 ---
    // pub savepoint: Option<Savepoint>,       // 嵌套事务 Savepoint
    // pub new_transaction: bool,               // 是否新建事务
    // pub suspended_resources: Option<SuspendedResources>, // 挂起的资源
}
```

#### 待补齐

- [x] `TransactionStatus` 基本字段
- [ ] `Savepoint` 支持（嵌套事务）
- [ ] `SuspendedResources`（传播行为挂起/恢复）
- [ ] `new_transaction` 标志

---

## 三、隔离级别与传播行为

### 3.1 Isolation —— 4 种隔离级别

**现状**：已定义 enum（5 变体，含 Default）。
**语义参照**：spring-tx `Isolation`。

#### Spring API（Java）

```java
public abstract class Isolation {
    public static final int DEFAULT = -1;
    public static final int READ_UNCOMMITTED = Connection.TRANSACTION_READ_UNCOMMITTED;
    public static final int READ_COMMITTED = Connection.TRANSACTION_READ_COMMITTED;
    public static final int REPEATABLE_READ = Connection.TRANSACTION_REPEATABLE_READ;
    public static final int SERIALIZABLE = Connection.TRANSACTION_SERIALIZABLE;
}
```

#### Rust enum（已有）

```rust
/// 事务隔离级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    Default,           // 使用数据库默认隔离级别
    ReadUncommitted,   // 脏读
    ReadCommitted,     // 不可重复读
    RepeatableRead,    // 幻读（MySQL 默认）
    Serializable,      // 最高隔离，性能最低
}
```

#### 数据库映射

| Isolation 变体 | PostgreSQL | MySQL | SQLite |
|:---|:---|:---|:---|
| `Default` | READ COMMITTED | REPEATABLE READ | SERIALIZABLE |
| `ReadUncommitted` | READ UNCOMMITTED | READ UNCOMMITTED | （忽略，始终 SERIALIZABLE） |
| `ReadCommitted` | READ COMMITTED | READ COMMITTED | （忽略） |
| `RepeatableRead` | REPEATABLE READ | REPEATABLE READ | （忽略） |
| `Serializable` | SERIALIZABLE | SERIALIZABLE | SERIALIZABLE |

#### 待补齐

- [x] `Isolation` enum 定义
- [ ] `to_sql_string()` 方法（生成 SET TRANSACTION ISOLATION LEVEL SQL）
- [ ] 各数据库驱动适配

---

### 3.2 Propagation —— 7 种传播行为

**现状**：已定义 enum（7 变体）。
**语义参照**：spring-tx `Propagation`。

#### Spring API（Java）

```java
public enum Propagation {
    REQUIRED(TransactionDefinition.PROPAGATION_REQUIRED),
    SUPPORTS(TransactionDefinition.PROPAGATION_SUPPORTS),
    MANDATORY(TransactionDefinition.PROPAGATION_MANDATORY),
    REQUIRES_NEW(TransactionDefinition.PROPAGATION_REQUIRES_NEW),
    NOT_SUPPORTED(TransactionDefinition.PROPAGATION_NOT_SUPPORTED),
    NEVER(TransactionDefinition.PROPAGATION_NEVER),
    NESTED(TransactionDefinition.PROPAGATION_NESTED);
}
```

#### Rust enum（已有）

```rust
/// 事务传播行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    Required,       // 有则加入，无则新建（默认）
    RequiresNew,    // 总是新建，挂起当前
    Mandatory,      // 有则加入，无则抛错
    Nested,         // 嵌套事务（Savepoint）
    Supports,       // 有则加入，无则非事务
    NotSupported,   // 总是非事务，挂起当前
    Never,          // 总是非事务，有则抛错
}
```

#### 传播行为语义表

| Propagation | 当前有事务 | 当前无事务 | 对应 Spring 行为 |
|:---|:---|:---|:---|
| `Required` | 加入 | 新建 | `PROPAGATION_REQUIRED` |
| `RequiresNew` | 挂起 → 新建 | 新建 | `PROPAGATION_REQUIRES_NEW` |
| `Mandatory` | 加入 | 抛 `TransactionRequiredException` | `PROPAGATION_MANDATORY` |
| `Nested` | 创建 Savepoint | 新建 | `PROPAGATION_NESTED` |
| `Supports` | 加入 | 非事务运行 | `PROPAGATION_SUPPORTS` |
| `NotSupported` | 挂起 → 非事务 | 非事务运行 | `PROPAGATION_NOT_SUPPORTED` |
| `Never` | 抛异常 | 非事务运行 | `PROPAGATION_NEVER` |

#### 传播行为实现状态

| Propagation | 实现复杂度 | 状态 | 说明 |
|:---|:---|:---|:---|
| `Required` | 低 | ⬜ 待实现 | 加入或新建，最常用 |
| `RequiresNew` | 高 | ⬜ 待实现 | 需要挂起/恢复资源 |
| `Mandatory` | 低 | ⬜ 待实现 | 仅检查是否存在 |
| `Nested` | 中 | ⬜ 待实现 | 依赖 Savepoint |
| `Supports` | 低 | ⬜ 待实现 | 条件分支 |
| `NotSupported` | 高 | ⬜ 待实现 | 需要挂起资源 |
| `Never` | 低 | ⬜ 待实现 | 仅检查是否不存在 |

---

## 四、TransactionTemplate —— 编程式事务门面

### 4.1 TransactionTemplate

**现状**：⬜ 待实现。
**语义参照**：spring-tx `TransactionTemplate`。

#### Spring API（Java）

```java
// spring-tx 编程式事务模板
public class TransactionTemplate extends DefaultTransactionDefinition
        implements TransactionOperations {
    public <T> T execute(TransactionCallback<T> action) throws TransactionException {
        TransactionStatus status = transactionManager.getTransaction(this);
        T result;
        try {
            result = action.doInTransaction(status);
        } catch (RuntimeException ex) {
            transactionManager.rollback(status);
            throw ex;
        }
        transactionManager.commit(status);
        return result;
    }
}
```

#### Rust 目标设计

```rust
/// 编程式事务模板。
/// 对标 Spring TransactionTemplate，提供 execute 闭包模式。
pub struct TransactionTemplate<M: PlatformTransactionManager> {
    manager: Arc<M>,
    definition: TransactionDefinition,
}

impl<M: PlatformTransactionManager> TransactionTemplate<M> {
    pub fn new(manager: Arc<M>) -> Self;
    pub fn with_definition(manager: Arc<M>, definition: TransactionDefinition) -> Self;

    /// 执行事务回调。
    /// 自动 begin → callback → commit/rollback。
    pub async fn execute<F, T, E>(&self, f: F) -> Result<T, TransactionError>
    where
        F: FnOnce(TransactionStatus) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: Into<TransactionError>;
}

/// 事务操作 trait（便于 mock 测试）。
pub trait TransactionOperations: Send + Sync {
    fn execute<F, T, E>(&self, f: F) -> Pin<Box<dyn Future<Output = Result<T, TransactionError>> + Send>>
    where
        F: FnOnce(TransactionStatus) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>> + Send,
        E: Into<TransactionError>;
}
```

#### 与 Spring 的关键差异

| 维度 | Spring TransactionTemplate | Rust TransactionTemplate |
|:---|:---|:---|
| 回调签名 | `TransactionCallback<T>.doInTransaction(status)` | `FnOnce(TransactionStatus) -> Future<Result<T, E>>` |
| 异步 | 同步阻塞 | async + Pin<Box<dyn Future>> |
| 错误处理 | RuntimeException → rollback | `Result::Err` → rollback |
| 类型擦除 | 泛型 + 接口 | 泛型 + trait object |

---

### 4.2 TransactionCallback

**语义参照**：spring-tx `TransactionCallback<T>`。

```rust
/// 事务回调 trait。
/// 对标 Spring TransactionCallback<T>。
pub trait TransactionCallback<T>: Send {
    fn do_in_transaction(
        self,
        status: &mut TransactionStatus,
    ) -> Pin<Box<dyn Future<Output = Result<T, TransactionError>> + Send>>;
}

/// 为闭包自动实现 TransactionCallback。
impl<T, F> TransactionCallback<T> for F
where
    F: FnOnce(&mut TransactionStatus) -> Pin<Box<dyn Future<Output = Result<T, TransactionError>> + Send>>
        + Send,
{
    fn do_in_transaction(
        self,
        status: &mut TransactionStatus,
    ) -> Pin<Box<dyn Future<Output = Result<T, TransactionError>> + Send>> {
        (self)(status)
    }
}
```

---

## 五、TransactionSynchronizationManager —— 任务级资源绑定

### 5.1 task_local 资源管理

**现状**：⬜ 待实现。
**语义参照**：spring-tx `TransactionSynchronizationManager`（ThreadLocal 版）。

#### Spring API（Java）

```java
// spring-tx ThreadLocal 资源管理器
public abstract class TransactionSynchronizationManager {
    private static final ThreadLocal<Map<Object, Object>> resources = new ThreadLocal<>();
    private static final ThreadLocal<Set<TransactionSynchronization>> synchronizations = new ThreadLocal<>();
    private static final ThreadLocal<String> currentTransactionName = new ThreadLocal<>();
    private static final ThreadLocal<Boolean> currentTransactionReadOnly = new ThreadLocal<>();
    private static final ThreadLocal<Integer> currentTransactionIsolationLevel = new ThreadLocal<>();

    public static Object getResource(Object key);
    public static void bindResource(Object key, Object value);
    public static Object unbindResource(Object key);
    public static boolean hasResource(Object key);
}
```

#### Rust 目标设计

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 事务资源键（通常是 DataSource 或 ConnectionPool 的标识）。
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceKey(String);

/// 任务级事务资源管理器。
/// 使用 tokio::task::task_local! 替代 ThreadLocal。
///
/// 注意：task_local 要求值实现 Send，且在 task 生命周期内有效。
pub struct TransactionSynchronizationManager;

// task_local 存储
tokio::task::task_local! {
    static RESOURCES: Mutex<HashMap<ResourceKey, Arc<dyn Any + Send + Sync>>>;
    static SYNCHRONIZATIONS: Mutex<Vec<Box<dyn TransactionSynchronization>>>;
    static CURRENT_TRANSACTION_NAME: Mutex<Option<String>>;
    static CURRENT_TRANSACTION_READ_ONLY: Mutex<bool>;
    static CURRENT_TRANSACTION_ISOLATION: Mutex<Option<Isolation>>;
}

impl TransactionSynchronizationManager {
    /// 获取当前事务绑定的资源。
    pub async fn get_resource(key: &ResourceKey) -> Option<Arc<dyn Any + Send + Sync>>;

    /// 绑定资源到当前事务。
    pub async fn bind_resource(key: ResourceKey, value: Arc<dyn Any + Send + Sync>);

    /// 解绑当前事务的资源。
    pub async fn unbind_resource(key: &ResourceKey) -> Option<Arc<dyn Any + Send + Sync>>;

    /// 当前任务是否绑定指定资源。
    pub async fn has_resource(key: &ResourceKey) -> bool;

    /// 注册事务同步回调。
    pub async fn register_synchronization(sync: Box<dyn TransactionSynchronization>);

    /// 触发所有同步回调（before_commit / after_commit 等）。
    pub async fn trigger_synchronization(phase: SyncPhase);

    /// 清理当前任务的所有事务资源（事务完成后调用）。
    pub async fn clear();
}
```

---

### 5.2 TransactionSynchronization —— 同步回调

**语义参照**：spring-tx `TransactionSynchronization`。

#### Spring API（Java）

```java
public interface TransactionSynchronization extends Flushable {
    int STATUS_COMMITTED = 0;
    int STATUS_ROLLED_BACK = 1;
    int STATUS_UNKNOWN = 2;

    default void suspend() {}
    default void resume() {}
    default void flush() {}
    default void beforeCommit(boolean readOnly) {}
    default void beforeCompletion() {}
    default void afterCommit() {}
    default void afterCompletion(int status) {}
}
```

#### Rust trait

```rust
/// 同步回调阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPhase {
    BeforeCommit,
    BeforeCompletion,
    AfterCommit,
    AfterCompletion(CompletionStatus),
    Suspend,
    Resume,
    Flush,
}

/// 完成状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionStatus {
    Committed,
    RolledBack,
    Unknown,
}

/// 事务同步回调 trait。
/// 对标 Spring TransactionSynchronization。
pub trait TransactionSynchronization: Send + Sync + 'static {
    fn before_commit(&mut self, _read_only: bool) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
    fn before_completion(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
    fn after_commit(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
    fn after_completion(&mut self, _status: CompletionStatus) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
    fn suspend(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
    fn resume(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
    fn flush(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
}
```

---

### 5.3 SuspendedResources —— 资源挂起/恢复

```rust
/// 挂起的事务资源。
/// 在 RequiresNew / NotSupported 传播行为时使用。
pub struct SuspendedResources {
    key: ResourceKey,
    resource: Arc<dyn Any + Send + Sync>,
    synchronizations: Vec<Box<dyn TransactionSynchronization>>,
}

impl SuspendedResources {
    /// 挂起当前事务资源。
    pub async fn suspend(key: &ResourceKey) -> Option<Self>;
    /// 恢复挂起的资源。
    pub async fn resume(self);
}
```

---

## 六、集成约束与路线图

### 6.1 JTA 不支持决策

| 维度 | 说明 |
|:---|:---|
| 决策 | 🚫 **不支持 JTA（Java Transaction API）** |
| 原因 | Rust 生态无 JTA/XA 标准等价物 |
| 替代方案 | 分布式事务使用 Saga 模式（编排 / 协同） |
| 影响范围 | `UserTransaction` / `TransactionManager`（javax）不移植 |

#### JTA 概念替代方案

| JTA 概念 | Rust 替代 | 说明 |
|:---|:---|:---|
| `UserTransaction.begin()` | `TransactionTemplate.execute()` | 编程式事务 |
| `@Transactional` | `#[transactional]` 过程宏 | 声明式事务 |
| XA DataSource | 单库事务 + Saga 跨库 | 无 XA 支持 |
| `TransactionSynchronization` | `TransactionSynchronization` trait | 直接对标 |

---

### 6.2 声明式事务宏（待实现）

```rust
/// 声明式事务宏。
/// 对标 Spring @Transactional 注解。
#[transactional(
    propagation = Propagation::Required,
    isolation = Isolation::Default,
    read_only = false,
    timeout = 30,
    rollback_for = [AppError],
    no_rollback_for = [NotFoundError],
)]
async fn save_user(user: &User) -> Result<User, AppError> {
    // 业务逻辑
}
```

#### 宏展开伪代码

```rust
// 展开后的等价代码
async fn save_user(user: &User) -> Result<User, AppError> {
    let template = TransactionTemplate::new(current_manager());
    let def = TransactionDefinition {
        propagation: Propagation::Required,
        isolation: Isolation::Default,
        read_only: false,
        timeout_secs: 30,
    };
    template
        .with_definition(def)
        .execute(|_status| Box::pin(async { /* 业务逻辑 */ }))
        .await
        .map_err(|e| AppError::from(e))
}
```

---

### 6.3 Async 适配约束

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| 运行时 | Tokio | `task_local!` 依赖 Tokio 运行时 |
| Future 类型 | `Pin<Box<dyn Future + Send>>` | 避免 `async-trait` 依赖 |
| 所有权 | `TransactionStatus` 按值移动 | 防止重复 commit/rollback |
| 连接获取 | async `acquire()` | 连接池异步获取 |

---

### 6.4 测试策略

| 测试类型 | 内容 | 工具 |
|:---|:---|:---|
| 单元测试 | Propagation / Isolation 语义 | `#[test]` |
| 集成测试 | 事务 begin/commit/rollback 流程 | `vernal-test` + 内存数据库 |
| 传播行为测试 | 7 种传播行为组合 | mock `PlatformTransactionManager` |
| 宏测试 | `#[transactional]` 展开 | `trybuild` |

---

### 6.5 待补齐工作路线图

| 阶段 | 内容 | 优先级 | 预估工作量 |
|:---|:---|:---|:---|
| S1 | `TransactionTemplate` + `TransactionCallback` | P0 | 1 天 |
| S2 | `TransactionSynchronizationManager`（task_local） | P0 | 1.5 天 |
| S3 | `TransactionSynchronization` 回调 | P1 | 1 天 |
| S4 | `DataSourceTransactionManager`（vernal-jdbc） | P0 | 1 天 |
| S5 | `#[transactional]` 过程宏 | P1 | 1.5 天 |
| S6 | 7 种传播行为完整实现 | P1 | 2 天 |
| S7 | Savepoint（嵌套事务） | P2 | 1 天 |

---

### 6.6 成熟度状态

| 维度 | 当前 | 目标 |
|:---|:---|:---|
| 文件数 | 4 | 12+ |
| 行数 | 128 | 800+ |
| 核心 trait | 3（Manager / Definition / Status） | 5+（+ Template / Synchronization） |
| 隔离级别 | 4（已定义） | 4 |
| 传播行为 | 7（已定义） | 7（全部实现） |
| 事务模板 | 0 | 1（TransactionTemplate） |
| 声明式宏 | 0 | 1（#[transactional]） |
| 与 spring-tx 语义对标度 | ~30% | 85%+ |

---

## 附录：spring-tx 语义覆盖全景

| spring-tx 包 | 类数 | vernal-tx 状态 | 说明 |
|:---|:---|:---|:---|
| 根包（PlatformTransactionManager） | 15 | ✅ 已定义 trait | 骨架完成 |
| support（TransactionSynchronizationManager） | 12 | ⬜ 待实现 | task_local 版 |
| transaction（TransactionDefinition/Status） | 8 | ✅ 已定义 | 骨架完成 |
| interceptor（TransactionInterceptor） | 5 | ⬜ 待实现 | 过程宏替代 |
| jta（JtaTransactionManager） | 8 | 🚫 不支持 | Rust 无 JTA |
| document（JtaTransactionManager XML） | 3 | 🚫 不支持 | Rust 无 XML |

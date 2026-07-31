<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-tx 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> 当前权威要求。基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`，遵循[迁移验收规范](../迁移验收规范.md)。

## 当前事实

[自动对象审计](../migration-audit/vernal-tx.md)识别 121 个 Spring 业务对象；严格完成数为 0，121 个均为 `MISSING`。现有 `definition.rs`、`manager.rs`、`status.rs` 是聚合抽象，不能抵扣一对象一文件要求。

## 目标结构

| Spring 路径 | 目标 Rust 路径 |
|---|---|
| `interceptor/TransactionInterceptor.java` | `interceptor/transaction_interceptor.rs` |
| `annotation/Transactional.java` | `annotation/transactional.rs` |
| `support/AbstractPlatformTransactionManager.java` | `support/abstract_platform_transaction_manager.rs` |
| `reactive/TransactionSynchronizationManager.java` | `reactive/transaction_synchronization_manager.rs` |
| `jta/JtaTransactionManager.java` | `jta/jta_transaction_manager.rs` |

包层级超过两层时只保留末两层；根包对象留在 crate 根目录。`mod.rs`、`lib.rs` 不定义类型。

## 语义边界

- 同步事务：传播、隔离、超时、只读、保存点、挂起/恢复、同步回调、提交/回滚。
- 声明式事务：属性解析、管理器选择、目标调用、rollback rule 与异常传播。
- 异步事务：使用 future/stream 与任务局部上下文，不以 Reactor 类型为由整体豁免。
- JTA/JCA/JVM 专属能力只能逐对象证明 `PLATFORM_NA`。
- sqlx、Diesel、SeaORM、RBatis 只是后端候选；只有精确符号和集成测试才能标 `DEPENDENCY_REUSED`。

## 门禁

对象路径、中文来源注释、公开方法文档和语义测试同时满足才可标 `IMPLEMENTED`。stub、聚合对象、形态近似、仅编译通过均属于未完成。

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-tx 技术要求（对标 spring-tx）

> **版本**：v3.0（2026-07-28）
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
| 线程模型 | ThreadLocal 绑定 Connection | `tokio::task::task_local!` 绑定 | 1:1 对应，语义等价 |
| 声明式 | `@Transactional` 注解 + AOP 代理 | `#[transactional]` 过程宏 + vernal-aop | 编译期织入 |
| JTA | 支持（javax.transaction） | **不支持**（🚫） | Rust 生态无 JTA 等价物 |
| 隔离级别 | 5 级（DEFAULT + 4 标准） | 5 级（同） | 完全对齐 |
| 传播行为 | 7 种 | 7 种（同） | 完全对齐 |
| Savepoint | 支持（嵌套事务） | 支持（待补齐） | `Nested` 传播行为依赖 |
| 事务同步 | `TransactionSynchronizationManager` | 同名 trait + task_local（待补齐） | before/after_commit 回调 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  声明式层：#[transactional] 过程宏（vernal-aspects） │
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
| JTA | 🚫 不支持 | Rust 生态无 JTA/XA 标准，分布式事务用 Saga 模式替代（vernal-context 提供） |
| 事务同步 | `TransactionSynchronization` trait | 与 Spring 对齐，支持 before_commit / after_commit / after_completion 回调 |
| 声明式 | `#[transactional]` 过程宏（vernal-aspects） | 编译期织入，无运行时代理开销 |
| Savepoint | 支持（嵌套事务） | `Nested` 传播行为依赖 Savepoint |
| 错误模型 | `TransactionError`（thiserror） | 后续可拆分为 `TransactionException` / `UnexpectedRollbackException` 等 |
| 资源绑定 | `TransactionResource` trait | 让 sqlx::Transaction / rbatis::Tx 都能绑定到当前 task_local |
| Rollback 规则 | `RollbackRule` 默认 RuntimeException → rollback | 与 Spring 一致；checked error 不触发 |

### 1.4 当前骨架文件

| 文件 | 行数 | 内容 |
|:---|:---|:---|
| `lib.rs` | 10 | 模块声明 + re-export（Isolation / Propagation / TransactionDefinition / PlatformTransactionManager / TransactionStatus） |
| `definition.rs` | 43 | `Propagation`(7) + `Isolation`(4) + `TransactionDefinition` |
| `manager.rs` | 34 | `PlatformTransactionManager` trait（3 方法）+ `TransactionError` |
| `status.rs` | 41 | `TransactionStatus`（read_only / completed / rollback_only + 3 方法） |

### 1.5 与其他 crate 的边界

| crate | 关系 | 说明 |
|:---|:---|:---|
| `vernal-core` | 依赖 | 基础 `BoxError`（当前未实际引用，预留） |
| `vernal-aspects` | 上层增强 | `#[transactional]` 过程宏织入 |
| `vernal-jdbc` | 下游实现 | `DataSourceTransactionManager` 基于 sqlx |
| `vernal-orm` | 下游实现 | `JpaTransactionManager` 基于 rbatis |
| `vernal-context` | 同级独立 | 提供 `@Transactional` AOP 装配 |
| `vernal-r2dbc` | 后续接入 | 响应式事务管理器（待评估） |

---

## 二、核心 Trait 体系

### 2.1 PlatformTransactionManager —— 事务管理器契约

**现状**：已定义 trait 骨架（3 个方法）。
**语义参照**：spring-tx `PlatformTransactionManager`。

#### Spring API（Java）

```java
public interface PlatformTransactionManager {
    TransactionStatus getTransaction(TransactionDefinition definition)
        throws TransactionException;
    void commit(TransactionStatus status) throws TransactionException;
    void rollback(TransactionStatus status) throws TransactionException;
}
```

#### Rust trait（已有）

```rust
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
| `commit` | **消费 `TransactionStatus`** | 按值移动，防止重复提交 |
| `rollback` | **消费 `TransactionStatus`** | 按值移动，防止重复回滚 |

#### 待补齐

- [x] `PlatformTransactionManager` trait 定义
- [ ] `AbstractPlatformTransactionManager`（提供事务边界检查逻辑）
- [ ] `DataSourceTransactionManager`（vernal-jdbc 实现）
- [ ] `JpaTransactionManager`（vernal-orm 实现）
- [ ] `ResourcelessTransactionManager`（测试用）

---

### 2.2 TransactionDefinition —— 事务元数据

**现状**：已定义完整结构体（4 字段 + Default）。
**语义参照**：spring-tx `TransactionDefinition`。

#### Spring API（Java）

```java
public interface TransactionDefinition {
    int getPropagationBehavior();
    int getIsolationLevel();
    int getTimeout();
    boolean isReadOnly();
    String getName();  // 4.2+
}
```

#### Rust 结构体（已有）

```rust
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

#### 待补齐

- [x] `TransactionDefinition` 结构体
- [x] `Default` 实现（`Required` + `Default` 隔离 + 写 + 不超时）
- [ ] `TransactionDefinitionBuilder`（链式 API）
- [ ] `name: Option<String>` 字段（事务名称，用于日志 / 监控）

---

### 2.3 Isolation —— 隔离级别

**现状**：已定义枚举（5 变体）。
**语义参照**：spring-tx `TransactionDefinition.ISOLATION_*`。

#### 完整对照

| Spring 常量 | vernal-tx 变体 | 说明 |
|:---|:---|:---|
| `ISOLATION_DEFAULT` | `Isolation::Default` | 使用数据库默认 |
| `ISOLATION_READ_UNCOMMITTED` | `Isolation::ReadUncommitted` | 读未提交 |
| `ISOLATION_READ_COMMITTED` | `Isolation::ReadCommitted` | 读已提交（多数 DB 默认） |
| `ISOLATION_REPEATABLE_READ` | `Isolation::RepeatableRead` | 可重复读（MySQL InnoDB 默认） |
| `ISOLATION_SERIALIZABLE` | `Isolation::Serializable` | 串行化 |

#### Spring API vs Rust 命名风格

| 项 | Spring | vernal-tx |
|:---|:---|:---|
| 命名 | `int` 常量（1-8） | enum 变体 |
| 类型安全 | ❌ 任意 int | ✅ 编译期 |
| 默认值 | `ISOLATION_DEFAULT = -1` | `Isolation::Default` |

#### 当前实现

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Isolation {
    Default,
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}
```

✅ 完全对齐 Spring。

---

### 2.4 Propagation —— 传播行为

**现状**：已定义枚举（7 变体）。
**语义参照**：spring-tx `TransactionDefinition.PROPAGATION_*`。

#### 完整对照

| Spring 常量 | vernal-tx 变体 | 行为 |
|:---|:---|:---|
| `PROPAGATION_REQUIRED` | `Propagation::Required` | 有则加入，无则新建（**默认**） |
| `PROPAGATION_SUPPORTS` | `Propagation::Supports` | 有则加入，无则非事务执行 |
| `PROPAGATION_MANDATORY` | `Propagation::Mandatory` | 有则加入，无则抛异常 |
| `PROPAGATION_REQUIRES_NEW` | `Propagation::RequiresNew` | 始终新建，挂起当前事务 |
| `PROPAGATION_NOT_SUPPORTED` | `Propagation::NotSupported` | 非事务执行，挂起当前事务 |
| `PROPAGATION_NEVER` | `Propagation::Never` | 非事务执行，当前有事务则抛异常 |
| `PROPAGATION_NESTED` | `Propagation::Nested` | 有则嵌套（Savepoint），无则新建 |

#### 当前实现

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Propagation {
    Required,
    RequiresNew,
    Mandatory,
    Nested,
    Supports,
    NotSupported,
    Never,
}
```

✅ 完全对齐 Spring（7 种全覆盖）。

#### 行为矩阵（待补齐文档）

| 当前已有事务 \ Propagation | Required | Supports | Mandatory | RequiresNew | NotSupported | Never | Nested |
|:---|:---|:---|:---|:---|:---|:---|:---|
| 无 | 新建 | 非事务 | 抛异常 | 新建 | 非事务 | 非事务 | 新建 |
| 有 | 加入 | 加入 | 加入 | 挂起+新建 | 挂起 | 抛异常 | 嵌套（Savepoint） |

> 实现位于 `AbstractPlatformTransactionManager::handleExistingTransaction`，
> 需要在 `get_transaction` 中根据当前 task_local 是否已绑定事务来分支。

---

### 2.5 TransactionStatus —— 事务运行时状态

**现状**：已定义结构体（3 字段 + 3 方法）。
**语义参照**：spring-tx `TransactionStatus` + `SavepointManager`。

#### Spring API（Java）

```java
public interface TransactionStatus extends SavepointManager {
    boolean isNewTransaction();
    boolean hasSavepoint();
    void setRollbackOnly();
    boolean isRollbackOnly();
    boolean isCompleted();
}

public interface SavepointManager {
    Object createSavepoint() throws TransactionException;
    void rollbackToSavepoint(Object savepoint) throws TransactionException;
    void releaseSavepoint(Object savepoint) throws TransactionException;
}
```

#### Rust 结构体（已有 + 待扩展）

```rust
#[derive(Debug)]
pub struct TransactionStatus {
    pub read_only: bool,
    pub completed: bool,
    pub rollback_only: bool,
}
```

#### 待补齐字段

| 字段 | 类型 | 用途 |
|:---|:---|:---|
| `is_new_transaction` | `bool` | 区分 `Required` 加入 vs 新建 |
| `savepoint` | `Option<Savepoint>` | 支持 `Nested` 传播 |
| `suspended_resources` | `Option<Box<dyn Any>>` | 挂起的事务资源（RequiresNew / NotSupported） |
| `transaction_name` | `Option<String>` | 日志 / 监控关联 |

#### 当前方法

```rust
impl TransactionStatus {
    pub fn new(read_only: bool) -> Self;
    pub fn set_rollback_only(&mut self);
    pub fn is_rollback_only(&self) -> bool;
    pub fn is_completed(&self) -> bool;
}
```

✅ 基础方法完整。

---

### 2.6 TransactionError —— 异常模型

**现状**：已定义简单结构体（1 字段，`message: String`）。

**待重构**：对齐 Spring 异常层级，改为 enum + thiserror derive，包含 `UnexpectedRollback` / `HeuristicCompletion` / `IllegalState` / `InvalidIsolation` / `InvalidPropagation` / `NoTransaction` / `SuspensionUnsupported` / `Other` 共 8 个变体。

---

## 三、待补齐：抽象基类与模板

### 3.1 AbstractPlatformTransactionManager

**目标**：对标 Spring `AbstractPlatformTransactionManager`，提供事务边界处理骨架。
Spring 在抽象基类中实现：
- `get_transaction`：根据 Propagation 决定新建/加入/挂起；
- `commit`：rollback_only 检查 → before_commit → doCommit → after_commit；
- `rollback`：before_completion → doRollback → after_completion。

#### Rust 设计（trait 继承）

```rust
#[async_trait]
pub trait AbstractPlatformTransactionManager: PlatformTransactionManager {
    /// 模板方法：包含传播行为处理
    async fn get_transaction_template(
        &self,
        definition: &TransactionDefinition,
    ) -> Result<TransactionStatus, TransactionError> {
        // 1. 检查当前 task_local 是否有事务
        if let Some(existing) = TransactionSynchronizationManager::current_transaction() {
            return self.handle_existing_transaction(definition, existing).await;
        }
        self.handle_propagation(definition).await
    }

    async fn commit_template(&self, mut status: TransactionStatus)
        -> Result<(), TransactionError> {
        if status.is_rollback_only() {
            // 触发 rollback on commit exception
            self.rollback(status).await?;
            return Err(TransactionError::UnexpectedRollback(...));
        }
        // 触发 before_commit
        TransactionSynchronizationManager::trigger_before_commit().await?;
        self.commit(status).await?;
        TransactionSynchronizationManager::trigger_after_commit().await?;
        Ok(())
    }

    async fn rollback_template(&self, mut status: TransactionStatus)
        -> Result<(), TransactionError> {
        if status.is_completed() { return Ok(()); }
        TransactionSynchronizationManager::trigger_before_completion(false).await?;
        self.rollback(status).await?;
        TransactionSynchronizationManager::trigger_after_completion(false).await?;
        Ok(())
    }

    /// 子类实现
    async fn do_begin(&self, definition: &TransactionDefinition)
        -> Result<TransactionStatus, TransactionError>;
    async fn do_commit(&self, status: TransactionStatus)
        -> Result<(), TransactionError>;
    async fn do_rollback(&self, status: TransactionStatus)
        -> Result<(), TransactionError>;
}
```

---

### 3.2 TransactionTemplate —— 编程式事务入口

**目标**：对标 Spring `TransactionTemplate`。

```rust
pub struct TransactionTemplate {
    manager: Arc<dyn PlatformTransactionManager>,
    definition: TransactionDefinition,
}

impl TransactionTemplate {
    pub fn new(manager: Arc<dyn PlatformTransactionManager>) -> Self;
    pub fn with_definition(mut self, def: TransactionDefinition) -> Self;

    /// 同步执行回调
    pub fn execute<F, T>(&self, callback: F) -> Result<T, TransactionError>
    where
        F: FnOnce(TransactionStatus) -> Result<T, TransactionError>;

    /// 异步执行回调（Tokio-first）
    pub async fn execute_async<F, Fut, T>(&self, callback: F)
        -> Result<T, TransactionError>
    where
        F: FnOnce(TransactionStatus) -> Fut,
        Fut: Future<Output = Result<T, TransactionError>>;
}
```

#### 使用示例

```rust
let tmpl = TransactionTemplate::new(tx_manager);
let result = tmpl.execute_async(|status| async move {
    user_repo.save(&user).await?;
    order_repo.save(&order).await?;
    Ok(())
}).await?;
```

---

### 3.3 TransactionSynchronizationManager —— 事务同步

**目标**：对标 Spring `TransactionSynchronizationManager`，
使用 `task_local!` 替代 ThreadLocal。

```rust
tokio::task_local! {
    static TX_CONTEXT: RefCell<Option<TxContext>>;
}

pub struct TxContext {
    pub status: TransactionStatus,
    pub synchronizations: Vec<Box<dyn TransactionSynchronization>>,
}

pub struct TransactionSynchronizationManager;

impl TransactionSynchronizationManager {
    pub fn is_active() -> bool;
    pub fn is_current_transaction_read_only() -> bool;
    pub fn current_transaction_name() -> Option<String>;
    pub fn current_transaction_status() -> Option<TransactionStatus>;

    pub async fn init_synchronization() -> Result<(), TransactionError>;
    pub fn register_synchronization(sync: Box<dyn TransactionSynchronization>);

    pub async fn trigger_before_commit() -> Result<(), TransactionError>;
    pub async fn trigger_before_completion(commit: bool) -> Result<(), TransactionError>;
    pub async fn trigger_after_commit() -> Result<(), TransactionError>;
    pub async fn trigger_after_completion(commit: bool) -> Result<(), TransactionError>;
}

#[async_trait]
pub trait TransactionSynchronization: Send + Sync {
    async fn before_commit(&self) -> Result<(), TransactionError> { Ok(()) }
    async fn before_completion(&self) -> Result<(), TransactionError> { Ok(()) }
    async fn after_commit(&self) -> Result<(), TransactionError> { Ok(()) }
    async fn after_completion(&self, commit: bool) -> Result<(), TransactionError> { Ok(()) }
}
```

---

### 3.4 SavepointManager

**目标**：对标 Spring `SavepointManager`，支持 `Nested` 传播。

```rust
pub trait SavepointManager: Send + Sync {
    fn create_savepoint(&self) -> Result<Savepoint, TransactionError>;
    fn rollback_to_savepoint(&self, savepoint: Savepoint) -> Result<(), TransactionError>;
    fn release_savepoint(&self, savepoint: Savepoint) -> Result<(), TransactionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Savepoint(String);

impl Savepoint {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn name(&self) -> &str;
}
```

---

## 四、声明式集成：vernal-aspects

### 4.1 注解清单

| Spring 注解 | vernal 过程宏 | 引入版本 |
|:---|:---|:---|
| `@Transactional` | `#[transactional]` | Java 1.0+ |
| `@TransactionalEventListener` | `#[tx_event_listener]` | Java 4.2+ |
| `@Propagation` / `@Isolation` | 宏参数 | Java 1.0+ |

### 4.2 `#[transactional]` 过程宏形态

```rust
#[transactional(
    propagation = Propagation::RequiresNew,
    isolation = Isolation::ReadCommitted,
    read_only = false,
    timeout_secs = 30,
    rollback_for = ["MyError"],   // 自定义异常触发 rollback
    no_rollback_for = ["NotFoundError"],
)]
pub async fn create_user(&self, name: String) -> Result<User, BoxError> {
    self.user_repo.save(&User::new(name)).await
}
```

### 4.3 织入位置

均在 `vernal-aspects` crate 的 `transaction.rs` 中实现，
通过 `vernal-aop` 的 `Advice` trait + 编译期宏织入。

```rust
// vernal-aspects/src/transaction.rs（伪代码）
pub struct TransactionalAdvice {
    manager: Arc<dyn PlatformTransactionManager>,
    definition: TransactionDefinition,
}

#[async_trait]
impl Advice for TransactionalAdvice {
    async fn around(&self, ctx: &mut AdviceCtx) -> Result<Arc<dyn Any + Send + Sync>, BoxError> {
        let status = self.manager.get_transaction(&self.definition)?;
        match ctx.proceed().await {
            Ok(result) => {
                self.manager.commit(status)?;
                Ok(result)
            }
            Err(e) => {
                self.manager.rollback(status)?;
                Err(e)
            }
        }
    }
}
```

### 4.4 Rollback 规则

默认对 `BoxError`（任意错误）触发 rollback；
可通过 `rollback_for` / `no_rollback_for` 精细控制；
业务代码可调用 `TransactionStatus::set_rollback_only()` 强制 rollback。

---

## 五、测试与验证

### 5.1 单元测试（待补齐）

| 测试项 | 目标 |
|:---|:---|
| `propagation_required_join` | 嵌套调用 `Required` 加入已有事务 |
| `propagation_requires_new_suspend` | `RequiresNew` 挂起外层事务 |
| `propagation_mandatory_no_tx` | `Mandatory` 在无事务时返回 `NoTransaction` |
| `propagation_never_with_tx` | `Never` 在有事务时返回 `IllegalState` |
| `propagation_nested_savepoint` | `Nested` 创建 savepoint，rollback 部分不影响外层 |
| `isolation_default_passthrough` | `Default` 不向 DB 传隔离级别 |
| `rollback_only_propagation` | 内层 `setRollbackOnly` 导致外层 commit 失败 |
| `transaction_synchronization_order` | before/after commit 回调顺序正确 |
| `transaction_template_async_execute` | `execute_async` 自动 begin/commit |
| `transaction_template_async_rollback_on_error` | 回调错误自动 rollback |

### 5.2 集成测试（待补齐）

```rust
// crates/vernal-tx/tests/integration.rs（待创建）
#[tokio::test(flavor = "multi_thread")]
async fn test_required_propagation_with_sqlx() {
    // 1. 启动 sqlx 连接池
    // 2. 创建 DataSourceTransactionManager
    // 3. 嵌套调用两次 save()
    // 4. 验证外层 rollback 时内层也被 rollback
}
```

### 5.3 性能基准（待补齐）

| 基准 | 目标 |
|:---|:---|
| `bench_simple_commit` | 单事务 commit 延迟 < 100µs |
| `bench_nested_required` | 嵌套 10 层 `Required` 无显著开销 |
| `bench_synchronization_overhead` | 10 个 sync 回调 < 50µs 开销 |

### 5.4 编译期验证

`cargo check` / `cargo clippy -D warnings` / `cargo test` 必须全部通过。

---

## 六、迁移路线

| 阶段 | 版本 | 任务 |
|:---|:---|:---|
| P0 骨架 | v0.1 | ✅ Propagation / Isolation / TransactionDefinition / TransactionStatus / PlatformTransactionManager；⏳ TransactionError 重构为 enum |
| P1 抽象 | v0.2 | `AbstractPlatformTransactionManager` + `TransactionSynchronizationManager`（task_local）+ `TransactionSynchronization` + `TransactionTemplate` |
| P2 集成 | v0.3 | `DataSourceTransactionManager`（vernal-jdbc）+ `JpaTransactionManager`（vernal-orm）+ `SavepointManager` 实现 |
| P3 声明 | v0.4 | `#[transactional]` 宏（vernal-aspects）+ SpEL 解析 `#root` / `#result` + rollback 规则 |
| P4 发布 | v1.0 | 完整文档 + `cargo doc` + 性能基准 + 100% 测试覆盖 |

---

## 附录 A：与 Spring TX 的完整 API 对照

| Spring 接口 | vernal-tx 类型 | 状态 |
|:---|:---|:---|
| `PlatformTransactionManager` | `PlatformTransactionManager` trait | ✅ |
| `AbstractPlatformTransactionManager` | `AbstractPlatformTransactionManager` trait | ⏳ |
| `TransactionDefinition` | `TransactionDefinition` struct | ✅ |
| `TransactionStatus` | `TransactionStatus` struct | ✅ |
| `SavepointManager` | `SavepointManager` trait | ⏳ |
| `TransactionTemplate` | `TransactionTemplate` struct | ⏳ |
| `TransactionSynchronizationManager` | `TransactionSynchronizationManager` struct | ⏳ |
| `TransactionSynchronization` | `TransactionSynchronization` trait | ⏳ |
| `Isolation`（5 级） | `Isolation` enum（5 变体） | ✅ |
| `Propagation`（7 种） | `Propagation` enum（7 变体） | ✅ |
| `UnexpectedRollbackException` | `TransactionError::UnexpectedRollback` | ⏳ |
| `HeuristicCompletionException` | `TransactionError::HeuristicCompletion` | ⏳ |
| `IllegalTransactionStateException` | `TransactionError::IllegalState` | ⏳ |
| `NoTransactionException` | `TransactionError::NoTransaction` | ⏳ |
| `InvalidPropagationException` | `TransactionError::InvalidPropagation` | ⏳ |
| `JtaTransactionManager` | 🚫 不支持 | ✅（决策） |

## 附录 B：依赖清单

| 依赖 | 版本 | 用途 |
|:---|:---|:---|
| `vernal-core` | path = `../vernal-core` | 基础错误类型（BoxError，预留） |
| `thiserror` | workspace（**待集成**） | `TransactionError` derive |
| `async-trait` | 0.1（**待集成**） | `AbstractPlatformTransactionManager` 异步 trait |
| `tokio` | 1.52.4（**待集成**） | `task_local!` 运行时 |
| `vernal-aspects` | path（**待集成**） | `#[transactional]` 过程宏 |

## 附录 C：向后兼容与弃用策略

- 本 crate 处于 v0.x 阶段，**允许 breaking change**。
- `Propagation` / `Isolation` 是 enum，新增变体是 minor change（不影响 match 穷尽性之外的代码）。
- `TransactionDefinition` 字段如需扩展，使用 `#[non_exhaustive]` + builder 模式。
- `PlatformTransactionManager` 新增方法必须提供默认实现。
- JTA 永远不会被支持；如有需求，使用 Saga 模式（vernal-context 提供）。
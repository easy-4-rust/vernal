<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-orm 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> 本模块只对标当前仓库的 `spring-orm`，不把 Spring Data JPA 或 Toasty 自身对象混入 Spring 对象计数。目标 crate 尚未建立。

固定提交中有 59 个 Spring ORM 业务对象，当前全部 `MISSING`。Toasty、SeaORM、Diesel 等只有逐对象精确符号及集成测试时才可标 `DEPENDENCY_REUSED`。

| Java 来源 | 目标 Rust |
|---|---|
| `jpa/JpaTransactionManager.java` | `jpa/jpa_transaction_manager.rs` |
| `jpa/EntityManagerFactoryUtils.java` | `jpa/entity_manager_factory_utils.rs` |
| `jpa/support/OpenEntityManagerInViewInterceptor.java` | `jpa/support/open_entity_manager_in_view_interceptor.rs` |
| `jpa/hibernate/HibernateExceptionTranslator.java` | `jpa/hibernate/hibernate_exception_translator.rs` |

Provider 专属对象仍需逐项迁移、精确复用或以平台证据标记。遵循[迁移验收规范](../迁移验收规范.md)。

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-orm 技术要求（对标 spring-orm / spring-data-jpa）

> **版本**：v2.0（2026-07-28）
> **定位**：vernal-orm crate 技术交接文档，对标 Spring Framework 7.0.8 spring-orm + spring-data-jpa。
> **主线**：toasty 0.9（编译期 codegen）为实现主线，sea-orm 0.12 为备选方案。
> **现状**：⬜ 待创建，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》6.1 节（ORM）。

---

## 一、总览

### 1.1 定位与边界

vernal-orm 是 Vernal Framework 的 **Tokio-first 异步 ORM 抽象内核**，
对标 spring-orm + spring-data-jpa 模块，提供 Repository 模式和实体管理能力。

**核心范式转换**：JPA 反射 + 运行时代理 → Toasty 编译期 codegen。

| 维度 | spring-orm/spring-data-jpa（语义参考） | vernal-orm（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（反射 + 注解处理器） | Rust（过程宏 + codegen） | 无反射，编译期静态分发 |
| ORM 框架 | Hibernate（运行时代理） | Toasty 0.9（编译期 codegen） | 编译期生成 SQL |
| 实体映射 | `@Entity` / `@Column` 注解 | `#[derive(Model)]` 过程宏 | 语义等价，形式不同 |
| 查询语言 | JPQL / Criteria API | Toasty DSL / sqlx 原生 | Rust 原生类型安全查询 |
| Repository | `JpaRepository<T, ID>` | `Repository<T>` trait | 语义对齐 |
| 事务管理 | `JpaTransactionManager` | 集成 vernal-tx | 跨 crate 协作 |
| 延迟加载 | Hibernate 代理 | **不支持**（🚫） | Rust 无运行时代理 |
| 缓存 | Hibernate L1/L2 缓存 | 外部缓存（vernal-cache） | 分离关注点 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  Repository 层：Repository<T> trait                  │
│  → find_by_id / find_all / save / delete / count     │
├─────────────────────────────────────────────────────┤
│  Service 层：#[service] + #[transactional]           │
│  → 业务逻辑 + 声明式事务                              │
├─────────────────────────────────────────────────────┤
│  Entity Manager 层：EntityManager<T>                 │
│  → persist / merge / remove / find / flush           │
├─────────────────────────────────────────────────────┤
│  代码生成层：Toasty codegen（编译期）                  │
│  → #[derive(Model)] → 生成 SQL / Row 映射 / Builder  │
├─────────────────────────────────────────────────────┤
│  事务层：集成 vernal-tx JpaTransactionManager         │
│  → 跨 Repository 事务边界                            │
├─────────────────────────────────────────────────────┤
│  驱动层：sqlx（PostgreSQL / MySQL / SQLite）          │
│  → 原生 async 连接                                   │
└─────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| ORM 主线 | Toasty 0.9 | 编译期 codegen，零反射开销，Rust 原生 |
| 备选 ORM | sea-orm 0.12 | 社区成熟，宏丰富，作为 fallback |
| 延迟加载 | 🚫 不支持 | Rust 无运行时代理（Hibernate Proxy） |
| N+1 问题 | 编译期检测 + `eager` 加载 | Toasty codegen 可生成 JOIN 查询 |
| 实体继承 | 🚫 不支持 | Rust 无类继承，用 trait 组合替代 |
| 缓存 | 外部 vernal-cache | L1 用 HashMap，L2 用 Redis/moka |

### 1.3.0 Toasty 生态定位分析（vernal-orm 选型依据）

#### 核心判断：Toasty 不会成为"Rust 生态标准 ORM"，但会成为"tokio 生态事实默认 ORM"

**三个关键事实**：

1. **Toasty 不是标准，是 ORM**：它没有像 `java.sql.Connection` 那样定义抽象接口
   - 它直接定义自己的 `toasty::Model` derive 宏
   - 没有 `Statement` / `PreparedStatement` / `ResultSet` 这种 JDBC 标准 API
   - 它的 driver 抽象是私有的（`toasty-driver-*`），不像 `java.sql.Driver` 那样有 ServiceLoader 机制
2. **Tokio 影响力 ≠ 标准制定**：Java 标准是 JSR 专家组（`javax.persistence`、`java.sql`）制定的
   - Rust 生态没有类似机构，**事实标准**由社区采用率决定
   - Tokio 团队影响力大，但 Toasty 仍需经社区验证
3. **Toasty 的真正定位是"tokio 系 ORM"**：
   - 与 Axum（HTTP 路由）形成组合拳
   - 与 Topcoat（全栈 Web）形成 Axum + Toasty + Topcoat 的 tokio-rs 全家桶
   - 这套组合在 **tokio 生态内部** 是默认推荐

#### 与 Java 对比

| Java 标准 | Rust 现状 | Toasty 角色 |
|---|---|---|
| `javax.persistence`（JPA 标准）| 无对等标准 | Toasty 不是标准 |
| `java.sql`（JDBC 标准）| 无对等标准 | sqlx 是事实 JDBC |
| `JPA Providers`：Hibernate / EclipseLink | 无 Provider 模式 | toasty 是单一实现 |
| `JDBC Drivers`：MySQL/PostgreSQL 各自 | sqlx 支持多 dialect | toasty-driver-* 是 toasty 专属 |

**结论**：Toasty **没有制定 Rust ORM 标准**，它只是 tokio 系一个高质量 ORM 实现。就像 Hibernate 是 Java ORM 标杆，但 Hibernate 也不是 JPA 标准的制定者。

#### 对 vernal-orm 的影响

| 维度 | 影响 | vernal-orm 应对 |
|---|---|---|
| **tokio 生态** | 正向——vernal 全栈基于 tokio，与 Toasty 同源天然整合 | 维持 Toasty 锁定主线 |
| **Dialect 支持** | Toasty 已支持 SQLite/Turso/PostgreSQL/MySQL/DynamoDB | vernal-orm 通过 Toasty 间接获得这 5 种方言 |
| **Async 原生** | Toasty 完全 async-first（tokio 系）| vernal-orm 完美契合 |
| **编译期 codegen** | Toasty 强项，零反射 | vernal-orm 同样受益于零反射 |
| **社区采用** | Axum + Toasty 组合可能成为 tokio 系默认 ORM | vernal 受益于生态普及 |
| **MyBatis 用户迁移** | MyBatis 习惯用户可能更倾向 rbatis 而非 Toasty | vernal-rbatis 仍需保留 |

#### vernal-orm 当前定位（不变）

```
spring-orm (Java)   →  vernal-orm (Rust)
   ↓                       ↓
Hibernate (JPA 实现)   →  Toasty (tokio-rs ORM 标杆)
   ↓                       ↓
JPA 标准 (javax)     →  无对等标准（Toasty 是实现，不是标准）
   ↓                       ↓
多 Provider 切换     →  多 dialect 切换（但需 toasty-sql 中转）
```

#### 4 条优化指导

**指导 1：vernal-orm 选型不变，锁定 Toasty**

理由：
- Toasty 与 vernal 全栈（tokio 系）同源
- 编译期 codegen 对标 Hibernate（JPA 主实现）
- 已规划在 `vernal-orm/Spring-orm-技术要求.md` 中

```toml
# vernal-orm/Cargo.toml（规划）
[dependencies]
toasty = "0.9"           # ORM 主线
toasty-sql = "0.9"       # SQL dialect
toasty-driver-postgres = "0.9"
toasty-driver-mysql = "0.9"
toasty-driver-sqlite = "0.9"
```

**指导 2：抽象层设计——vernal-orm 包装 Toasty，不泄漏到 public API**

```rust
// ❌ 直接暴露 toasty 类型（强耦合）
pub fn save_user(user: &toasty::Model) -> Result<(), toasty::Error> { ... }

// ✅ vernal-orm 抽象层（解耦）
pub trait Repository<E: Entity> {
    async fn save(&self, entity: &E) -> Result<(), OrmError>;
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<E>, OrmError>;
    // ... 抽象方法
}

// vernal-orm-toasty 内部实现
struct ToastyRepositoryImpl { ... }
impl<E: ToastyModel> Repository<E> for ToastyRepositoryImpl { ... }
```

**指导 3：多 ORM 共存——保留 rbatis 作为迁移路径**

```rust
// 业务层用 vernal-orm 抽象
use vernal_orm::Repository;

#[derive(Repository)]
#[repository(impl = "toasty")]   // 编译期选择
pub struct UserRepo;

// 也支持：
// #[repository(impl = "rbatis")]  // 旧 MyBatis 习惯
// #[repository(impl = "sqlx")]    // 需要 SQL 控制的场景
```

**指导 4：与 Toasty 生态同步的版本策略**

| Toasty 版本 | vernal 应对 |
|---|---|
| 0.x 阶段 | vernal-orm 跟踪 minor 版本，major 升级时做 breaking change review |
| 1.0 GA | 锁定 1.0.x，**正式纳入 vernal 发布** |
| 2.0+ | 评估迁移成本，保留旧版本兼容层（参考 rbatis 4.x→5.x 升级策略） |

#### 与 Topcoat 的协同价值

```
tokio-rs 全家桶（2026-07 发布节奏）
├── Tokio（异步运行时）           ← vernal 全栈基础
├── Axum（HTTP 路由）            ← vernal-web 10 适配器之一
├── tracing（日志）              ← vernal-log 底层
├── Toasty（ORM）                ← vernal-orm 锁定主线
└── Topcoat（全栈 Web 框架）    ← vernal-web/webmvc/webflux 整合
```

**vernal 的位置**：把"tokio-rs 全家桶"用 vernal 中间层 API 统一封装，提供给业务层使用。

#### 结论

1. **Toasty 不会成为 Rust 生态的标准 ORM**——它没有制定标准，只是实现
2. **Toasty 会成为 tokio 系的事实默认 ORM**——凭借 tokio-rs 的影响力和与 Axum/Topcoat 的协同
3. **vernal-orm 锁定 Toasty 是正确选择**——与 vernal 全栈的 tokio 血统一致
4. **vernal-orm 关键设计是抽象层包装**——不直接暴露 toasty 类型，保留切换 rbatis/sqlx 的能力
5. **生态护城河**——vernal 的价值不在 ORM 本身（sqlx/rbatis/Toasty 各有优势），而在 **34 crate 一站式整合** + **10 Web 框架适配** + **23 份完整文档**

### 1.3.1 Toasty 生态完整地址（ORM 主线 crate）

| crate | 版本 | crates.io | docs.rs | GitHub |
|---|---|---|---|---|
| toasty | 0.9.0 | [crates.io](https://crates.io/crates/toasty) | [docs.rs](https://docs.rs/toasty/0.9.0/toasty/) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-core | 0.9.0 | [crates.io](https://crates.io/crates/toasty-core) | [docs.rs](https://docs.rs/toasty-core) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-macros | 0.9.0 | [crates.io](https://crates.io/crates/toasty-macros) | [docs.rs](https://docs.rs/toasty-macros) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-sql | 0.9.0 | [crates.io](https://crates.io/crates/toasty-sql) | [docs.rs](https://docs.rs/toasty-sql) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-cli | 0.9.0 | [crates.io](https://crates.io/crates/toasty-cli) | [docs.rs](https://docs.rs/toasty-cli) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-driver-sqlite | 0.9.0 | [crates.io](https://crates.io/crates/toasty-driver-sqlite) | [docs.rs](https://docs.rs/toasty-driver-sqlite) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-driver-turso | 0.9.0 | [crates.io](https://crates.io/crates/toasty-driver-turso) | [docs.rs](https://docs.rs/toasty-driver-turso) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-driver-mysql | 0.9.0 | [crates.io](https://crates.io/crates/toasty-driver-mysql) | [docs.rs](https://docs.rs/toasty-driver-mysql) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-driver-postgresql | 0.9.0 | [crates.io](https://crates.io/crates/toasty-driver-postgresql) | [docs.rs](https://docs.rs/toasty-driver-postgresql) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |
| toasty-driver-dynamodb | 0.9.0 | [crates.io](https://crates.io/crates/toasty-driver-dynamodb) | [docs.rs](https://docs.rs/toasty-driver-dynamodb) | [tokio-rs/toasty](https://github.com/tokio-rs/toasty) |

> 所有 toasty-* 子 crate 共享同一 GitHub 仓库 [tokio-rs/toasty](https://github.com/tokio-rs/toasty)。
> vernal-orm 的 `Cargo.toml` 依赖 `toasty`（核心）+ 对应 driver（按需），不直接依赖 `toasty-core`/`toasty-macros`/`toasty-sql`。

### 1.4 JPA 反射 → Toasty 编译期 codegen 对比

| JPA 概念 | 运行时机制 | Toasty 编译期替代 |
|:---|:---|:---|
| `@Entity` | 反射扫描类路径 | `#[derive(Model)]` 过程宏 |
| `@Column` | 反射读取字段注解 | `#[model(column = "...")]` 属性 |
| `@Id` | 反射识别主键 | `#[model(primary_key)]` 属性 |
| `@OneToMany` | 代理对象延迟加载 | 编译期生成 JOIN 查询 |
| `@NamedQuery` | 运行时解析 JPQL | 编译期类型安全 DSL |
| EntityListener | 反射调用回调方法 | `#[model(before_insert)]` 钩子 |
| PersistenceContext | 运行时一级缓存 | 手动管理 / vernal-cache |

---

## 二、Repository 模式

### 2.1 Repository trait —— 通用仓库契约

**现状**：⬜ 待实现。
**语义参照**：spring-data-jpa `Repository<T, ID>` / `JpaRepository<T, ID>`。

#### Spring API（Java）

```java
// spring-data-jpa 核心 Repository 接口
public interface JpaRepository<T, ID> extends
        PagingAndSortingRepository<T, ID>,
        QueryByExampleExecutor<T> {
    List<T> findAll();
    List<T> findAll(Sort sort);
    Page<T> findAll(Pageable pageable);
    Optional<T> findById(ID id);
    <S extends T> S save(S entity);
    <S extends T> List<S> saveAll(Iterable<S> entities);
    void deleteById(ID id);
    void delete(T entity);
    long count();
    boolean existsById(ID id);
}
```

#### Rust trait

```rust
/// 通用 Repository trait。
/// 对标 Spring JpaRepository<T, ID>。
#[async_trait]
pub trait Repository<T: Model>: Send + Sync {
    /// 主键类型。
    type Id: Send + Sync + Clone + PartialEq + std::fmt::Debug;

    /// 根据 ID 查询。
    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<T>, OrmError>;

    /// 查询所有。
    async fn find_all(&self) -> Result<Vec<T>, OrmError>;

    /// 分页查询。
    async fn find_page(&self, page: PageRequest) -> Result<Page<T>, OrmError>;

    /// 排序查询。
    async fn find_all_sorted(&self, sort: Sort) -> Result<Vec<T>, OrmError>;

    /// 保存（INSERT 或 UPDATE）。
    async fn save(&self, entity: &T) -> Result<T, OrmError>;

    /// 批量保存。
    async fn save_all(&self, entities: &[T]) -> Result<Vec<T>, OrmError>;

    /// 根据 ID 删除。
    async fn delete_by_id(&self, id: &Self::Id) -> Result<(), OrmError>;

    /// 删除实体。
    async fn delete(&self, entity: &T) -> Result<(), OrmError>;

    /// 批量删除。
    async fn delete_all(&self, entities: &[T]) -> Result<(), OrmError>;

    /// 计数。
    async fn count(&self) -> Result<u64, OrmError>;

    /// 是否存在。
    async fn exists_by_id(&self, id: &Self::Id) -> Result<bool, OrmError>;
}
```

---

### 2.2 PagingAndSorting —— 分页与排序

**语义参照**：spring-data `Pageable` / `Page` / `Sort`。

```rust
/// 分页请求。
/// 对标 Spring Pageable。
#[derive(Debug, Clone)]
pub struct PageRequest {
    pub page: u32,
    pub size: u32,
    pub sort: Option<Sort>,
}

impl PageRequest {
    pub fn new(page: u32, size: u32) -> Self;
    pub fn with_sort(page: u32, size: u32, sort: Sort) -> Self;
}

/// 排序。
/// 对标 Spring Sort。
#[derive(Debug, Clone)]
pub struct Sort {
    pub orders: Vec<Order>,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub property: String,
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Asc,
    Desc,
}

/// 分页结果。
/// 对标 Spring Page<T>。
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub total_elements: u64,
    pub total_pages: u32,
    pub current_page: u32,
    pub page_size: u32,
    pub is_first: bool,
    pub is_last: bool,
}
```

---

### 2.3 QueryByExample —— 示例查询

**语义参照**：spring-data `QueryByExampleExecutor<T>`。

```rust
/// 示例查询 trait。
/// 对标 Spring QueryByExampleExecutor<T>。
#[async_trait]
pub trait QueryByExample<T: Model>: Repository<T> {
    /// 按示例查询。
    async fn find_by_example(&self, example: Example<T>) -> Result<Vec<T>, OrmError>;

    /// 按示例查询单个。
    async fn find_one_by_example(&self, example: Example<T>) -> Result<Option<T>, OrmError>;

    /// 按示例计数。
    async fn count_by_example(&self, example: Example<T>) -> Result<u64, OrmError>;

    /// 按示例是否存在。
    async fn exists_by_example(&self, example: Example<T>) -> Result<bool, OrmError>;
}

/// 查询示例。
#[derive(Debug, Clone)]
pub struct Example<T: Model> {
    pub probe: T,
    pub matcher: ExampleMatcher,
}

/// 示例匹配器。
#[derive(Debug, Clone)]
pub struct ExampleMatcher {
    pub default_string_matcher: StringMatcher,
    pub ignore_case: bool,
    pub null_handling: NullHandling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringMatcher { Exact, StartingWith, EndingWith, Containing, IgnoreCase }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullHandling { Include, Exclude }
```

---

## 三、EntityManager —— 实体管理

### 3.1 EntityManager trait

**现状**：⬜ 待实现。
**语义参照**：JPA `EntityManager`。

#### Spring API（Java）

```java
// JPA EntityManager
public interface EntityManager {
    void persist(Object entity);
    <T> T merge(T entity);
    void remove(Object entity);
    <T> T find(Class<T> entityClass, Object primaryKey);
    void flush();
    void clear();
    boolean contains(Object entity);
}
```

#### Rust trait

```rust
/// 实体管理器 trait。
/// 对标 JPA EntityManager。
#[async_trait]
pub trait EntityManager<T: Model>: Send + Sync {
    /// 持久化新实体（INSERT）。
    async fn persist(&self, entity: &T) -> Result<(), OrmError>;

    /// 合并实体（UPDATE）。
    async fn merge(&self, entity: &T) -> Result<T, OrmError>;

    /// 移除实体（DELETE）。
    async fn remove(&self, entity: &T) -> Result<(), OrmError>;

    /// 根据主键查找。
    async fn find(&self, id: &T::Id) -> Result<Option<T>, OrmError>;

    /// 刷新到数据库。
    async fn flush(&self) -> Result<(), OrmError>;

    /// 清除一级缓存。
    fn clear(&self);

    /// 实体是否在持久化上下文中。
    fn contains(&self, entity: &T) -> bool;
}
```

---

### 3.2 Model trait —— 实体模型契约

```rust
/// 实体模型 trait。
/// 由 #[derive(Model)] 自动生成。
pub trait Model: Send + Sync + Clone + 'static {
    /// 主键类型。
    type Id: Send + Sync + Clone + PartialEq + std::fmt::Debug;

    /// 表名。
    fn table_name() -> &'static str;

    /// 获取主键值。
    fn id(&self) -> &Self::Id;

    /// 从行数据构建。
    fn from_row(row: &dyn Row) -> Result<Self, OrmError> where Self: Sized;

    /// 转为插入参数。
    fn to_insert_params(&self) -> Vec<Box<dyn sqlx::Encode<'_, DB>>>;

    /// 转为更新参数。
    fn to_update_params(&self) -> Vec<Box<dyn sqlx::Encode<'_, DB>>>;
}
```

#### Model derive 宏目标

```rust
/// 使用 toasty derive 宏定义实体。
/// 编译期生成 Model trait 实现。
#[derive(Debug, Clone, Model)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key, auto_increment)]
    pub id: i64,
    #[model(column = "name", not_null)]
    pub name: String,
    #[model(column = "email", unique)]
    pub email: String,
    #[model(column = "created_at", default = "now()")]
    pub created_at: chrono::NaiveDateTime,
}
```

---

### 3.3 QueryBuilder —— 类型安全查询构建

```rust
/// 查询构建器。
/// 对标 JPA CriteriaBuilder。
pub struct QueryBuilder<T: Model> {
    pool: Arc<dyn AnyPool>,
    conditions: Vec<Condition>,
    orders: Vec<Order>,
    limit: Option<u32>,
    offset: Option<u32>,
    includes: Vec<String>,
}

impl<T: Model> QueryBuilder<T> {
    pub fn new(pool: Arc<dyn AnyPool>) -> Self;

    /// 条件（WHERE）。
    pub fn where_eq(mut self, field: &str, value: impl Into<SqlValue>) -> Self;
    pub fn where_ne(mut self, field: &str, value: impl Into<SqlValue>) -> Self;
    pub fn where_gt(mut self, field: &str, value: impl Into<SqlValue>) -> Self;
    pub fn where_lt(mut self, field: &str, value: impl Into<SqlValue>) -> Self;
    pub fn where_like(mut self, field: &str, pattern: &str) -> Self;
    pub fn where_in(mut self, field: &str, values: Vec<SqlValue>) -> Self;
    pub fn where_null(mut self, field: &str) -> Self;
    pub fn where_not_null(mut self, field: &str) -> Self;

    /// 排序（ORDER BY）。
    pub fn order_by(mut self, field: &str, direction: Direction) -> Self;

    /// 分页。
    pub fn limit(mut self, limit: u32) -> Self;
    pub fn offset(mut self, offset: u32) -> Self;

    /// 关联加载（JOIN）。
    pub fn include(mut self, relation: &str) -> Self;

    /// 执行查询。
    pub async fn fetch_all(self) -> Result<Vec<T>, OrmError>;
    pub async fn fetch_one(self) -> Result<Option<T>, OrmError>;
    pub async fn fetch_first(self) -> Result<Option<T>, OrmError>;
    pub async fn count(self) -> Result<u64, OrmError>;
    pub async fn exists(self) -> Result<bool, OrmError>;
}
```

---

## 四、JpaTransactionManager —— ORM 事务管理

### 4.1 事务管理器

**现状**：⬜ 待实现。
**语义参照**：spring-orm `JpaTransactionManager`。

#### Spring API（Java）

```java
// spring-orm JPA 事务管理器
public class JpaTransactionManager extends AbstractPlatformTransactionManager
        implements ResourceTransactionManager {
    public JpaTransactionManager(EntityManagerFactory emf) { ... }
}
```

#### Rust 目标设计

```rust
/// JPA 事务管理器。
/// 对标 spring-orm JpaTransactionManager。
/// 实现 vernal-tx PlatformTransactionManager trait。
pub struct JpaTransactionManager {
    pool: Arc<dyn AnyPool>,
    name: String,
}

impl JpaTransactionManager {
    pub fn new(pool: Arc<dyn AnyPool>, name: impl Into<String>) -> Self;
}

/// 内部事务对象。
struct JpaTransactionObject {
    connection: Option<Box<dyn AnyConnection>>,
    auto_commit: bool,
    flush_mode: FlushMode,
}

/// 刷新模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlushMode {
    Auto,       // 自动刷新（默认）
    Commit,     // 仅提交时刷新
    Manual,     // 手动刷新
    Always,     // 每次查询前刷新
}

impl PlatformTransactionManager for JpaTransactionManager {
    fn get_transaction(&self, definition: &TransactionDefinition) -> Result<TransactionStatus, TransactionError> {
        // 根据 Propagation 决定行为 → 获取连接 → 关闭 auto_commit → 开始事务
        todo!()
    }
    fn commit(&self, status: TransactionStatus) -> Result<(), TransactionError> {
        // flush 所有变更 → 提交事务 → 释放连接
        todo!()
    }
    fn rollback(&self, status: TransactionStatus) -> Result<(), TransactionError> {
        // 回滚事务 → 清除持久化上下文 → 释放连接
        todo!()
    }
}
```

---

### 4.2 SimpleJpaRepository —— 默认 Repository 实现

**语义参照**：spring-data-jpa `SimpleJpaRepository<T, ID>`。

```rust
/// 默认 Repository 实现。
/// 对标 spring-data-jpa SimpleJpaRepository<T, ID>。
pub struct SimpleJpaRepository<T: Model> {
    pool: Arc<dyn AnyPool>,
    _phantom: PhantomData<T>,
}

impl<T: Model> SimpleJpaRepository<T> {
    pub fn new(pool: Arc<dyn AnyPool>) -> Self;
}

#[async_trait]
impl<T: Model> Repository<T> for SimpleJpaRepository<T> {
    type Id = T::Id;

    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<T>, OrmError> {
        let sql = format!(
            "SELECT * FROM {} WHERE id = $1",
            T::table_name()
        );
        // 执行查询并映射
        todo!()
    }

    async fn save(&self, entity: &T) -> Result<T, OrmError> {
        // UPSERT 逻辑：存在则 UPDATE，不存在则 INSERT
        todo!()
    }

    async fn delete_by_id(&self, id: &Self::Id) -> Result<(), OrmError> {
        let sql = format!(
            "DELETE FROM {} WHERE id = $1",
            T::table_name()
        );
        // 执行删除
        todo!()
    }

    // ... 其他方法实现
}
```

---

### 4.3 Repository 注册

```rust
/// Repository 注册表。
/// 对标 spring-data-jpa RepositoryFactory。
pub struct RepositoryRegistry {
    repositories: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl RepositoryRegistry {
    pub fn new() -> Self;

    /// 注册 Repository。
    pub fn register<T: Model>(&mut self, repo: Arc<dyn Repository<T>>) {
        self.repositories.insert(TypeId::of::<T>(), repo);
    }

    /// 获取 Repository。
    pub fn get<T: Model>(&self) -> Option<Arc<dyn Repository<T>>> {
        self.repositories
            .get(&TypeId::of::<T>())
            .and_then(|r| r.clone().downcast().ok())
    }
}
```

---

## 五、Toasty Codegen 集成

### 5.1 Toasty 0.9 选型

**状态**：⬜ 待集成。
**理由**：Toasty 是 Rust 生态中唯一提供 JPA 风格编译期 codegen 的 ORM。

#### Toasty 核心特性

| 特性 | 说明 | JPA 对应 |
|:---|:---|:---|
| `#[derive(Model)]` | 编译期生成实体映射 | `@Entity` + `@Table` |
| `#[model(column)]` | 字段映射 | `@Column` |
| `#[model(primary_key)]` | 主键标记 | `@Id` + `@GeneratedValue` |
| `#[model(has_one)]` | 一对一关系 | `@OneToOne` |
| `#[model(has_many)]` | 一对多关系 | `@OneToMany` |
| `#[model(belongs_to)]` | 多对一关系 | `@ManyToOne` |
| 编译期 SQL 生成 | 零运行时开销 | Hibernate HQL 编译 |
| 类型安全查询 | 编译期检查 | Criteria API |

#### Toasty 代码生成示例

```rust
// toasty 模型定义
#[derive(Debug, Model)]
pub struct User {
    #[model(primary_key)]
    pub id: Id<User>,
    #[model(unique)]
    pub email: String,
    pub name: String,
    #[model(has_many)]
    pub posts: HasMany<Post>,
}

#[derive(Debug, Model)]
pub struct Post {
    #[model(primary_key)]
    pub id: Id<Post>,
    pub title: String,
    pub body: String,
    #[model(belongs_to)]
    pub user: BelongsTo<User>,
}

// toasty 生成的查询 API（编译期类型安全）
let user = User::find_by_id(&id).get(&mut db).await?;
let posts = User::find_by_id(&id).posts().all().get(&mut db).await?;
```

---

### 5.2 sea-orm 0.12 备选方案

**状态**：⬜ 备选（Toasty 不满足时切换）。

#### sea-orm 核心特性

| 特性 | 说明 | 与 Toasty 对比 |
|:---|:---|:---|
| `Entity` 宏 | 编译期生成实体 | 类似，但宏语法不同 |
| `DeriveEntityModel` | 模型派生 | 类似 `#[derive(Model)]` |
| `DeriveRelation` | 关系定义 | 类似 `has_many` / `belongs_to` |
| `ActiveModel` | 可变模型 | Toasty 无此概念 |
| `Select` / `Insert` / `Update` / `Delete` | CRUD 操作 | 类似 QueryBuilder |
| SeaQuery | 底层查询构建器 | Toasty 内建 |
| 社区活跃度 | 高 | Toasty 较新，社区小 |

#### sea-orm 实体定义

```rust
// sea-orm 实体定义（备选方案）
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Posts,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef { Relation::Posts.def() }
}
```

---

### 5.3 Toasty vs sea-orm 决策矩阵

| 维度 | Toasty 0.9 | sea-orm 0.12 | 决策 |
|:---|:---|:---|:---|
| 编译期 codegen | ✅ 核心设计 | ✅ 宏派生 | 平手 |
| JPA 语义对齐 | 高（has_many / belongs_to） | 中（Relation enum） | Toasty 胜 |
| 社区成熟度 | 低（新项目） | 高（活跃社区） | sea-orm 胜 |
| 异步支持 | 原生 async | 原生 async | 平手 |
| 查询类型安全 | 编译期 | 编译期 | 平手 |
| 关联加载 | 编译期 JOIN | 运行时 N+1（可优化） | Toasty 胜 |
| 最终决策 | **主线** | **备选** | — |

---

### 5.4 代码生成工作流

```
用户定义实体（#[derive(Model)]）
  ↓
Toasty 过程宏解析
  ↓
编译期生成：
  ├── Model trait 实现（from_row / to_insert_params / to_update_params）
  ├── SQL 常量（INSERT / UPDATE / DELETE / SELECT）
  ├── QueryBuilder 类型安全方法
  ├── 关联查询（JOIN / 子查询）
  └── 变更检测（dirty tracking）
  ↓
运行时：
  ├── 零反射开销
  ├── 直接调用生成的 SQL
  └── sqlx 异步执行
```

---

## 六、集成约束与路线图

### 6.1 依赖约束

| 依赖 | 版本 | 说明 |
|:---|:---|:---|
| `toasty` | `0.9` | ORM codegen（主线） |
| `sea-orm` | `0.12` | ORM 备选方案 |
| `sqlx` | `0.9` | 底层数据库驱动 |
| `vernal-tx` | `workspace` | 事务管理 |
| `vernal-cache` | `workspace` | L1/L2 缓存（可选） |

---

### 6.2 延迟加载不支持决策

| JPA 延迟加载概念 | Rust 不支持原因 | 替代方案 |
|:---|:---|:---|
| `@OneToMany(fetch = LAZY)` | Rust 无运行时代理对象 | 编译期 `include()` 显式加载 |
| `Hibernate.initialize(proxy)` | 无代理机制 | `QueryBuilder::include()` |
| `@LazyCollection` | 无集合代理 | `HasMany::load()` 显式调用 |
| 字节码增强 | Rust 无字节码 | 编译期 codegen |

#### 显式加载替代方案

```rust
// JPA 风格（延迟加载，运行时代理）
// User user = entityManager.find(User.class, 1L);
// List<Post> posts = user.getPosts(); // 代理对象，首次访问时加载

// Rust 风格（显式加载，编译期确定）
let user = User::find_by_id(&id).get(&mut db).await?;
let posts = User::find_by_id(&id)
    .posts()    // 显式声明关联
    .all()      // 生成 JOIN 查询
    .get(&mut db).await?;
```

---

### 6.3 实体生命周期钩子

```rust
/// 实体生命周期钩子。
/// 对标 JPA EntityListener / @PrePersist / @PostPersist。
pub trait ModelLifecycle: Model {
    fn before_insert(&mut self) -> Result<(), OrmError> { Ok(()) }
    fn after_insert(&self) -> Result<(), OrmError> { Ok(()) }
    fn before_update(&mut self) -> Result<(), OrmError> { Ok(()) }
    fn after_update(&self) -> Result<(), OrmError> { Ok(()) }
    fn before_delete(&mut self) -> Result<(), OrmError> { Ok(()) }
    fn after_delete(&self) -> Result<(), OrmError> { Ok(()) }
    fn after_load(&mut self) -> Result<(), OrmError> { Ok(()) }
}
```

---

### 6.4 错误处理

```rust
/// ORM 错误类型。
#[derive(Debug, thiserror::Error)]
pub enum OrmError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("实体未找到: {0}")]
    NotFound(String),
    #[error("唯一约束冲突: {0}")]
    UniqueViolation(String),
    #[error("外键约束冲突: {0}")]
    ForeignKeyViolation(String),
    #[error("映射错误: {0}")]
    Mapping(String),
    #[error("查询错误: {0}")]
    Query(String),
    #[error("事务错误: {0}")]
    Transaction(String),
}
```

---

### 6.5 待补齐工作路线图

| 阶段 | 内容 | 优先级 | 预估工作量 |
|:---|:---|:---|:---|
| S1 | `Model` trait + `#[derive(Model)]` 宏（Toasty 集成） | P0 | 3 天 |
| S2 | `Repository<T>` trait + `SimpleJpaRepository` | P0 | 2 天 |
| S3 | `EntityManager<T>` trait | P1 | 1.5 天 |
| S4 | `JpaTransactionManager`（集成 vernal-tx） | P0 | 1.5 天 |
| S5 | `QueryBuilder` 类型安全查询 | P1 | 2 天 |
| S6 | 分页与排序（PageRequest / Page / Sort） | P1 | 1 天 |
| S7 | `QueryByExample` 示例查询 | P2 | 1 天 |
| S8 | sea-orm 备选适配层 | P2 | 2 天 |

---

### 6.6 成熟度状态

| 维度 | 当前 | 目标 |
|:---|:---|:---|
| 文件数 | 0 | 20+ |
| 行数 | 0 | 1500+ |
| 核心 trait | 0 | 4（Model / Repository / EntityManager / Lifecycle） |
| 事务管理器 | 0 | 1（JpaTransactionManager） |
| derive 宏 | 0 | 1（#[derive(Model)]） |
| ORM 引擎 | 0 | 1（Toasty 0.9，备选 sea-orm 0.12） |
| 与 spring-orm 语义对标度 | 0% | 75%+ |

---

## 附录：spring-orm / spring-data-jpa 语义覆盖全景

| spring-orm 包 | 类数 | vernal-orm 状态 | 说明 |
|:---|:---|:---|:---|
| orm（EntityManagerFactory） | 12 | ⬜ 待实现 | Toasty 集成 |
| jpa（JpaTransactionManager） | 18 | ⬜ 待实现 | 集成 vernal-tx |
| hibernate5/6（HibernateJpaVendorAdapter） | 15 | 🚫 不适用 | Toasty 替代 Hibernate |
| spring-data-jpa 根包 | 25 | ⬜ 待实现 | Repository 模式 |
| repository（JpaRepository） | 30 | ⬜ 待实现 | trait 定义 |
| support（QueryByExample） | 10 | ⬜ 待实现 | 示例查询 |
| config（JpaNamespaceHandler） | 8 | 🚫 不适用 | Rust 无 XML 配置 |

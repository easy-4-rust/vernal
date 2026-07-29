# vernal-async 技术要求（对标 spring-async）

> **版本**：v1.0（2026-07-28）
> **定位**：vernal-async crate 技术交接文档，对标 Spring Framework 7.0.8 spring-async。
> **现状**：2 文件 / 28 行骨架，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》8.7 节（工具库 — 异步调度）。

---

## 一、总览

### 1.1 定位与边界

vernal-async 是 Vernal Framework 的 **Tokio-first 异步执行抽象内核**，
对标 spring-async 模块，提供异步任务提交与生命周期管理能力。

| 维度 | spring-async（语义参考） | vernal-async（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（Executor + Callable） | Rust（trait + tokio） | 无 Future 阻塞 |
| 异步 | `Future<T>` / `CompletableFuture<T>` | `Future<Output = T>` | 1:1 对应 |
| 执行器 | `TaskExecutor` / `AsyncTaskExecutor` | `AsyncTaskExecutor` trait | 已简化 |
| 默认实现 | `SimpleAsyncTaskExecutor`（每任务一线程） | `SimpleAsyncTaskExecutor`（`tokio::spawn`） | 复用 Tokio 调度器 |
| 异常处理 | `AsyncUncaughtExceptionHandler` | 同名 trait | — |
| 声明式 | `@Async` | `#[async_task]`（vernal-aspects） | 编译期织入 |
| 调度 | `ScheduledExecutorService` | **不在本 crate**（由 tokio + vernal-aspects 提供） | — |
| Reactive | `ReactiveAdapter` | **不在本 crate**（vernal-webflux 提供） | — |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────────┐
│  声明式层：#[async_task] 过程宏（vernal-aspects）          │
│  → 解析 executor / timeout / exception_handler          │
├─────────────────────────────────────────────────────────┤
│  扩展层：AsyncTaskExecutor（待扩展）                      │
│  → submit_with_future / submit_with_timeout / submit_listenable │
├─────────────────────────────────────────────────────────┤
│  接口层：AsyncTaskExecutor trait                          │
│  → submit(F) -> Result<(), BoxError>                    │
├─────────────────────────────────────────────────────────┤
│  管理器层：AsyncConfigurer（待补齐）                       │
│  → get_async_executor / get_async_uncaught_exception_handler │
├─────────────────────────────────────────────────────────┤
│  默认实现：SimpleAsyncTaskExecutor                        │
│  → tokio::spawn（不创建线程，复用调度器）                  │
├─────────────────────────────────────────────────────────┤
│  底层：tokio 1.52.4 + tokio-util 0.7.16                   │
│  → TaskTracker / CancellationToken                       │
└─────────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 默认执行器 | `tokio::spawn` | 不创建线程，复用 Tokio 调度器；与 Java `SimpleAsyncTaskExecutor` 1:1 语义 |
| 异常处理 | `AsyncUncaughtExceptionHandler` trait | 用户自定义 panic 处理逻辑 |
| 类型擦除 | `Future<Output = Result<(), BoxError>>` | 与 dyn-compatible 兼容，返回类型固定 |
| 超时 | `tokio::time::timeout` 包装 | 用户在调用方控制，不需要执行器内置 |
| 取消 | `CancellationToken`（tokio-util） | 对齐 Spring `@Async` 任务取消 |
| Scheduled | 🚫 不在本 crate | 由 `#[scheduled]` 宏 + tokio Interval 提供（vernal-aspects） |
| 线程池 | 🚫 不内置专用线程池 | tokio 调度器已是最佳线程池 |
| Java 适配 | `AsyncConfigurer` 抽象 | 暴露给用户注入自定义执行器 |

### 1.4 当前骨架文件

| 文件 | 行数 | 内容 |
|:---|:---|:---|
| `lib.rs` | 7 | 模块声明 + re-export（`AsyncTaskExecutor`） |
| `executor.rs` | 14 | `AsyncTaskExecutor` trait + `BoxError` 引用 |

### 1.5 与其他 crate 的边界

| crate | 关系 | 说明 |
|:---|:---|:---|
| `vernal-core` | 依赖 | `BoxError`（已使用） |
| `vernal-aspects` | 上层增强 | `#[async_task]` 过程宏织入 |
| `vernal-context` | 同级装配 | `AsyncConfigurer` 配置入口 |
| `vernal-web` | 下游使用 | Controller 异步处理 |
| `vernal-messaging` | 下游使用 | 异步消息监听 |
| `tokio` | 上游 | 异步运行时 |
| `tokio-util` | 上游 | CancellationToken / TaskTracker |
| `futures-util` | 上游 | Future 组合器 |

---

## 二、核心 Trait 体系

### 2.1 AsyncTaskExecutor —— 异步任务执行契约

**现状**：已定义最简 trait（1 方法）。
**语义参照**：spring-async `AsyncTaskExecutor` / `TaskExecutor`。

#### Spring API（Java）

```java
// TaskExecutor
public interface TaskExecutor {
    void execute(Runnable task);
}

// AsyncTaskExecutor
public interface AsyncTaskExecutor extends TaskExecutor {
    void execute(Runnable task);
    void execute(Runnable task, long startTimeout);
    Future<?> submit(Runnable task);
    <T> Future<T> submit(Callable<T> task);
    <T> ListenableFuture<T> submitListenable(Runnable task);
    <T> ListenableFuture<T> submitListenable(Callable<T> task);
}
```

#### Rust trait（已有 + 待扩展）

```rust
use std::future::Future;
use vernal_core::BoxError;

pub trait AsyncTaskExecutor: Send + Sync {
    /// 提交异步任务到执行器。
    fn submit<F>(&self, task: F) -> Result<(), BoxError>
    where
        F: Future<Output = Result<(), BoxError>> + Send + 'static;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享，可存入 Arc |
| `submit` | 接收 `'static + Send` Future | 与 dyn-compatible 兼容 |
| 返回值 | `Result<(), BoxError>` | 提交失败时（如 shutdown）返回错误 |

#### 待补齐（对齐 Spring AsyncTaskExecutor）

| Spring 方法 | vernal-async 方法 | 引入版本 |
|:---|:---|:---|
| `execute(Runnable)` | `submit(F)` | Java 3.0+ |
| `execute(Runnable, long)` | `submit_with_start_timeout` | Java 6.1+ |
| `submit(Runnable)` | `submit()` 已提供 | Java 3.0+ |
| `submit(Callable)` | `submit_callable` | Java 3.0+ |
| `submitListenable` | `submit_listenable` | Java 3.0+（返回 listener 句柄） |
| — | `submit_with_cancellation`（Vernal 独有） | 接受 `CancellationToken` |

#### 为什么返回 `Result<(), BoxError>`？

- Spring 的 `execute()` 返回 void，不关心提交失败。
- 但 Spring 的 `submit()` 返回 `Future`，失败信息在 Future 中。
- Vernal 的 `submit` 在 `tokio::spawn` 失败时（如运行时关闭）应立即返回错误，
  比 Spring 更早暴露问题。

---

### 2.2 AsyncUncaughtExceptionHandler —— 异常回调

**目标**：对标 Spring `AsyncUncaughtExceptionHandler`。

```rust
use std::future::Future;

#[async_trait]
pub trait AsyncUncaughtExceptionHandler: Send + Sync {
    /// 处理未被任务捕获的异常。
    async fn handle_exception(
        &self,
        throwable: BoxError,
        task: &(dyn Any + Send + Sync),
    ) -> Result<(), BoxError>;
}
```

#### 使用模式

```rust
struct LoggingHandler;

#[async_trait]
impl AsyncUncaughtExceptionHandler for LoggingHandler {
    async fn handle_exception(
        &self,
        throwable: BoxError,
        task: &(dyn Any + Send + Sync),
    ) -> Result<(), BoxError> {
        tracing::error!("Async task failed: {} (task: {:?})", throwable, task.type_id());
        Ok(())
    }
}
```

---

### 2.3 AsyncConfigurer —— 配置入口

**目标**：对标 Spring `@EnableAsync` + `AsyncConfigurer`。

```rust
#[async_trait]
pub trait AsyncConfigurer: Send + Sync {
    /// 返回默认执行器
    fn get_async_executor(&self) -> Arc<dyn AsyncTaskExecutor>;

    /// 返回默认异常处理器（可选）
    fn get_async_uncaught_exception_handler(&self) -> Option<Arc<dyn AsyncUncaughtExceptionHandler>> {
        None
    }
}
```

#### 默认实现

```rust
pub struct DefaultAsyncConfigurer;

#[async_trait]
impl AsyncConfigurer for DefaultAsyncConfigurer {
    fn get_async_executor(&self) -> Arc<dyn AsyncTaskExecutor> {
        Arc::new(SimpleAsyncTaskExecutor::new())
    }
}
```

---

## 三、默认实现：SimpleAsyncTaskExecutor

### 3.1 现状

**当前状态**：未实现，仅在文档中提及。

### 3.2 设计目标

对标 Spring `SimpleAsyncTaskExecutor`：每提交一个任务就启动新的执行单元。
Spring 是"每任务一线程"（Thread per task），Rust 改为"`tokio::spawn`"。

### 3.3 结构

```rust
use std::future::Future;
use std::sync::Arc;
use vernal_core::BoxError;
use crate::executor::AsyncTaskExecutor;

pub struct SimpleAsyncTaskExecutor {
    /// tokio runtime handle（None 表示使用 #[tokio::main] 当前运行时）
    handle: Option<tokio::runtime::Handle>,
    /// 异常处理器
    exception_handler: Option<Arc<dyn AsyncUncaughtExceptionHandler>>,
    /// 任务追踪（shutdown 时等待）
    tracker: Arc<tokio_util::task::TaskTracker>,
}

impl SimpleAsyncTaskExecutor {
    /// 创建默认执行器（使用当前运行时）
    pub fn new() -> Self;

    /// 使用指定 runtime
    pub fn with_handle(handle: tokio::runtime::Handle) -> Self;

    /// 设置异常处理器
    pub fn with_exception_handler(mut self, handler: Arc<dyn AsyncUncaughtExceptionHandler>) -> Self;

    /// 启动 graceful shutdown
    pub async fn shutdown(&self, timeout: Duration) -> bool;
}

impl Default for SimpleAsyncTaskExecutor {
    fn default() -> Self { Self::new() }
}

impl AsyncTaskExecutor for SimpleAsyncTaskExecutor {
    fn submit<F>(&self, task: F) -> Result<(), BoxError>
    where
        F: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        let handler = self.exception_handler.clone();
        let tracker = self.tracker.clone();
        let handle = match &self.handle {
            Some(h) => h.clone(),
            None => tokio::runtime::Handle::try_current()
                .map_err(|e| Box::new(e) as BoxError)?,
        };
        tracker.spawn_on(handle, async move {
            match task.await {
                Ok(()) => Ok(()),
                Err(e) => {
                    if let Some(h) = handler {
                        let _ = h.handle_exception(e, &()).await;
                    } else {
                        tracing::error!("Async task failed: {}", e);
                    }
                    Ok(())
                }
            }
        });
        Ok(())
    }
}
```

### 3.4 与 Spring 的语义对照

| 行为 | Spring | vernal-async |
|:---|:---|:---|
| 每任务一执行单元 | `new Thread(task).start()` | `tokio::spawn(task)` |
| 线程池 | 无（每个任务一线程） | 复用 Tokio 调度器（更高效） |
| 任务取消 | `Future.cancel(true)` | `CancellationToken` |
| Shutdown | `executor.shutdown()` | `tracker.close() + wait` |

---

## 四、高级执行器（待补齐）

### 4.1 BoundedAsyncTaskExecutor

**目标**：限制并发任务数，超过阈值时阻塞。

```rust
pub struct BoundedAsyncTaskExecutor {
    inner: Arc<dyn AsyncTaskExecutor>,
    semaphore: Arc<tokio::sync::Semaphore>,
    max_concurrent: usize,
}

impl AsyncTaskExecutor for BoundedAsyncTaskExecutor {
    fn submit<F>(&self, task: F) -> Result<(), BoxError>
    where
        F: Future<Output = Result<(), BoxError>> + Send + 'static,
    {
        let sem = self.semaphore.clone();
        let inner = self.inner.clone();
        self.inner.submit(async move {
            let _permit = sem.acquire().await?;
            inner.submit(async move {
                task.await?;
                Ok(())
            }).await?;
            Ok(())
        })?;
        Ok(())
    }
}
```

### 4.2 ThreadPoolTaskExecutor（可命名线程池）

**目标**：对标 Spring `ThreadPoolTaskExecutor`。

```rust
pub struct ThreadPoolTaskExecutor {
    inner: Arc<tokio::runtime::Runtime>,
    core_pool_size: usize,
    max_pool_size: usize,
    queue_capacity: usize,
    thread_name_prefix: String,
}

impl ThreadPoolTaskExecutor {
    pub fn builder() -> ThreadPoolTaskExecutorBuilder;
}
```

> 注：Tokio 没有"动态扩缩容"线程池概念，此类型仅提供**专用 runtime**。
> 如果只是限流，使用 `BoundedAsyncTaskExecutor` 即可。

### 4.3 ScheduledTaskExecutor

**目标**：对标 Spring `ScheduledTaskExecutor` / `TaskScheduler`。

**不在本 crate**，将由 `vernal-aspects` 的 `#[scheduled]` + tokio Interval 提供。

---

## 五、声明式集成：vernal-aspects

### 5.1 注解清单

| Spring 注解 | vernal 过程宏 | 引入版本 |
|:---|:---|:---|
| `@Async` | `#[async_task]` | Java 3.0+ |
| `@EnableAsync` | `#[enable_async]` | Java 3.1+ |

### 5.2 `#[async_task]` 形态

```rust
#[async_task(
    executor = "my_executor",            // 可选，指定执行器 bean 名称
    timeout_secs = 30,                   // 可选，Future 超时
    exception_handler = "my_handler",    // 可选，异常处理器 bean 名称
)]
pub async fn send_email(&self, to: String, body: String) -> Result<(), BoxError> {
    self.email_client.send(&to, &body).await
}
```

### 5.3 织入位置

```rust
// vernal-aspects/src/async_task.rs（伪代码）
pub struct AsyncTaskAdvice {
    executor: Arc<dyn AsyncTaskExecutor>,
    timeout: Option<Duration>,
    handler: Option<Arc<dyn AsyncUncaughtExceptionHandler>>,
}

#[async_trait]
impl Advice for AsyncTaskAdvice {
    async fn around(&self, ctx: &mut AdviceCtx) -> Result<Arc<dyn Any + Send + Sync>, BoxError> {
        let fut = ctx.proceed();
        let executor = self.executor.clone();
        let timeout = self.timeout;
        let handler = self.handler.clone();
        // 提交到执行器（不等待完成）
        executor.submit(async move {
            let result = if let Some(t) = timeout {
                match tokio::time::timeout(t, fut).await {
                    Ok(r) => r,
                    Err(_) => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::TimedOut, "async task timeout"
                    )) as BoxError),
                }
            } else {
                fut.await
            };
            if let Err(e) = result {
                if let Some(h) = handler {
                    let _ = h.handle_exception(e, &()).await;
                } else {
                    tracing::error!("Async task failed: {}", e);
                }
            }
            Ok(())
        })?;
        // 立即返回"提交成功"，原调用方不阻塞
        Ok(Arc::new(()))
    }
}
```

### 5.4 注意事项

| 项 | 说明 |
|:---|:---|
| 返回类型 | `#[async_task]` 标注的函数必须返回 `Future<Output = Result<(), BoxError>>`，调用方不接收返回值 |
| 异常处理 | 默认走 `tracing::error!`，可注入 `exception_handler` 覆盖 |
| Self 用法 | 不能在 `#[async_task]` 标注的方法内访问 `self` 的非 Send 字段 |
| 上下文传递 | 通过 `task_local!`（vernal-context）传递 traceId / userId |

---

## 六、测试与验证

### 6.1 单元测试（待补齐）

| 测试项 | 目标 |
|:---|:---|
| `simple_executor_submit_executes` | submit 后任务被 tokio 调度执行 |
| `simple_executor_concurrent_100` | 并发 100 任务全部完成 |
| `simple_executor_handler_called_on_error` | 任务 panic 时 handler 被调用 |
| `simple_executor_handler_called_on_result_err` | 任务返回 Err 时 handler 被调用 |
| `bounded_executor_limit_concurrency` | max=10 时最多 10 个并发 |
| `simple_executor_shutdown_waits` | shutdown 等待所有任务完成 |
| `simple_executor_shutdown_timeout` | 超时后返回 false |
| `async_configurer_default` | `DefaultAsyncConfigurer` 返回 `SimpleAsyncTaskExecutor` |

### 6.2 集成测试（待补齐）

```rust
// crates/vernal-async/tests/integration.rs（待创建）
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_async_task_macro_execution() {
    // 1. 创建 SimpleAsyncTaskExecutor
    // 2. 用 #[async_task] 标注函数
    // 3. 验证函数被异步执行
    // 4. 验证 exception_handler 被调用
}
```

### 6.3 性能基准（待补齐）

| 基准 | 目标 |
|:---|:---|
| `bench_submit_overhead` | `submit()` 调用延迟 < 1µs |
| `bench_throughput_1k_tasks` | 1k 任务吞吐量 > 100k/s |
| `bench_bounded_throughput` | 限制 100 并发下吞吐量 |
| `bench_handler_overhead` | 自定义 handler 开销 < 100ns |

### 6.4 编译期验证

- `cargo check -p vernal-async --all-features` 必须通过。
- `cargo clippy -p vernal-async --all-features -- -D warnings` 必须 0 警告。
- `cargo test -p vernal-async` 必须 100% 通过。

---

## 附录 A：与 Spring Async 的完整 API 对照

| Spring 接口 | vernal-async 类型 | 状态 |
|:---|:---|:---|
| `TaskExecutor` | （合并到 `AsyncTaskExecutor`） | ✅ |
| `AsyncTaskExecutor` | `AsyncTaskExecutor` trait | ✅ |
| `TaskScheduler` | 🚫 不在本 crate | ✅（决策） |
| `SchedulingTaskExecutor` | ⏳ 待补齐 | — |
| `AsyncUncaughtExceptionHandler` | `AsyncUncaughtExceptionHandler` trait | ⏳ |
| `AsyncConfigurer` | `AsyncConfigurer` trait | ⏳ |
| `SimpleAsyncTaskExecutor` | `SimpleAsyncTaskExecutor` struct | ⏳ |
| `ThreadPoolTaskExecutor` | `ThreadPoolTaskExecutor` struct | ⏳ |
| `ConcurrentTaskExecutor` | ⏳ 待补齐（包装 tokio Runtime） | — |
| `WorkManagerTaskExecutor` | 🚫 不支持（无 Java EE 等价） | ✅（决策） |
| `@Async` | `#[async_task]` 宏 | ⏳ |
| `@EnableAsync` | `#[enable_async]` 宏 | ⏳ |

## 附录 B：依赖清单

| 依赖 | 版本 | 用途 |
|:---|:---|:---|
| `vernal-core` | path = `../vernal-core` | `BoxError`（已使用） |
| `async-trait` | 0.1（**待集成**） | 异步 trait 支持 |
| `tokio` | 1.52.4（**待集成**） | 异步运行时 + spawn |
| `tokio-util` | 0.7.16（**待集成**） | `TaskTracker` / `CancellationToken` |
| `tracing` | 0.1.41（**待集成**） | 默认异常日志 |
| `vernal-aspects` | path（**待集成**） | `#[async_task]` 过程宏 |

## 附录 C：迁移路线

### C.1 P0（v0.1，骨架完成）

- [x] `AsyncTaskExecutor` trait（最简）
- [ ] `SimpleAsyncTaskExecutor` 实现
- [ ] `AsyncUncaughtExceptionHandler` trait + 默认实现

### C.2 P1（v0.2，扩展 API）

- [ ] `submit_with_start_timeout` / `submit_callable`
- [ ] `submit_with_cancellation`（接受 `CancellationToken`）
- [ ] `AsyncConfigurer` trait + `DefaultAsyncConfigurer`

### C.3 P2（v0.3，专用执行器）

- [ ] `BoundedAsyncTaskExecutor`（信号量限流）
- [ ] `ThreadPoolTaskExecutor`（专用 runtime）
- [ ] `ConcurrentTaskExecutor`（包装 tokio Runtime）

### C.4 P3（v0.4，声明式）

- [ ] `#[async_task]` 过程宏（vernal-aspects）
- [ ] `#[enable_async]` 宏
- [ ] 与 `task_local!` 集成（传递 traceId 等上下文）

### C.5 P4（v1.0，发布）

- [ ] 完整文档 + 示例
- [ ] `cargo doc` 公开 API
- [ ] 100% 测试覆盖

## 附录 D：向后兼容与弃用策略

- 本 crate 处于 v0.x 阶段，**允许 breaking change**。
- `AsyncTaskExecutor` 新增方法必须提供默认实现。
- `SimpleAsyncTaskExecutor` 字段如需扩展，使用 `#[non_exhaustive]` + builder 模式。
- 任何 `pub` 类型重命名需经 vernal-architecture RFC 评审。
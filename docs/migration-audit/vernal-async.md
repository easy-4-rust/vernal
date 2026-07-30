<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-async 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 14 个 class/interface/enum/record；`package-info.java` 不计入 |
| 目录算法 | 去掉组织和模块根包，保留末 2 层包目录 |
| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |
| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |
| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |
| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |
| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |

本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。
<!-- current-migration-contract-end -->

## 汇总

| 指标 | 数量 |
|---|---:|
| Java 业务对象 | 14 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 0 |
| `MISSING` | 14 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 0 |

## 结构红线

- 未发现 `lib.rs`/`mod.rs` 类型定义或生产 wildcard import。

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.core.task.AsyncTaskExecutor` | `AsyncTaskExecutor.java` | `async_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.SimpleAsyncTaskExecutor` | `SimpleAsyncTaskExecutor.java` | `simple_async_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.SyncTaskExecutor` | `SyncTaskExecutor.java` | `sync_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskCallback` | `TaskCallback.java` | `task_callback.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskDecorator` | `TaskDecorator.java` | `task_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskExecutor` | `TaskExecutor.java` | `task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskRejectedException` | `TaskRejectedException.java` | `task_rejected_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.TaskTimeoutException` | `TaskTimeoutException.java` | `task_timeout_exception.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.VirtualThreadDelegate` | `VirtualThreadDelegate.java` | `virtual_thread_delegate.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.VirtualThreadTaskExecutor` | `VirtualThreadTaskExecutor.java` | `virtual_thread_task_executor.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.CompositeTaskDecorator` | `support/CompositeTaskDecorator.java` | `support/composite_task_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.ContextPropagatingTaskDecorator` | `support/ContextPropagatingTaskDecorator.java` | `support/context_propagating_task_decorator.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.ExecutorServiceAdapter` | `support/ExecutorServiceAdapter.java` | `support/executor_service_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.core.task.support.TaskExecutorAdapter` | `support/TaskExecutorAdapter.java` | `support/task_executor_adapter.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |

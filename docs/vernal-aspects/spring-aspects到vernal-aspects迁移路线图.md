<!-- migration-doc: authority=historical canonical=迁移路线图.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [迁移路线图.md](迁移路线图.md) 为准。

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->
# spring-aspects → vernal-aspects 全量迁移路线图

> 版本：v1.1（2026-07-27）｜基线：Spring Framework **7.0.8** spring-aspects
> 仓库：easy-4-rust/vernal @ dev ｜ 底盘：aspect-rs（编译期织入） + vernal-aop（运行期环绕）
> **目录命名 100% 镜像 Spring 5 个 aspectj 子包路径**
> 本文档随代码同步维护，每阶段完成时更新

## 一、总目标

以 Spring Framework 7.0.8 的 spring-aspects 模块为蓝本，实现功能语义完全对齐的 vernal-aspects crate。
**目标 46+ 个 Rust 文件**，覆盖 Spring-aspects 全部 5 大横切关注点（事务 / 缓存 / 异步 / DI / Spring Configured），
**46+ 个单元测试**，4 个核心切面与 Spring 行为 100% 一致。

### 设计原则

1. **目录命名 100% 镜像 Spring 包路径**：Spring `transaction.aspectj` → vernal `transaction/aspectj`，依此类推
2. **一个 .rs 文件只对应一个 Java 对象**（含 `.aj` 等价类），与 liteflow-rust、vernal-expression 规范保持一致
3. **类型名 100% 与 Spring 一致**：`AspectJTransactionManagementConfiguration` 保留全名（**不**简化为 `Config`）
4. **aspect-rs 是织入底盘**：编译期 `#[aspect]` 宏替代 AspectJ LTW，零运行时反射
5. **vernal-aop 是拦截底盘**：运行时环绕拦截，对接 tokio `Future`
6. **公开类型必须有中文文档注释**：结构体、枚举、trait、方法、字段
7. **不实现 AspectJ LTW**：aspect-rs 编译期织入已经覆盖该用例

### vernal-aspects 模块镜像表

| Spring 包 | vernal 模块路径 |
|---|---|
| `org.springframework.transaction.aspectj` | `src/transaction/aspectj/` |
| `org.springframework.cache.aspectj` | `src/cache/aspectj/` |
| `org.springframework.scheduling.aspectj` | `src/scheduling/aspectj/` |
| `org.springframework.beans.factory.aspectj` | `src/beans/factory/aspectj/` |
| `org.springframework.context.annotation.aspectj` | `src/context/annotation/aspectj/` |
| （META-INF + 桥接）| `src/weaver/` + `src/support/` |

### 与 vernal-aspects 当前状态的差距

| 维度 | 当前 | 目标 |
|------|------|------|
| 文件数 | 5（lib + 4 aspects） | **46+** |
| 代码行数 | ~250 | **4000+** |
| 模块数 | 0 | **5 aspectj 子模块 + 2 桥接** |
| 切面抽象层 | 无 | **5**（Transactional/Cacheable/Async/DI/Configured） |
| 注解驱动切面 | 无 | **5**（Annotation 版本） |
| 配置类 | 无 | **5**（AspectJ*Configuration 等价） |
| 配置枚举/struct | 4（Propagation/Isolation/CacheOperation/ScheduleConfig） | **10+** |
| 单元测试 | 0 | **46+** |

---

## 二、进度总览

| 阶段 | 内容 | 状态 | 验收 |
|---|---|---|---|
| S0 | 对象级对照表 + 语义迁移对照表 + 路线图 + 一致性检查 | ✅ | docs/vernal-expression/ 四份 spring-aspects 文档 |
| S1 | 织入机制层（weaver 模块 + aspect-rs 适配器） | ⬜ | aop_xml.rs + advice_kind.rs + pointcut_matcher.rs + aspect_adapter.rs |
| S2 | 事务切面（`transaction/aspectj/` 模块，11 文件） | ⬜ | 7 Propagation + 5 Isolation + Annotation 版本编译通过 + 12+ 测试 |
| S3 | 缓存切面（`cache/aspectj/` 模块，14 文件） | ⬜ | Cacheable/Put/Evict + JCache + 10+ 测试 |
| S4 | 异步切面（`scheduling/aspectj/` 模块，7 文件） | ⬜ | void/Future 派发 + executor 桥接 + 6+ 测试 |
| S5 | 可配置对象切面（`beans/factory/aspectj/` 模块，11 文件） | ⬜ | pre/post-construction 注入 + 反序列化 reattach + 8+ 测试 |
| S6 | Spring Configured 启用（`context/annotation/aspectj/` 模块，2 文件） | ⬜ | enable proc-macro + Configuration 注册 |
| S7 | 集成测试与文档收尾 | ⬜ | 4 份文档 + 中文注释覆盖率 100% |

---

## 三、阶段详细计划

### S1 — 织入机制层（预估 2 天）

**范围**：建立 vernal-aspects 与 aspect-rs / vernal-aop 之间的桥接层

**目标文件**：

| 文件 | Java 类 / 概念 | 工作量 |
|------|---------|--------|
| `weaver/mod.rs` | 模块入口 | 0.1 天 |
| `weaver/aop_xml.rs` | `META-INF/aop.xml` 等价 | 0.5 天 |
| `weaver/advice_kind.rs` | Advice 类型枚举 | 0.25 天 |
| `weaver/pointcut_matcher.rs` | Pointcut 模式匹配 trait | 0.5 天 |
| `support/mod.rs` | 模块入口 | 0.1 天 |
| `support/aspect_adapter.rs` | vernal_aop::Interceptor ↔ aspect_core::Aspect 双向适配 | 0.5 天 |
| `support/async_support.rs` | tokio Future ↔ aspect-rs 异步桥接 | 0.25 天 |

**验收**：
- `AopXml::register_all()` 能列出全部切面（与 Spring aop.xml 一致）
- `AdviceKind` 枚举覆盖 4 种 advice 类型
- `PointcutMatcher::matches(method_meta, pointcut_str)` 能正确解析 `execution(@Tx * *(..))` 等模式
- `AspectAdapter::to_aspect()` 与 `to_interceptor()` 双向转换通过测试

---

### S2 — 事务切面（预估 3 天）

**范围**：`transaction/aspectj/` 模块镜像 `org.springframework.transaction.aspectj` 5 个类 + 7 种 Propagation + 5 种 Isolation + 6 个支撑类

**目标文件**（共 11 个）：

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `transaction/aspectj/mod.rs` | 模块入口 | 0.1 天 |
| `transaction/aspectj/abstract_transaction_aspect.rs` | `AbstractTransactionAspect` | 0.5 天 |
| `transaction/aspectj/annotation_transaction_aspect.rs` | `AnnotationTransactionAspect` | 0.5 天 |
| `transaction/aspectj/jta_annotation_transaction_aspect.rs` | `JtaAnnotationTransactionAspect` | 0.25 天 |
| `transaction/aspectj/aspectj_transaction_management_configuration.rs` | `AspectJTransactionManagementConfiguration` | 0.25 天 |
| `transaction/aspectj/aspectj_jta_transaction_management_configuration.rs` | `AspectJJtaTransactionManagementConfiguration` | 0.25 天 |
| `transaction/aspectj/propagation.rs` | `Propagation` 枚举（7 变体） | 0.25 天 |
| `transaction/aspectj/isolation.rs` | `Isolation` 枚举（5 变体） | 0.25 天 |
| `transaction/aspectj/transaction_attribute.rs` | `TransactionAttribute` 接口 | 0.25 天 |
| `transaction/aspectj/transaction_attribute_source.rs` | `AnnotationTransactionAttributeSource` trait | 0.5 天 |
| `transaction/aspectj/transaction_aspect_support.rs` | `TransactionAspectSupport` 支撑类 | 0.5 天 |
| `transaction/aspectj/rethrower.rs` | `Rethrower` checked 异常透传（仅集成测试用） | 0.25 天 |

**验收**：
- 7 种 Propagation 全部有单元测试（Required / RequiresNew / Nested / Mandatory / Supports / NotSupported / Never）
- 5 种 Isolation 全部有单元测试（Default / ReadUncommitted / ReadCommitted / RepeatableRead / Serializable）
- `rollback_for` / `no_rollback_for` 规则正确
- `TransactionAttribute::default()` 与 Spring 默认值一致
- 嵌套事务保存点行为正确
- 11 个文件全部独立编译通过

---

### S3 — 缓存切面（预估 3 天）

**范围**：`cache/aspectj/` 模块镜像 `org.springframework.cache.aspectj` 5 个类 + 9 个支撑类

**目标文件**（共 14 个）：

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `cache/aspectj/mod.rs` | 模块入口 | 0.1 天 |
| `cache/aspectj/abstract_cache_aspect.rs` | `AbstractCacheAspect` | 0.5 天 |
| `cache/aspectj/annotation_cache_aspect.rs` | `AnnotationCacheAspect` | 0.5 天 |
| `cache/aspectj/jcache_cache_aspect.rs` | `JCacheCacheAspect`（feature: `jcache`） | 0.5 天 |
| `cache/aspectj/aspectj_caching_configuration.rs` | `AspectJCachingConfiguration` | 0.25 天 |
| `cache/aspectj/aspectj_jcache_configuration.rs` | `AspectJJCacheConfiguration` | 0.25 天 |
| `cache/aspectj/any_throw.rs` | `AnyThrow` checked 异常透传（仅集成测试用） | 0.25 天 |
| `cache/aspectj/cache_operation.rs` | `CacheOperation` 枚举（Cacheable/Put/Evict） | 0.25 天 |
| `cache/aspectj/cache_operation_source.rs` | `AnnotationCacheOperationSource` trait | 0.5 天 |
| `cache/aspectj/cache_operation_invoker.rs` | `CacheOperationInvoker` trait | 0.25 天 |
| `cache/aspectj/cache_aspect_support.rs` | `CacheAspectSupport` 支撑类 | 0.5 天 |
| `cache/aspectj/cache_interceptor.rs` | `CacheInterceptor` | 0.25 天 |
| `cache/aspectj/jcache_aspect_support.rs` | `JCacheAspectSupport` | 0.5 天 |
| `cache/aspectj/cache_config.rs` | `CacheConfig`（Rust 独立 struct） | 0.5 天 |

**验收**：
- Cacheable 命中 / 未命中分支有测试
- CachePut 总是更新缓存测试
- CacheEvict `beforeInvocation` 顺序测试
- `allEntries` 批量驱逐测试
- 复合 `@Caching` 多操作测试
- 错误处理器 `errorHandler` 调用测试
- 14 个文件全部独立编译通过

---

### S4 — 异步切面（预估 2 天）

**范围**：`scheduling/aspectj/` 模块镜像 `org.springframework.scheduling.aspectj` 3 个类 + 4 个支撑类

**目标文件**（共 7 个）：

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `scheduling/aspectj/mod.rs` | 模块入口 | 0.1 天 |
| `scheduling/aspectj/abstract_async_execution_aspect.rs` | `AbstractAsyncExecutionAspect` | 0.5 天 |
| `scheduling/aspectj/annotation_async_execution_aspect.rs` | `AnnotationAsyncExecutionAspect`（含 return type 校验） | 0.5 天 |
| `scheduling/aspectj/aspectj_async_configuration.rs` | `AspectJAsyncConfiguration` | 0.25 天 |
| `scheduling/aspectj/async_execution_aspect_support.rs` | `AsyncExecutionAspectSupport` 支撑类 | 0.5 天 |
| `scheduling/aspectj/async_task_executor.rs` | `AsyncTaskExecutor` trait | 0.25 天 |
| `scheduling/aspectj/async_uncaught_exception_handler.rs` | `AsyncUncaughtExceptionHandler` trait | 0.25 天 |
| `scheduling/aspectj/async_annotation_beans.rs` | `@Async` 注解定义 | 0.25 天 |

**验收**：
- `void` 返回类型的 `@Async` 方法 fire-and-forget 测试
- `Future` / `CompletableFuture` 返回类型测试
- `getExecutorQualifier` 从 `@Async#value()` 解析测试
- executor 派发测试（默认 vs 命名 executor）
- 异步异常处理器调用测试
- 7 个文件全部独立编译通过

---

### S5 — 可配置对象切面（预估 3 天）

**范围**：`beans/factory/aspectj/` 模块镜像 `org.springframework.beans.factory.aspectj` 5 个类 + 4 个支撑类 + 1 个嵌套接口

**目标文件**（共 11 个）：

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `beans/factory/aspectj/mod.rs` | 模块入口 | 0.1 天 |
| `beans/factory/aspectj/configurable_object.rs` | `ConfigurableObject` marker trait | 0.25 天 |
| `beans/factory/aspectj/abstract_dependency_injection_aspect.rs` | `AbstractDependencyInjectionAspect` | 0.5 天 |
| `beans/factory/aspectj/abstract_interface_driven_dependency_injection_aspect.rs` | `AbstractInterfaceDrivenDependencyInjectionAspect` | 0.5 天 |
| `beans/factory/aspectj/annotation_bean_configurer_aspect.rs` | `AnnotationBeanConfigurerAspect` | 0.5 天 |
| `beans/factory/aspectj/generic_interface_driven_dependency_injection_aspect.rs` | `GenericInterfaceDrivenDependencyInjectionAspect<I>` | 0.25 天 |
| `beans/factory/aspectj/configurable_deserialization_support.rs` | `ConfigurableDeserializationSupport`（嵌套接口，独立文件） | 0.25 天 |
| `beans/factory/aspectj/bean_configurer_support.rs` | `BeanConfigurerSupport` | 0.5 天 |
| `beans/factory/aspectj/bean_wiring_info.rs` | `BeanWiringInfo` | 0.25 天 |
| `beans/factory/aspectj/bean_wiring_info_resolver.rs` | `AnnotationBeanWiringInfoResolver` | 0.5 天 |

**验收**：
- pre-construction 注入顺序测试（leastSpecificSuperTypeConstruction）
- post-construction 注入测试（mostSpecificSubTypeConstruction）
- 反序列化 reattach 测试（readResolve）
- `@Configurable` 注解驱动测试
- 泛型 `configure(I)` 类型安全派发测试
- `BeanFactoryAware` / `InitializingBean` / `DisposableBean` 生命周期测试
- 11 个文件全部独立编译通过

---

### S6 — Spring Configured 启用（预估 1 天）

**范围**：`context/annotation/aspectj/` 模块镜像 `org.springframework.context.annotation.aspectj` 2 个类 + proc-macro

**目标文件**（共 2 个 + 1 proc-macro）：

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `context/annotation/aspectj/mod.rs` | 模块入口 | 0.1 天 |
| `context/annotation/aspectj/enable_spring_configured.rs` | `EnableSpringConfigured` proc-macro | 0.5 天 |
| `context/annotation/aspectj/spring_configured_configuration.rs` | `SpringConfiguredConfiguration` | 0.25 天 |

**验收**：
- `#[enable_spring_configured]` 宏能自动注册 `SpringConfiguredConfiguration`
- `BEAN_CONFIGURER_ASPECT_BEAN_NAME` 常量命名对齐（vernal 化前缀）
- 与 `vernal-context` 的 `Configuration` trait 集成测试
- 2 个文件全部独立编译通过

---

### S7 — 集成测试与文档收尾（预估 1 天）

**范围**：跨模块集成测试 + 中文文档收尾

**目标文件**：

| 文件 | 说明 | 工作量 |
|------|------|--------|
| `support/tests/transactional_tests.rs` | 端到端事务场景 | 0.25 天 |
| `support/tests/cacheable_tests.rs` | 端到端缓存场景 | 0.25 天 |
| `support/tests/async_tests.rs` | 端到端异步场景 | 0.25 天 |
| `support/tests/configurable_tests.rs` | 端到端 DI 场景 | 0.25 天 |

**验收**：
- 4 个端到端测试场景全部通过
- 所有公开类型中文 doc 注释覆盖率 100%
- `lib.rs` 文档对齐 `vernal-expression` 的 `lib.rs` 风格
- README 更新（如有）

---

## 四、工程规范

### 命名规则

| 项目 | 规范 |
|------|------|
| 目录命名 | **100% 镜像 Spring 包路径**（如 `transaction/aspectj/`、`beans/factory/aspectj/`）|
| 目录/文件名 | snake_case（`abstract_transaction_aspect.rs`、`annotation_transaction_aspect.rs`）|
| 类型名 | **PascalCase 与 Spring 完全一致**（`AspectJTransactionManagementConfiguration` 保留全名）|
| 方法名 | snake_case（`invoke_within_transaction`、`determine_async_executor`）|
| 切面后缀 | `Aspect`（与 Spring `*Aspect` 对齐）|
| trait | `trait` 关键字，与 Spring `interface` 对齐 |
| 枚举 | 与 Spring 一致：`Propagation`、`Isolation`、`CacheOperation` |

### 文件结构规范

```rust
//! 对应 Java 类：org.springframework.transaction.aspectj.AbstractTransactionAspect
//!
//! 事务切面抽象层：封装 TransactionAspectSupport 的 invokeWithinTransaction 行为，
//! 由子类实现 transactionalMethodExecution pointcut。
//!
//! 对应 Spring 的织入位置：spring-aspects/.../transaction/aspectj/AbstractTransactionAspect.aj

use std::sync::Arc;

use aspect_core::{Aspect, AspectError, JoinPoint, ProceedingJoinPoint};
use vernal_aop::{Invocation, InvocationFuture, Next};

/// 事务切面抽象基类。
///
/// 对应 Spring 的 `AbstractTransactionAspect`。
pub struct AbstractTransactionAspect<T: TransactionAttributeSource> {
    attribute_source: Arc<T>,
}

impl<T: TransactionAttributeSource + 'static> AbstractTransactionAspect<T> {
    /// 创建抽象切面。
    pub fn new(attribute_source: Arc<T>) -> Self {
        Self { attribute_source }
    }
}
```

### 测试规范

- 每个 trait 至少 1 个 mock 实现测试
- 每个 Aspect 至少 1 个真实织入测试（用 aspect-rs 的 `#[aspect]` 宏）
- 端到端场景测试：transactional/cacheable/async/configurable 各 1 个

---

## 五、风险评估

| 风险 | 影响 | 对策 |
|------|------|------|
| AspectJ ITD（declare parents）无法直接迁移 | `ConfigurableObject` 标记接口的强制实现 | trait + blanket impl 替代 |
| AspectJ `declare error` 无法直接迁移 | `@Async` 返回类型校验 | trait bound + `static_assertions` |
| `Rethrower` / `AnyThrow` checked 异常透传 | AspectJ around advice 的限制 | Rust 无对应概念，保留为可选辅助层 |
| aspect-rs 编译期织入 vs vernal-aop 运行期拦截 | 两套 AOP 抽象需并存 | `support/aspect_adapter.rs` 双向适配 |
| `pointcut` 模式解析复杂度 | AspectJ 语法丰富 | 仅实现 vernal-aspects 实际使用的子集 |
| 中文注释工作量大 | 交付延迟 | 分阶段进行，优先核心切面 |
| 包路径 4 层深度（`beans/factory/aspectj/`） | 模块嵌套深，路径冗长 | 接受此开销换取与 Spring 100% 镜像 |

---

## 六、预期成果

完成全部 7 个阶段后，vernal-aspects 将成为 **Rust 生态最完整的 Spring-aspects 等价实现**：

| 维度 | 当前 | 完成后 |
|------|------|--------|
| 文件数 | 5 | **46+** |
| 代码行数 | ~250 | **4000+** |
| 镜像的 Spring 包数 | 0 | **5**（100% 镜像） |
| 切面抽象层 | 4（占位） | **5**（全部完成）|
| 注解驱动切面 | 0 | **5** |
| 配置类 | 0 | **5**（保留 `Configuration` 全名） |
| 单元测试 | 0 | **46+** |
| 与 Spring 7.x 行为一致度 | 10%（仅骨架） | **90%+** |
| 与 vernal-aop / aspect-rs 双轨集成 | 无 | **完整** |
| 与 vernal-context（IoC）集成 | 无 | **完整** |

### 与 spring-framework 7.0.8 spring-aspects 的对标度

| 维度 | 完成前 | 完成后 |
|------|------|--------|
| 类覆盖 | 4/21 = 19% | **21/21 = 100%**（按对象级对照表） |
| 包路径镜像 | 0/5 = 0% | **5/5 = 100%**（5 个 aspectj 子包全部镜像） |
| 切面织入 | 0/6 = 0% | **6/6 = 100%**（Annotation/JTA/JCache/Async/Configurer/SpringConfigured） |
| Pointcut 解析 | 0/15+ = 0% | **15+ pointcut 全部解析** |
| 单元测试 | 0 | **46+** |

### 与 vernal-expression 的对标度（对比）

| 维度 | vernal-expression | vernal-aspects |
|---|---|---|
| 文件数 | 4 → 87+（目标） | 5 → 46+（目标）|
| 镜像 Spring 包 | 4 个（`expression`、`spel`、`common` 等）| **5 个 aspectj 子包** |
| 一致性规范 | 87+ 文件 / 4564 行 / 96 测试 | **46+ 文件 / 4000+ 行 / 46+ 测试** |
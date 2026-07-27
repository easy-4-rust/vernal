# spring-aspects → vernal-aspects 对象级对照表（验收清单）

> 基线：Spring Framework **7.0.8** spring-aspects 模块，共 **21 个 Java 类**（含 1 个 `package-info` 与 1 个 `aop.xml`）。
> 现状基线：vernal-aspects 已有 **4 个核心切面**（`TransactionalAspect` / `CacheableAspect` / `AsyncAspect` / `ScheduledAspect`），均为占位骨架。
> 本表回答"**每个 Spring aspect 对象落在哪个 vernal-aspects 文件**"，是补缺与拆分的验收清单。

## 命名对齐原则（100% 对齐 spring-aspects）

### 目录命名规则

Spring 包路径最后一层目录（含 `aspectj` 子包）必须**完全镜像**到 vernal-aspects：

| Spring 包 | vernal-aspects 目录 | 备注 |
|---|---|---|
| `org.springframework.transaction.aspectj` | `transaction/aspectj/` | 完全镜像 |
| `org.springframework.cache.aspectj` | `cache/aspectj/` | 完全镜像 |
| `org.springframework.scheduling.aspectj` | `scheduling/aspectj/` | 完全镜像 |
| `org.springframework.beans.factory.aspectj` | `beans/factory/aspectj/` | 完全镜像 |
| `org.springframework.context.annotation.aspectj` | `context/annotation/aspectj/` | 完全镜像 |

**镜像规则**：
- Spring 包的最后两层目录（`X.aspectj` 或 `X.Y.aspectj`）作为 vernal-aspects 的目录路径
- 同一 Java 包内的所有类放同一目录，**不**再二次按业务语义拆分子模块
- 包名保持 `snake_case`（与 Spring 全小写一致），不再做"业务词根"改写

### 文件命名规则

| Spring 文件后缀 | vernal-aspects 文件命名 |
|---|---|
| `*.aj`（AspectJ 切面）| `{ClassName}.rs`（去 `.aj` 后缀，加 `.rs`）|
| `*.java`（普通 Java 类）| `{ClassName}.rs`（加 `.rs` 后缀）|
| `META-INF/aop.xml`（资源）| `aop_xml.rs`（放在 `weaver/` 模块下，Rust 等价物）|

### 类型命名规则

- Spring `aspect` 类型 → vernal `struct`（不是 `trait`），保留 `Aspect` 后缀
- Spring `interface` 类型 → vernal `trait`，保留原名
- Spring `enum` 类型 → vernal `enum`，保留原名
- Spring `Configuration` 后缀 → vernal 保留 `Configuration` 后缀（**不**简化为 `Config`），保持与 Spring 完全一致

### 方法命名规则

- Java `camelCase` → Rust `snake_case`（Rust 惯用法，不可避免）
- 字段名同理
- 类型名 `PascalCase` → Rust 仍 `PascalCase`（与 Java 一致）

---

## 目标工程结构（Cargo crate）

```
vernal-framework/crates/vernal-aspects/
├── Cargo.toml                                  # crate 元数据
└── src/
    ├── lib.rs                                  # crate 门面（mod 声明 + re-export）
    │
    ├── transaction/                            # 对标 org.springframework.transaction.aspectj
    │   └── aspectj/                            # 镜像 Spring 包路径的 aspectj 子包
    │       ├── mod.rs                          # 模块入口（只 mod 声明 + re-export）
    │       ├── abstract_transaction_aspect.rs  # AbstractTransactionAspect.aj
    │       ├── annotation_transaction_aspect.rs# AnnotationTransactionAspect.aj
    │       ├── jta_annotation_transaction_aspect.rs # JtaAnnotationTransactionAspect.aj
    │       ├── aspectj_transaction_management_configuration.rs # AspectJTransactionManagementConfiguration.java
    │       ├── aspectj_jta_transaction_management_configuration.rs # AspectJJtaTransactionManagementConfiguration.java
    │       ├── propagation.rs                  # Propagation 枚举（Spring 同名）
    │       ├── isolation.rs                    # Isolation 枚举（Spring 同名）
    │       ├── transaction_attribute.rs        # TransactionAttribute 接口
    │       ├── transaction_attribute_source.rs # AnnotationTransactionAttributeSource
    │       ├── rethrower.rs                    # Rethrower checked 异常透传
    │       └── transaction_aspect_support.rs   # TransactionAspectSupport 支撑类
    │
    ├── cache/                                  # 对标 org.springframework.cache.aspectj
    │   └── aspectj/                            # 镜像 Spring 包路径的 aspectj 子包
    │       ├── mod.rs
    │       ├── abstract_cache_aspect.rs        # AbstractCacheAspect.aj
    │       ├── annotation_cache_aspect.rs      # AnnotationCacheAspect.aj
    │       ├── jcache_cache_aspect.rs          # JCacheCacheAspect.aj
    │       ├── aspectj_caching_configuration.rs       # AspectJCachingConfiguration.java
    │       ├── aspectj_jcache_configuration.rs        # AspectJJCacheConfiguration.java
    │       ├── any_throw.rs                    # AnyThrow.java
    │       ├── cache_operation.rs              # CacheOperation 枚举（Cacheable/Put/Evict）
    │       ├── cache_config.rs                 # CacheConfig（Rust 独立 struct）
    │       ├── cache_operation_source.rs       # AnnotationCacheOperationSource
    │       ├── cache_operation_invoker.rs      # CacheOperationInvoker 接口
    │       ├── cache_aspect_support.rs         # CacheAspectSupport 支撑类
    │       ├── cache_interceptor.rs            # CacheInterceptor 拦截器
    │       └── jcache_aspect_support.rs        # JCacheAspectSupport 支撑类
    │
    ├── scheduling/                             # 对标 org.springframework.scheduling.aspectj
    │   └── aspectj/                            # 镜像 Spring 包路径的 aspectj 子包
    │       ├── mod.rs
    │       ├── abstract_async_execution_aspect.rs # AbstractAsyncExecutionAspect.aj
    │       ├── annotation_async_execution_aspect.rs # AnnotationAsyncExecutionAspect.aj
    │       ├── aspectj_async_configuration.rs  # AspectJAsyncConfiguration.java
    │       ├── async_execution_aspect_support.rs # AsyncExecutionAspectSupport 支撑类
    │       ├── async_task_executor.rs          # AsyncTaskExecutor trait
    │       ├── async_uncaught_exception_handler.rs # AsyncUncaughtExceptionHandler trait
    │       └── async_annotation_beans.rs       # Async 注解元数据（@Async）
    │
    ├── beans/                                  # 对标 org.springframework.beans.factory.aspectj
    │   └── factory/                            # 镜像 Spring beans.factory 中间包
    │       └── aspectj/                        # 镜像 Spring 包路径的 aspectj 子包
    │           ├── mod.rs
    │           ├── configurable_object.rs          # ConfigurableObject.java 标记接口
    │           ├── abstract_dependency_injection_aspect.rs # AbstractDependencyInjectionAspect.aj
    │           ├── abstract_interface_driven_dependency_injection_aspect.rs # AbstractInterfaceDrivenDependencyInjectionAspect.aj
    │           ├── annotation_bean_configurer_aspect.rs # AnnotationBeanConfigurerAspect.aj
    │           ├── generic_interface_driven_dependency_injection_aspect.rs # GenericInterfaceDrivenDependencyInjectionAspect.aj
    │           ├── bean_configurer_support.rs      # BeanConfigurerSupport
    │           ├── bean_wiring_info.rs             # BeanWiringInfo
    │           ├── bean_wiring_info_resolver.rs    # AnnotationBeanWiringInfoResolver
    │           └── configurable_deserialization_support.rs # ConfigurableDeserializationSupport 嵌套接口
    │
    ├── context/                                # 对标 org.springframework.context.annotation.aspectj
    │   └── annotation/                         # 镜像 Spring context.annotation 中间包
    │       └── aspectj/                        # 镜像 Spring 包路径的 aspectj 子包
    │           ├── mod.rs
    │           ├── enable_spring_configured.rs # EnableSpringConfigured.java 注解
    │           └── spring_configured_configuration.rs # SpringConfiguredConfiguration.java
    │
    ├── weaver/                                 # vernal 特有：aop.xml + 织入机制桥接
    │   ├── mod.rs
    │   ├── aop_xml.rs                          # META-INF/aop.xml 等价物
    │   ├── advice_kind.rs                      # 4 类 Advice 类型枚举（Before/After/Around/AfterError）
    │   └── pointcut_matcher.rs                 # Pointcut 模式匹配（trait 入口）
    │
    └── support/                                # vernal 特有：bridge 到 aspect-rs / vernal-aop
        ├── mod.rs
        ├── aspect_adapter.rs                   # vernal_aop::Interceptor → aspect_core::Aspect 适配
        └── async_support.rs                    # Async 适配（tokio future）
```

**重要约定**：
- 每个 Spring Java 类（除 `package-info.java` 外）→ 一个同名 Rust 文件
- `.aj` 后缀文件 → `.rs`（去除 `.aj`）
- 嵌套内部接口（如 `ConfigurableDeserializationSupport`）也拆为独立 `.rs` 文件，保留 `outer::Inner` 的归属语义（在本表的归属列注明）

---

## 命名与组织规则

1. **目录命名 100% 镜像 Spring 包路径**：Spring `transaction.aspectj` → vernal `transaction/aspectj`，依此类推
2. **目录、文件名一律 snake_case**；类型 PascalCase；方法 snake_case
3. **每个 .rs 文件只对应一个 Java 对象**（含 AspectJ 切面与 .aj 等价类）
4. `mod.rs` 只做模块声明与 re-export，**禁止定义任何类型/逻辑**
5. 每个文件头部必须有中文 doc 注释：说明对应 Java 类的全限定名、核心职责、所在包
6. 方法级中文注释从 Java 源码同步翻译
7. **配置类保留 `Configuration` 后缀**（如 `AspectJTransactionManagementConfiguration`），不简化为 `Config`

## 状态图例

| 标记 | 含义 |
|---|---|
| ✅ | 已迁移，独立文件已对齐 |
| 🔀 | 语义已迁移但与其他对象合并在同一文件，**待拆分** |
| ⬜ | 未迁移，缺失 |
| 🔶 | 语义等价但形态不同；文件保留，内部记录差异 |
| 🚫 | 不迁移（Java 生态特有），说明栏给出 Rust 化替代或理由 |
| 🆕 | Rust 侧新增、无 Java 对偶 |

## 统计汇总（21 类）

| 状态 | 数量 | 占比 |
|---|---|---|
| ✅ 已对齐 | 4 | 19% |
| 🔀 合并未拆 | 0 | 0% |
| ⬜ 待迁移 | 15 | 71% |
| 🔶 形态不同 | 1 | 5% |
| 🚫 不迁移 | 1 | 5% |
| **合计** | **21** | 100% |

---

## 1. 事务切面（`org.springframework.transaction.aspectj`）（5 类）

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AbstractTransactionAspect` | `transaction/aspectj/abstract_transaction_aspect.rs` | ⬜ | 事务切面抽象层，封装 `TransactionAspectSupport` 行为 |
| `AnnotationTransactionAspect` | `transaction/aspectj/annotation_transaction_aspect.rs` | ⬜ | Spring `@Transactional` 注解驱动的具体切面 |
| `JtaAnnotationTransactionAspect` | `transaction/aspectj/jta_annotation_transaction_aspect.rs` | ⬜ | JTA 1.2 `jakarta.transaction.Transactional` 注解驱动 |
| `AspectJTransactionManagementConfiguration` | `transaction/aspectj/aspectj_transaction_management_configuration.rs` | ⬜ | 注册 AnnotationTransactionAspect Bean 的 @Configuration |
| `AspectJJtaTransactionManagementConfiguration` | `transaction/aspectj/aspectj_jta_transaction_management_configuration.rs` | ⬜ | JTA 版 @Configuration，注册 JtaAnnotationTransactionAspect |

### 1.1 事务切面相关支撑类（与本切面在同一包下，**不另起模块**）

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `Propagation`（`org.springframework.transaction.annotation.Propagation`，spring-tx 模块） | `transaction/aspectj/propagation.rs` | ⬜ | 7 种传播行为；与 Spring 同名 |
| `Isolation`（`org.springframework.transaction.annotation.Isolation`，spring-tx 模块） | `transaction/aspectj/isolation.rs` | ⬜ | 5 种隔离级别 |
| `TransactionAttribute`（spring-tx） | `transaction/aspectj/transaction_attribute.rs` | ⬜ | 事务属性接口 |
| `AnnotationTransactionAttributeSource`（spring-tx） | `transaction/aspectj/transaction_attribute_source.rs` | ⬜ | 基于注解的事务属性源 |
| `TransactionAspectSupport`（spring-tx） | `transaction/aspectj/transaction_aspect_support.rs` | ⬜ | 事务切面支撑基类 |
| `Rethrower`（本包私有） | `transaction/aspectj/rethrower.rs` | ⬜ | checked 异常透传 |

> 注：`Propagation` / `Isolation` / `TransactionAttribute` / `AnnotationTransactionAttributeSource` / `TransactionAspectSupport` 物理上位于 spring-tx 模块，但在 spring-aspects 切面代码中被直接依赖。vernal-aspects 必须在 crate 内提供等价定义（因为 vernal-tx crate 还在演化中），故放在 `transaction/aspectj/` 同包下。

---

## 2. 缓存切面（`org.springframework.cache.aspectj`）（5 类）

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AbstractCacheAspect` | `cache/aspectj/abstract_cache_aspect.rs` | ⬜ | 缓存切面抽象层，封装 `CacheAspectSupport` 行为 |
| `AnnotationCacheAspect` | `cache/aspectj/annotation_cache_aspect.rs` | ⬜ | `@Cacheable` / `@CachePut` / `@CacheEvict` / `@Caching` 注解驱动 |
| `JCacheCacheAspect` | `cache/aspectj/jcache_cache_aspect.rs` | ⬜ | JSR-107 注解（`@CacheResult` / `@CachePut` 等）驱动 |
| `AspectJCachingConfiguration` | `cache/aspectj/aspectj_caching_configuration.rs` | ⬜ | 注册 AnnotationCacheAspect Bean 的 @Configuration |
| `AspectJJCacheConfiguration` | `cache/aspectj/aspectj_jcache_configuration.rs` | ⬜ | JCache 版 @Configuration，注册 JCacheCacheAspect |

### 2.1 缓存切面相关支撑类

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AnyThrow`（本包私有） | `cache/aspectj/any_throw.rs` | ⬜ | checked 异常透传工具 |
| `CacheOperation`（`org.springframework.cache.annotation.CacheOperation`，spring-context） | `cache/aspectj/cache_operation.rs` | ⬜ | 缓存操作元数据 |
| `AnnotationCacheOperationSource`（spring-context） | `cache/aspectj/cache_operation_source.rs` | ⬜ | 基于注解的缓存操作源 |
| `CacheOperationInvoker`（spring-context） | `cache/aspectj/cache_operation_invoker.rs` | ⬜ | 缓存操作调用器接口 |
| `CacheAspectSupport`（spring-context） | `cache/aspectj/cache_aspect_support.rs` | ⬜ | 缓存切面支撑基类 |
| `CacheInterceptor`（spring-context） | `cache/aspectj/cache_interceptor.rs` | ⬜ | 缓存拦截器（MethodInterceptor） |
| `JCacheAspectSupport`（`org.springframework.cache.jcache.interceptor.JCacheAspectSupport`，spring-context） | `cache/aspectj/jcache_aspect_support.rs` | ⬜ | JCache 切面支撑 |

> 同样地：`Cache*` 系列类物理上属于 spring-context，vernal-aspects 在 crate 内提供等价定义以保证切面模块独立编译。

---

## 3. 异步切面（`org.springframework.scheduling.aspectj`）（3 类）

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AbstractAsyncExecutionAspect` | `scheduling/aspectj/abstract_async_execution_aspect.rs` | ⬜ | 异步切面抽象层，封装 `AsyncExecutionAspectSupport` 行为 |
| `AnnotationAsyncExecutionAspect` | `scheduling/aspectj/annotation_async_execution_aspect.rs` | ⬜ | `@Async` 注解驱动的具体切面（含编译期 `declare error` 校验） |
| `AspectJAsyncConfiguration` | `scheduling/aspectj/aspectj_async_configuration.rs` | ⬜ | 注册 AnnotationAsyncExecutionAspect Bean 的 @Configuration |

### 3.1 异步切面相关支撑类

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AsyncExecutionAspectSupport`（`org.springframework.aop.interceptor.AsyncExecutionAspectSupport`，spring-aop） | `scheduling/aspectj/async_execution_aspect_support.rs` | ⬜ | 异步切面支撑基类 |
| `AsyncTaskExecutor`（`org.springframework.core.task.AsyncTaskExecutor`，spring-core） | `scheduling/aspectj/async_task_executor.rs` | ⬜ | 异步任务执行器接口 |
| `AsyncUncaughtExceptionHandler`（`org.springframework.aop.interceptor.AsyncUncaughtExceptionHandler`，spring-aop） | `scheduling/aspectj/async_uncaught_exception_handler.rs` | ⬜ | 异步异常处理器接口 |
| `Async` 注解（`org.springframework.scheduling.annotation.Async`） | `scheduling/aspectj/async_annotation_beans.rs` | ⬜ | `@Async` 注解定义 |

---

## 4. 可配置对象切面（`org.springframework.beans.factory.aspectj`）（5 类）

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AbstractDependencyInjectionAspect` | `beans/factory/aspectj/abstract_dependency_injection_aspect.rs` | ⬜ | DI 切面抽象层，定义 pre/post-construction 与 deserialization advice |
| `AbstractInterfaceDrivenDependencyInjectionAspect` | `beans/factory/aspectj/abstract_interface_driven_dependency_injection_aspect.rs` | ⬜ | 接口驱动的 DI 切面（实现 `ConfigurableObject` 的对象） |
| `AnnotationBeanConfigurerAspect` | `beans/factory/aspectj/annotation_bean_configurer_aspect.rs` | ⬜ | `@Configurable` 注解驱动的具体切面 |
| `GenericInterfaceDrivenDependencyInjectionAspect` | `beans/factory/aspectj/generic_interface_driven_dependency_injection_aspect.rs` | ⬜ | 泛型驱动的 DI 切面（类型安全的 configure 方法） |
| `ConfigurableObject` | `beans/factory/aspectj/configurable_object.rs` | ⬜ | 标记接口，标识需要 DI 注入的对象 |

### 4.1 嵌套接口

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `ConfigurableDeserializationSupport`（`AbstractInterfaceDrivenDependencyInjectionAspect` 内嵌套 static interface） | `beans/factory/aspectj/configurable_deserialization_support.rs` | ⬜ | 反序列化支持标记接口；归属 `AbstractInterfaceDrivenDependencyInjectionAspect` 内 |

### 4.2 可配置对象切面相关支撑类

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `BeanConfigurerSupport`（`org.springframework.beans.factory.wiring.BeanConfigurerSupport`，spring-beans） | `beans/factory/aspectj/bean_configurer_support.rs` | ⬜ | Bean 配置支撑 |
| `BeanWiringInfo`（`org.springframework.beans.factory.wiring.BeanWiringInfo`，spring-beans） | `beans/factory/aspectj/bean_wiring_info.rs` | ⬜ | Bean 装配信息 |
| `AnnotationBeanWiringInfoResolver`（`org.springframework.beans.factory.annotation.AnnotationBeanWiringInfoResolver`，spring-beans） | `beans/factory/aspectj/bean_wiring_info_resolver.rs` | ⬜ | 基于注解的 Bean 装配信息解析器 |

---

## 5. Spring Configured 启用（`org.springframework.context.annotation.aspectj`）（2 类）

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `EnableSpringConfigured` | `context/annotation/aspectj/enable_spring_configured.rs` | ⬜ | 启用注解（proc-macro attribute），引入 `SpringConfiguredConfiguration` |
| `SpringConfiguredConfiguration` | `context/annotation/aspectj/spring_configured_configuration.rs` | ⬜ | `@Configuration`，注册 `AnnotationBeanConfigurerAspect` 单例 |

---

## 6. 资源（resources）（1 类）

| 资源 | 目标位置 | 状态 | 说明 |
|---|---|---|---|
| `META-INF/aop.xml` | `weaver/aop_xml.rs` | 🆕 | AspectJ LTW 配置；Vernal 用 `weaver/aop_xml.rs` + `build.rs` 注册；aspect-rs 不需要 XML，由 `#[aspect]` 宏在编译期织入 |

---

## Rust 侧新增、无 Java 对偶（🆕）

| Rust 文件 | 承载的 Java 语义 | 处置建议 |
|---|---|---|
| `weaver/aop_xml.rs` | AspectJ LTW 配置清单 | 替代 Spring 的 `META-INF/aop.xml`，用 Rust struct + `build.rs` 注册 |
| `weaver/advice_kind.rs` | Advice 类型枚举（Before/After/Around/AfterError） | aspect-rs 的 `Aspect` trait 已经涵盖；本枚举作为 vernal-aspects 内部转发到 `aspect_core::Aspect` 时的元数据 |
| `weaver/pointcut_matcher.rs` | Pointcut 模式匹配 | aspect-rs 的 `Pointcut::parse` 已实现，vernal-aspects 直接复用 |
| `support/aspect_adapter.rs` | `vernal_aop::Interceptor` → `aspect_core::Aspect` 适配 | vernal-aop 当前用 `Interceptor` + `Next` 模型，aspect-rs 用 `Aspect` + `ProceedingJoinPoint`；本适配器让 vernal-aspects 既能跑在 vernal-aop 上也能跑在 aspect-rs 上 |
| `support/async_support.rs` | tokio `Future` → `InvocationFuture` 桥接 | 解决两个 AOP 框架对异步语义的不同抽象 |

---

## 测试基线

每个新文件至少 1 个单元测试。核心场景：
- `transaction/aspectj/abstract_transaction_aspect.rs`：7 种 Propagation 行为正确；rollback_for 规则
- `transaction/aspectj/annotation_transaction_aspect.rs`：从函数属性读取 `@Transactional` 配置
- `cache/aspectj/annotation_cache_aspect.rs`：`@Cacheable` 命中/未命中分支
- `cache/aspectj/jcache_cache_aspect.rs`：JSR-107 注解映射
- `scheduling/aspectj/annotation_async_execution_aspect.rs`：返回 `void`/`Future` 的方法识别
- `beans/factory/aspectj/annotation_bean_configurer_aspect.rs`：pre/post-construction 注入顺序
- `beans/factory/aspectj/generic_interface_driven_dependency_injection_aspect.rs`：泛型 configure 派发
- `weaver/pointcut_matcher.rs`：`execution(@Tx * *(..))` 模式解析

---

## 验收清单（合并 4 类文档的最终清单）

1. ✅ 21 个 Spring 类全部落到独立 .rs 文件（合并 0、缺失 0）
2. ✅ 5 个 vernal 子模块目录完全镜像 Spring 5 个包路径（`transaction/aspectj/`、`cache/aspectj/`、`scheduling/aspectj/`、`beans/factory/aspectj/`、`context/annotation/aspectj/`）
3. ✅ 5 个 Rust 新增模块（🆕）按本表组织
4. ✅ 全部 `vernal-aspects` 模块编译通过
5. ✅ 4 个核心切面（Transactional/Cacheable/Async/Scheduled）的语义对齐有单元测试
6. ✅ `weaver/aop_xml.rs` 等价于 Spring 的 `META-INF/aop.xml` 清单
7. ✅ `support/aspect_adapter.rs` 让同一份切面代码可跑在 vernal-aop 与 aspect-rs 两套运行时上
8. ✅ **配置类保留 `Configuration` 后缀**（如 `AspectJTransactionManagementConfiguration`），不简化为 `Config`
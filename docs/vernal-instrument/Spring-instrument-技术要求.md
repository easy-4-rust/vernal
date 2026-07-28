# vernal-instrument 技术要求（对标 spring-instrument）

> **版本**：v1.0（2026-07-28）
> **对标**：`spring-instrument` 6.1 / Spring Framework 6.1
> **Rust 基线**：edition 2024 / rustc 1.88
> **crate 现状**：不迁移（Rust 无 JVM / 无运行时字节码 / 无 Agent 机制）
> **替代方案**：编译期过程宏（`vernal-macros`）

---

## 一、概述与不迁移决策

### 1.1 spring-instrument 是什么

`spring-instrument` 是 Spring Framework 的 **JVM Agent 模块**，提供以下能力：

1. **类加载期织入（Load-time Weaving, LTW）**：在 JVM 加载 `.class` 文件时，
   通过 `java.lang.instrument` API 动态修改字节码。
2. **类转换器（ClassFileTransformer）**：拦截类加载事件，注入 AOP 代理代码。
3. **Spring 代理上下文支持**：为 `@Configurable` 注解的 POJO 提供依赖注入，
   即使这些对象不是由 Spring 容器创建的。
4. **AspectJ 织入支持**：配合 `spring-aspects` 实现编译期和加载期 AOP 织入。

### 1.2 为什么 Rust 不需要迁移

**核心结论：`spring-instrument` 的所有功能在 Rust 中要么不存在对应需求，
要么已被编译期机制替代。** 不建 crate，不迁移。

| spring-instrument 功能 | Rust 对应情况 | 结论 |
|:---|:---|:---|
| JVM Agent（`-javaagent`） | Rust 无 JVM，无 Agent 机制 | 不适用 |
| 类加载期字节码修改 | Rust 编译为原生机器码，无字节码 | 不适用 |
| ClassFileTransformer | Rust 无类加载器概念 | 不适用 |
| `@Configurable` LTW 织入 | 过程宏在编译期完成 | 已替代 |
| AspectJ LTW 支持 | vernal-aop 编译期 AOP | 已替代 |
| 运行时类增强 | Rust 无运行时反射 | 不适用 |

### 1.3 Rust 与 JVM 的根本差异

```
JVM 执行模型：
  .java → javac → .class → JVM 类加载器 → 字节码验证 → LTW 织入 → JIT 编译 → 执行
                                    ↑
                            spring-instrument 在此介入

Rust 执行模型：
  .rs → rustc → LLVM IR → 机器码 → 直接执行
      ↑
  过程宏在此介入（编译期，非运行时）
```

关键差异：

| 维度 | JVM | Rust |
|:---|:---|:---|
| 编译产物 | 字节码（.class） | 原生机器码 |
| 加载机制 | 类加载器（运行时加载） | 静态链接（编译时确定） |
| 字节码修改 | 运行时可行（Agent） | 不可行（无字节码） |
| 元编程 | 反射 + 字节码生成 | 过程宏（编译期） |
| AOP 织入 | LTW / CTW / 运行时代理 | 过程宏 + trait 对象 |
| 代理模式 | 动态代理（JDK/CGLIB） | trait 对象 / 装饰器模式 |

---

## 二、逐类功能说明：为何不迁移

### 2.1 JVM Agent 机制

**spring-instrument 核心**：提供 `premain()` 方法作为 JVM Agent 入口。

```java
// spring-instrument 的 Agent 入口
public class InstrumentationSavingAgent {
    public static void premain(String agentArgs, Instrumentation inst) {
        inst.addTransformer(new ClassFileTransformer() { ... });
    }
}
```

**Rust 为何不迁移**：
- Rust 编译为原生机器码，没有 JVM，没有 `-javaagent` 启动参数。
- Rust 没有 `Instrumentation` API，没有类加载器，没有字节码格式。
- Linux 的 `LD_PRELOAD` 和 macOS 的 `DYLD_INSERT_LIBRARIES` 是最接近的
  机制，但它们用于动态库注入，不是字节码修改，且与 Spring 的 AOP 语义无关。

**Rust 替代**：不需要。编译期过程宏（`vernal-macros`）在编译时完成所有
代码增强，无需运行时介入。

### 2.2 ClassFileTransformer

**spring-instrument 功能**：拦截类加载事件，修改字节码。

```java
public class MyTransformer implements ClassFileTransformer {
    public byte[] transform(ClassLoader loader, String className,
            Class<?> classBeingRedefined, ProtectionDomain protectionDomain,
            byte[] classfileBuffer) {
        // 修改字节码，注入 AOP 代理代码
        return modifiedBytecode;
    }
}
```

**Rust 为何不迁移**：
- Rust 没有字节码，没有 `.class` 文件，没有类加载器。
- Rust 编译后的二进制文件是原生机器码，无法在运行时修改。
- 即使使用 `LD_PRELOAD` 注入共享库，也无法修改已编译的 Rust 代码。

**Rust 替代**：过程宏在编译期扫描源代码 AST，注入增强逻辑。

```rust
// vernal-macros 提供的编译期增强
#[derive(Component)]  // 编译期注入 IoC 注册代码
struct MyService {
    #[inject]         // 编译期注入依赖注入代码
    repository: Arc<dyn UserRepository>,
}

#[transactional]     // 编译期注入事务管理代码
async fn transfer(&self, from: i64, to: i64, amount: Decimal) -> Result<()> {
    // 业务逻辑
}
```

### 2.3 @Configurable LTW 织入

**spring-instrument 功能**：为 `new` 创建的对象注入 Spring 依赖。

```java
@Configurable(preConstruction = true)
public class Order {
    @Autowired
    private OrderRepository repository;  // 即使 new Order() 也会注入

    public void process() {
        repository.save(this);  // repository 不是 null
    }
}

// 使用
Order order = new Order();  // repository 已被 LTW 注入
order.process();
```

**Rust 为何不迁移**：
- Rust 没有 `new` 关键字创建对象的统一机制（直接结构体构造）。
- Rust 没有运行时反射，无法在对象创建后动态注入字段。
- `@Configurable` 的核心价值是让非容器管理的对象也能使用依赖注入，
  这在 Rust 中通过构造函数参数传递依赖更自然。

**Rust 替代**：构造函数注入 + 工厂模式。

```rust
// Rust 的依赖注入通过构造函数参数传递
struct Order {
    repository: Arc<dyn OrderRepository>,
}

impl Order {
    /// 通过 IoC 容器创建（vernal-beans Container）。
    pub fn new(repository: Arc<dyn OrderRepository>) -> Self {
        Self { repository }
    }

    /// 工厂方法（vernal-beans FactoryBean trait）。
    pub fn create(container: &Container) -> Result<Self> {
        let repository = container.get::<dyn OrderRepository>()?;
        Ok(Self::new(repository))
    }
}
```

### 2.4 AspectJ 织入支持

**spring-instrument 功能**：配合 `spring-aspects` 实现 AspectJ 编译期/加载期
AOP 织入。

```java
// AspectJ 切面
@Aspect
public class TransactionAspect {
    @Around("@annotation(transactional)")
    public Object around(ProceedingJoinPoint pjp, Transactional transactional) {
        Transaction tx = beginTransaction();
        try {
            Object result = pjp.proceed();
            tx.commit();
            return result;
        } catch (Exception e) {
            tx.rollback();
            throw e;
        }
    }
}
```

**Rust 为何不迁移**：
- AspectJ 的 LTW 模式依赖 `spring-instrument` Agent 在类加载时织入切面代码。
- Rust 没有字节码，无法在运行时织入切面。
- Rust 的 AOP 通过过程宏 + trait 对象在编译期实现，语义等价但机制不同。

**Rust 替代**：`vernal-aop` + `vernal-macros` 编译期 AOP。

```rust
// vernal-aop 的切面定义
#[intercept(TransactionInterceptor)]
async fn transfer(&self, from: i64, to: i64, amount: Decimal) -> Result<()> {
    // 业务逻辑（TransactionInterceptor 在编译期织入）
}

// 拦截器实现
pub struct TransactionInterceptor;

impl Interceptor for TransactionInterceptor {
    fn intercept(&self, invocation: &Invocation) -> BoxFuture<Result<Value>> {
        Box::pin(async move {
            begin_transaction().await?;
            match invocation.proceed().await {
                Ok(value) => {
                    commit_transaction().await?;
                    Ok(value)
                }
                Err(e) => {
                    rollback_transaction().await?;
                    Err(e)
                }
            }
        })
    }
}
```

### 2.5 Spring 代理上下文

**spring-instrument 功能**：为非容器管理的对象提供 Spring 上下文访问。

```java
@Configurable
public class DomainEvent {
    @Autowired
    private ApplicationEventPublisher publisher;

    public void publish() {
        publisher.publishEvent(this);
    }
}
```

**Rust 为何不迁移**：
- Rust 没有全局的 Spring 上下文概念。
- Rust 的依赖注入通过显式参数传递，不依赖全局上下文。
- `DomainEvent` 的 `publish` 能力通过构造时传入 `EventPublisher` 实现。

**Rust 替代**：显式依赖传递 + vernal-context `ApplicationContext`。

```rust
struct DomainEvent {
    publisher: Arc<dyn EventPublisher>,
}

impl DomainEvent {
    pub fn new(publisher: Arc<dyn EventPublisher>) -> Self {
        Self { publisher }
    }

    pub fn publish(&self) {
        self.publisher.publish(Box::new(self.clone()));
    }
}
```

---

## 三、Rust 替代方案：vernal-macros

### 3.1 vernal-macros 定位

`vernal-macros` 是 Vernal Framework 的 **过程宏 crate**，在编译期提供
spring-instrument 的等价功能。它是 `vernal-aop`、`vernal-beans`、
`vernal-context` 的编译期基础设施。

```
vernal-macros 功能矩阵
  ┌─────────────────────────────────────────────────┐
  │  vernal-macros (编译期过程宏)                     │
  │    ├─ #[derive(Component)]    → IoC 组件注册      │
  │    ├─ #[inject]               → 依赖注入          │
  │    ├─ #[transactional]        → 事务管理          │
  │    ├─ #[cacheable]            → 缓存              │
  │    ├─ #[async]                → 异步执行          │
  │    ├─ #[scheduled]            → 定时任务          │
  │    ├─ #[intercept(...)]       → AOP 拦截          │
  │    └─ #[configuration]        → 配置类            │
  ├─────────────────────────────────────────────────┤
  │  vernal-aop (AOP 内核)                           │
  │    └─ 拦截器链 + 切面匹配 + 代理生成             │
  ├─────────────────────────────────────────────────┤
  │  vernal-beans (IoC 容器)                         │
  │    └─ Bean 定义 + 生命周期 + 依赖注入            │
  ├─────────────────────────────────────────────────┤
  │  vernal-context (应用上下文)                      │
  │    └─ ApplicationContext + Environment + Events   │
  └─────────────────────────────────────────────────┘
```

### 3.2 过程宏 vs Agent 对比

| 维度 | spring-instrument (Agent) | vernal-macros (过程宏) |
|:---|:---|:---|
| 执行时机 | 运行时（类加载期） | 编译期 |
| 修改对象 | 字节码（.class 文件） | 源代码 AST |
| 性能开销 | 运行时零开销（已织入） | 编译时开销（一次性） |
| 调试体验 | 字节码不可读 | 生成的 Rust 代码可读 |
| 安全性 | 运行时修改，可能破坏不变量 | 编译期检查，类型安全 |
| 灵活性 | 可修改任意类 | 仅限标注了宏的代码 |
| 依赖 | JVM + Agent 参数 | rustc + 过程宏插件 |
| 错误报告 | 运行时异常 | 编译期错误 |

### 3.3 过程宏实现原理

```rust
// vernal-macros/src/lib.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// #[transactional] 过程宏
#[proc_macro_attribute]
pub fn transactional(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;

    let expanded = quote! {
        #fn_vis #fn_sig {
            // 编译期注入事务管理代码
            let __tx_ctx = vernal_tx::TransactionContext::current();
            __tx_ctx.begin().await?;
            match (|| async { #fn_block })().await {
                Ok(__result) => {
                    __tx_ctx.commit().await?;
                    Ok(__result)
                }
                Err(__e) => {
                    __tx_ctx.rollback().await?;
                    Err(__e)
                }
            }
        }
    };

    TokenStream::from(expanded)
}
```

### 3.4 与 vernal-aop 的集成

`vernal-macros` 生成的代码调用 `vernal-aop` 的拦截器链：

```rust
// 使用示例
#[derive(Component)]
struct OrderService {
    repository: Arc<dyn OrderRepository>,
}

impl OrderService {
    #[transactional]
    #[cacheable("orders", ttl = 300)]
    async fn find_order(&self, id: i64) -> Result<Order> {
        self.repository.find_by_id(id).await
    }

    #[transactional]
    async fn create_order(&self, order: Order) -> Result<Order> {
        let saved = self.repository.save(order).await?;
        Ok(saved)
    }
}

// 编译后展开为：
impl OrderService {
    async fn find_order(&self, id: i64) -> Result<Order> {
        let __cache_key = format!("orders:{}", id);
        if let Some(cached) = vernal_cache::get(&__cache_key).await {
            return Ok(cached);
        }
        let __tx_ctx = vernal_tx::TransactionContext::current();
        __tx_ctx.begin().await?;
        let __result = match self.repository.find_by_id(id).await {
            Ok(value) => {
                __tx_ctx.commit().await?;
                vernal_cache::put(&__cache_key, &value, 300).await;
                Ok(value)
            }
            Err(e) => {
                __tx_ctx.rollback().await?;
                Err(e)
            }
        };
        __result
    }
    // ... create_order 类似
}
```

---

## 四、其他编译期替代机制

### 4.1 linkme 分布式注册

`spring-instrument` 的 Agent 可以扫描类路径上的组件。Rust 通过 `linkme`
实现编译期分布式注册：

```rust
// vernal-context-indexer 使用 linkme
use linkme::distributed_slice;

#[distributed_slice]
pub static COMPONENTS: [fn() -> ComponentDef] = [..];

// 各 crate 注册自己的组件
#[distributed_slice(COMPONENTS)]
fn register_order_service() -> ComponentDef {
    ComponentDef::new::<OrderService>()
        .with_scope(Scope::Singleton)
        .with_dependencies(&[Dependency::new::<dyn OrderRepository>()])
}
```

### 4.2 inventory 函数注册

`spring-instrument` 的 Agent 可以注册全局事件监听器。Rust 通过 `inventory`
实现：

```rust
use inventory;

pub struct EventListenerDef {
    pub event_type: &'static str,
    pub handler: fn(&dyn Any),
}

inventory::collect!(EventListenerDef);

// 各 crate 注册事件监听器
inventory::submit! {
    EventListenerDef {
        event_type: "OrderCreated",
        handler: |event| {
            let order = event.downcast_ref::<OrderCreated>().unwrap();
            // 处理事件
        },
    }
}
```

### 4.3 Rust 编译期反射（有限）

Rust 没有完整的运行时反射，但通过 trait 和过程宏可以实现有限的编译期反射：

```rust
/// 编译期类型信息 trait（vernal-core 提供）。
pub trait TypeInfo {
    fn type_name() -> &'static str;
    fn module_path() -> &'static str;
    fn fields() -> &'static [FieldInfo];
    fn methods() -> &'static [MethodInfo];
}

/// 字段信息。
pub struct FieldInfo {
    pub name: &'static str,
    pub type_name: &'static str,
    pub offset: usize,
}

// 通过 derive 宏自动生成
#[derive(TypeInfo)]
struct User {
    name: String,
    email: String,
}
```

---

## 五、迁移决策矩阵

### 5.1 逐功能迁移决策

| spring-instrument 功能 | 迁移决策 | Rust 替代 | 说明 |
|:---|:---:|:---|:---|
| `premain()` Agent 入口 | 不迁移 | 无 | Rust 无 JVM |
| `Instrumentation` API | 不迁移 | 无 | Rust 无类加载器 |
| `ClassFileTransformer` | 不迁移 | 无 | Rust 无字节码 |
| `@Configurable` LTW | 不迁移 | 构造函数注入 | 显式依赖传递 |
| AspectJ LTW 织入 | 不迁移 | `vernal-aop` 过程宏 | 编译期 AOP |
| 类路径扫描 | 不迁移 | `linkme` 分布式注册 | 编译期组件发现 |
| 事件监听器注册 | 不迁移 | `inventory` 分布式注册 | 编译期事件注册 |
| 运行时类增强 | 不迁移 | 无 | Rust 无运行时修改 |
| 代理上下文 | 不迁移 | 显式依赖传递 | 不依赖全局上下文 |

### 5.2 不迁移原因汇总

1. **技术栈不兼容**：spring-instrument 的核心机制（JVM Agent、字节码修改、
   类加载器）在 Rust 中不存在对应概念。
2. **编译期替代成熟**：vernal-macros 过程宏已覆盖 spring-instrument 的所有
   实际使用场景（AOP 织入、依赖注入、事务管理）。
3. **安全性更好**：编译期检查比运行时修改更安全，错误在编译时发现而非运行时。
4. **性能更优**：编译期代码增强无运行时开销，而 Agent 的 LTW 在类加载时有开销。
5. **调试更友好**：生成的 Rust 代码可读，而字节码修改后的类难以调试。

### 5.3 vernal-macros 功能清单

| 宏 | 对标 Spring | 说明 |
|:---|:---|:---|
| `#[derive(Component)]` | `@Component` | IoC 组件注册 |
| `#[inject]` | `@Autowired` | 依赖注入 |
| `#[transactional]` | `@Transactional` | 事务管理 |
| `#[cacheable]` | `@Cacheable` | 缓存 |
| `#[async]` | `@Async` | 异步执行 |
| `#[scheduled]` | `@Scheduled` | 定时任务 |
| `#[intercept(...)]` | `@Aspect` + `@Around` | AOP 拦截 |
| `#[configuration]` | `@Configuration` | 配置类 |
| `#[bean]` | `@Bean` | Bean 定义 |
| `#[scope]` | `@Scope` | 作用域 |
| `#[post_construct]` | `@PostConstruct` | 初始化回调 |
| `#[pre_destroy]` | `@PreDestroy` | 销毁回调 |
| `#[value("...")]` | `@Value` | 配置值注入 |
| `#[condition_on]` | `@Conditional` | 条件装配 |

---

## 六、总结

### 6.1 核心结论

`spring-instrument` 是 JVM 专属模块，其核心机制（JVM Agent、字节码修改、
类加载器织入）在 Rust 中不存在对应概念，也不需要迁移。Rust 通过编译期过程宏
（`vernal-macros`）实现了 spring-instrument 的所有实际使用场景，且在安全性、
性能和调试体验方面优于 JVM 的运行时织入方案。

### 6.2 决策记录

| 决策项 | 决策 | 理由 |
|:---|:---|:---|
| 是否建 vernal-instrument crate | 否 | Rust 无 JVM Agent 机制 |
| 是否建 vernal-instrument 文档 | 是（本文档） | 记录不迁移决策和替代方案 |
| spring-instrument 功能替代 | `vernal-macros` 过程宏 | 编译期等价替代 |
| @Configurable 替代 | 构造函数注入 | Rust 惯用模式 |
| AspectJ LTW 替代 | `vernal-aop` 编译期 AOP | 过程宏 + trait 对象 |
| 类路径扫描替代 | `linkme` 分布式注册 | 编译期组件发现 |

### 6.3 后续行动

无需后续行动。`vernal-macros` 已在 vernal-framework workspace 中实现，
覆盖了 spring-instrument 的所有实际使用场景。本文档仅作为技术交接记录，
证明不迁移决策的合理性。

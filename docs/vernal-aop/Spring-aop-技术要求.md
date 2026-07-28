# vernal-aop 技术要求（对标 spring-aop）

> **版本**：v2.0（2026-07-28）
> **定位**：vernal-aop crate 的技术交接文档，对标 Spring Framework 7.0.8 spring-aop。
> **主线**：aspect-rs（aspect-core 0.1.2 + aspect-macros + aspect-std）为实现主线，
> spring-aop 仅作语义参考。
> **现状**：64 文件 / 4649 行，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》8.4 节（校验）与 8.5 节（AOP）。

---

## 一、总览

### 1.1 定位与边界

vernal-aop 是 Vernal Framework 的 **Tokio-first 异步切面与拦截内核**。它对标
Spring Framework 的 `spring-aop` 模块，提供 AOP（面向切面编程）能力。

| 维度 | spring-aop（语义参考） | vernal-aop（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（反射 + CGLIB 字节码） | Rust（trait + 过程宏） | 无运行时代理，编译期宏替代 |
| 异步 | Reactor Mono/Flux | Tokio-first async | 原生异步，无桥接开销 |
| 代理工厂 | ProxyFactory + CGLIB/JDK | `#[aspect]` 过程宏 + InvocationPlan | 编译期织入，零反射 |
| 切点表达式 | AspectJ Weaver（编译期织入） | `parse_pointcut_expr()` DSL 解析器 | Rust 原生 DSL，无需外部工具链 |
| 业务切面 | interceptor 包（13 个调试类） | aspect-std（8 个生产级） | 业务价值更高 |
| Advice 类型 | 6 类（含 Introduction） | 4 类 + Introduction trait | 核心覆盖，Introduction 用 trait object |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  用户层：Aspect trait（四段式异步通知）               │
│  before / after / after_error / around              │
├─────────────────────────────────────────────────────┤
│  桥接层：AspectAdapter（Aspect → Interceptor）       │
├─────────────────────────────────────────────────────┤
│  执行层：Interceptor trait（Around-only 底层原语）    │
│  → Next 链式推进 → InvocationPlan 预编译             │
├─────────────────────────────────────────────────────┤
│  匹配层：Pointcut trait + PointcutExpr DSL           │
│  → Advisor 绑定 Pointcut + Interceptor + order       │
├─────────────────────────────────────────────────────┤
│  运行时：Invocation + Operation + CancellationToken  │
│  → InvocationPlanCatalog 启动期预编译目录             │
└─────────────────────────────────────────────────────┘
```

### 1.3 与 aspect-rs 的关系

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 结构主线 | aspect-rs | Rust 原生，无语言范式转换 |
| 语义参考 | spring-aop | 208 类提供功能完整度标尺 |
| 异步化 | aspect-rs 同步 → vernal-aop Tokio-first | 核心改造 |
| 命名冲突 | `Pointcut` enum → `PointcutExpr` | 避免与现有 `Pointcut` trait 冲突 |
| CGLIB/JDK | 不移植，用过程宏替代 | Rust 无运行时字节码代理 |

### 1.4 关键命名映射

| aspect-rs 原名 | vernal-aop 移植名 | 冲突原因 |
|:---|:---|:---|
| `Pointcut`（AST enum） | `PointcutExpr` | 与现有 `Pointcut` trait 冲突 |
| `Matcher` trait | `PointcutMatcher` | 避免歧义 |
| `FunctionInfo` | `FunctionDescriptor` | 与 `Operation` 概念区分 |
| `parse_pointcut` | `parse_pointcut_expr` | 强调解析表达式 |
| `Aspect` trait | 保留原名 `Aspect` | 与 `Interceptor` 共存（门面 vs 原语） |

---

## 二、核心 Trait 体系

本章涵盖 vernal-aop 的三大核心 trait：`Aspect`（用户门面）、`Interceptor`（底层原语）、
`AspectAdapter`（桥接器）。

### 2.1 Aspect trait —— 四段式异步切面门面

**来源**：移植自 aspect-rs `aspect-core/src/aspect.rs::Aspect`，改造为 Tokio-first 异步。
**语义参照**：spring-aop `MethodBeforeAdvice` / `AfterReturningAdvice` /
`ThrowsAdvice` / `MethodInterceptor`。

#### Spring API（Java）

```java
// spring-aop 6 类 Advice（vernal-aop 覆盖 4/6）
public interface MethodBeforeAdvice extends BeforeAdvice {
    void before(Method m, Object[] args, Object target) throws Throwable;
}
public interface AfterReturningAdvice extends AfterAdvice {
    void afterReturning(Object returnValue, Method m, Object[] args, Object target) throws Throwable;
}
public interface ThrowsAdvice extends AfterAdvice {
    // 反射调用 afterThrowing 方法，无固定接口
}
public interface MethodInterceptor {
    Object invoke(MethodInvocation invocation) throws Throwable;
}
```

#### Rust trait

```rust
/// 切面门面（四段式异步通知）。
/// 对应 aspect-rs 的 Aspect trait，before/after/after_error/around 四段式
/// 改造为异步。around 默认实现保留 aspect-rs 的 before→proceed→after 模式。
///
/// 与 Interceptor 的关系：Aspect 是用户友好门面，Interceptor 是底层执行原语。
/// 通过 AspectAdapter 自动转换。
pub trait Aspect: Send + Sync + 'static {
    /// 方法执行前调用。对应 spring-aop MethodBeforeAdvice.before()。
    fn before<'a>(
        &'a self, _inv: &'a Invocation,
    ) -> Pin<Box<dyn Future<Output = Result<(), InvocationError>> + Send + 'a>>;

    /// 方法成功执行后调用。对应 spring-aop AfterReturningAdvice.afterReturning()。
    fn after<'a>(
        &'a self, _inv: &'a Invocation, _result: &'a InvocationValue,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    /// 方法异常时调用。对应 spring-aop ThrowsAdvice（反射 afterThrowing）。
    fn after_error<'a>(
        &'a self, _inv: &'a Invocation, _error: &'a InvocationError,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    /// 包裹整个执行的环绕通知。对应 spring-aop MethodInterceptor.invoke()。
    /// 默认实现：before(inv) → next.run(inv) → Ok: after / Err: after_error
    fn around<'a>(&'a self, inv: Arc<Invocation>, next: Next<'a>) -> InvocationFuture<'a>;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享 |
| `'static` | 必须 | 无生命周期引用，可存入 Arc |
| 返回类型 | `Pin<Box<dyn Future + Send>>` | 手动 async trait，避免 `async-trait` 依赖 |
| 默认实现 | 所有 4 个方法均有 | `before` 返回 `Ok(())`，`after`/`after_error` 无操作 |
| around 默认语义 | `before → proceed → after/after_error` | `before` 失败则短路 |

#### 待补齐

- [x] `Aspect` trait 定义（S1 已完成）
- [x] 四段式默认实现（S1 已完成）
- [x] `Send + Sync + 'static` 约束验证测试
- [ ] `#[aspect]` 过程宏自动织入（S2 待做）

---

### 2.2 Interceptor trait —— Around-only 底层执行原语

**来源**：vernal-aop 现有内核，无 aspect-rs 对偶。
**语义参照**：spring-aop `MethodInterceptor.invoke()`。

#### Spring API（Java）

```java
// spring-aop 核心拦截器
public interface MethodInterceptor {
    Object invoke(MethodInvocation invocation) throws Throwable;
}
// MethodInvocation 提供 proceed() 控制执行链
public interface Invocation {
    Object[] getArguments();
    Method getMethod();
    Object proceed() throws Throwable;
}
```

#### Rust trait

```rust
/// Vernal AOP 的核心环绕拦截器。
/// 实现者可以在调用 Next::run 前后执行逻辑，也可以不调用 next 直接返回，
/// 形成鉴权拒绝、缓存命中或熔断短路。结果与错误均可在返回前被替换。
pub trait Interceptor: Send + Sync + 'static {
    /// 环绕当前调用。
    fn intercept<'a>(
        &'a self, invocation: Arc<Invocation>, next: Next<'a>,
    ) -> InvocationFuture<'a>;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享 |
| `'static` | 必须 | 存入 `Arc<dyn Interceptor>` |
| 方法签名 | `(&self, Arc<Invocation>, Next) -> InvocationFuture` | 接收共享调用对象和链后继 |
| 短路能力 | 可不调用 `next.run()` | 缓存命中、鉴权拒绝、熔断短路 |
| 结果替换 | 可替换 `Ok`/`Err` | 恢复、转换、包装 |

#### 与 Aspect 的对比

| 特性 | Interceptor | Aspect |
|:---|:---|:---|
| Advice 类型 | 1（around） | 4（before/after/after_error/around） |
| 定位 | 底层执行原语 | 用户友好门面 |
| 谁实现 | 框架内部 / 高级用户 | 普通用户 |
| 转换 | — | `AspectAdapter` 自动转为 `Interceptor` |
| 异步 | Tokio-first | Tokio-first |

#### 待补齐

- [x] `Interceptor` trait 定义（已有）
- [x] `Next` 链式推进（已有）
- [ ] `SimpleInterceptor` 与 `Interceptor` 桥接（待评估）

---

### 2.3 AspectAdapter —— Aspect 到 Interceptor 的桥接器

**来源**：vernal-aop 扩展（无 aspect-rs 对偶）。
**语义参照**：spring-aop `AdvisorAdapterRegistry` 桥接 Advice 到 Interceptor。

#### Spring API（Java）

```java
// spring-aop 的 AdvisorAdapterRegistry 将 Advice 适配为 Interceptor
public interface AdvisorAdapterRegistry {
    Interceptor[] getInterceptors(Advice advice) throws UnknownAdviceTypeException;
    Advisor wrap(Object advice) throws UnknownAdviceTypeException;
}
```

#### Rust 实现

```rust
/// 把 Aspect 自动适配为 Interceptor 的桥接器。
/// 用户实现 Aspect（四段式），AspectAdapter 自动转换为 Interceptor。
pub struct AspectAdapter<A: Aspect> {
    aspect: Arc<A>,
}

impl<A: Aspect> AspectAdapter<A> {
    pub fn new(aspect: A) -> Self { ... }
    pub fn shared(aspect: Arc<A>) -> Self { ... }
    pub fn aspect(&self) -> &A { ... }
}

impl<A: Aspect> Interceptor for AspectAdapter<A> {
    fn intercept<'a>(&'a self, invocation: Arc<Invocation>, next: Next<'a>) -> InvocationFuture<'a> {
        self.aspect.around(invocation, next)
    }
}

impl<A: Aspect> Clone for AspectAdapter<A> { ... }
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| 泛型参数 | `A: Aspect` | 必须实现 Aspect trait |
| 内部存储 | `Arc<A>` | 共享切面实例，支持 Clone |
| `Clone` | 通过 `Arc::clone` 实现 | 浅克隆，共享底层切面 |
| 生命周期 | `&self.aspect` 与 `&'a self` 绑定 | 满足 `around` 签名约束 |

#### 待补齐

- [x] `AspectAdapter` 定义（S1 已完成）
- [x] `Interceptor for AspectAdapter` 实现（S1 已完成）
- [x] `Clone` 实现（S1 已完成）
- [ ] 与 `InvocationPlanBuilder` 集成（S4 已完成）

---

## 三、Pointcut DSL 体系

本章涵盖切点匹配的全部组件：`Pointcut` trait（运行时匹配契约）、`PointcutExpr` enum
（DSL AST）、Pattern 体系、解析器、以及 vernal-aop 现有的具体切点对象。

### 3.1 Pointcut trait —— 切点匹配契约

**来源**：vernal-aop 现有内核。
**语义参照**：spring-aop `Pointcut`（`ClassFilter` + `MethodMatcher`）。

#### Spring API（Java）

```java
// spring-aop Pointcut = ClassFilter + MethodMatcher
public interface Pointcut {
    ClassFilter getClassFilter();
    MethodMatcher getMethodMatcher();
    Pointcut TRUE = TruePointcut.INSTANCE;
}
public interface MethodMatcher {
    boolean matches(Method method, Class<?> targetClass);
    boolean isRuntime();
    boolean matches(Method method, Class<?> targetClass, Object... args);
}
```

#### Rust trait

```rust
/// 判断一个切面是否应用于指定操作。
/// 闭包会自动实现该 trait，因此调用方既可以定义可复用切点对象，
/// 也可以直接使用 |operation: &Operation| ... 完成匹配。
pub trait Pointcut: Send + Sync + 'static {
    fn matches(&self, operation: &Operation) -> bool;
}

// 闭包自动实现
impl<F> Pointcut for F
where F: Fn(&Operation) -> bool + Send + Sync + 'static
{
    fn matches(&self, operation: &Operation) -> bool { self(operation) }
}
```

#### 现有 Pointcut 实现一览

| 类型 | 文件 | 匹配逻辑 | spring-aop 对应 |
|:---|:---|:---|:---|
| `AnyPointcut` | `any_pointcut.rs` | 永真 | `TruePointcut` |
| `ComponentPointcut` | `component_pointcut.rs` | 组件名精确匹配 | `ClassFilter`（简化） |
| `MethodPointcut` | `method_pointcut.rs` | 方法名精确匹配 | `NameMatchMethodPointcut` |
| `OperationPointcut` | `operation_pointcut.rs` | 组件+方法同时匹配 | `StaticMethodMatcherPointcut` |
| `TagPointcut` | `tag_pointcut.rs` | 标签匹配 | （spring-aop 无对应） |
| `QualifierPointcut` | `qualifier_pointcut.rs` | 限定符匹配 | （spring-aop 无对应） |
| `AndPointcut<L,R>` | `and_pointcut.rs` | 逻辑与（短路） | `ComposablePointcut` |
| `OrPointcut<L,R>` | `or_pointcut.rs` | 逻辑或（短路） | `ComposablePointcut` |
| `NotPointcut<P>` | `not_pointcut.rs` | 逻辑非 | `Pointcuts.not()` |

#### 待补齐

- [x] `Pointcut` trait 定义（已有）
- [x] 8 个具体 Pointcut 实现（已有）
- [x] `PointcutExt` 组合扩展（`.and()` / `.or()` / `.not()`）
- [ ] `args()` / `this()` / `target()` designator（P2）

---

### 3.2 PointcutExpr enum —— DSL 抽象语法树

**来源**：移植自 aspect-rs `aspect-core/src/pointcut/ast.rs`，改名 `Pointcut` →
`PointcutExpr`，扩展 `Tag` / `Qualifier` 变体。
**语义参照**：spring-aop AspectJ 表达式 AST。

#### Spring API（Java）

```java
// spring-aop 使用 AspectJ 表达式（编译期织入）
// AspectJ designator: execution(), within(), args(), this(), target(), @annotation()
@Pointcut("execution(public * com.example.service.*.*(..))")
public void serviceLayer() {}

@Around("serviceLayer() && @annotation(secured)")
public Object audit(ProceedingJoinPoint pjp) throws Throwable { ... }
```

#### Rust enum

```rust
/// 切点表达式（Rust 原生 DSL）。
/// 对应 aspect-rs Pointcut enum（改名 PointcutExpr），扩展 Tag/Qualifier 变体。
#[derive(Debug, Clone, PartialEq)]
pub enum PointcutExpr {
    /// 匹配函数执行：execution(pub fn save(..))
    Execution(ExecutionPattern),
    /// 匹配模块内函数：within(crate::api)
    Within(ModulePattern),
    /// 逻辑与
    And(Box<PointcutExpr>, Box<PointcutExpr>),
    /// 逻辑或
    Or(Box<PointcutExpr>, Box<PointcutExpr>),
    /// 逻辑非
    Not(Box<PointcutExpr>),
    /// 标签匹配（vernal 扩展）：tag(secured)
    Tag(TagPattern),
    /// 限定符匹配（vernal 扩展）：qualifier(primary)
    Qualifier(QualifierPattern),
}
```

#### DSL 语法参考

| 语法 | 示例 | 说明 |
|:---|:---|:---|
| `execution(pattern)` | `execution(pub fn save_*(..))` | 函数执行匹配 |
| `within(module)` | `within(crate::api)` | 模块内匹配 |
| `tag(name)` | `tag(secured)` | 标签匹配（vernal 扩展） |
| `qualifier(name)` | `qualifier(primary)` | 限定符匹配（vernal 扩展） |
| `&&` | `execution(...) && within(...)` | 逻辑与 |
| `\|\|` | `execution(save) \|\| execution(update)` | 逻辑或 |
| `!` | `!within(crate::internal)` | 逻辑非 |
| `()` | `(a \|\| b) && within(api)` | 分组（改变优先级） |

#### 便捷方法

```rust
impl PointcutExpr {
    pub fn parse(input: &str) -> Result<Self, PointcutParseError>;
    pub fn and(self, other: PointcutExpr) -> Self;
    pub fn or(self, other: PointcutExpr) -> Self;
    pub fn not(self) -> Self;
    pub fn public_functions() -> Self;
    pub fn all_functions() -> Self;
    pub fn within_module(module_path: impl Into<String>) -> Self;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `PointcutExpr` vs `Pointcut` trait | 共存，不冲突 | `PointcutExpr` 是 AST，`Pointcut` trait 是匹配契约 |
| `PointcutMatcher` 实现 | 为 `PointcutExpr` 实现 | 通过 `FunctionDescriptor::from_operation` 桥接 |
| 运算符优先级 | `!` > `&&` > `\|\|` | 与 Java/AspectJ 一致 |
| 括号分组 | 支持嵌套 `()` | 正确处理深度嵌套 |

#### 待补齐

- [x] `PointcutExpr` enum 定义（S1 已完成）
- [x] 7 个变体（Execution/Within/And/Or/Not/Tag/Qualifier）（S1 已完成）
- [x] 便捷方法（parse/and/or/not/public_functions/all_functions/within_module）
- [ ] `args()` / `this()` / `target()` 变体（P2）

---

### 3.3 Pattern 体系 —— 匹配模式类型

**来源**：直接移植自 aspect-rs `aspect-core/src/pointcut/pattern.rs`，4 类模式。
**语义参照**：spring-aop `ClassFilter` / `MethodMatcher` / `NameMatchMethodPointcut`。

#### Visibility 枚举

```rust
/// 函数可见性模式。对应 Rust 原生可见性概念（spring-aop 无直接对应）。
pub enum Visibility {
    Public,    // pub
    Crate,     // pub(crate)
    Super,     // pub(super)
    Private,   // 无修饰符
}
```

#### NamePattern 枚举

```rust
/// 函数名模式。对应 spring-aop NameMatchMethodPointcut。
pub enum NamePattern {
    Wildcard,            // * — 匹配任意名称
    Exact(String),       // save_user — 精确匹配
    Prefix(String),      // save* — 前缀匹配
    Suffix(String),      // *_user — 后缀匹配
    Contains(String),    // *save* — 包含匹配
}
```

#### ExecutionPattern 结构

```rust
/// 执行模式：匹配函数签名。对应 AspectJ execution() 设计器。
pub struct ExecutionPattern {
    pub visibility: Option<Visibility>,
    pub name: NamePattern,
    pub return_type: Option<String>,
}
// 便捷构造：any() / public() / named(name)
```

#### ModulePattern 结构

```rust
/// 模块模式：按模块路径匹配函数。对应 AspectJ within() 设计器。
pub struct ModulePattern {
    pub path: String,
}
// matches_path 支持精确匹配和前缀匹配（含 :: 子模块）
```

#### 待补齐

- [x] 4 类 Pattern 定义（S1 已完成）
- [x] Visibility::matches / NamePattern::matches / ModulePattern::matches_path（S1 已完成）
- [x] ExecutionPattern 便捷构造方法（S1 已完成）

---

### 3.4 PointcutMatcher 与 FunctionDescriptor

**来源**：移植自 aspect-rs `aspect-core/src/pointcut/matcher.rs`。
**改名**：`Matcher` → `PointcutMatcher`，`FunctionInfo` → `FunctionDescriptor`。
**语义参照**：spring-aop `MethodMatcher`。

#### Rust 实现

```rust
/// 函数描述符（用于切点匹配的中间结构）。
/// 从 vernal Operation 转换而来。
pub struct FunctionDescriptor {
    pub name: String,
    pub module_path: String,
    pub visibility: String,
    pub return_type: Option<String>,
    pub tags: Vec<String>,        // vernal 扩展
    pub qualifier: Option<String>, // vernal 扩展
}

impl FunctionDescriptor {
    /// 从 vernal Operation 转换为函数描述符。
    pub fn from_operation(operation: &Operation) -> Self { ... }
}

/// 切点匹配器 trait。对应 aspect-rs Matcher（改名）。
pub trait PointcutMatcher {
    fn matches_operation(&self, operation: &Operation) -> bool;
}

// 为 PointcutExpr 实现 PointcutMatcher
impl PointcutMatcher for PointcutExpr { ... }
```

#### 待补齐

- [x] `FunctionDescriptor` 定义（S1 已完成）
- [x] `from_operation` 转换（S1 已完成）
- [x] `PointcutMatcher` trait + `PointcutExpr` 实现（S1 已完成）

---

### 3.5 DSL 解析器

**来源**：移植自 aspect-rs `aspect-core/src/pointcut/parser.rs`。
**改名**：`parse_pointcut` → `parse_pointcut_expr`。
**语义参照**：spring-aop AspectJ Weaver 解析器。

#### Rust 实现

```rust
/// 解析切点表达式字符串。
pub fn parse_pointcut_expr(input: &str) -> Result<PointcutExpr, PointcutParseError>;

/// 切点表达式解析错误。
pub struct PointcutParseError {
    pub message: String,
    pub position: Option<usize>,
}
```

#### 支持的语法

| 语法 | 示例 | 说明 |
|:---|:---|:---|
| `execution(pattern)` | `execution(pub fn save_*(..))` | 函数执行匹配 |
| `within(module)` | `within(crate::api)` | 模块内匹配 |
| `tag(name)` | `tag(secured)` | 标签匹配（vernal 扩展） |
| `qualifier(name)` | `qualifier(primary)` | 限定符匹配（vernal 扩展） |
| `&&` / `\|\|` / `!` | `execution(...) && within(...)` | 逻辑组合 |
| `()` 分组 | `(a \|\| b) && c` | 改变优先级 |

#### 待补齐

- [x] `parse_pointcut_expr` 定义（S1 已完成）
- [x] `execution()` / `within()` / `tag()` / `qualifier()` 解析（S1 已完成）
- [x] `&&` / `\|\|` / `!` / `()` 解析（S1 已完成）
- [ ] `args()` / `this()` / `target()` 解析（P2）

---

## 四、Advisor 体系

本章涵盖顾问对象：`Advisor`（通用顾问）、`PointcutAdvisor`（切点顾问 trait）、
`IntroductionAdvisor`（引入顾问 trait）、`DefaultPointcutAdvisor`（默认实现）。

### 4.1 Advisor —— 通用顾问对象

**来源**：vernal-aop 现有内核。
**语义参照**：spring-aop `Advisor`（`Pointcut` + `Advice` + `order`）。

#### Spring API（Java）

```java
// spring-aop Advisor = Pointcut + Advice + order
public interface Advisor {
    Advice getAdvice();
    boolean isPerInstance();
}
public interface PointcutAdvisor extends Advisor {
    Pointcut getPointcut();
}
```

#### Rust 实现

```rust
/// 将一个切点、一个环绕拦截器和执行顺序组合成切面声明。
/// order 越小越先进入调用链、越后退出；相同顺序由注册顺序稳定决定。
#[derive(Clone)]
pub struct Advisor {
    pointcut: Arc<dyn Pointcut>,
    interceptor: Arc<dyn Interceptor>,
    order: i32,
}

impl Advisor {
    pub fn new<P: Pointcut, I: Interceptor>(pointcut: P, interceptor: I, order: i32) -> Self;
    pub fn shared(pointcut: Arc<dyn Pointcut>, interceptor: Arc<dyn Interceptor>, order: i32) -> Self;
    pub fn matches(&self, operation: &Operation) -> bool;
    pub fn order(&self) -> i32;
    pub fn interceptor(&self) -> Arc<dyn Interceptor>;
}
```

#### 待补齐

- [x] `Advisor` 定义（已有）
- [x] `matches` / `order` / `interceptor` 方法（已有）

---

### 4.2 PointcutAdvisor trait —— 切点驱动的顾问

**来源**：spring-aop 语义补缺（S5 已完成），aspect-rs 无对偶。
**语义参照**：spring-aop `PointcutAdvisor`。

#### Spring API（Java）

```java
// spring-aop PointcutAdvisor 覆盖除引入顾问之外的几乎所有顾问
public interface PointcutAdvisor extends Advisor {
    Pointcut getPointcut();
}
```

#### Rust trait

```rust
/// 切点顾问：由切点驱动的顾问超接口。
/// 对应 spring-aop PointcutAdvisor。
pub trait PointcutAdvisor: Send + Sync + 'static {
    fn pointcut(&self) -> &dyn Pointcut;
    fn interceptor(&self) -> &dyn Interceptor;
    fn order(&self) -> i32;
}
```

#### 待补齐

- [x] `PointcutAdvisor` trait 定义（S5 已完成）
- [ ] `PointcutAdvisor` 与 `InvocationPlanBuilder` 集成

---

### 4.3 IntroductionAdvisor trait —— 引入顾问

**来源**：spring-aop 语义补缺（S5 已完成），aspect-rs 无对偶。
**语义参照**：spring-aop `IntroductionAdvisor`。

#### Spring API（Java）

```java
// spring-aop IntroductionAdvisor：通过 AOP 通知实现目标未实现的额外接口
public interface IntroductionAdvisor extends Advisor, IntroductionInfo {
    ClassFilter getClassFilter();
    void validateInterfaces() throws IllegalArgumentException;
}
public interface IntroductionInfo {
    Class<?>[] getInterfaces();
}
```

#### Rust trait

```rust
/// 引入信息：描述通过 AOP 引入的额外接口。
pub trait IntroductionInfo: Send + Sync + 'static {
    fn interface_names(&self) -> &[&'static str];
}

/// 引入顾问：执行一个或多个 AOP 引入。
/// 引入是通过 AOP 通知实现目标未实现的额外接口。
pub trait IntroductionAdvisor: IntroductionInfo + Send + Sync + 'static {
    fn class_filter(&self) -> &dyn Fn(&str) -> bool;
    fn order(&self) -> i32;
    fn pointcut(&self) -> &dyn Pointcut;
}
```

#### 待补齐

- [x] `IntroductionInfo` trait 定义（S5 已完成）
- [x] `IntroductionAdvisor` trait 定义（S5 已完成）
- [ ] 具体 `IntroductionAdvisor` 实现（待业务需求）

---

### 4.4 DefaultPointcutAdvisor —— 默认切点顾问实现

**来源**：spring-aop 语义补缺（S5 已完成）。
**语义参照**：spring-aop `DefaultPointcutAdvisor`。

#### Spring API（Java）

```java
// spring-aop 最常用的 Advisor 实现
public class DefaultPointcutAdvisor extends AbstractGenericPointcutAdvisor {
    public DefaultPointcutAdvisor(Pointcut pointcut, Advice advice) { ... }
    public DefaultPointcutAdvisor(int order, Pointcut pointcut, Advice advice) { ... }
}
```

#### Rust 实现

```rust
/// 默认切点顾问。对应 spring-aop DefaultPointcutAdvisor。
/// 可与任何切点和通知类型配合使用，但不适用于引入。
pub struct DefaultPointcutAdvisor {
    pointcut: Arc<dyn Pointcut>,
    interceptor: Arc<dyn Interceptor>,
    order: i32,
}

impl DefaultPointcutAdvisor {
    pub fn new<I: Interceptor>(pointcut: impl Pointcut, interceptor: I) -> Self;
    pub fn with_order<I: Interceptor>(pointcut: impl Pointcut, interceptor: I, order: i32) -> Self;
    pub fn shared(pointcut: Arc<dyn Pointcut>, interceptor: Arc<dyn Interceptor>, order: i32) -> Self;
}

impl PointcutAdvisor for DefaultPointcutAdvisor { ... }
```

#### 待补齐

- [x] `DefaultPointcutAdvisor` 定义（S5 已完成）
- [x] `PointcutAdvisor` 实现（S5 已完成）

---

## 五、业务切面与工具切面

本章涵盖 aspect-std 的 8 个业务切面（待 S3 移植）和 spring-aop 的工具切面
（语义参考）。

### 5.1 aspect-std 8 个业务切面

**来源**：待移植自 aspect-rs `aspect-std`（S3 阶段）。
**语义参照**：spring-aop interceptor 包（13 个调试类）。
**实现方式**：所有切面实现 vernal-aop 的 `Interceptor` trait（而非 `Aspect`），
因为业务切面需要 around 全控制。

#### 8 个切面对照表

| 切面 | aspect-rs 源 | Rust crate 依赖 | spring-aop 语义参照 | 状态 |
|:---|:---|:---|:---|:---|
| `logging` | `logging.rs` | `tracing` | `DebugInterceptor` / `SimpleTraceInterceptor` | ⬜ S3 |
| `timing` | `timing.rs` | `tracing` + `tokio::time::Instant` | `PerformanceMonitorInterceptor` | ⬜ S3 |
| `metrics` | `metrics.rs` | `metrics` crate | （spring-aop 无对应） | ⬜ S3 |
| `caching` | `caching.rs` | `moka`（LRU/TTL） | （spring-aop 无对应） | ⬜ S3 |
| `ratelimit` | `ratelimit.rs` | `governor`（令牌桶） | （spring-aop 无对应） | ⬜ S3 |
| `circuitbreaker` | `circuitbreaker.rs` | `failsafe-rust`（状态机） | （spring-aop 无对应） | ⬜ S3 |
| `authorization` | `authorization.rs` | 自定义闭包 | （spring-aop 无对应） | ⬜ S3 |
| `validation` | `validation.rs` | `validator` crate | （spring-aop 无对应） | ⬜ S3 |

#### 目标文件结构

```
vernal-aop/src/interceptor/
├── mod.rs                      # 模块声明 + re-export
├── logging_aspect.rs           # 日志（进出方法）
├── timing_aspect.rs            # 计时（执行耗时）
├── metrics_aspect.rs           # 度量（自定义指标）
├── caching_aspect.rs           # 缓存（结果缓存）
├── ratelimit_aspect.rs         # 限流（令牌桶）
├── circuitbreaker_aspect.rs    # 熔断（closed/open/half-open）
├── authorization_aspect.rs     # 授权（权限校验）
└── validation_aspect.rs        # 校验（参数有效性）
```

#### 实现模式（以 caching 为例）

```rust
/// 缓存切面。对应 aspect-rs aspect-std caching。
/// 实现 Interceptor trait（非 Aspect），因为需要 around 全控制。
pub struct CachingAspect {
    cache: moka::future::Cache<String, Arc<dyn Any + Send + Sync>>,
}

impl Interceptor for CachingAspect {
    fn intercept<'a>(&'a self, inv: Arc<Invocation>, next: Next<'a>) -> InvocationFuture<'a> {
        Box::pin(async move {
            let key = format!("{}:{}", inv.operation().component(), inv.operation().method());
            if let Some(cached) = self.cache.get(&key).await {
                return Ok(cached);
            }
            let result = next.run(inv).await;
            if let Ok(value) = &result {
                self.cache.insert(key, value.clone()).await;
            }
            result
        })
    }
}
```

#### 与 spring-aop interceptor 包的互补

| 切面类别 | spring-aop interceptor 包 | aspect-std | 互补关系 |
|:---|:---|:---|:---|
| 调试/追踪 | `DebugInterceptor` / `SimpleTraceInterceptor` | `logging` / `timing` | 重叠（合并） |
| 性能监控 | `PerformanceMonitorInterceptor` | `metrics` / `timing` | 重叠（合并） |
| 缓存 | （无） | `caching` | **aspect-std 独有** |
| 熔断 | （无） | `circuitbreaker` | **aspect-std 独有** |
| 限流 | （无） | `ratelimit` | **aspect-std 独有** |
| 授权 | （无） | `authorization` | **aspect-std 独有** |
| 校验 | （无） | `validation` | **aspect-std 独有** |

#### 待补齐

- [ ] `logging_aspect.rs`（S3 待做）
- [ ] `timing_aspect.rs`（S3 待做）
- [ ] `metrics_aspect.rs`（S3 待做）
- [ ] `caching_aspect.rs`（S3 待做）
- [ ] `ratelimit_aspect.rs`（S3 待做）
- [ ] `circuitbreaker_aspect.rs`（S3 待做）
- [ ] `authorization_aspect.rs`（S3 待做）
- [ ] `validation_aspect.rs`（S3 待做）

---

### 5.2 spring-aop 工具切面（语义参考）

以下 spring-aop 工具切面在 vernal-aop 中由 aspect-std 对应切面覆盖或互补。
**不单独移植**，语义由 aspect-std 业务切面承载。

#### Spring API（Java）

```java
// spring-aop 调试工具切面
public class DebugInterceptor implements MethodInterceptor {
    public Object invoke(MethodInvocation invocation) throws Throwable {
        logger.debug("Invocation: " + invocation);
        return invocation.proceed();
    }
}

// spring-aop 性能监控切面
public class PerformanceMonitorInterceptor extends AbstractMonitoringInterceptor {
    protected Object invokeUnderTrace(MethodInvocation invocation, Log logger) throws Throwable {
        long start = System.nanoTime();
        try { return invocation.proceed(); }
        finally { logger.trace("Invocation took " + (System.nanoTime() - start) + "ns"); }
    }
}

// spring-aop 并发节流切面
public class ConcurrencyThrottleInterceptor extends ConcurrencyThrottleSupport
        implements MethodInterceptor {
    public Object invoke(MethodInvocation invocation) throws Throwable {
        // 限制并发数
    }
}
```

#### vernal-aop 对应

| spring-aop 工具切面 | vernal-aop 对应 | 说明 |
|:---|:---|:---|
| `DebugInterceptor` | `logging_aspect`（S3） | 日志记录 |
| `PerformanceMonitorInterceptor` | `timing_aspect`（S3） | 执行耗时 |
| `ConcurrencyThrottleInterceptor` | `ratelimit_aspect`（S3） | 并发限流 |
| `SimpleTraceInterceptor` | `logging_aspect`（S3） | 追踪日志 |
| `AbstractMonitoringInterceptor` | `timing_aspect` + `metrics_aspect`（S3） | 监控基类 |

---

## 六、运行时与集成约束

本章涵盖运行时对象（Invocation / Operation / InvocationPlan）、与现有体系的
集成约束、以及待补齐工作的路线图。

### 6.1 运行时核心对象

#### Operation —— 操作描述

```rust
/// 描述一次可被切面匹配的稳定操作身份及其声明元数据。
/// component 通常是组件类型或逻辑服务名，method 是方法或端点名称。
/// 相等性与 Hash 只使用 component + method，不包含标签和限定符。
pub struct Operation {
    component: Arc<str>,
    method: Arc<str>,
    metadata: Arc<OperationMetadata>,
}
```

| 对比 | aspect-rs `JoinPoint` | vernal-aop `Operation` |
|:---|:---|:---|
| 函数名 | `function_name: &'static str` | `method: Arc<str>` |
| 模块路径 | `module_path: &'static str` | `component: Arc<str>` |
| 元数据 | 无 | `metadata: OperationMetadata`（tags/qualifier） |

#### Invocation —— 调用对象

```rust
/// 一次被拦截调用的不可变身份与共享运行时状态。
/// 截止时间采用 Tokio Instant，取消采用 CancellationToken。
pub struct Invocation {
    id: InvocationId,
    operation: Operation,
    context: Arc<InvocationContext>,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
}
```

#### InvocationPlan —— 预编译调用计划

```rust
/// 针对一个操作预编译的有序拦截器链。
/// 计划构建后只保存不可变 Arc 切片，可被任意数量的 Tokio task 并发调用。
pub struct InvocationPlan {
    operation: Operation,
    interceptors: Arc<[Arc<dyn Interceptor>]>,
}
```

#### InvocationPlanCatalog —— 预编译目录

```rust
/// 保存应用上下文中全部预编译 AOP 调用计划。
/// 相同身份且元数据一致的 Operation 重复声明会被合并；元数据冲突由计划建造器拒绝。
pub struct InvocationPlanCatalog {
    plans: Arc<OnceLock<Arc<HashMap<Operation, InvocationPlan>>>>,
}
```

### 6.2 与 spring-aop 概念的不移植项

| spring-aop 概念 | 不移植原因 | vernal-aop 替代 |
|:---|:---|:---|
| `ProxyFactory` / CGLIB / JDK 代理 | Rust 无运行时字节码代理 | `#[aspect]` 过程宏 + InvocationPlan |
| `TargetSource`（17 类） | `InvocationTarget` 已覆盖 | 现有 `InvocationTarget` |
| `Advised` / `AdvisedSupport` | `Advised<T>` 已覆盖 | 现有 `Advised<T>` |
| XML 配置（17 类） | Rust 无 Spring XML | 过程宏 + vernal IoC |
| Scope 代理（5 类） | vernal-context 已覆盖 | vernal-context scope |
| AspectJ Weaver | Rust 原生 DSL 替代 | `parse_pointcut_expr()` |

### 6.3 引用约定文档

crate 选型依据、集成模式和硬约束见《Spring 组件替换约定》：

- **8.4 节（校验）**：`validator` crate 对标 `javax.validation` / Hibernate Validator，
  用于 `validation_aspect`（S3 待移植）。
- **8.5 节（AOP）**：`vernal-aop` 对标 `spring-aop`，`vernal-aspects` 对标内建切面
  （@Transactional / @Cacheable / @Async / @Scheduled），`vernal-macros` 对标
  过程宏（`#[derive(Component)]`、`#[intercept]`），`vernal-context-indexer` 对标
  linkme 分布式 slice 注册。

### 6.4 待补齐工作路线图

| 阶段 | 内容 | 优先级 | 预估工作量 |
|:---|:---|:---|:---|
| S2 | `#[aspect]` / `#[advice]` 过程宏 | P0 | 2 天 |
| S3 | 8 个 aspect-std 业务切面（异步化 + Rust crate） | P1 | 2.5 天 |
| P2 | `args()` / `this()` / `target()` Pointcut designator | P2 | 2 天 |

#### S2 过程宏（`#[aspect]` / `#[advice]`）

```rust
// aspect-rs 原版（同步）
#[aspect(LoggingAspect)]
fn save_user(user: User) -> Result<User> { ... }

#[advice(pointcut = "execution(pub fn *(..))", advice = "around", order = 10)]
fn log_around(pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> { ... }

// vernal-aop 移植版（异步）
#[aspect(LoggingAspect)]
async fn save_user(user: User) -> Result<User> { ... }

#[advice(
    pointcut = "execution(pub fn *(..)) && within(crate::api) || tag(secured)",
    advice = "around",  // 或 before / after / after_error
    order = 10
)]
async fn log_around(inv: Arc<Invocation>, next: Next) -> InvocationResult { ... }
```

目标文件结构：

```
vernal-aop/src/advice/
├── mod.rs                 # 宏模块声明
├── aspect_macro.rs        # #[aspect] attribute macro
└── advice_macro.rs        # #[advice] attribute macro
```

### 6.5 测试基线

#### 已完成（121 测试全部通过）

| 阶段 | 测试内容 | 数量 |
|:---|:---|:---|
| S1 | Aspect trait 四段式异步 | 5 |
| S1 | AspectError 三变体 + From 转换 | 7 |
| S1 | AspectAdapter 桥接 | 2 |
| S1 | PointcutExpr 解析（execution/within/and/or/not/tag/qualifier） | 17 |
| S1 | Pattern 匹配（Visibility/NamePattern/ExecutionPattern/ModulePattern） | 11 |
| S1 | PointcutMatcher + FunctionDescriptor | 7 |
| S5 | DefaultPointcutAdvisor / PointcutAdvisor / IntroductionAdvisor | 6 |
| parity | spring-aop 语义对齐 | 33 |
| 其他 | 集成测试 + 回归测试 | 33 |

#### 待做

| 阶段 | 测试内容 | 预估数量 |
|:---|:---|:---|
| S2 | `#[aspect]` / `#[advice]` 宏 + trybuild 负例 | 8+ |
| S3 | 8 个业务切面（caching/moka, circuitbreaker/failsafe） | 8+ |

### 6.6 成熟度状态

| 维度 | 当前状态 | 目标状态 |
|:---|:---|:---|
| 文件数 | 64 | 76+ |
| 行数 | 4649 | 6000+ |
| Advice 类型 | 4（异步四段式）+ Introduction | 4 + Introduction |
| Pointcut | 8 个具体 + 7 个 DSL 变体 | 8 + 10+ DSL |
| Pointcut 解析器 | 1（`parse_pointcut_expr`） | 1 |
| 过程宏 | 0 | 2（`#[aspect]` / `#[advice]`） |
| 业务切面 | 0 | 8（aspect-std） |
| 与 spring-aop 语义对标度 | ~80% | 85%+ |
| 测试数 | 121 | 140+ |

---

## 附录：Spring-aop 语义覆盖全景

| spring-aop 包 | 类数 | aspect-rs 覆盖 | vernal-aop 状态 | 说明 |
|:---|:---|:---|:---|:---|
| 根包（Advice/Pointcut/Advisor） | 24 | aspect-core | ✅ 已移植 | Aspect/Pointcut DSL/Advisor |
| framework（Proxy 工厂） | 50 | aspect-macros | 🚫 过程宏替代 | CGLIB/JDK 无 Rust 对等 |
| aspectj（AspectJ 表达式） | 42 | aspect-core/pointcut/ | ✅ 已移植 | Rust 原生 DSL 解析器 |
| support（Pointcut/Advisor 实现） | 31 | aspect-core/pointcut/ | ✅ 已移植 | Pattern 体系 |
| target（TargetSource） | 17 | 无 | 🚫 InvocationTarget 替代 | 已有等价物 |
| interceptor（工具切面） | 13 | aspect-std | ⬜ S3 待移植 | 业务价值更高 |
| config（XML/宏配置） | 17 | 无 | 🚫 过程宏替代 | Rust 无 Spring XML |
| scope（作用域代理） | 5 | 无 | 🚫 vernal-context 替代 | 已有等价物 |

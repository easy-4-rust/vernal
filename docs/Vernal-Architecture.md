# 句芒 · Vernal Framework Architecture

> **Purpose:** Define Vernal's brand semantics, system boundaries, IoC/AOP/
> ApplicationContext contracts, crate dependency rules, web and ecosystem
> integration strategy, and the refactoring boundary for ideas adopted from
> `tx-di`.
>
> **Architecture version:** 0.1.0<br>
> **Applicable code:** `0.0.0-dev` Phase 1–4 callable foundations<br>
> **Status:** Draft, awaiting architecture review<br>
> **Last updated:** 2026-07-24

## 1. Document control and status

### 1.1 Readers

| Reader | Primary sections | Expected outcome |
|:---|:---|:---|
| Vernal maintainers | 5–10, 14–17 | Kernel boundaries, contracts, delivery gates |
| Web adapter authors | 4, 11, 14 | Request context and middleware boundaries |
| Hutool-Rust / Sa-Token-Rust / Ddd4r maintainers | 4, 12 | Integration ownership and dependency rules |
| Test and security reviewers | 13–16 | Failure, security, and acceptance evidence |

### 1.2 Status labels

| Label | Meaning |
|:---|:---|
| `[Confirmed]` | Verified from the workspace, reviewed source, or current commands |
| `[Skeleton]` | Crate and dependency boundary exists; public behavior does not |
| `[Target]` | Proposed contract requiring review and implementation |
| `[Experimental]` | Requires a prototype or benchmark before commitment |
| `[Non-goal]` | Explicitly outside Vernal's responsibility |

### 1.3 Current state

- `[Confirmed]` The manifest declares Edition 2024, resolver 3, and MSRV
  1.85.0. The workspace passes `cargo +1.85.0 check --workspace --all-targets`
  locally; automated MSRV CI remains a target.
- `[Confirmed]` Six kernel/composition crates and fourteen web-related crates
  exist; four web foundation crates now expose callable behavior.
- `[Confirmed]` Every crate is `publish = false`; no crates.io or stable API
  claim is made.
- `[Confirmed]` `vernal-core` and `vernal-ioc` provide an explicit registry,
  deterministic graph planning, isolated containers, singleton/transient
  scopes, named/primary/all Trait bindings, and structured failures.
- `[Confirmed]` `vernal-aop` provides object-safe async Send and Local
  Around/Next execution planes, operation pointcuts, immutable plans, typed
  extensions, cancellation, and deadlines.
- `[Confirmed]` `vernal-context` provides a serialized Tokio lifecycle state
  machine, dependency-order initialize/start, cancellation, rollback,
  reverse idempotent shutdown, and context-local typed events.
  `VernalApplicationBuilder` registers the Tokio handle, application
  cancellation token, event bus, and precompiled AOP plan catalog before
  dependency-graph freezing.
- `[Confirmed]` `vernal-web`, `vernal-http`, `vernal-tower`, and
  `vernal-hyper` provide request scope, standard HTTP body frames/trailers,
  Tower lifecycle layers, and a real Hyper transport bridge.
- `[Confirmed]` `vernal-axum` provides native Router assembly and typed
  Context, component, and request-scope extractors; `vernal-actix-web` provides
  native Transform/Service middleware, body-bound Scope cleanup, matched
  resource operation identity, and strict Local-AOP;
  `vernal-rocket` provides managed state, request guards, and a body-aware
  fairing; `vernal-warp` provides native extension filters and a Tower Service
  body scope; `vernal-salvo` provides a native Hoop, typed Depot access, and
  frame/trailer-preserving body scope; `vernal-poem` provides native
  Middleware/Endpoint composition, typed extractors, body-bound Scope cleanup,
  matched-route operation identity, and fail-closed strict Around AOP;
  `vernal-ntex` provides native Middleware/Service composition,
  App State/Extension extractors, and body-bound Scope cleanup;
  `vernal-gotham` provides StateData, type-safe State access, Pipeline
  middleware, and frame/trailer-preserving body cleanup; `vernal-tide` provides
  native Middleware, typed Request Extension access, and reader-bound Scope
  cleanup; `vernal-tonic` provides a Context interceptor, typed Request
  extensions, `Status` mapping, and Tower composition.
- `[Confirmed]` `vernal-macros` provides `#[derive(Component)]` for explicit
  `Arc<T>`, `Arc<dyn Trait>`, and `Vec<Arc<dyn Trait>>` constructor injection,
  Singleton/Transient scope, default fields, and field qualifiers, with
  runtime and compile-fail tests. It uses neither linkme nor global
  auto-registration.
- `[Confirmed]` verbatim tx-di references now live under each crate's
  read-only `upstream/tx-di/` evidence directory instead of appearing as
  uncompiled parallel Store/App/global-registry implementations in production
  `src/`.
- `[Confirmed]` `#[component(aop)]` and `#[intercept]` connect context-local
  invocation plans, cancellation, and async component methods. Missing plans,
  cancellation, and return-type mismatches remain structured errors, without a
  global instance map.
- `[Target]` Broader method signatures, remaining consumer ecosystem bridges,
  benchmarks, and later production gates remain.

## 2. Brand meaning and architecture thesis

### 2.1 Dual brand

| Dimension | Content |
|:---|:---|
| Chinese brand | **句芒** |
| English brand | **Vernal** |
| Formal name | **Vernal Framework** |
| English description | **Vernal is a lightweight IoC, AOP and application context framework for Rust.** |
| Chinese description | **句芒是面向 Rust 生态的轻量级 IoC、AOP 与应用上下文框架。** |
| Brand line | **Vernal — Let components grow.** |
| Capability line | **Grow components. Weave capabilities.** |

句芒 evokes spring and the growth of living things; Vernal means “of or
relating to spring.” The metaphor is constrained by engineering rules:

```mermaid
flowchart LR
    Spring["Spring: awakening and growth"] --> Context["ApplicationContext: discover, start, close"]
    Root["Roots: dependency and nourishment"] --> IoC["IoC: explicit and deterministic resolution"]
    Weave["Weaving: cross-cutting capabilities"] --> AOP["AOP: ordered composition"]
    Seasons["Seasons: ordered change"] --> Lifecycle["Lifecycle: state and reverse cleanup"]
    Diversity["Living systems: bounded diversity"] --> Adapters["Adapters: coexist without contamination"]
```

### 2.2 Architecture in one sentence

**Vernal is a Rust-native framework composed of independent IoC and AOP
kernels plus an ApplicationContext that coordinates them; thin macros and
adapters connect the kernels to web frameworks and downstream ecosystems.**

### 2.3 Metaphor translated into rules

| Meaning | Engineering rule | Prohibited shortcut |
|:---|:---|:---|
| Growth, not fabrication | Build components from explicit dependencies | Arbitrary runtime string reflection |
| Weaving, not intrusion | Compose cross-cutting behavior around Invocation | Mutate business types to simulate AOP |
| Ordered seasons | Start in dependency order, stop in reverse | Unordered startup and abandoned tasks |
| Bounded diversity | Separate kernels, context, adapters, consumers | Put web/auth/ORM utilities into core |
| Renewal | Roll back failures; isolate and rebuild contexts | Unclearable process-global registries |

## 3. Drivers, constraints, and non-goals

### 3.1 Drivers

| ID | Driver | Priority | Architecture response |
|:---|:---|:---:|:---|
| D-001 | Any Rust project can use IoC or AOP independently | P0 | Mutually independent kernels |
| D-002 | Integrate Axum, Actix Web, Salvo, and Poem | P0 | Tower-first, native when needed |
| D-003 | Enable Hutool-Rust, Sa-Token-Rust, and Ddd4r | P0 | Consumer-owned bridges/starters |
| D-004 | Reuse proven `tx-di` ideas | P1 | Migration ledger and contract tests |
| D-005 | Remain Rust-native and diagnosable | P0 | Explicit types, errors, state, graph reports |
| D-006 | Use ecosystem standards without dependency pollution | P0 | Tokio-first; concrete web/ORM/config dependencies stay in integrations |

### 3.2 Hard constraints

1. `vernal-ioc` and `vernal-aop` do not depend on each other.
2. `vernal-core`, `vernal-aop`, and `vernal-context` may use Tokio and
   `tokio-util` task, synchronization, time, and cancellation primitives, but
   do not depend on concrete web, ORM, authentication, or configuration
   implementations.
3. Adapters point inward; kernels never depend on adapters.
4. Missing components, cycles, interception rejection, and shutdown failure
   return structured errors instead of panicking.
5. Contexts are instance-isolated; parallel contexts cannot overwrite one
   another.
6. “Zero cost,” “zero allocation,” and “Spring compatible” require measured,
   explicitly scoped evidence.

### 3.3 Non-goals

- `[Non-goal]` Reproduce Java reflection, CGLIB, or the complete historical
  BeanFactory API.
- `[Non-goal]` Build a router, HTTP server, ORM, authentication system, or
  distributed configuration product.
- `[Non-goal]` Make domain code depend on one framework's Request/Response.
- `[Non-goal]` Hot-swap arbitrary Rust types at runtime.
- `[Non-goal]` Ship a full “automatic configuration for everything” platform
  before a stable 1.0 kernel.

## 4. System context and ownership

```mermaid
flowchart TB
    App["Rust application"] --> Facade["vernal facade"]
    Facade --> Context["vernal-context"]
    Facade --> IoC["vernal-ioc"]
    Facade --> AOP["vernal-aop"]
    Macros["vernal-macros"] -. generates metadata .-> IoC
    Macros -. generates wrappers .-> AOP
    Web["Tower / Hyper / ten HTTP-RPC targets"] --> Adapters["vernal-* adapters"]
    Adapters --> Context
    Consumers["Hutool-Rust / Sa-Token-Rust / Ddd4r"] --> Bridges["consumer-owned bridges"]
    Bridges --> Facade
```

| Vernal owns | Vernal does not own | Owner |
|:---|:---|:---|
| Definitions, bindings, scopes, resolution | Domain rules | Application / Ddd4r |
| Interception contract, order, invocation chain | Authentication semantics | Sa-Token-Rust |
| Context lifecycle and local events | Routing, body, transport limits | Web framework |
| Framework integration ports | General utility algorithms | Hutool-Rust |
| Startup diagnostics and dependency graph | Deployment/config-center products | Infrastructure |

## 5. Source evidence and tx-di boundary

### 5.1 Reviewed evidence

| Project | Reviewed source | Confirmed finding |
|:---|:---|:---|
| `tx-di` | core component, registry, store, scope, topology, lifecycle, AOP, and intercept macro files | Typed metadata, topological construction, scopes, lifecycle, and interceptor chains exist |
| `Sa-Token-Rust` | Router flow, adapter contracts, and ten Web/RPC plugin families | One auth flow is reused through framework request/response ports |
| `Ddd4r` | Root manifest and implementation plan | Target stack needs a request-context bridge while retaining DDD/CQRS ownership |
| `Hutool-Rust` | AOP symbols and Reqwest-based HTTP client contracts | Utility/client interception is reusable evidence, but it does not provide server framework integration |

This is a local source snapshot from 2026-07-24, not evidence that any consumer
already integrates Vernal.

### 5.2 Adopted ideas

| tx-di mechanism | Vernal decision | Target crate |
|:---|:---|:---|
| Explicit `Component::Deps` | Retain describable constructor dependencies; redesign the stable contract | `vernal-ioc` |
| Link-time component metadata | Evaluate as an optional registration frontend | macros / optional adapter |
| `TypeId` plus erased store | Keep typed entrypoints and constrain erasure | `vernal-ioc` |
| Kahn topological ordering | Rebuild as a deterministic, testable planner | `vernal-ioc` |
| `debug_registry()` log table | Upgrade to serializable read-only snapshots that reuse the frozen plan | `vernal-ioc` / `vernal-context` |
| Singleton / Prototype | Evolve to Singleton / Transient / Scope SPI | `vernal-ioc` |
| Lifecycle hooks | Move to a Context-owned state machine | `vernal-context` |
| Forward before / reverse after | Preserve stack order through a true Around chain | `vernal-aop` |
| Generated interception wrapper | Keep compile-time generation; remove hard-coded paths and panic | `vernal-macros` |

### 5.3 Mandatory redesigns

| Current tx-di design | Risk | Vernal response |
|:---|:---|:---|
| Core mixes config, tracing, utilities, and shared errors | Unrelated concerns expand the kernel | Keep Tokio foundations; move config formats, logging implementations, and utilities outward |
| Global `HashMap<usize, Arc<InterceptorChain>>` | Address reuse, cleanup, locking, context isolation | Chain owned by wrapper/definition/context |
| Macro panics on missing chain or rejected before | Failure cannot compose | Structured caller-visible errors |
| Every argument becomes a Debug string | Secret leakage, allocation, lost type | No values by default; explicit redacted opt-in |
| `after` changes only a result description | Not true Around semantics | Introduce `Next`/continuation |
| Lifecycle owns Tokio tasks without one state machine | Cancellation and rollback semantics scatter | Tokio-native context with one lifecycle state machine |
| Global link-time registry is the only entrypoint | Weak isolation and dynamic assembly | Explicit `RegistryBuilder` baseline |

## 6. Architecture decisions

| ADR | Decision | Rationale | Rejected | Reversal condition |
|:---|:---|:---|:---|:---|
| ADR-001 | Publish IoC and AOP independently | Maximum foundational reuse | One behavioral mega-core | An unavoidable circular contract appears |
| ADR-002 | Context composes kernels | Keep kernels clean | Put lifecycle/config/events in IoC | Composition cannot stay type-safe |
| ADR-003 | Explicit registry is authoritative | Isolation and testability | Process-global registry | Global model proves safe and unloadable |
| ADR-004 | Around/Next interception | Express before/after/error/short-circuit | before/after only | Equivalent lower-cost model is proven |
| ADR-005 | Tower-first integration | Reuse a shared Rust service abstraction | Duplicate each pipeline | A framework cannot preserve semantics |
| ADR-006 | Separate adapters and bridges | Isolate dependency/version churn | All framework features in facade | Stable ecosystem ABI emerges |
| ADR-007 | Tokio-first runtime | Reuse the server ecosystem's tasks, synchronization, cancellation, and time | Custom runtime abstraction or multi-executor parity | Tokio stops being the target ecosystem standard |

## 7. Layers and crate dependencies

```mermaid
flowchart TB
    Consumer["Consumer<br/>Hutool-Rust / Sa-Token-Rust / Ddd4r / apps"]
    Adapter["Integration<br/>Tower / Web / bridges"]
    Facade["Facade<br/>vernal"]
    Context["Composition<br/>vernal-context"]
    IoC["Kernel<br/>vernal-ioc"]
    AOP["Kernel<br/>vernal-aop"]
    Core["Contracts<br/>vernal-core"]
    Macros["Compile-time frontend<br/>vernal-macros"]

    Consumer --> Adapter
    Consumer --> Facade
    Adapter --> Context
    Adapter --> IoC
    Adapter --> AOP
    Facade --> Context
    Facade --> IoC
    Facade --> AOP
    Facade --> Macros
    Context --> IoC
    Context --> AOP
    IoC --> Core
    AOP --> Core
    Macros -. generated contracts .-> IoC
    Macros -. generated wrappers .-> AOP
```

Forbidden directions:

```text
core ─X→ ioc / aop / context / web
ioc  ─X→ aop / context / concrete web / ORM
aop  ─X→ ioc / context / concrete web / ORM
context ─X→ concrete web framework
adapter A ─X→ adapter B
```

## 8. IoC kernel

### 8.1 Model

| Concept | Target responsibility |
|:---|:---|
| `ComponentKey` | Context-local identity: `TypeId` plus optional qualifier |
| `TraitKey` | Trait `TypeId` plus optional qualifier used for binding selection |
| `TraitBinding` | Type-safe upcast from the same concrete component `Arc` to `Arc<dyn Trait>` |
| `ComponentDefinition` | Constructor, dependencies, scope, order, lifecycle metadata |
| `RegistryBuilder` | Explicitly register definitions, then freeze |
| `GraphPlanner` | Validate missing/ambiguous/cyclic dependencies; create deterministic plan |
| `Container` | Resolve components and own instance/scope state |
| `Scope` | Define caching and construction semantics |
| `Resolver` | Restricted construction-time resolution view |

### 8.2 Build flow

```mermaid
sequenceDiagram
    participant A as Application
    participant R as RegistryBuilder
    participant G as GraphPlanner
    participant C as Container
    participant F as ComponentFactory

    A->>R: register definitions + trait bindings
    R->>G: freeze and validate
    alt missing, ambiguous, or cyclic
        G-->>A: structured GraphError
    else valid graph
        G-->>C: deterministic BuildPlan
        A->>C: resolve root
        C->>F: construct resolved dependencies
        F-->>C: owned instance
        C-->>A: typed handle
    end
```

Phase 1 implements only per-container `Singleton` and per-resolution
`Transient`. Request/task/tenant scopes arrive through a later Scope SPI and do
not introduce HTTP concepts into the kernel.

### 8.3 Tokio and framework-native components

Vernal does not require ecosystem objects to be wrapped in framework-specific
bean types. Any `Send + Sync + 'static` value can be registered directly,
including Tokio synchronization primitives and task handles, HTTP clients,
database pools, message clients, Tower services, framework state, and
application services.

Being usable as a component does not make its crate a kernel dependency. The
application or a `vernal-*` adapter imports the concrete framework and supplies
the `ComponentDefinition`; IoC sees only Rust types, factories, dependencies,
and scopes.

Prebuilt native objects can be added with
`ComponentDefinition::shared_value` or `ComponentDefinition::shared_arc`
without introducing wrapper types. `shared_arc` resolves the original
`Arc<T>`, not an `Arc<Arc<T>>`. Such values are shared by all containers
created from the same registry; use factory-based
`ComponentDefinition::singleton` when each container needs an isolated
instance.

Vernal is explicitly Tokio-first. Kernel crates may depend on Tokio directly
when tasks, asynchronous synchronization, time, or cancellation require it;
the framework will not invent a second runtime SPI. The IoC contract tests
register a native `tokio::runtime::Handle` and use it to run a real Tokio task.

### 8.4 Named, primary, and multiple Trait bindings

Trait bindings join the existing immutable Registry and graph rather than
enabling tx-di's former parallel Store:

```mermaid
flowchart LR
    DEF["ComponentDefinition<C>"] --> TARGET["Arc<C>"]
    BIND["TraitBinding<T, C>"] --> DEF
    BIND --> KEY["TraitKey<br/>TypeId + qualifier"]
    KEY --> SELECT["unique / named / primary / all"]
    SELECT --> TARGET
    TARGET --> TRAIT["Arc<dyn T><br/>same allocation"]
```

- `TraitBinding::new::<dyn T, C, _>` is checked by the Rust compiler.
- Unqualified single-value injection requires one candidate or exactly one
  primary candidate.
- A qualifier selects one named binding; duplicate names fail during
  registration.
- `resolve_all_traits` and `Vec<Arc<dyn T>>` preserve binding registration
  order and return an empty collection for zero implementations.
- Trait dependencies become real edges to their concrete targets and
  participate in missing-target, cycle, and startup-order validation.
- `register_bundle` preflights definitions and bindings together and leaves no
  partial module on failure.
- The Component derive emits the same restricted Resolver calls and explicit
  metadata for `Arc<dyn T>`, field qualifiers, and `Vec<Arc<dyn T>>`.

### 8.5 Failure contract

| Error | Retry | Required diagnostic |
|:---|:---:|:---|
| Duplicate definition | No | Both origins and key |
| Missing dependency | No | Full dependency path |
| Ambiguous binding | No | Candidates and qualifiers |
| Cycle | No | Short readable cycle |
| Construction failure | Source-dependent | Component, phase, source chain |
| Closed scope | No | Scope identity and close reason |

## 9. AOP kernel

### 9.1 Two API levels

The AOP kernel serves:

1. a typed generic API for library authors and static composition;
2. a controlled type-erased heterogeneous chain for Context and macros.

Both must share ordering, short-circuit, error, and context propagation
semantics.

### 9.2 Invocation contract

Target metadata includes method identity, declaring type, tags/qualifier,
read-only context extensions, deadline, cancellation, and nesting depth.
Argument values are not captured by default. Explicit capture must support
field-level redaction. Business errors retain their source rather than becoming
strings.

### 9.3 Around flow

```mermaid
sequenceDiagram
    participant C as Caller
    participant I1 as Interceptor 1
    participant I2 as Interceptor 2
    participant T as Target

    C->>I1: invoke(context, next)
    I1->>I2: next(context)
    I2->>T: next(context)
    T-->>I2: result
    I2-->>I1: transformed/observed result
    I1-->>C: final result
```

Lower `order` enters first and exits last. Equal order preserves registration
order. Pointcuts compile into immutable `InvocationPlan` values during
high-level application construction; context refresh consumes the frozen
catalog.

### 9.4 Two execution planes, one semantic model

Rust web frameworks do not agree that every Service future is `Send`. Vernal
therefore exposes two explicit execution planes instead of weakening either
contract:

| Plane | Interceptor chain | Target and future | Erased value |
|:---|:---|:---|:---|
| Thread-safe | `Interceptor` / `Next` / `InvocationPlan` | `Arc` target and `Send` future | `Box<dyn Any + Send + Sync>` |
| Worker-local | `LocalInterceptor` / `LocalNext` / `LocalInvocationPlan` | `Rc` target and non-`Send` future | `Box<dyn Any>` |

Both planes share `Operation`, `Invocation`, pointcut semantics, deterministic
ordering, short-circuit behavior, Tokio cancellation/deadline handling, and
request-context snapshots. Local interceptor declarations remain `Send +
Sync`, so `LocalInvocationPlanCatalog` is still a native ApplicationContext
component and its counts appear separately in startup diagnostics. A consumer
that needs both planes deliberately implements and registers both contracts;
Vernal never pretends that an arbitrary Send interceptor automatically
supports a local target.

### 9.5 No instance-pointer map

Vernal does not use `self as *const Self as usize` as durable identity:

- static composition uses `Advised<T>` owning target and chain;
- context components keep plans in immutable definitions;
- web adapters obtain plans from the context and pass request-scoped
  extensions into Invocation.

Ownership and cleanup therefore follow the actual wrapper/context lifecycle.

### 9.6 Method macro safety contract

The first weaving frontend intentionally exposes a narrow Rust contract:

1. `#[component(aop)]` requires explicit `Arc<InvocationPlanCatalog>` and
   `Arc<CancellationToken>` fields.
2. An intercepted method is `async fn` with `self: Arc<Self>`.
3. Arguments are owned and the return type is
   `Result<T, InvocationError>`.
4. A one-shot `Mutex<Option<Tuple>>` transfers arguments into the target
   without requiring business values to implement `Clone`.
5. Missing plans, repeated target advancement, cancellation, and return-type
   mismatches are structured errors rather than panics.

The owned receiver and arguments let `InvocationTarget` produce a safe
`'static` future across `.await`. `Next` is a one-shot continuation; a custom
interceptor that advances it twice receives `TargetAlreadyInvoked` instead of
silently repeating a side-effecting business method.

```mermaid
sequenceDiagram
    participant Caller
    participant Macro as "intercept wrapper"
    participant Component as "AopComponent"
    participant Catalog as "Context-local plan catalog"
    participant Plan as "InvocationPlan"
    participant Target as "Business method"

    Caller->>Macro: Arc<Service>.method(owned arguments)
    Macro->>Component: read catalog and cancellation
    Macro->>Catalog: find plan by Operation
    Catalog-->>Macro: Arc<InvocationPlan>
    Macro->>Plan: invoke(context, one-shot target)
    Plan->>Target: advance Around chain
    Target-->>Plan: Result<T, InvocationError>
    Plan-->>Macro: type-erased value
    Macro-->>Caller: recover T or structured error
```

## 10. ApplicationContext and lifecycle

The context composes registry, container, AOP plans, local typed events,
rollback, reverse cleanup, and read-only diagnostics. It directly uses Tokio
tasks, synchronization, time, cancellation, and signals. Configuration
formats, web servers, and external configuration centers remain adapters whose
native objects may still be registered as ordinary components.

Before graph freezing, `VernalApplicationBuilder` automatically registers the
current `tokio::runtime::Handle`, the application `CancellationToken`, the
context-local `EventBus`, and the immutable `InvocationPlanCatalog`. Components
declare them with ordinary `depends_on::<T>()` metadata, and the context plus
container receive the same `Arc<T>` instances. The lower-level
`Registry -> ApplicationContextBuilder` path remains available for library
composition that does not want runtime capture or built-in registration.

```mermaid
sequenceDiagram
    participant App as "Application assembly"
    participant Builder as "VernalApplicationBuilder"
    participant AOP as "InvocationPlanBuilder"
    participant Graph as "RegistryBuilder"
    participant Context as "ApplicationContext"

    App->>Builder: register definitions, advisors, operations
    Builder->>AOP: compile plan catalog
    Builder->>Graph: register Tokio/cancellation/events/catalog
    Builder->>Graph: freeze and validate complete graph
    Graph-->>Builder: Registry
    Builder->>Context: create with identical shared resources
    Context-->>App: refresh/start
```

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Refreshing: refresh
    Refreshing --> Refreshed: graph and plans valid
    Refreshing --> Failed: validation/build failure
    Refreshed --> Starting: start
    Starting --> Ready: required components started
    Starting --> RollingBack: startup failure
    Ready --> Draining: close requested
    RollingBack --> Closed: reverse cleanup
    Draining --> Closed: reverse shutdown
    Failed --> Closed: clean partial state
    Closed --> [*]
```

Lifecycle order:

```text
register → freeze → validate graph and pointcuts
→ construct singletons → initialize → start → Ready
→ drain → reverse stop → release scopes
```

Failures record completed steps and roll back only successful components.
`close()` is idempotent.

## 11. Web, HTTP, and framework integration

Vernal reuses Spring's separation of responsibilities, not its JVM product
names. Rust web frameworks generally expose async handlers, while streaming is
a capability of HTTP bodies, RPC calls, SSE, and WebSocket connections rather
than a separate application model.

| Layer | Vernal crate | Contract | Boundary |
|:---|:---|:---|:---|
| Web application | `vernal-web` | Request context, request scope, handler invocation, extraction, validation, error mapping | No transport or framework types |
| HTTP protocol | `vernal-http` | Request, response, body frames, streaming, cancellation, backpressure | Uses Rust `Future`/`Stream`; owns no runtime |

```mermaid
flowchart TD
    Core["Invocation / ApplicationContext"]
    Web["vernal-web common contracts"]
    Http["vernal-http protocol contracts"]
    Tower["vernal-tower"]
    Hyper["vernal-hyper"]
    HttpAdapters["Nine HTTP framework adapters"]
    Tonic["vernal-tonic RPC adapter"]

    Web --> Core
    Http --> Web
    Tower --> Web
    Hyper --> Http
    HttpAdapters --> Http
    HttpAdapters --> Tower
    Tonic --> Tower
```

The versioned coverage set is owned by
[`web-integration-manifest.toml`](../web-integration-manifest.toml):

| Priority | Framework | Crate | Protocol/capabilities | Target mechanism |
|:---:|:---|:---|:---|:---|
| 1 | Axum | `vernal-axum` | HTTP, body streaming, Tower | Tower Layer, Service, extractor/context bridge |
| 2 | Actix Web | `vernal-actix-web` | HTTP, body streaming, strict Local-AOP | Transform/Service middleware, app data, matched resource pattern |
| 3 | Rocket | `vernal-rocket` | HTTP request/response, optional streaming | Fairing, request guard, managed state |
| 4 | Warp | `vernal-warp` | HTTP, body streaming | Filter composition and rejection mapping |
| 5 | Salvo | `vernal-salvo` | HTTP, body streaming | Handler, Hoop, Depot scope |
| 6 | Poem | `vernal-poem` | HTTP, body streaming, strict AOP | Middleware, Endpoint, request data |
| 7 | Ntex | `vernal-ntex` | Network HTTP, body streaming | Service/middleware and worker-local state |
| 8 | Gotham | `vernal-gotham` | HTTP request/response | State middleware and handler pipeline |
| 9 | Tide | `vernal-tide` | HTTP, body streaming | Middleware, request state, endpoint |
| 10 | Tonic | `vernal-tonic` | gRPC / RPC streaming | Tower Service, interceptor, extensions |

Tower and Hyper are foundations and do not consume two slots in the ten-target
set. Tonic is explicitly RPC rather than an HTTP router. The selection is a
versioned coverage priority derived from the reviewed local integration
superset and current registry presence, not an objective universal popularity
ranking.

The workspace has fourteen web-related crates: `vernal-web`, `vernal-http`,
Tower/Hyper, and ten adapters. The four foundations now provide callable
request-scope, HTTP frame/trailer, cancellation, Tower lifecycle, AOP
invocation, and Hyper
transport behavior. Axum adds native Router assembly and typed extractors;
Actix Web adds App Data/Extensions, body-aware middleware, and strict Around
interception for its `Rc`-based non-`Send` services through Vernal Local-AOP.
Strict middleware wraps a concrete Resource after matching, uses the
low-cardinality resource pattern as operation identity, fail-closes missing
metadata/plans, and preserves native Actix errors. Rocket adds
managed state, request guards, and a body-aware fairing; Warp adds extension
filters and an official Tower Service lifecycle; Salvo adds a Hoop, typed Depot
access, and frame/trailer-preserving body lifecycle; Poem adds
Middleware/Endpoint, request-extension extractors, body lifecycle integration,
and strict Around AOP using matched low-cardinality `PathPattern` metadata;
Ntex adds native Middleware/Service, App State/Extensions, typed
  extractors, and response-body lifecycle integration; Gotham adds native
  StateData, type-safe State access, Pipeline middleware, and a frame/trailer
  body lifecycle; Tide adds native Middleware, typed Request Extension access,
  and reader-bound Scope cleanup; Tonic adds native Request/Metadata/Status and
  Tower integration.
The detailed contract is
[Vernal Web Architecture](./Vernal-Web-Architecture.md).

## 12. Consumer integrations

### 12.1 Hutool-Rust

Hutool-Rust remains a utility library. Its Reqwest-based HTTP client
interceptors may use `vernal-aop`, but Hutool-Rust must not own server
ApplicationContext or become a Vernal kernel dependency.

The local Hutool-Rust checkout now contains a consumer-owned, unpublished
`hutool-vernal` bridge pinned to a verified Vernal Git revision. It atomically
registers Hutool `HttpConfig` and the Tokio/Reqwest `HttpClient` as
container-local singletons, exposes explicit URL/SSRF policy selection, and
keeps the configuration edge visible to Vernal graph validation. Its runtime
test proves singleton resolution, duplicate-bundle rejection, and local-target
rejection before network I/O. Clean remote integration remains pending because
that checkout is concurrently undergoing a separate one-object-per-file
history/refactor stream.

### 12.2 Sa-Token-Rust

Sa-Token-Rust is the only retained security integration target. Vernal will
register its manager/runtime graph as explicit components, compose its shared
authentication flow as framework middleware or interceptors, and propagate the
result through request scope. Token, session, role, permission, cookie, and
401/403 semantics remain owned by Sa-Token-Rust.

`sa-token-vernal` is now implemented in the Sa-Token-Rust repository as a
consumer-owned, unpublished bridge. It pins a verified Vernal Git revision,
adapts `HttpRequestSnapshot` to `SaRequest`, projects authenticated roles into
`SecurityPrincipal`, and runs downstream futures inside request-level
`SaTokenContext`. `SaTokenComponents` additionally preserves the caller's exact
`Arc<SaTokenManager>`, bridge, and policy identities, atomically installs all
three as a validated component graph, and registers an authentication and
authorization Advisor. `VernalSaTokenInterceptor` authenticates, then enforces
operation-scoped all/any role and permission requirements, including Sa-Token
global and prefix wildcard semantics, across the complete Tokio invocation
future. Anonymous protected calls return 401, authenticated but insufficient
calls return 403, and backend failures remain internal 500 errors. Axum, Poem,
and Tonic adapters translate these `WebFailure` values into native HTTP/gRPC
failures. `PathAuthConfig` remains the sole source of path login policy. Its
existing ten plugin families remain input evidence for the Vernal adapter
matrix, not code that Vernal silently vendors.

### 12.3 Ddd4r

Ddd4r now owns an unpublished, consumer-side `ddd4r-vernal` bridge pinned to a
verified Vernal Git revision. It atomically installs the caller's native
`Registry` and `DefaultCommandBus` while preserving their exact `Arc`
identities, producing an explicit
`Registry + DefaultCommandBus -> VernalDdd4rBridge` graph.

At each asynchronous entry point, the bridge creates an isolated shallow
registry snapshot and enters Ddd4r's own Tokio task-local `ContextScope`.
Request identities, tenants, transactions, and repository overrides remain
visible across `.await` without leaking after scope exit. Vernal composes
services and cross-cutting contracts; Ddd4r retains aggregates, domain events,
CQRS, repositories, outbox, and transaction semantics, and `ddd4r-core` does
not depend on Vernal.

```mermaid
flowchart LR
    Components["Ddd4rComponents"] --> VC["Vernal Context"]
    VC --> Registry["Ddd4r Registry"]
    VC --> Bus["DefaultCommandBus"]
    Registry --> Bridge["VernalDdd4rBridge"]
    Bus --> Bridge
    Bridge -->|"snapshot per async entry"| Scope["Tokio ContextScope"]
    Scope --> Domain["Repository / Event / Runtime facades"]
```

A real Tokio test in an isolated dependency graph proves native `Arc` identity,
task-local resolution after `yield_now().await`, request-service cleanup after
scope exit, and atomic duplicate-bundle rejection. Clippy with `-D warnings`
and Rustdoc also pass for the target crate. Full Ddd4r workspace verification
remains blocked by a pre-existing, currently unavailable `rbatis-r2dbc` Git
revision; this is neither reported as a bridge failure nor as a passing
workspace gate.

## 13. Security, privacy, and global state

| Risk | Default policy | Acceptance |
|:---|:---|:---|
| Sensitive invocation arguments | Do not capture values by default | Redaction and opt-in tests |
| Faulty interceptor blocks a chain | Deadline, cancellation, error boundary | Timeout/cancellation tests |
| Cross-context leakage | No process-global mutable authority | Parallel isolation tests |
| Macro exposes private fields | Generate minimum metadata | trybuild and token snapshots |
| Feature supply-chain growth | Independent adapters, minimal defaults | cargo tree / deny |
| Unsafe in workspace | `unsafe_code = "forbid"` | Compile/Clippy gate |

The unsafe rule covers Vernal-owned source, not the entire third-party graph.

## 14. Errors, reliability, and diagnostics

| Category | Examples | Caller action |
|:---|:---|:---|
| Definition | Duplicate, invalid qualifier | Fix registration |
| Graph | Missing, ambiguous, cyclic | Fix dependency relation |
| Resolution | Constructor failure, closed scope | Inspect source or stop use |
| Interception | Rejection, pointcut, chain failure | Follow business error contract |
| Lifecycle | Init/start/close failure | Roll back or report degraded |
| Adapter | Conversion or missing context | Return stable native error |

A context refresh now produces a serializable, read-only, redacted report
containing version/features, definition and scope counts, graph summary,
pointcut matches, lifecycle timing/failures, adapter state, warnings, and
unused definitions.

```mermaid
flowchart LR
    Registry["Registry<br/>definitions + bindings + BuildPlan"]
    Catalog["InvocationPlanCatalog"]
    Static["Static diagnostics<br/>features / adapters / external / warnings"]
    Lifecycle["Context state machine<br/>warm-up / resolve / init / start / stop"]
    Snapshot["RegistrySnapshot<br/>owned read-only value"]
    Report["StartupReport<br/>owned redacted value"]
    Output["Serde serializer<br/>logs / admin endpoints / tests"]

    Registry -->|"reuse validated order"| Snapshot
    Snapshot --> Report
    Catalog -->|"plan and interceptor slot counts"| Report
    Static --> Report
    Lifecycle -->|"phase, outcome, microseconds"| Report
    Report --> Output
```

The implemented contract is:

- `Registry::snapshot()` follows the real dependency-first build order without
  rerunning graph planning or exposing factories, upcast closures, instances,
  or addresses.
- `ApplicationContext::startup_report().await` returns an owned clone that
  later start/close operations cannot mutate.
- warm-up, component resolution, initialize, start, and stop record stable
  phases, subjects, outcomes, and microsecond durations.
- raw error chains remain available through `ContextError::source`; the report
  type has no `error_message` field.
- features and warnings use static names/codes; adapters and external
  dependencies accept only a name and fixed `DiagnosticState`, not connection
  strings, tokens, or arbitrary error details.
- unused definitions cannot be inferred safely from “no incoming edges”; the
  collection remains empty until exact resolution tracking exists. Adapter
  auto-discovery is likewise delegated to future integration-crate wiring;
  applications can register current states explicitly.

## 15. Verification and acceptance

| Level | Required evidence |
|:---|:---|
| Static | Crate direction, Tokio feature budget, and no concrete Web/ORM implementation in generic kernels |
| Unit | Graph, qualifier, ordering, scope, state machine |
| Property | Deterministic planning and cycle detection for arbitrary graphs |
| Compile | Macro diagnostics, bounds, lifetimes, generics |
| Concurrency | Once-only singleton, context isolation, close races |
| Contract | Typed and dynamic AOP have identical semantics |
| Web conformance | Equivalent policy result across frameworks |
| Consumer integration | No reverse dependency from bridges |
| Benchmark | Measured resolution, chain depth, startup graph only |

Phase 1 minimum acceptance:

1. Deterministically plan a 1,000-node DAG.
2. Missing, ambiguous, and cycle errors contain readable paths.
3. Parallel containers do not share singleton instances.
4. `cargo tree` proves no concrete web or ORM framework in `vernal-ioc`;
   Tokio is allowed when needed.
5. Normal failures use `Result`, not panic.

As of 2026-07-24, all five items have local evidence: 24 IoC contract tests
cover a 1,000-node graph, missing/ambiguous/cycle paths, singleton isolation
across two concurrent containers, transient creation, qualifiers, hidden
dependency rejection, native-value registration, a real task spawned through
an injected Tokio handle, named/primary/all Trait bindings, empty sets,
missing targets, Trait cycles, naming conflicts, batch atomicity, and
deterministic Registry serialization without factories or instance addresses.
`register_all` atomically installs definition-only batches; `register_bundle`
atomically commits definitions and bindings together. Tokio remains a
contract-test dependency for IoC rather than runtime state in its resolution
hot path.

The Phase 2 AOP kernel additionally has eight contract tests for
ordered entry/reverse exit, short circuit, result/error transformation, typed
context across `.await`, cancellation/deadline, pointcut filtering, and
64-task concurrent reuse, plus deduplicated plan-catalog compilation. The macro
frontend additionally has four runtime tests for singleton Component
injection, transient construction, Trait Object injection, and context-local
intercepted invocation, plus four compile-fail cases for invalid component
fields, invalid collection qualifiers, non-async methods, and borrowed
receivers. Phase 2 now has a callable loop, while broader
signatures, diagnostic coverage, benchmarks, and stability guarantees remain
open.

The Phase 3 kernel has eleven contract tests for dependency-order
startup, reverse shutdown, initialize/start rollback, invalid transitions,
idempotent close, concurrent close serialization, and context-local typed
event isolation, plus runtime-unavailable diagnostics, same-instance injection
of the four built-in resources, and owned/redacted serialization of successful
and failed startup reports.

## 16. Delivery roadmap

| Phase | Deliverable | Exit evidence |
|:---|:---|:---|
| 0 | Brand, bilingual README, architecture, buildable skeleton | Docs and Cargo gates pass |
| 1 | Core + IoC minimum loop | Registry, graph, scope, resolution tests |
| 2 | AOP + macros | Around, order, pointcut, trybuild tests |
| 3 | ApplicationContext | Lifecycle, events, rollback, shutdown tests |
| 4 | Web/HTTP contracts, Tower/Hyper, and ten adapters | Cross-framework conformance |
| 5 | Three ecosystem bridges | Consumer examples and dependency checks |
| 6 | Preview release | MSRV, SemVer, security, packaging, docs |

No phase is complete merely because a crate exists or `cargo check` is green.

## 17. Risks and open decisions

| ID | Risk / open decision | Impact | Validation |
|:---|:---|:---|:---|
| R-001 | Object-safe async Around allocation cost | AOP performance | Benchmark the implemented boxed-future path |
| R-002 | Macro support for impl/trait/async methods | Usability | trybuild matrix |
| R-003 | Cross-platform link-time registration | Portability | Linux/macOS/Windows CI |
| R-004 | Request-scope cancellation differences | Resource safety | Cross-framework failure tests |
| R-005 | Spring terminology overwhelms Rust API style | Maintenance | API review and Rust guidelines |
| R-006 | Premature zero-cost claims | Trust | Measure static and dynamic paths separately |

## 18. Definition of architecture done

- [x] IoC and AOP can be depended on, built, and used independently.
- [x] Tokio usage and feature budgets are explicit; Context does not leak
  configuration formats or concrete web/ORM types into generic kernels.
- [ ] Graph, interception, and lifecycle include success/failure/rollback tests.
- [ ] No pointer-address chain map, normal-flow panic, or hidden cross-context state.
- [ ] Web adapters pass one conformance suite while preserving native semantics.
- [x] Hutool-Rust, Sa-Token-Rust, and Ddd4r boundaries are proven by consumer examples.
- [ ] English and Chinese docs share commands, crate names, status, and diagrams.
- [ ] SemVer, MSRV, security policy, and registry verification precede release.

---

**Document version:** 0.1.0<br>
**Last updated:** 2026-07-24<br>
**Status:** Draft, awaiting architecture review

# 句芒 · Vernal Web Integration Architecture

> **Vernal Framework**  
> **Grow components. Weave capabilities.**

**Status:** Architecture draft  
**Baseline:** 2026-07-24  
**Scope:** `vernal-web`, `vernal-http`, `vernal-web-testkit`, Tower/Hyper
foundations, and ten adapters

[简体中文](./Vernal-Web-Architecture.zh_CN.md) | [Back to README](../README.md)

## 1. Current facts

The workspace contains fifteen web-related crates:

- two shared contracts: `vernal-web` and `vernal-http`;
- one shared conformance utility: `vernal-web-testkit`;
- two foundations: `vernal-tower` and `vernal-hyper`;
- ten adapters: Axum, Actix Web, Rocket, Warp, Salvo, Poem, Ntex, Gotham,
  Tide, and Tonic.

The four shared/foundation crates now expose callable contracts:

- `vernal-web` owns typed request context, request scope, invocation bridging,
  security-principal carriage, and stable problem details. Its request scope
  delegates directly to IoC `ScopeContext`, and business components may declare
  `scope = WebRequestScope`;
- `vernal-web-testkit` gives every adapter the same application-context,
  scope-identity, and component-`Arc` binding assertions;
- `vernal-http` uses standard `http`/`http-body` types, preserves frames and
  trailers, supports cancellation, and only buffers through an explicit limit;
- `vernal-tower` injects `ApplicationContext` and closes request scopes after
  body completion, upstream error, or dropped request futures;
- `vernal-hyper` converts `Incoming` into the same frame stream without
  buffering and is verified over a real TCP/HTTP connection.

Axum depends on Axum 0.8 and implements native Router assembly plus typed
Context, component, and request-scope extractors. Actix Web uses the
MSRV-compatible 4.11/actix-http 3.11 line and implements native
Transform/Service middleware, App Data/Extension extractors, and body-bound
Scope cleanup. Rocket 0.5.1 implements managed state, request guards, a
request/response body-aware fairing, and strict Send-AOP around native Route
handlers while preserving Success/Error/Forward Outcomes. Warp 0.4.3 combines native `warp::ext`
filters with the official `warp::service` Tower boundary for typed extraction,
full body lifecycle, and fail-closed strict Send-AOP using an explicit route
pattern. Salvo uses 0.85.0, the last release compatible with
Rust 1.85, and implements a native Hoop, typed Depot access, and a
frame/trailer-preserving body scope plus strict Send-AOP over the complete
Handler chain. Poem uses the MSRV-aligned 3.1.12 release
and implements native Middleware/Endpoint composition, request extension
extractors, and body-bound Scope cleanup. Tonic uses the MSRV-compatible 0.12
line and implements a Context interceptor, typed Request extensions,
`GrpcMethod` routing metadata, stable `Status` mapping, and Tower composition.
Ntex uses the Rust-1.85-compatible 2.18.0 line and implements native
Middleware/Service composition, App State/Extension extractors, and a
MessageBody-bound request scope. Its strict Local-AOP path uses an explicit
resource pattern and a borrowed worker-local target. Gotham 0.8 implements
native StateData, type-safe State access, Pipeline middleware,
frame/trailer-preserving body cleanup, and strict Send-AOP over the complete
Pipeline Chain with an explicit route pattern. Tide 0.17.0-beta.1 implements native Middleware, typed Request
Extension access, response-reader-bound Scope cleanup, and strict Send-AOP over
borrowed `Next` on Vernal's Tokio runtime.

The versioned selection is recorded in
[`web-integration-manifest.toml`](../web-integration-manifest.toml).

## 2. Goals and non-goals

### 2.1 Goals

1. Reuse the same components, request scope, and AOP policies across web and RPC
   frameworks.
2. Preserve each framework's native router, request, response, middleware, and
   runtime semantics.
3. Confine framework-specific types to adapter crates.
4. Give Sa-Token-Rust a stable authentication context and native rejection path.
5. Let Hutool-Rust and Ddd4r consume Vernal without reverse dependencies.
6. Prove behavioral equivalence with one cross-framework contract suite.

### 2.2 Non-goals

- No Rust Servlet container.
- No Project Reactor clone or new async runtime.
- No replacement for native routing DSLs.
- No concrete web-framework implementation in generic kernels. Tokio is the
  official runtime; Tower and Hyper remain owned by their foundation crates.
- No reimplementation of Sa-Token-Rust authentication or authorization.
- No claim that the ten targets form a permanent worldwide popularity ranking.

## 3. Local source findings

| Source | Observed capability | Vernal adopts | Vernal rejects |
|:---|:---|:---|:---|
| tx-di | Axum extractor, request extensions, Tower Layer/Service, route and layer registration; Tonic dependency evidence | Context injection, Tower composition, adapter assembly concepts | Global containers, pointer identity, framework code in kernels |
| Sa-Token-Rust | Ten plugin families: Actix Web, Axum, Gotham, Ntex, Poem, Rocket, Salvo, Tide, Tonic, Warp | Coverage union and native security middleware seams | Copying the security kernel into Vernal |
| Hutool-Rust | Reqwest-based HTTP client and client interceptors | Injectable tools and outbound client capabilities | Treating a client wrapper as a server adapter |
| Ddd4r | Domain, application, and infrastructure boundaries | A Ddd4r-owned starter that wires services and ports | A Vernal kernel dependency on DDD implementations |

There is one retained security brand and integration target:
**Sa-Token-Rust**.

## 4. Web and protocol contracts

```mermaid
flowchart TB
    K["Vernal kernels<br/>Core / IoC / AOP / Context"]
    W["vernal-web<br/>application contracts"]
    HTTP["vernal-http<br/>HTTP protocol contracts"]
    TOWER["vernal-tower<br/>Layer / Service"]
    HYPER["vernal-hyper<br/>HTTP transport"]
    HTTPADAPTERS["HTTP framework adapters"]
    RPC["vernal-tonic<br/>RPC adapter"]

    W --> K
    HTTP --> W
    TOWER --> W
    HYPER --> HTTP
    HTTPADAPTERS --> HTTP
    RPC --> TOWER
```

### 4.1 `vernal-web`

This crate defines framework-neutral application contracts only:

- `RequestContext`: request ID, route metadata, security principal, and
  extensions;
- `WebRequestScope`: the Web facade over IoC `ScopeContext`, including scoped
  component resolution, native-object caching, close hooks, cancellation, and
  release state;
- `HandlerInvocation`: handler and method metadata plus resolved arguments;
- `ProblemDetails`: stable application error categories;
- `ContextCarrier`: context propagation across futures, streams, and tasks;
- `WebIntegration`: adapter capability and diagnostics metadata.

Its public API must not expose Axum, Actix Web, or Hyper types. Tokio-native
types are allowed for task, cancellation, deadline, and context-propagation
contracts.

### 4.2 `vernal-http`

This crate defines protocol contracts:

- HTTP request and response parts;
- finite and streaming request/response bodies;
- extraction, validation, and HTTP error mapping;
- cancellation propagation and scope cleanup;
- preservation of upstream backpressure;
- extension points for WebSocket and SSE.

Ordinary request/response and streaming are capabilities of the same HTTP
contract. Rust `Future` and `Stream` are used directly; Vernal does not create
a second reactive type system or own an async runtime.

## 5. Request execution

```mermaid
sequenceDiagram
    participant F as Web/RPC Framework
    participant A as Vernal Adapter
    participant C as ApplicationContext
    participant S as Request Scope
    participant P as AOP Chain
    participant H as Component Handler

    F->>A: Native request
    A->>C: Read explicit context handle
    A->>S: Open request scope
    A->>A: Build RequestContext
    A->>P: invoke(context, next)
    P->>H: Resolve component and call handler
    H-->>P: Result / Future / Stream
    P-->>A: Result or policy rejection
    A->>A: Map to native response
    A->>S: close on success, error, or cancellation
    A-->>F: Native response / status
```

Constraints:

1. `ApplicationContext` is passed through app state, an extension, or explicit
   construction.
2. Adapters never guess the current context from process-global state.
3. Policy short-circuiting skips the handler but still closes an opened scope.
4. Around advice observes success, application errors, transport errors, and
   cancellation.
5. Native bodies and streams are not buffered merely to satisfy a common API.

## 6. Streaming and cancellation

```mermaid
stateDiagram-v2
    [*] --> ScopeOpened
    ScopeOpened --> HeadersReady: handler accepted
    HeadersReady --> Streaming: first item
    Streaming --> Streaming: demand / item
    Streaming --> Completed: end of stream
    ScopeOpened --> Failed: extraction or policy error
    HeadersReady --> Failed: body error
    Streaming --> Cancelled: peer disconnect
    Failed --> ScopeClosed
    Cancelled --> ScopeClosed
    Completed --> ScopeClosed
    ScopeClosed --> [*]
```

- A finite HTTP response closes the request scope after its body completes.
- Streaming HTTP and Tonic keep the scope alive until stream completion,
  failure, or cancellation.
- An adapter bridges upstream backpressure; it must not hide it behind an
  unbounded queue.
- Cancellation cleanup is idempotent and cannot depend on async work in `Drop`.
- Resources requiring async release use explicit `scope.close().await`; the
  application-owned `ScopeCleanupPolicy` supplies the wait bound.
- The default bound is 30 seconds. Timeout ends only the current adapter wait;
  the shared Tokio cleanup coordinator continues and later callers can join the
  same result without executing hooks twice.

## 7. Tower and Hyper foundations

### 7.1 `vernal-tower`

Implemented reusable facilities for Axum, Tonic, and other Tower services:

- `VernalLayer` injects the context handle;
- `RequestScopeLayer` explicitly receives `Arc<ApplicationContext>`, derives
  request scope from its Container, and closes it across response body
  completion, service error, cancellation, and dropped futures;
- `ContextPropagationLayer` creates or preserves `RequestContext`, captures an
  owned HTTP metadata snapshot, and exposes the Scope-owned cancellation token;
- `AopLayer` resolves `RouteMetadata` into a precompiled `InvocationPlan` for
  security, transaction, audit, and observability interceptors;
- `ErrorMappingLayer` delegates both readiness and call failures to a typed
  `TowerErrorMapper`, which can recover them as framework-native responses or
  convert them into a new `Service::Error`;
- `TowerRouteResolver` lets adapters derive low-cardinality route metadata from
  native routing information;
- `TowerResponse<R>` preserves the native response and body across type erasure
  without reading or buffering it, and requires only `Send`;
- `AopServiceError<E>` separates missing context/scope/route failures, AOP
  failures, and recoverable native `Service::Error` values.

Missing plans are rejected by default, so a missing Sa-Token-Rust security plan
cannot silently bypass authorization. Routes that intentionally do not use AOP
must explicitly select `MissingPlanPolicy::Proceed`. A short-circuiting
interceptor never transfers request ownership to the downstream service, while
downstream errors traverse the complete AOP chain before being restored to
their native error type.

Layer ordering is contractual and tested. Error mapping wraps AOP so it can
recover both infrastructure and interceptor failures:

```text
request: Trace -> Context -> RequestScope -> ContextPropagation -> ErrorMapping -> Security/AOP -> Handler
error:   Handler -> Security/AOP -> ErrorMapping
```

### 7.2 `vernal-hyper`

This crate implements HTTP transport concerns only:

- lightweight bridges for Hyper requests and responses;
- body-frame and trailer fidelity, verified on a real chunked request;
- request-cancellation signals shared by the request and body;
- no application router and no place in the ten-framework count.

## 8. Ten framework adapters

| # | Framework | Crate | Protocol | Native integration seam | Current state |
|:--:|:---|:---|:---|:---|:---:|
| 1 | Axum | `vernal-axum` | HTTP + Tower | `Layer`, State/Extension, Extractor, IntoResponse | Phase 5 adapter |
| 2 | Actix Web | `vernal-actix-web` | HTTP | `Transform`/`Service`, App Data, Extractor, Responder | Adapter + strict Local-AOP |
| 3 | Rocket | `vernal-rocket` | HTTP | Fairing, Request Guard, Route Handler, Responder | Adapter + strict AOP |
| 4 | Warp | `vernal-warp` | HTTP | Filter, Rejection, Reply, Tower Service | Adapter + strict AOP |
| 5 | Salvo | `vernal-salvo` | HTTP | Handler, Hoop, Depot, Writer | Adapter + strict AOP |
| 6 | Poem | `vernal-poem` | HTTP | Middleware, Endpoint, Data, IntoResponse | Adapter + strict AOP |
| 7 | Ntex | `vernal-ntex` | HTTP | Service/Middleware, App State, Extractor | Adapter + strict Local-AOP |
| 8 | Gotham | `vernal-gotham` | HTTP | State Middleware, Pipeline, Handler | Adapter + strict AOP |
| 9 | Tide | `vernal-tide` | HTTP | Middleware, Request Extension, Response | Adapter + strict AOP |
| 10 | Tonic | `vernal-tonic` | RPC Streaming + Tower | Layer, Interceptor, Extension, Status, Streaming | Phase 5 adapter |

### 8.1 Framework-specific constraints

- **Axum:** reuse `vernal-tower`; component extractors read router state or
  request extensions and never create a second container. Its protocol mapper
  is installed through the shared `ErrorMappingLayer`, rather than a dedicated
  Axum error-handling middleware.
- **Actix Web:** context lives in app data; multi-worker singleton and request
  scope boundaries need explicit tests. Actix services commonly use `Rc` and
  local non-`Send` futures, so strict middleware uses Vernal's separate
  `LocalInterceptor`/`LocalNext`/`LocalInvocationPlan` contract. It wraps a
  concrete `web::resource(...)` after route matching, drives the complete
  Service future, uses the matched resource pattern as low-cardinality
  operation identity, fail-closes missing metadata/plans, maps AOP failures
  through `EitherBody`, and preserves native Actix service errors.
- **Rocket:** the ignite fairing registers managed context, the request fairing
  creates scope, request guards resolve context/components/scope, and the
  response fairing retains scope until the native body reaches EOF or is
  cancelled. Rocket 0.5 exposes its public body only as `AsyncRead`, so wrapping
  converts it to a streamed body: bytes, backpressure, and errors remain, while
  the original known-length/seekable classification cannot be reconstructed
  through public APIs. Strict mode applies `VernalRocketRoutesExt` to the
  existing `routes![...]` collection before mount. It replaces only the public
  `Route.handler`, retaining name, method, URI, rank, format, and discovered
  sentinels. The runtime-matched Route supplies the complete low-cardinality URI
  template after mount, so mount prefixes are retained; that template and the
  real request method form the Operation. An owned cross-model snapshot
  propagates through `RequestContext`. The complete Handler Future runs through a borrowed
  Send-AOP target, missing plans fail closed, policy failures do not invoke the
  Handler, and Rocket's Success, Error, and Forward Outcomes remain native.
- **Warp:** `warp::ext` composition filters extract context, components, and
  scope; missing extraction becomes a typed rejection rather than panic. Warp
  0.4's public Reply contains a private body type, so full body scope is
  composed at the official `warp::service(route)` boundary. `VernalWarpLayer`
  provides the lifecycle-only path; `VernalWarpAopLayer` additionally places
  the complete Filter Service behind strict Tower AOP while preserving frames,
  trailers, backpressure, and cancellation. Warp does not expose the matched
  route template through its public Service request, so one strict layer wraps
  one concrete Filter Service and receives the same full low-cardinality route
  pattern explicitly. The real HTTP method and owned request snapshot propagate
  through `RequestContext`; empty patterns and missing plans fail closed,
  policy failures map to native Warp responses without executing the Filter,
  and successful native responses retain status, headers, extensions, and
  private Body.
- **Salvo:** a Hoop wraps the invocation, typed Depot entries carry context,
  components, and request scope, and handlers retain native signatures.
  `ResBody` is wrapped directly as `http_body::Body`, preserving data frames,
  trailers, upstream errors, and backpressure while closing scope on completion
  or cancellation. Strict mode enables Salvo's zero-dependency `matched-path`
  feature, combines the low-cardinality matched template with the real HTTP
  method, propagates an owned metadata snapshot, and drives the complete
  Handler chain through Send-AOP. `BorrowedInvocationTarget` ties
  `Request`/`Depot`/`Response`/`FlowCtrl` borrows to one plan await without
  cloning them. Missing metadata or plans fail closed, policy failures become
  stable responses, and native Handler responses retain their status, headers,
  extensions, and body. Salvo 0.85.0 is the last release declaring Rust 1.85;
  version 0.86 and later require Rust 1.89 or newer.
- **Poem:** native middleware wraps the concrete endpoint after Route matching;
  request extensions carry context, components, scope, and `RequestContext`.
  Strict mode derives `Operation(path_pattern, http_method)` from Poem's
  low-cardinality `PathPattern`, rejects missing metadata or plans, wraps the
  complete Endpoint future, and preserves native Poem errors. The response byte
  stream preserves errors and backpressure and closes scope on completion or
  cancellation. Poem 3's
  public `into_bytes_stream()` does not expose trailers, so this adapter cannot
  promise trailer fidelity; use `vernal-hyper` when frame/trailer fidelity is
  required.
- **Ntex:** native Middleware/Service injects the explicit application context
  and per-request scope into Extensions; extractors may also read the shared
  context from App State. A native `MessageBody` wrapper retains the scope
  through EOF, upstream failure, or cancellation while preserving Ntex body
  size and backpressure. Strict middleware must wrap a concrete
  `web::resource(...)` and receive that resource's full low-cardinality path
  pattern explicitly: Ntex 2 does not expose the matched `ResourceDef` through
  public request APIs. The HTTP method still comes from the real request.
  `BorrowedLocalInvocationTarget` ties the complete Service future to the
  current `ServiceCtx`; the borrow cannot escape
  `LocalInvocationPlan::invoke_borrowed`. Missing plans fail closed, the
  one-shot request is never cloned, and native Service errors are restored.
  Vernal enables Ntex's Tokio backend and pins 2.18.0: the current 3.x release
  exceeds the workspace Rust 1.85 MSRV. Worker-local non-`Send` state is never
  silently promoted to a cross-worker singleton.
- **Gotham:** native `StateData` wrappers carry the explicit application context
  and per-request scope, while a typed State extension resolves IoC components
  without a service locator. `Middleware`/`NewMiddleware` binds the scope to
  Gotham's `http-body` 1.0 response, preserving data frames, trailers, size
  hints, upstream errors, backpressure, and cancellation. Strict mode drives
  the complete Pipeline Chain through `BorrowedInvocationTarget`. Gotham State
  exposes owned standard HTTP metadata but not the final matched route
  template, so a concrete Pipeline receives the same full low-cardinality
  route pattern explicitly. The real method and owned request snapshot
  propagate through `RequestContext`; empty patterns and missing plans fail
  closed, policy failures become native responses without invoking the
  Handler, and native `HandlerError` status plus cause are restored. The
  middleware factory narrowly marks its synchronized `Arc<ApplicationContext>`
  holder as unwind-safe to satisfy Gotham's pipeline contract; it does not
  weaken Context locking or thread-safety requirements.
- **Tide:** native Middleware injects the explicit application context and
  per-request scope into Request Extensions; a typed request extension resolves
  IoC components without a service locator. The response `AsyncBufRead` wrapper
  retains scope through EOF, upstream failure, or cancellation while preserving
  bytes, known length, errors, and backpressure. Tide's public body API does not
  expose HTTP trailers, so this adapter cannot promise trailer fidelity. Tide
  exposes route parameter values but not the matched template, so strict
  middleware is installed on a concrete Route and receives the same full
  low-cardinality pattern explicitly. It combines that pattern with the real
  method, converts `http-types` metadata into an owned `http` 1.x snapshot, and
  drives the complete Middleware/Endpoint chain through
  `BorrowedInvocationTarget`. Empty patterns and missing plans fail closed;
  native responses retain status, headers, extensions, and Body. The adapter
  requires an active Tokio runtime for deterministic asynchronous scope closure.
  Tide 0.17.0-beta.1 remains beta, so its compatibility surface must be
  revalidated before a stable Vernal release.
- **Tonic:** unary and streaming calls use the Tower path; Vernal errors map to
  stable `Status` values without losing metadata or extensions.

## 9. Sa-Token-Rust integration

Sa-Token-Rust owns authentication, session, login state, and authorization
rules. Vernal resolves components, invokes pointcuts, and carries request
context.

```mermaid
flowchart LR
    REQ["Native request"] --> ADAPTER["vernal-* adapter"]
    ADAPTER --> RC["RequestContext"]
    RC --> AOP["Vernal AOP pointcut"]
    AOP --> SAT["Sa-Token-Rust policy / API"]
    SAT -->|allow| HANDLER["Application component"]
    SAT -->|deny| DENY["Native 401/403 or RPC Status"]
    HANDLER --> RESP["Native response"]
```

Sa-Token-Rust now owns the experimental `sa-token-vernal` bridge:

- depend on a verified Vernal Git revision without creating a kernel reverse
  dependency;
- adapt `HttpRequestSnapshot` to `SaRequest` and reuse `run_auth_flow`;
- project login identity and roles into `RequestContext::SecurityPrincipal`;
- run downstream futures inside the request's `SaTokenContext`;
- use `SaTokenComponents` as the named `sa-token.security`
  `ApplicationModule` to atomically register the manager, bridge, policy, and
  matching Send/Local authentication/authorization Advisors; duplicate module
  names or definitions leave no partial security plan;
  `VernalSaTokenPointcut` covers declared operations and
  `VernalSaTokenInterceptor` authenticates before enforcing operation-scoped
  all/any role and permission requirements;
- align exactly with Axum/Poem `Operation(path_template, http_method)` and
  Tonic `Operation(service_name, method_name)`, mapping rejection through
  `WebFailure` into native framework responses;
- preserve native web plugins for users who do not use Vernal.

The bridge implements authentication plus immutable operation authorization.
Role matching is exact; permissions preserve Sa-Token exact, global `*`, and
prefix wildcard semantics. Empty declared requirements fail closed. Protected
anonymous calls map to 401, insufficient authenticated identities to 403, and
permission backend failures to an internal 500 without exposing the source.
Sa-Token-Rust `PathAuthConfig` remains the sole source of path login policy,
and Vernal never depends on Sa-Token-Rust.

## 10. Hutool-Rust and Ddd4r

- **Hutool-Rust:** HTTP clients, serializers, caches, and other tools can be
  registered as components. Outbound client interception is separate from
  server adapters.
- **Ddd4r:** a Ddd4r-owned `ddd4r-vernal` starter binds domain services,
  application services, repository ports, and transaction/audit interceptors.
  The current bridge registers native Registry/CommandBus objects and enters
  Ddd4r's Tokio task-local `ContextScope` with an isolated snapshot.
- All ecosystem dependencies point from the consumer to Vernal.

## 11. Errors, security, and observability

| Failure point | Vernal category | Adapter responsibility |
|:---|:---|:---|
| Missing/closed context | Infrastructure | Stable 5xx/Status; never panic |
| Missing/ambiguous component | Resolution | Redacted dependency path and stable mapping |
| Extraction/validation | ClientInput | Native 4xx or rejection |
| Authentication/authorization | PolicyDenied | Sa-Token-Rust determines 401/403 semantics |
| Handler application error | Application | Invoke the user error mapper |
| Body/stream error | Transport | Preserve cancellation class, close scope, and record a redacted cleanup warning on failure |

Guardrails:

- never log tokens, cookies, authorization headers, or component secrets;
- no unbounded-cardinality raw paths or user IDs in trace/metric labels;
- diagnostics expose adapter, version, status, scope counts, and static redacted
  warning codes; scope cleanup failures use
  `web.request-scope.cleanup-failed`, never the close-hook error text;
- cleanup timeout, hook failure, and hook-task panic share that redacted warning
  boundary; the background coordinator still reaches `Closed`;
- panic is not normal control flow for denial, resolution failure, or cancel;
- adapters keep native body limits and timeouts unless explicitly configured.

## 12. Cross-framework conformance

Every adapter must reuse the same `vernal-web-testkit` contract suite. The
shared request-binding contract makes all ten adapters pass their native
`ApplicationContext`, `WebRequestScope`, and request-scoped component to
`WebAdapterContract`, which resolves again within the same scope and compares
`Arc` identity. `ScopeCloseProbe` observes, but never closes, the real adapter
scope. Together with `ScopeRejectingInterceptor`, it proves normal body
completion, policy short-circuit, and response-body drop cleanup on all ten
adapters. This catches both scope bypass and test-assisted cleanup.
`FailingHttpBody`, `FailingByteStream`, `FailingTokioReader`, and
`FailingFuturesReader` only emit deterministic upstream failures. All ten
adapters use their native body, stream, or reader wrapper to prove that the
error path waits for scope closure before restoring the original transport
failure. `ScopeCleanupTimeoutFixture` supplies a real application-bound scope,
bounded cleanup policy, and controllable hanging hook. Every adapter now proves
that its native response surface returns the structured timeout within budget,
records only `web.request-scope.cleanup-failed`, leaves the single cleanup
coordinator running, and reaches `Closed` exactly once after release. Each
adapter also consumes one native frame, chunk, or byte without polling the
terminal boundary and then drops that consumer. This deterministic
framework-boundary disconnect proves that `DropGuard` cancellation wakes the
pre-started cleanup task and closes the still-open scope. Live-socket disconnect
E2E and remaining matrix paths continue incrementally. The Axum cancellation
contract additionally proves that an
ignored background close error reaches the owning Context as a redacted warning:

| Contract | Required coverage |
|:---|:---|
| Context | Explicit injection, missing, closed, parallel-app isolation |
| Scope | Per-request identity, nested reuse, close on success/error/cancel |
| IoC | Singleton, transient, qualifier, multi-binding, resolution errors |
| AOP | Ordering, short circuit, error, async, stream completion/cancel |
| HTTP | Headers, status, body, trailers, extensions, size limits |
| Security | Allow, unauthenticated, forbidden, policy error, redaction |
| Streaming | Backpressure, half-close, disconnect, cleanup timeout |
| Lifecycle | Startup, rollback, graceful shutdown, worker isolation |

An adapter is complete only when:

1. upstream dependencies and features are locked;
2. native Hello/DI/AOP/Security examples run;
3. the shared contract suite passes;
4. a version compatibility matrix is recorded;
5. `cargo tree` proves kernels have no framework dependency;
6. unsupported cases and measured performance boundaries are documented.

## 13. Delivery waves

| Wave | Scope | Exit evidence |
|:---|:---|:---|
| P0 | `vernal-web` and `vernal-http` contracts | No framework type leaks; contract tests pass |
| P1 | Tower, Hyper, Axum, Actix Web | Two middleware families pass one suite |
| P2 | Salvo, Poem, Rocket, Warp, Tonic | HTTP/RPC and finite/streaming matrix |
| P3 | Ntex, Gotham, Tide | Worker/state/compatibility risks tested and recorded |
| P4 | Sa-Token-Rust, Hutool-Rust, Ddd4r bridge examples | Consumer-owned dependencies; no kernel coupling |

Priorities can change with registry maintenance and downstream demand. Any
change updates the manifest, bilingual docs, and compatibility matrix together.

## 14. Definition of architecture done

- [x] Web and HTTP contracts are implemented and independently tested.
- [x] Tower/Hyper do not enter Core, IoC, AOP, or Context.
- [x] All ten adapters use native extension points and no global context.
- [ ] Non-streaming, streaming, cancellation, and cleanup semantics have tests.
- [x] Tonic is classified as RPC, not an HTTP router.
- [x] Sa-Token-Rust is the sole retained security integration target.
- [x] Hutool-Rust, Sa-Token-Rust, and Ddd4r names and boundaries match in both languages.
- [x] Skeleton, runnable, contract-passing, and production-ready are never conflated.

---

**Vernal is a lightweight IoC, AOP and application context framework for Rust.**

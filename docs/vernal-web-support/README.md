# vernal-web-support 迁移文档

> 本目录收纳 `vernal-web` 端到端迁移到 rust 的全部规划文档。
> 对应源码：`crates/vernal-web`（核心契约）、`crates/vernal-http`（通用 HTTP 抽象）、`crates/vernal-web-testkit`（跨运行时 contract test）、12 个 runtime crate（actix-web / axum / gotham / hyper / ntex / poem / rocket / salvo / tide / warp / tonioc / tower）。
> 目标蓝本：Spring Framework 7.0.8 `spring-web`、`spring-webmvc`、`spring-webflux`。
> 基线原则：按"功能语义完全迁移 + Rust 化实现"对齐；以"对象一文件 / snake_case / PascalCase / 中文注释 / 同步维护"作为规范。

## 索引

### 一、基础契约层（`spring-web` → `vernal-web`）

| 文档 | 描述 | 状态 |
|---|---|---|
| `spring-web到vernal-web迁移路线图.md` | `vernal-web` 端规划：HTTP 核心、消息/编解码、客户端/服务端请求管线、Middleware、CORS、SSE、WebSocket、表达式集成、收尾 W0–W10。 | ✅ v1.0 |
| `spring-web到vernal-web对象级对照表.md` | 451 个 Spring `web/*` 公共对象（含 `org.springframework.http.*`、`http.client.*`、`http.codec.*`、`http.converter.*`、`web.*`）到 Rust `.rs` 文件与对象名的结构验收清单。 | ✅ v1.0 |
| `spring-web到vernal-web语义迁移对照表.md` | Java 反射 / 字节码 / 反应式 publisher / Servlet 等语义到 Rust trait / 闭包 / `Future` + `Stream` / 泛型宏的对应；与 `vernal-expression` 桥接点。 | ✅ v1.0 |
| `spring-web到vernal-web对象名称一致性检查.md` | 现状 Python 文件名 + 类型名 + `ProblemDetails` 单复数冲突 + http 内部命名一致性 + `cargo description` 模板 | ✅ v1.0 |

### 二、运行时适配层（`spring-webmvc` + `spring-webflux` → `vernal-web-{runtime}`）

| 文档 | 描述 | 状态 |
|---|---|---|
| `spring-webmvc和spring-flux到vernal-web-{runtime}迁移路线图.md` | R0–R8 路线：在 `vernal-web::contract` 抽出 HandlerAdapter/HandlerMapping/ArgumentResolver/ReturnValueHandler/HandlerInterceptor/WebFilter/HandlerExceptionHandler/ContentNegotiationManager/LocaleContextResolver/FlashMapManager；12 个运行时 crate 完备；`vernal-web-testkit` 跨运行时契约测试矩阵。 | ✅ v1.0 |
| `spring-webmvc和spring-flux到vernal-web-{runtime}对象级对照表.md` | Spring WebMVC(363 文件) + WebFlux(275 文件) 顶部与子包 → `vernal-web::contract` + 12 个运行时运行时内部 `<runtime>_<concept>.rs`。 | ✅ v1.0 |
| `spring-webmvc和spring-flux到vernal-web-{runtime}语义迁移对照表.md` | Dispatcher 请求生命周期、WebFlux reactive 栈、HandlerMapping/ArgumentResolver/HandlerInterceptor/AOP、消息转换、协商/Locale/FlashMap/View、WebFlux 专属；12 个不迁移项。 | ✅ v1.0 |
| `spring-webmvc和spring-flux到vernal-web-{runtime}对象名称一致性检查.md` | 12 个运行时现有命名 + 跨 crate 一致性 (`Middleware` / `Layer` / `Hoop` / `Fairing` / `RouteMetadata` / `RouteResolver` 等) 严点记录。 | ✅ v1.0 |

## 三、八份文档使用说明

- 仅读 文档 1–4：足以理解 `vernal-web` 核心 + 怎么到 Spring `web/*` 。
- 仅读 文档 5–8：足以理解 12 个运行时 crate + 怎么到 Spring WebMVC/WebFlux。
- 两套合并读：可对任何场景书写 Spring → Vernal 迁移指南。

## 四、文件状态汇总

| 文档 | 行数 | 状态 |
|---|---:|---|
| `spring-web到vernal-web迁移路线图.md` | 110 | ✅ |
| `spring-web到vernal-web对象级对照表.md` | 131 | ✅ |
| `spring-web到vernal-web语义迁移对照表.md` | 103 | ✅ |
| `spring-web到vernal-web对象名称一致性检查.md` | 85 | ✅ |
| `spring-webmvc和spring-flux到vernal-web-{runtime}迁移路线图.md` | 156 | ✅ |
| `spring-webmvc和spring-flux到vernal-web-{runtime}对象级对照表.md` | 246 | ✅ |
| `spring-webmvc和spring-flux到vernal-web-{runtime}语义迁移对照表.md` | 152 | ✅ |
| `spring-webmvc和spring-flux到vernal-web-{runtime}对象名称一致性检查.md` | 169 | ✅ |

## 五、同步规则

每次合并运行时适配或 `vernal-web` 与 `vernal-http` 变更时，**必须**同时更新本目录下列文档：

1. 路线图 2 份
2. 对象级对照表 2 份
3. 语义迁移对照表 2 份
4. 对象名称一致性检查 2 份

并检查是否需要补加以下文档中的表格、状态、CodeGraph 引用：

- `crates/vernal-web/{src,tests}`、`crates/vernal-http/{src,tests}`、`crates/vernal-web-testkit/{src,tests}`
- `crates/vernal-{runtime}/{src,tests}` × 12
- `crates/vernal-tower/{src,tests}`
- `crates/vernal-hyper/{src,tests}`

不遗漏。

## 六、仓库总量统计

| 仓库 | 节点 | 边 | 文件 |
|---|---:|---:|---:|
| `vernal-framework` | 4,606 | 34,731 | 754 |
| `spring-framework` | 115,037 | 1,029,168 | 9,680 |

| Spring 模块 | Java 文件 |
|---|---:|
| `spring-webmvc` | 363 |
| `spring-webflux` | 275 |
| `spring-web/http` | 31 |
| `spring-web/http.client` | 35 |
| `spring-web/http.codec` | 29 |
| `spring-web/http.converter` | 23 |
| `spring-web/web` | 448 |

| Vernal crate | Rust 文件 |
|---|---:|
| `vernal-web` | 15 |
| `vernal-http` | 7 |
| `vernal-web-testkit` | 9 |
| `vernal-tower` | 22 |
| `vernal-hyper` | 2 |
| `vernal-actix-web` | 14 |
| `vernal-axum` | 12 |
| `vernal-gotham` | 13 |
| `vernal-ntex` | 14 |
| `vernal-poem` | 11 |
| `vernal-rocket` | 14 |
| `vernal-salvo` | 10 |
| `vernal-tide` | 10 |
| `vernal-warp` | 11 |
| `vernal-tonic` | 9 |


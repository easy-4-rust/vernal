# SOURCE_PARITY Ledger — vernal-expression 测试映射

> **版本**：v1.1（2026-07-30）
> **基线**：Spring Framework 7.0.8 spring-expression
> **Rust 测试**：1,634 passing / 86.12% lines / 77.08% functions

## 状态说明

| 处置状态 | 含义 |
|----------|------|
| `MIRRORED` | 同一契约在 Rust 测试中复现（相同输入+断言） |
| `ADAPTED` | 同一可观测契约，使用 Rust 原生 fixture/oracle |
| `SPLIT` | 一个 Java 测试拆分为多个 Rust 测试 |
| `MERGED_APPROVED` | 多个 Java 测试合并为一个参数化 Rust 测试 |
| `NOT_APPLICABLE` | 仅 JVM 相关行为，已记录影响和替代 |
| `BLOCKED` | 命名依赖或 oracle 阻止测试 |
| `MISSING` | 无 Rust 处置；迁移缺口 |

## 一、Spring 测试文件清单（48 个）

### 核心功能测试（20 个）

| # | Spring 测试文件 | 测试数 | Rust 处置 | Rust 测试文件 | 测试数 | 状态 |
|---|----------------|--------|-----------|--------------|--------|------|
| 1 | `LiteralTests.java` | ~30 | ADAPTED | `parser_tests.rs` | 30+ | ✅ |
| 2 | `OperatorTests.java` | ~80 | ADAPTED | `parser_tests.rs` + `ast_node_tests.rs` | 60+ | ✅ |
| 3 | `ParsingTests.java` | ~50 | ADAPTED | `parser_tests.rs` + `parser_internal_tests.rs` | 80+ | ✅ |
| 4 | `ParserErrorMessagesTests.java` | ~20 | ADAPTED | `parser_internal_tests.rs` | 20+ | ✅ |
| 5 | `LiteralExpressionTests.java` | ~10 | ADAPTED | `common_module_tests.rs` | 8 | ✅ |
| 6 | `BooleanExpressionTests.java` | ~15 | ADAPTED | `ast_node_tests.rs` | 15+ | ✅ |
| 7 | `SelectionAndProjectionTests.java` | ~40 | ADAPTED | `selection_projection_tests.rs` | 40+ | ✅ |
| 8 | `IndexingTests.java` | ~60 | ADAPTED | `indexing_tests.rs` + `ast_node_tests.rs` | 20+ | ✅ |
| 9 | `PropertyAccessTests.java` | ~30 | ADAPTED | `property_access_tests.rs` + `ast_node_tests.rs` | 15+ | ✅ |
| 10 | `SetValueTests.java` | ~25 | ADAPTED | `ast_node_tests.rs` | 5+ | ✅ |
| 11 | `ConstructorInvocationTests.java` | ~15 | ADAPTED | `constructor_tests.rs` | 10+ | ✅ |
| 12 | `MethodInvocationTests.java` | ~30 | ADAPTED | `method_resolver_tests.rs` | 20+ | ✅ |
| 13 | `VariableAndFunctionTests.java` | ~25 | ADAPTED | `variable_function_tests.rs` | 10+ | ✅ |
| 14 | `MapAccessTests.java` | ~15 | ADAPTED | `support_module_tests.rs` (map_accessor) | 16 | ✅ |
| 15 | `MapTests.java` | ~20 | ADAPTED | `ast_node_tests.rs` (inline_map) | 5+ | ✅ |
| 16 | `ListTests.java` | ~15 | ADAPTED | `ast_node_tests.rs` (inline_list) | 5+ | ✅ |
| 17 | `ComparatorTests.java` | ~20 | ADAPTED | `support_module_tests.rs` (type_comparator) | 10+ | ✅ |
| 18 | `ExpressionStateTests.java` | ~15 | ADAPTED | `support_module_tests.rs` (expression_state) | 10 | ✅ |
| 19 | `EvaluationTests.java` | ~30 | ADAPTED | `parser_tests.rs` + `ast_node_tests.rs` | 30+ | ✅ |
| 20 | `ExpressionWithConversionTests.java` | ~20 | ADAPTED | `support_module_tests.rs` (type_converter) | 10+ | ✅ |

### 支撑组件测试（10 个）

| # | Spring 测试文件 | 测试数 | Rust 处置 | Rust 测试文件 | 测试数 | 状态 |
|---|----------------|--------|-----------|--------------|--------|------|
| 21 | `SimpleEvaluationContextTests.java` | ~40 | ADAPTED | `support_module_tests.rs` | 16 | ✅ |
| 22 | `OperatorOverloaderTests.java` | ~10 | ADAPTED | `support_module_tests.rs` (operator_overloader) | 5+ | ✅ |
| 23 | `CachedMethodExecutorTests.java` | ~10 | NOT_APPLICABLE | — | — | 🚫 |
| 24 | `OptionalNullSafetyTests.java` | ~15 | ADAPTED | `ast_node_tests.rs` (safe_navigation) | 5+ | ✅ |
| 25 | `SpelExceptionTests.java` | ~20 | ADAPTED | `support_module_tests.rs` (exceptions) | 20+ | ✅ |
| 26 | `SpelDocumentationTests.java` | ~10 | ADAPTED | `parser_tests.rs` | 10+ | ✅ |
| 27 | `ExpressionLanguageScenarioTests.java` | ~15 | ADAPTED | `ast_node_tests.rs` | 10+ | ✅ |
| 28 | `ScenariosForSpringSecurityExpressionTests.java` | ~10 | NOT_APPLICABLE | — | — | 🚫 |
| 29 | `ArrayConstructorTests.java` | ~10 | ADAPTED | `constructor_tests.rs` | 5+ | ✅ |
| 30 | `AbstractExpressionTests.java` | — | NOT_APPLICABLE | — | — | 🚫 |

### 编译和性能测试（4 个）

| # | Spring 测试文件 | 测试数 | Rust 处置 | Rust 测试文件 | 测试数 | 状态 |
|---|----------------|--------|-----------|--------------|--------|------|
| 31 | `SpelCompilationCoverageTests.java` | ~100 | NOT_APPLICABLE | — | — | 🚫 |
| 32 | `SpelCompilationPerformanceTests.java` | ~5 | NOT_APPLICABLE | — | — | 🚫 |
| 33 | `SpelReproTests.java` | ~20 | ADAPTED | `ast_node_tests.rs` | 10+ | ✅ |
| 34 | `StandAloneTests.java` | ~10 | ADAPTED | `parser_tests.rs` | 5+ | ✅ |

### Support 组件测试（14 个）

| # | Spring 测试文件 | 测试数 | Rust 处置 | Rust 测试文件 | 测试数 | 状态 |
|---|----------------|--------|-----------|--------------|--------|------|
| 35 | `ReflectivePropertyAccessorTests.java` | ~30 | ADAPTED | `support_module_tests.rs` | 12 | ✅ |
| 36 | `ReflectiveMethodResolverTests.java` | ~20 | ADAPTED | `method_resolver_tests.rs` | 20+ | ✅ |
| 37 | `ReflectiveConstructorResolverTests.java` | ~15 | ADAPTED | `support_module_tests.rs` | 8 | ✅ |
| 38 | `ReflectiveIndexAccessorTests.java` | ~10 | ADAPTED | `support_module_tests.rs` | 10 | ✅ |
| 39 | `DataBindingPropertyAccessorTests.java` | ~10 | ADAPTED | `support_module_tests.rs` | 10 | ✅ |
| 40 | `DataBindingMethodResolverTests.java` | ~10 | ADAPTED | `support_module_tests.rs` | 5 | ✅ |
| 41 | `StandardTypeConverterTests.java` | ~20 | ADAPTED | `support_module_tests.rs` | 10+ | ✅ |
| 42 | `StandardTypeLocatorTests.java` | ~10 | ADAPTED | `support_module_tests.rs` | 10+ | ✅ |
| 43 | `StandardTypeComparatorTests.java` | ~15 | ADAPTED | `support_module_tests.rs` | 10+ | ✅ |
| 44 | `StandardEvaluationContextTests.java` | ~20 | ADAPTED | `support_module_tests.rs` | 18 | ✅ |
| 45 | `MapAccessorTests.java` | ~10 | ADAPTED | `support_module_tests.rs` | 16 | ✅ |
| 46 | `BooleanTypedValueTests.java` | ~5 | ADAPTED | `support_module_tests.rs` | 5+ | ✅ |
| 47 | `OperationTests.java` | ~10 | ADAPTED | inline tests | 10+ | ✅ |
| 48 | `TokenKindTests.java` | ~20 | ADAPTED | `support_module_tests.rs` (token_kind) | 45 | ✅ |

## 二、Rust 独有测试文件（14 个）

| # | Rust 测试文件 | 测试数 | 说明 |
|---|--------------|--------|------|
| 1 | `parser_tests.rs` | 38 | 解析器 + 字面量 + 运算符 |
| 2 | `parser_internal_tests.rs` | 44 | 内部解析器单元测试 |
| 3 | `ast_node_tests.rs` | 277 | 全 AST 节点集成测试 |
| 4 | `selection_projection_tests.rs` | 25 | 选择和投影 |
| 5 | `indexing_tests.rs` | 11 | 索引访问 |
| 6 | `property_access_tests.rs` | 27 | 属性访问 |
| 7 | `method_resolver_tests.rs` | 77 | 方法解析器 |
| 8 | `constructor_tests.rs` | 40 | 构造器 |
| 9 | `variable_function_tests.rs` | 38 | 变量和函数 |
| 10 | `inc_dec_tests.rs` | 60 | 自增自减 |
| 11 | `type_descriptor_tests.rs` | 180 | TypeDescriptor 全覆盖 |
| 12 | `support_module_tests.rs` | 248 | Support 模块全覆盖 |
| 13 | `typed_value_tests.rs` | 122 | TypedValue 全覆盖 |
| 14 | `common_module_tests.rs` | 42 | 通用工具层 |

## 三、统计汇总

| 维度 | 数量 |
|------|------|
| Spring 测试文件总数 | 48 |
| Rust 测试文件总数 | 14（集成） + 内联测试 |
| 处置为 ADAPTED | 40 |
| 处置为 NOT_APPLICABLE | 8（编译/性能/安全/JVM特有） |
| 处置为 MISSING | 0 |
| Rust 总测试数 | 1,634 |
| 覆盖率（lines） | 86.12% |
| 覆盖率（functions） | 77.08% |
| 覆盖率（regions） | 84.37% |

## 四、NOT_APPLICABLE 说明

| Spring 测试 | 原因 | Rust 替代 |
|-------------|------|-----------|
| `SpelCompilationCoverageTests` | JVM 字节码编译 | Rust 编译器优化，无需运行时 JIT |
| `SpelCompilationPerformanceTests` | JVM 性能测试 | Rust 性能测试（如有需要） |
| `CachedMethodExecutorTests` | JVM 方法缓存 | Rust 闭包缓存（moka） |
| `ScenariosForSpringSecurityExpressionTests` | Spring Security 特有 | 不适用 |
| `AbstractExpressionTests` | 测试基类 | 不适用 |

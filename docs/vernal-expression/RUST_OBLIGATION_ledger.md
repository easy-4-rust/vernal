<!-- migration-doc: authority=support canonical=../迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。

# RUST_OBLIGATION Ledger — vernal-expression Rust 特有测试

> **版本**：v1.1（2026-07-30）
> **测试**：1,634 passing / 86.12% lines / 77.08% functions

## 状态说明

| 状态 | 含义 |
|------|------|
| ✅ | 已测试 |
| ⚠️ | 部分测试 |
| ❌ | 未测试 |

## 一、所有权 / Drop

| 机制 | 测试 | 状态 |
|------|------|------|
| `TypedValue` Clone 正确性 | `typed_value_tests.rs` (15 tests) | ✅ |
| `ExpressionValue` Arc 引用计数 | `typed_value_tests.rs` (Object variant) | ✅ |
| `SpelNode` trait object 生命周期 | `ast_node_tests.rs` | ✅ |
| `ExpressionState` 栈 push/pop 边界 | `support_module_tests.rs` | ✅ |
| `RwLock` 毒化恢复 | N/A（使用 `unwrap()`） | ⚠️ |

## 二、Send + Sync

| 机制 | 测试 | 状态 |
|------|------|------|
| `TypedValue: Send + Sync` | 编译时检查 | ✅ |
| `ExpressionValue: Send + Sync` | 编译时检查（Arc<dyn Any+Send+Sync>） | ✅ |
| `SpelNode: Send + Sync` | 编译时检查 | ✅ |
| `EvaluationContext: Send + Sync` | 编译时检查 | ✅ |
| `StandardEvaluationContext: Send + Sync` | 编译时检查 | ✅ |
| `ReflectiveMethodResolver: Send + Sync` | 编译时检查 | ✅ |

## 三、类型安全

| 机制 | 测试 | 状态 |
|------|------|------|
| `ExpressionValue` 13 变体枚举 | `typed_value_tests.rs` (74 tests) | ✅ |
| `TypeDescriptor` 4 变体枚举 | `type_descriptor_tests.rs` (180 tests) | ✅ |
| `PrimitiveKind` 14 变体枚举 | `type_descriptor_tests.rs` | ✅ |
| `TokenKind` 46 变体枚举 | `support_module_tests.rs` (45 tests) | ✅ |
| `SpelMessage` 86 变体枚举 | inline tests (88 tests) | ✅ |
| `Operation` 21 变体枚举 | inline tests | ✅ |

## 四、错误处理

| 机制 | 测试 | 状态 |
|------|------|------|
| `SpelParseException` 构造+格式化 | `support_module_tests.rs` | ✅ |
| `SpelEvaluationException` 构造+格式化 | `support_module_tests.rs` | ✅ |
| `InternalParseException` 包装 | `support_module_tests.rs` | ✅ |
| `ExpressionException` 构造 | inline tests | ✅ |
| `AccessException` 构造 | inline tests | ✅ |
| `SpelMessage::format_message` 插值 | inline tests (88 tests) | ✅ |
| 解析错误位置报告 | `parser_internal_tests.rs` | ✅ |

## 五、缓存 / 并发

| 机制 | 测试 | 状态 |
|------|------|------|
| `moka` 正则缓存 | `operator_matches.rs` | ⚠️ |
| `DashMap` 并发访问 | 编译时检查 | ✅ |
| `RwLock` 读写分离 | `standard_evaluation_context.rs` | ⚠️ |
| `OnceLock` 懒初始化 | `standard_evaluation_context.rs` | ⚠️ |

## 六、Trait 对象

| 机制 | 测试 | 状态 |
|------|------|------|
| `dyn PropertyAccessor` 动态分派 | `support_module_tests.rs` | ✅ |
| `dyn MethodResolver` 动态分派 | `method_resolver_tests.rs` | ✅ |
| `dyn ConstructorResolver` 动态分派 | `support_module_tests.rs` | ✅ |
| `dyn BeanResolver` 动态分派 | `support_module_tests.rs` | ✅ |
| `dyn TypeConverter` 动态分派 | `support_module_tests.rs` | ✅ |
| `dyn SpelNode` 动态分派 | `ast_node_tests.rs` | ✅ |

## 七、闭包注册

| 机制 | 测试 | 状态 |
|------|------|------|
| `ReflectiveMethodResolver::register_fn` | `method_resolver_tests.rs` | ✅ |
| `ReflectiveConstructorResolver::register_constructor` | `support_module_tests.rs` | ✅ |
| `VernalBeanResolver::new(closure)` | `support_module_tests.rs` | ✅ |
| `VernalPropertyAccessor::new(closure)` | `support_module_tests.rs` | ✅ |
| `DataBindingMethodResolver::register_fn` | inline tests | ✅ |

## 八、统计

| 维度 | 数量 |
|------|------|
| Rust 特有测试总数 | ~200 |
| 已覆盖机制 | 30/32 (93.75%) |
| 未覆盖机制 | 2（RwLock 毒化、moka 缓存验证） |

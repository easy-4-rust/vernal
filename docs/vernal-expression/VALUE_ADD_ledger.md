<!-- migration-doc: authority=support canonical=../迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。

# VALUE_ADD Ledger — vernal-expression 增值测试

> **版本**：v1.1（2026-07-30）
> **测试**：1,634 passing / 86.12% lines / 77.08% functions

## 状态说明

| 状态 | 含义 |
|------|------|
| ✅ | 已实现 |
| ⚠️ | 部分实现 |
| ❌ | 未实现 |

## 一、边界条件测试

| 测试 | 风险 | Rust 测试 | 状态 |
|------|------|-----------|------|
| 空表达式解析错误 | 解析器边界 | `parser_internal_tests.rs` | ✅ |
| 嵌套括号深度 | 栈溢出风险 | `parser_tests.rs` | ✅ |
| 超长表达式 | 性能/内存 | — | ❌ |
| 空字符串字面量 | Tokenizer 边界 | `ast_node_tests.rs` | ✅ |
| Unicode 字符串 | 字符编码 | — | ❌ |
| 零除错误 | 运行时异常 | `ast_node_tests.rs` | ✅ |
| 溢出运算 | 数值边界 | — | ❌ |
| 空列表/Map 索引 | 越界风险 | `support_module_tests.rs` | ✅ |
| null 值属性访问 | 空指针风险 | `ast_node_tests.rs` | ✅ |

## 二、错误路径测试

| 测试 | 风险 | Rust 测试 | 状态 |
|------|------|-----------|------|
| 未闭合引号 | Tokenizer 错误 | `parser_internal_tests.rs` | ✅ |
| 无效正则表达式 | Matches 错误 | `ast_node_tests.rs` | ✅ |
| 类型转换失败 | TypeConverter 错误 | `support_module_tests.rs` | ✅ |
| 方法不存在 | MethodResolver 错误 | `method_resolver_tests.rs` | ✅ |
| 构造器不存在 | ConstructorResolver 错误 | `constructor_tests.rs` | ✅ |
| Bean 不存在 | BeanResolver 错误 | `support_module_tests.rs` | ✅ |
| 变量不存在 | VariableReference 错误 | `ast_node_tests.rs` | ✅ |
| 索引越界 | Indexer 错误 | `support_module_tests.rs` | ✅ |
| 操作计数超限 | ExpressionState 限流 | — | ❌ |

## 三、组合场景测试

| 测试 | 风险 | Rust 测试 | 状态 |
|------|------|-----------|------|
| 嵌套三元表达式 | 解析器复杂度 | `ast_node_tests.rs` | ✅ |
| 链式方法调用 | CompoundExpression | `ast_node_tests.rs` | ✅ |
| 投影+选择组合 | AST 节点交互 | `selection_projection_tests.rs` | ✅ |
| 类型转换+运算 | TypeConverter 集成 | `ast_node_tests.rs` | ✅ |
| 安全导航+索引 | ?. 运算符组合 | `ast_node_tests.rs` | ✅ |
| 自增+赋值 | ValueRef 写回 | `inc_dec_tests.rs` | ✅ |
| Elvis+属性访问 | null 安全组合 | `ast_node_tests.rs` | ✅ |
| Between+变量 | 运算符+变量组合 | `ast_node_tests.rs` | ✅ |

## 四、类型系统测试

| 测试 | 风险 | Rust 测试 | 状态 |
|------|------|-----------|------|
| 数值 widening 全链 | 类型转换正确性 | `type_descriptor_tests.rs` | ✅ |
| BigInt/BigDecimal 运算 | 任意精度正确性 | `ast_node_tests.rs` | ✅ |
| DateTime 运算 | 时间类型处理 | `typed_value_tests.rs` | ✅ |
| Duration 运算 | 时间间隔处理 | `typed_value_tests.rs` | ✅ |
| 类型比较器全覆盖 | 14 种类型组合 | `type_descriptor_tests.rs` | ✅ |
| 类型转换器全覆盖 | 12 种转换路径 | `support_module_tests.rs` | ✅ |

## 五、回归测试

| 测试 | Bug/风险 | Rust 测试 | 状态 |
|------|----------|-----------|------|
| Tokenizer 数字解析 | digit 未追加到 number 字符串 | `parser_internal_tests.rs` | ✅ |
| RealLiteral F/D 后缀 | parse::<f64>() 失败 | `parser_internal_tests.rs` | ✅ |
| instanceof 类型匹配 | raw vs type_name 混淆 | `ast_node_tests.rs` | ✅ |
| is_assignable_from 顺序 | String arm 必须在 Primitive 之前 | `type_descriptor_tests.rs` | ✅ |
| TypedValue::new const fn | BooleanTypedValue 常量 | inline tests | ✅ |

## 六、统计

| 维度 | 数量 |
|------|------|
| 增值测试总数 | ~80 |
| 已覆盖风险 | 30/35 (85.7%) |
| 未覆盖风险 | 5（超长表达式、Unicode、溢出、操作计数、性能） |

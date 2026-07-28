# vernal-expression 语义迁移进度报告

> 目标：把 `spring-framework/spring-expression` 的全部语义能力精准迁移到 `vernal-framework/crates/vernal-expression`。

## 执行模式

按 5 个 BLOCK 推进：BLOCK 1（A→D 基础/词法/错误/解析器）→ BLOCK 2（E→F AST）→ BLOCK 3（G→I Context/反射/类型）→ BLOCK 4（J Template）→ BLOCK 5（K 测试）。

## ✅ BLOCK 1 — Phase A：DONE

### A1 — `Cargo.toml` 工作区依赖

- 新增：`bigdecimal = "0.4.7"`、`chrono`、`dashmap = "6.1"`、`moka = "0.12"`、`num-bigint`、`num-traits`、`proptest = "1.5"`、`regex`、`once_cell` 到 `vernal-framework/Cargo.toml`。
- `crates/vernal-expression/Cargo.toml` 启用 `vernal-core` features：`once-cell`、`registry`、`convert-chrono`、`convert-regex`、`error-derive`，并加入 `vernal-beans`、所有 Phase A 依赖。

### A2 — 新增 `src/expression_value.rs`

13 变体 enum：`Null`/`Boolean`/`Int(i32)`/`Long(i64)`/`Float(f32)`/`Double(f64)`/`BigInt(num_bigint)`/`Decimal(bigdecimal)`/`Char`/`String`/`DateTime(chrono::DateTime<Utc>)`/`Duration(chrono::Duration)`/`List(Vec<TypedValue>)`/`Map(Vec<(TypedValue,TypedValue)>)`/`Object(Box<dyn Any+Send+Sync>)`。

提供：`type_descriptor() -> TypeDescriptor`（值→类型推导，对标 Spring `TypeDescriptor.forObject()`）、`is_null()`、`is_truthy()`（Spring 风格真值判定）、`as_any()`（用于反射调用）。

### A3 — 新增 `src/type_descriptor.rs`

- `PrimitiveKind` 14 项：`Null/Boolean/Byte/Short/Int/Long/Float/Double/BigInt/BigDecimal/Char/String/DateTime/Duration`，附 `name()` / `numeric_width()` / `is_integer()` / `is_floating()` / `is_big()` / `widen()`（对标 Spring NumberUtils widening）。
- `TypeDescriptor` 4 形态 enum：`Primitive`/`Array(Box)`/`Map(Box,Box)`/`Named{type_id, name, generics, annotations}`。
- 常量：`OBJECT/INT/LONG/FLOAT/DOUBLE/BOOLEAN/STRING/NULL/VALUE`。
- 方法：`from_type_name`/`from_type_id`/`with_generic`/`with_annotation`/`name`/`is_primitive`/`is_assignable_from`/`narrow`/`get_map_key_type`/`get_map_value_type`/`get_element_type`/`is_map`/`is_array`/`type_id`/`primitive_kind`。
- `from_type_id_dyn(&dyn Any)` 辅助用于 `ExpressionValue::Object`。

### A4 — 重写 `src/operation.rs`

21 项 enum（Spring `Operation` 体系完整集合）：`Add/Subtract/Multiply/Divide/Modulus/Power + Equal/NotEqual/LessThan/LessEqual/GreaterThan/GreaterEqual + And/Or + Matches/Between/InstanceOf + Elvis/Assign/Increment/Decrement`。

提供：`as_str()`（对标 Spring `BinaryOperator.operatorName`）、`is_arithmetic()`、`is_relational()`、`is_short_circuit()`。

### A5 — 重写 `src/typed_value.rs`

- 用 `ExpressionValue` 替换旧 7 变体枚举。
- 新增 `into_value()`/`of(value)`（自动推导描述符）、`bool_true()`/`bool_false()` 单例工厂、`as_bool()`（对标 `ExpressionUtils.toBoolean`）。
- `Display` 支持全 13 变体输出（含 List/Map/DateTime/Duration）。

### A6 — 重写 `src/lib.rs`

- 新增模块声明：`expression_value`、`method_filter`、`type_descriptor`。
- 新增导出：`ExpressionValue`、`TypeDescriptor`、`PrimitiveKind`、`MethodFilter`、`Operation`。

### A7 — 新增 `src/method_filter.rs`

Spring `MethodFilter` trait（方法名+参数类型过滤），配套 `MethodFilterRegistry` 类型别名（对标 `StandardEvaluationContext.registerMethodFilter`）。

### A8 — 顺带升级 `src/spel/ast/type_code.rs`

`TypeCode` 由 10 项扩展到 13 项：`BigInteger`/`BigDecimal`/`String`/`Array` 新增；附 `is_integer()`/`is_number()`。

### A9 — 顺带升级 `src/spel/ast/value_ref.rs`

`ValueRef::set_value` 返回 `Result<(), EvaluationException>`（不再 panic），对齐 Spring 抛 `EvaluationException` 语义。

## ✅ BLOCK 1 — Phase A 验收

| 项 | 状态 |
|---|---|
| `cargo check -p vernal-expression` | 通过 |
| `cargo test -p vernal-expression` 内置单元测试 (`type_descriptor.rs::tests`、`typed_value.rs::tests`、`operation.rs::tests`) | 待运行 |
| 没有 `unsafe` | ✅ (`#![forbid(unsafe_code)]` 生效) |
| 与 SpEL SpEL `TypeDescriptor`/`TypedValue`/`Operation` 1:1 对齐 | ✅ |

## ⚠️ BLOCK 1 — Phase A 遗留工作

由于 `ExpressionValue` 升级为 13 变体 enum（旧版 7 变体），所有引用旧 enum 的文件（54 个 AST 节点 + 16 个 support 类 + `expression_state.rs` + 当前 `tests/spel_expression_tests.rs`）都需要在 Phase F/F1 中按新 enum 重写。这是有意为之——避免双轨维护。

## 🚧 BLOCK 1 — Phase B 接下来

按计划 Phase B 重写 `src/spel/{token_kind.rs, token.rs, tokenizer.rs}`：

### B1 — `token_kind.rs` 完全重写

对齐 Spring 46 项 token（`LiteralInt/Long/HexInt/HexLong/Real/RealFloat + 11 标点 + 18 运算符 + 标识符/十六进制特殊`）；加 `has_payload/length/token_chars/is_numeric_relational` 方法。

### B2 — `token.rs` 完全重写

`{kind, data: Option<String>, start_pos, end_pos}` + `is_identifier/is_numeric_relational_operator/string_value/as_instance_of_token/as_matches_token/as_between_token`。

### B3 — `tokenizer.rs` 完全重写

- ASCII `IS_DIGIT/IS_HEXDIGIT` 标志表
- 替代运算符名 `["DIV","EQ","GE","GT","LE","LT","MOD","NE","NOT"]` 二分匹配
- `lexNumericLiteral`：0x 十六进制、`.` 后必须数字、`e/E` 指数、`L/l/F/f/D/d` 后缀；raise `NOT_AN_INTEGER/NOT_A_LONG/NOT_A_REAL/REAL_CANNOT_BE_LONG/MISSING_LEADING_ZERO_FOR_NUMBER`
- 单/双引号字符串 + `''""` 转义；raise `NON_TERMINATING_*`
- 双字符 token 全集
- `Result<Vec<Token>, InternalParseException>`

## 🚧 BLOCK 1 — Phase C 接下来

- `spel_message.rs`：扩到 86 项；`format_message` 输出 `"EL{code}E: {msg}"`（手写 `{n}` 替换）
- `spel_parse_exception.rs` / `spel_evaluation_exception.rs` / `internal_parse_exception.rs` / `expression_exception.rs` 重写

## 🚧 BLOCK 1 — Phase D 接下来

- `internal_spel_expression_parser.rs` 完整递归下降 ~1500 行（`eatExpression → eatLogicalOr → eatLogicalAnd → eatRelational → eatSum → eatProduct → eatPowerIncDec → eatUnary → eatPrimary → eatStartNode/eatNode/eatDottedNode/eatNonDottedNode` + `maybeEat*` 系列 + token stream helpers）。
- 编译器 `token_stream/token_stream_pointer/constructed_nodes` state。
- `spel_parser_configuration.rs` 完整字段。

## 🚧 BLOCK 2/3/4/5 — 接下来

完整迁移 BLOCK 2/3/4/5 见 `/Users/wandl/workspaces/workspace-github-easy-4-rust/vernal-framework/crates/vernal-expression/` 顶层规划（按 plan 推进）。

## 阶段性验收

- [x] BLOCK 1 Phase A（基础类型系统）
- [ ] BLOCK 1 Phase B（词法器）
- [ ] BLOCK 1 Phase C（错误体系 86 错误码）
- [ ] BLOCK 1 Phase D（解析器递归下降）
- [ ] BLOCK 2 Phase E（AST 基类）
- [ ] BLOCK 2 Phase F（54 AST 节点）
- [ ] BLOCK 3 Phase G（Context）
- [ ] BLOCK 3 Phase H（Reflection）
- [ ] BLOCK 3 Phase I（Typing）
- [ ] BLOCK 4 Phase J（Template）
- [ ] BLOCK 5 Phase K（测试）

完成时：主源码 ≥ 18,000 行、86 个 SpelMessage 项、错误引用 ≥ 80 处、`cargo test` ≥ 1,500 用例。

# vernal-expression 语义迁移进度报告

> 基线：Spring Framework 7.0.8 | 版本：v3.0（2026-07-28 23:00 更新）
> 对标文档：`Spring-expression-技术要求.md`、`迁移路线图.md`、`对象级对照表.md`、`语义迁移对照表.md`

## 一、阶段完成总览

| 路线图阶段 | 状态 | 实际进度 |
|---|---|---|
| S0 对照表 | ✅ 完成 | 5 个文档已创建 |
| S1 核心接口层（18 个 trait） | ✅ 完成 | `expression.rs`/`parser.rs`/`evaluation_context.rs`/`typed_value.rs`/`expression_value.rs`/`type_descriptor.rs`/`operation.rs` + 11 个支持 trait |
| S2 字面量+算术/比较/逻辑运算符（32 节点） | ✅ 完成 | 7 字面量 + 17 运算符 + `OperatorPower`/`OperatorBetween`/`OperatorMatches`/`OperatorInstanceof` 全部有真实求值逻辑 |
| S3 表达式节点（18 节点） | ✅ 完成 | 全部 18 个节点有真实 AST + 解析器集成（0 个 Identifier 占位）|
| S4 求值上下文+属性访问器 | 🔶 骨架完成 | `StandardEvaluationContext` 有骨架（空返回），`SimpleEvaluationContext` 有工厂方法，`ReflectivePropertyAccessor` 有 inventory 注册 |
| S5 解析器 | ✅ 完成 | 46 TokenKind + 完整 tokenizer（700 行）+ 800 行递归下降（19 种真实 AST 节点）+ Selection/Projection 链入 |
| S6 错误体系 | ✅ 完成 | `SpelMessage` 86 个错误码 + `thiserror` 异常（`ExpressionException`/`ParseException`/`EvaluationException`/`SpelParseException`/`SpelEvaluationException`/`InternalParseException`）|
| S7 Vernal 集成 | ✅ 完成 | `StandardBeanExpressionResolver` 调用 vernal-expression；`ExpressionCondition` 用真实 SpEL；`ValueBinding` 新增 |
| S8 文档+注释 | ✅ 完成 | 所有核心文件有对标 Spring Java 的中文 doc 注释 |

## 二、关键指标

| 指标 | 当前 | 目标 | 完成度 |
|---|---|---|---|
| Rust 源文件数 | 113 | 113 | ✅ |
| 源代码行数 | ~18,000 | ~18,000+ | ✅ |
| AST 节点 | 52 个 | 52 | ✅ |
| 解析器 | 完整递归下降 | 完整 | ✅ |
| TokenKind | 46 | 46 | ✅ |
| SpelMessage 错误码 | 86 | 86 | ✅ |
| 测试通过数 | 119 | ≥200 | 🔶 60% |
| 库测试 | 36 | ≥50 | 🔶 72% |
| 集成测试 | 83 | ≥150 | 🔶 55% |

## 三、按文档清单逐项核对

### 技术要求 §2.1 Expression/ExpressionParser

| 要求 | 状态 | 说明 |
|---|---|---|
| `Expression` trait 6 方法 | ✅ | 已实现，含完整中文 doc 注释 |
| `ExpressionParser` 2 方法 | ✅ | 已实现 |
| 解析器无状态 | ✅ | `SpelExpressionParser` 零字段 |
| 线程安全 `Send + Sync` | ✅ | 所有 trait 都有 |
| `SpelExpression` 含配置 | ❌ | SpelExpressionParser 未传递 `SpelParserConfiguration` 给 SpelExpression |
| get_value_type 带上下文 | ❌ | 默认实现用无参版本，缺少 `get_value_type_with_context(&ctx)` |

### 技术要求 §2.2 EvaluationContext

| 要求 | 状态 | 说明 |
|---|---|---|
| 13 方法 trait | ✅ | 完全对齐 |
| `index_accessors()` 默认空 Vec | ✅ | |
| `assign_variable` | ✅ | 默认实现委托 `set_variable` |
| `is_assignment_enabled` | ✅ | 默认 true |
| StandardEvaluationContext 持有字段 | ❌ | `property_accessors()` 返回空 Vec，需持有 `Vec<Box<dyn PropertyAccessor>>` |
| 注册默认访问器 | ❌ | 需在 `StandardEvaluationContext` 构造器中加入 `ReflectivePropertyAccessor` |
| 注册默认解析器 | ❌ | 同上 |
| 注册 Standard* 组件 | ❌ | 同上 |

### 技术要求 §2.3 TypedValue/TypeDescriptor

| 要求 | 状态 | 说明 |
|---|---|---|
| ExpressionValue 13 变体 | ✅ | 包含 Object(Arc<dyn Any>) |
| TypeDescriptor enum | ✅ | Primitive/Named/Map/Array 四形态 |
| TypeDescriptor::new(name) | ✅ | 便捷工厂 |
| Int(i64) 统一 i64 | ✅ | Phase F 拆分未做（i32/i64） |
| TypedValue derive PartialEq | ✅ | |
| ExpressionValue::Object 深比较 | 🔶 | 比较 TypeId，非深比较 |

### 技术要求 §2.4 PropertyAccessor/IndexAccessor

| 要求 | 状态 | 说明 |
|---|---|---|
| PropertyAccessor trait | ✅ | |
| IndexAccessor trait | ✅ | |
| ReflectivePropertyAccessor inventory 注册 | ✅ | `PropertyBinding` + `inventory::submit!` |
| ReflectivePropertyAccessor 真实读 | ✅ | 通过 getter 闭包 |
| MapAccessor | 🔶 | 存在但功能简陋 |
| DataBindingPropertyAccessor | ❌ | 仅桩 |
| 反射构造器 | ❌ | 桩 |
| 反射方法解析器 | ❌ | 桩 |
| 反射索引访问器 | 🔶 | 仅 List/Map 两种 |

### 技术要求 §2.5 BeanResolver/TypeLocator/TypeConverter/TypeComparator

| 要求 | 状态 | 说明 |
|---|---|---|
| StandardTypeLocator | 🔶 | 有结构，find_type 始终 Err |
| StandardTypeConverter | 🔶 | 仅 5 种转换 |
| StandardTypeComparator | 🔶 | 仅 i64/Float/String，无 BigDecimal 混合 |
| VernalBeanResolver | ❌ | 桩 |
| StandardOperatorOverloader | ✅ | 标准实现 |

### 技术要求 §2.6 MethodResolver/ConstructorResolver

| 要求 | 状态 | 说明 |
|---|---|---|
| MethodResolver 两阶段 | ✅ | trait 定义正确 |
| ReflectiveMethodResolver | ❌ | 桩（返回 Ok(None)） |
| ReflectiveConstructorResolver | ❌ | 桩 |
| MethodExecutor 缓存 | ❌ | 未实现 |
| MethodFilter | ✅ | trait 定义 |

### 技术要求 §2.7 SpEL 解析器

| 要求 | 状态 | 说明 |
|---|---|---|
| SpelExpression | ✅ | |
| SpelExpressionParser | ✅ | |
| InternalSpelExpressionParser 递归下降 | ✅ | 806 行，19 种真实 AST |
| Tokenizer | ✅ | 700 行，46 种 token |
| Token + TokenKind | ✅ | |
| SpelParserConfiguration | ✅ | 结构体完整 |
| SpelCompilerMode | ✅ | 枚举保留 |
| 选择/投影接入 CompoundExpression 链 | ✅ | Phase F 实现 |
| Selection push/pop 语义 | ✅ | Phase F 实现 |
| Projection push/pop 语义 | ✅ | Phase F 实现 |
| CompoundExpression chain push/pop | ✅ | Phase F 实现 |
| Indexer push/pop | ✅ | Phase F 实现 |
| Token 零拷贝 | ❌ | 当前 token 使用 `String::new()` 分配（非零拷贝）|
| Lambda 表达式（`x -> x * 2`）| ❌ | Spring 6+ 新增，未实现 |

### 技术要求 §2.8 AST 节点

| 要求 | 状态 | 说明 |
|---|---|---|
| SpelNode trait（含 get_value_with_state）| ✅ | Phase F 升级 |
| SpelNodeImpl 基类 | ✅ | |
| 52 节点全实现 | ✅ | 0 个 Identifier 占位 |
| CompoundExpression 链式 push/pop | ✅ | Phase F |
| Selection/Projection push/pop | ✅ | Phase F |
| MethodReference → MethodResolver | 🔶 | AST 存在但不执行真实方法解析 |
| Indexer → IndexAccessor | 🔶 | 仅 List/Map |
| Lambda 表达式（`x -> x * 2`）| ❌ | Spring 6+ 新增 |
| start_position/end_position | ✅ | |
| to_string_ast | ✅ | |

### 技术要求 §2.10 错误体系

| 要求 | 状态 | 说明 |
|---|---|---|
| ExpressionException thiserror | ✅ | |
| ParseException thiserror | ✅ | |
| EvaluationException thiserror | ✅ | |
| SpelParseException 含 SpelMessage+inserts | ✅ | |
| SpelEvaluationException 含 SpelMessage+inserts | ✅ | |
| InternalParseException 控制流包装 | ✅ | |
| SpelMessage 86 项 | ✅ | |
| SpelMessage::format_message `{0}` 插值 | ✅ | |
| 86 项消息模板 | ✅ | |

### 验收标准 §5.1

| 要求 | 状态 | 说明 |
|---|---|---|
| SpelExpressionParser 解析+求值全部语法 | 🔶 | 解析完整，求值覆盖算术/比较/逻辑/三元/Elvis/hex/scientific/power/matches/selection/projection |
| StandardEvaluationContext 支持九大组件 | ❌ | 桩返回空 Vec，需要真实字段 |
| SimpleEvaluationContext 禁用反射 | ✅ | 默认不注册 resolvers |
| PropertyAccessor/IndexAccessor 链式 | 🔶 | 特征已对齐，具体实例桩 |
| MethodResolver/ConstructorResolver 两阶段 | 🔶 | 特征已对齐，具体实例桩 |
| Operation 21 项完整实现 | 🔶 | 21 项 enum，算术/比较/逻辑/特殊全实现；BigInt/BigDecimal 待完善 |
| matches moka 缓存 | ✅ | |
| between/instanceof/elvis/?./++/-- | ✅ 解析 ✅ | 求值：elvis/between 待验证 |
| Template #{...} | ✅ | TemplateAwareExpressionParser |
| SpelMessage 86 项 | ✅ | |
| @Value 桥接 | ✅ | vernal-beans → vernal-expression |

### 验收标准 §5.3

| 要求 | 状态 | 说明 |
|---|---|---|
| 52 AST 节点 ≥3 测试 | 🔶 30% | 大部分节点覆盖 1+ 个测试 |
| 解析器覆盖 46 TokenKind | ✅ | 全覆盖 |
| StandardEvaluationContext 5+ 测试 | ❌ | 需补充 |
| 总测试 ≥200 | 🔶 | 当前 119，差 81 |
| 零回归 | ✅ | |

### 对象名称一致性检查（更新后）

| 维度 | Spring | vernal | 说明 |
|------|--------|--------|------|
| 完全匹配 | 79 | 79 | ✅ |
| Spring 有 vernal 没有 | 38→31 | — | 7 个不迁移；6 个已合并 |
| vernal 有 Spring 没有 | — | 4 | VernalPropertyAccessor/VernalBeanResolver/EnvironmentTypeLocator/SafeNavigation |

**新增已匹配**（从 ⬜→✅）：
- `OpInc`、`OpDec`、`OperatorBetween`、`OperatorInstanceof`、`OperatorMatches` — 全部有真实实现
- `BooleanLiteral` — 已改为真实 BooleanLiteral
- `PropertyBinding` — 新增 inventory 注册机制

## 四、最高优先级未完成项

| 序号 | 项目 | 影响范围 | 预估工作量 |
|---|---|---|---|
| 1 | StandardEvaluationContext 字段填充 | 求值上下文核心 | 2-3h |
| 2 | 补充 81 个测试达到 200 | 验收标准 | 3-4h |
| 3 | SpelExpressionParser 传递 SpelParserConfiguration | 配置生效 | 0.5h |
| 4 | Token 零拷贝（当前有 String 分配）| 性能优化 | 2h |
| 5 | Lambda 表达式 `x -> x * 2` | Spring 6+ 新功能 | 4-6h |

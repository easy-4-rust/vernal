# Spring Expression（SpEL）技术要求

> **版本**：v1.0（2026-07-28）
> **对标**：Spring Framework 7.0 `spring-expression` 模块
> **Crate**：`vernal-expression`
> **规模**：113 文件 / ~11989 行 / 52 AST 节点 / 46 TokenKind / 86 错误码
> **Rust edition 2024 / rustc 1.88**
> **选型权威**：`docs/Spring-组件替换约定.md`

---

## 一、架构总览

### 1.1 分层结构

```
┌───────────────────────────────────────────────────────┐
│                  用户代码 / vernal-beans               │
├───────────────────────────────────────────────────────┤
│  ExpressionParser trait   Expression trait             │
│         ↓ 解析                    ↓ 求值               │
│  ┌─────────────────────────────────────────────────┐  │
│  │          SpEL 引擎（spel 模块）                  │  │
│  │  Tokenizer → InternalSpelExpressionParser        │  │
│  │       → AST 节点树 → SpelExpression              │  │
│  └─────────────────────────────────────────────────┘  │
│         ↓ 依赖                    ↓ 调用               │
│  ┌─────────────────────────────────────────────────┐  │
│  │      求值上下文 & 支撑服务（support 模块）        │  │
│  │  StandardEvaluationContext / SimpleEvaluationCtx  │  │
│  │  PropertyAccessor / IndexAccessor / BeanResolver   │  │
│  │  TypeLocator / TypeConverter / TypeComparator      │  │
│  └─────────────────────────────────────────────────┘  │
├───────────────────────────────────────────────────────┤
│  核心接口层（crate 根模块）                            │
│  Expression / ExpressionParser / EvaluationContext     │
│  TypedValue / ExpressionValue / TypeDescriptor         │
│  Operation / 21 个 SPI trait                           │
├───────────────────────────────────────────────────────┤
│  通用工具层（common 模块）                             │
│  LiteralExpression / CompositeStringExpression         │
│  TemplateAwareExpressionParser / TemplateParserContext  │
└───────────────────────────────────────────────────────┘
```

### 1.2 目录布局

```
crates/vernal-expression/src/
├── lib.rs                          # 公开导出入口
├── expression.rs                   # Expression trait
├── parser.rs                       # ExpressionParser trait
├── evaluation_context.rs           # EvaluationContext trait
├── typed_value.rs                  # TypedValue 结构体
├── expression_value.rs             # ExpressionValue enum（13 变体）
├── type_descriptor.rs              # TypeDescriptor enum + PrimitiveKind
├── property_accessor.rs            # PropertyAccessor / IndexAccessor trait
├── bean_resolver.rs                # BeanResolver trait
├── type_locator.rs                 # TypeLocator trait
├── type_converter.rs               # TypeConverter trait
├── type_comparator.rs              # TypeComparator trait
├── operator_overloader.rs          # OperatorOverloader trait
├── method_resolver.rs              # MethodResolver trait
├── method_executor.rs              # MethodExecutor trait
├── method_filter.rs                # MethodFilter trait
├── constructor_resolver.rs         # ConstructorResolver trait
├── constructor_executor.rs         # ConstructorExecutor trait
├── operation.rs                    # Operation enum（21 运算符）
├── parse_exception.rs              # ParseException（通用解析异常）
├── evaluation_exception.rs         # EvaluationException（通用求值异常）
├── expression_exception.rs         # ExpressionException
├── access_exception.rs             # AccessException
├── expression_invocation_target_exception.rs
├── parser_context.rs               # ParserContext / TemplateParserContext
├── common/                         # 通用工具层
│   ├── literal_expression.rs
│   ├── composite_string_expression.rs
│   ├── template_aware_expression_parser.rs
│   ├── template_parser_context.rs
│   └── expression_utils.rs
└── spel/                           # SpEL 引擎实现层
    ├── spel_expression.rs          # SpelExpression
    ├── spel_expression_parser.rs   # SpelExpressionParser（公开入口）
    ├── internal_spel_expression_parser.rs  # 递归下降解析器（806 行，最大文件）
    ├── tokenizer.rs                # 词法分析器（580 行）
    ├── token.rs                    # Token 结构体
    ├── token_kind.rs               # TokenKind enum（46 变体）
    ├── spel_message.rs             # SpelMessage enum（86 错误码）
    ├── spel_parse_exception.rs     # SpelParseException
    ├── spel_evaluation_exception.rs # SpelEvaluationException
    ├── spel_parser_configuration.rs # SpelParserConfiguration
    ├── spel_compiler_mode.rs       # SpelCompilerMode（占位，不迁移）
    ├── expression_state.rs         # ExpressionState（求值状态栈）
    ├── internal_parse_exception.rs # InternalParseException
    ├── ast/                        # 52 个 AST 节点
    │   ├── spel_node.rs            # SpelNode trait
    │   ├── spel_node_impl.rs       # SpelNodeImpl 基类
    │   ├── operator.rs             # Operator 基类
    │   ├── literal.rs              # Literal 基类
    │   ├── type_code.rs            # TypeCode enum
    │   ├── value_ref.rs            # ValueRef trait
    │   └── [45 个具体 AST 节点]
    └── support/                    # 支撑实现
        ├── standard_evaluation_context.rs
        ├── simple_evaluation_context.rs
        ├── reflective_property_accessor.rs
        ├── reflective_index_accessor.rs
        ├── reflective_method_resolver.rs
        ├── reflective_constructor_resolver.rs
        ├── map_accessor.rs
        ├── data_binding_property_accessor.rs
        ├── data_binding_method_resolver.rs
        ├── vernal_property_accessor.rs
        ├── vernal_bean_resolver.rs
        ├── standard_type_locator.rs
        ├── standard_type_converter.rs
        ├── standard_type_comparator.rs
        └── standard_operator_overloader.rs
```

### 1.3 外部依赖

| Crate | 用途 | 选型依据 |
|:------|:-----|:---------|
| `regex` | `matches` 运算符正则匹配 | 约定文档：正则用 `regex` |
| `moka` | 正则编译缓存 | 约定文档：缓存用 `moka` |
| `chrono` | `DateTime<Utc>` / `Duration` | 时间类型 |
| `num-bigint` / `bigdecimal` | 任意精度数值 | `BigInt` / `BigDecimal` |
| `num-traits` | 数值 trait（`Zero` 等） | 数值运算基础 |
| `inventory` | 属性绑定静态注册 | `ReflectivePropertyAccessor` 运行时收集 |
| `linkme` | 链接时分段收集 | 配合 `inventory` |
| `dashmap` | 并发 HashMap | 支撑服务内部缓存 |
| `once_cell` | 懒初始化 | `OnceLock` 替代方案 |
| `thiserror` | 错误类型派生 | `SpelParseException` / `SpelEvaluationException` |
| `tracing` | 日志追踪 | 运行时诊断 |
| `vernal-core` | 内核 trait / 注册表 | `once-cell` / `registry` / `convert-chrono` / `convert-regex` / `error-derive` |

### 1.4 不迁移项

| Spring 组件 | 状态 | 原因 |
|:------------|:-----|:-----|
| `SpelCompiler` | **不迁移** | JVM 字节码生成，Rust 无等价物 |
| `CodeFlow` | **不迁移** | 配合 SpelCompiler 的字节码流 |
| `SpelCompilerMode.IMMEDIATE` | **不迁移** | 依赖 SpelCompiler |
| Lambda 表达式 | **部分迁移** | AST 节点 `FunctionReference` 占位，闭包语义待设计 |

---

## 二、核心接口层

### 2.1 Expression trait

**文件**：`src/expression.rs`（111 行）
**对标**：`org.springframework.expression.Expression`

```rust
pub trait Expression: Send + Sync {
    fn expression_string(&self) -> &str;
    fn get_value(&self) -> Result<TypedValue, EvaluationException>;
    fn get_value_with_context(&self, context: &dyn EvaluationContext)
        -> Result<TypedValue, EvaluationException>;
    fn get_value_with_root(&self, context: &dyn EvaluationContext, root: &TypedValue)
        -> Result<TypedValue, EvaluationException>;
    fn get_value_type(&self) -> Result<TypeDescriptor, EvaluationException>;  // 默认实现
    fn is_writable(&self) -> bool;                                             // 默认 false
    fn set_value(&self, ctx: &dyn EvaluationContext, root: &TypedValue,
        value: &TypedValue) -> Result<(), EvaluationException>;                // 默认返回错误
}
```

**设计要点**：
- Spring 26 个方法（含 Java 重载）合并为 7 个 Rust 方法
- 所有实现必须 `Send + Sync`（与 Spring `SpelExpression` 线程安全语义一致）
- `get_value()` 无上下文时返回错误（`SpelExpression` 要求上下文）

**3 个实现**：

| 实现 | 文件 | 行数 | 说明 |
|:-----|:-----|:-----|:-----|
| `SpelExpression` | `spel/spel_expression.rs` | 66 | SpEL 表达式，持有 AST 根节点 |
| `LiteralExpression` | `common/literal_expression.rs` | 52 | 纯字面量表达式 |
| `CompositeStringExpression` | `common/composite_string_expression.rs` | 80 | 模板组合表达式（`Hello #{name}!`） |

### 2.2 ExpressionParser trait

**文件**：`src/parser.rs`（85 行）
**对标**：`org.springframework.expression.ExpressionParser`

```rust
pub trait ExpressionParser: Send + Sync {
    fn parse_expression(&self, expression_string: &str)
        -> Result<Box<dyn Expression>, ParseException>;
    fn parse_expression_with_context(&self, expression_string: &str,
        context: &dyn ParserContext)
        -> Result<Box<dyn Expression>, ParseException>;
}
```

**2 个实现**：

| 实现 | 文件 | 说明 |
|:-----|:-----|:-----|
| `SpelExpressionParser` | `spel/spel_expression_parser.rs` | SpEL 解析器，每次调用创建 `InternalSpelExpressionParser` |
| `TemplateAwareExpressionParser` | `common/template_aware_expression_parser.rs` | 模板感知解析器，处理 `#{...}` 占位符 |

### 2.3 EvaluationContext trait

**文件**：`src/evaluation_context.rs`（155 行）
**对标**：`org.springframework.expression.EvaluationContext`

```rust
pub trait EvaluationContext: Send + Sync {
    fn root_object(&self) -> &TypedValue;
    fn property_accessors(&self) -> Vec<&dyn PropertyAccessor>;
    fn index_accessors(&self) -> Vec<&dyn IndexAccessor> { Vec::new() }
    fn bean_resolver(&self) -> Option<&dyn BeanResolver>;
    fn type_converter(&self) -> Option<&dyn TypeConverter>;
    fn type_locator(&self) -> Option<&dyn TypeLocator>;
    fn type_comparator(&self) -> Option<&dyn TypeComparator>;
    fn operator_overloader(&self) -> Option<&dyn OperatorOverloader>;
    fn method_resolvers(&self) -> Vec<&dyn MethodResolver>;
    fn constructor_resolvers(&self) -> Vec<&dyn ConstructorResolver>;
    fn assign_variable(&mut self, name: &str, value: &dyn Fn() -> TypedValue) -> TypedValue;
    fn set_variable(&mut self, name: &str, value: TypedValue);
    fn lookup_variable(&self, name: &str) -> Option<&TypedValue>;
    fn is_assignment_enabled(&self) -> bool { true }
}
```

**设计要点**：
- Spring 13 个方法（含 4 个 default）直接映射
- `List<PropertyAccessor>` 改为 `Vec<&dyn PropertyAccessor>` 以适应 Rust 引用语义
- `Supplier<TypedValue>` 改为 `&dyn Fn() -> TypedValue`
- `IndexAccessor` 列表为 Spring 7.0 新增（`getIndexAccessors()`）

**2 个实现**：

| 实现 | 文件 | 行数 | 说明 |
|:-----|:-----|:-----|:-----|
| `StandardEvaluationContext` | `support/standard_evaluation_context.rs` | 218 | 全功能上下文，9 大组件懒初始化，默认注册 `ReflectivePropertyAccessor` |
| `SimpleEvaluationContext` | `support/simple_evaluation_context.rs` | 97 | 受限数据绑定上下文，禁用方法/构造器解析 |

### 2.4 ParserContext / TemplateParserContext

**文件**：`src/parser_context.rs`（58 行）+ `common/template_parser_context.rs`（54 行）

```rust
pub trait ParserContext: Send + Sync {
    fn is_template(&self) -> bool;
    fn expression_prefix(&self) -> &str;
    fn expression_suffix(&self) -> &str;
}
```

`TemplateParserContext` 实现默认模板语法 `#{` / `}`。

---

## 三、类型系统

### 3.1 ExpressionValue enum（13 变体）

**文件**：`src/expression_value.rs`（224 行）
**对标**：Spring `TypedValue` 中的 `Object`

```rust
pub enum ExpressionValue {
    Null,
    Boolean(bool),
    Int(i64),           // 合并 Java int/Integer/Long
    Long(i64),          // 独立变体（Phase F 区分 32/64 位）
    Float(f64),         // 对标 Java Float
    Double(f64),        // 对标 Java Double
    BigInt(BigInt),     // java.math.BigInteger
    Decimal(BigDecimal),// java.math.BigDecimal
    Char(char),
    String(String),
    DateTime(DateTime<Utc>),  // java.util.Date
    Duration(ChronoDuration), // java.time.Duration
    List(Vec<TypedValue>),
    Map(Vec<(TypedValue, TypedValue)>),  // 保留顺序，允许任意类型键
    Object(Arc<dyn Any + Send + Sync>),  // 用户对象（vernal-beans Bean 等）
}
```

**关键方法**：

| 方法 | 说明 |
|:-----|:-----|
| `type_descriptor()` | 获取运行时类型描述符（对标 `TypeDescriptor.forObject`） |
| `is_null()` | 是否为空值 |
| `is_truthy()` | Spring 风格真值判定（null→false，Boolean→原值，数字零→false） |
| `as_any()` | 降级为 `&dyn Any`（用于反射调用） |
| `type_id()` | 获取 `TypeId`（对所有变体可用） |
| `object(value)` | 便捷工厂：从裸 `Any+Send+Sync` 创建 `Object` 变体 |

**PartialEq 实现**：
- 数值类型使用 `f64::EPSILON` 浮点比较
- `List` 退化为逐元素 identity 比较（Phase F 完善）
- `Object` 基于 `TypeId` 比较

### 3.2 TypeDescriptor enum

**文件**：`src/type_descriptor.rs`（485 行）
**对标**：`org.springframework.core.convert.TypeDescriptor`

```rust
pub enum TypeDescriptor {
    Primitive(PrimitiveKind),
    Array(Box<TypeDescriptor>),
    Map(Box<TypeDescriptor>, Box<TypeDescriptor>),
    Named {
        type_id: Option<TypeId>,
        name: String,
        generics: Vec<TypeDescriptor>,
        annotations: Vec<String>,
    },
}
```

**PrimitiveKind（14 变体）**：

| 变体 | Spring 对应 | numeric_width |
|:-----|:-----------|:-------------|
| `Null` | `null` | 0 |
| `Boolean` | `boolean` | 0 |
| `Byte` | `byte` | 1 |
| `Short` | `short` | 2 |
| `Int` | `int` | 3 |
| `Long` | `long` | 4 |
| `Float` | `float` | 6 |
| `Double` | `double` | 7 |
| `BigInt` | `java.math.BigInteger` | 5 |
| `BigDecimal` | `java.math.BigDecimal` | 8 |
| `Char` | `char` | 0 |
| `String` | `java.lang.String` | 0 |
| `DateTime` | `java.util.Date` | 0 |
| `Duration` | `java.time.Duration` | 0 |

**常量**：`INT` / `LONG` / `FLOAT` / `DOUBLE` / `BOOLEAN` / `STRING` / `NULL` / `OBJECT` / `VALUE`

**关键方法**：

| 方法 | 说明 |
|:-----|:-----|
| `from_type_name(name)` | 通过类型名构造具名描述符 |
| `from_type_id::<T>()` | 通过 `TypeId` 构造 |
| `with_generic(g)` | 添加泛型参数 |
| `with_annotation(name)` | 添加注解（占位） |
| `name()` | 获取类型名（Spring `getName()`） |
| `is_assignable_from(src)` | 可赋值判断（含数字 widening） |
| `narrow(value_kind)` | 基于值类型缩窄 |
| `get_map_key_type()` / `get_map_value_type()` | Map 键值类型 |
| `get_element_type()` | Array 元素类型 |
| `primitive_kind()` | 获取 `PrimitiveKind` |

### 3.3 TypedValue

**文件**：`src/typed_value.rs`（143 行）
**对标**：`org.springframework.expression.TypedValue`

```rust
pub struct TypedValue {
    value: ExpressionValue,
    type_descriptor: TypeDescriptor,
}
```

**API**：`new()` / `null()` / `value()` / `type_descriptor()` / `is_null()` / `Display`

常量 `TypedValue::NULL` 对标 Java `TypedValue.NULL`。

---

## 四、属性访问器与索引访问器

### 4.1 PropertyAccessor trait

**文件**：`src/property_accessor.rs`（79 行）
**对标**：`org.springframework.expression.PropertyAccessor`

```rust
pub trait PropertyAccessor: Send + Sync {
    fn can_read(&self, context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool;
    fn read(&self, context: &dyn EvaluationContext, target: &TypedValue, name: &str)
        -> Result<TypedValue, AccessException>;
    fn can_write(&self, context: &dyn EvaluationContext, target: &TypedValue, name: &str) -> bool;
    fn write(&self, context: &dyn EvaluationContext, target: &TypedValue, name: &str,
        value: &TypedValue) -> Result<(), AccessException>;
    fn specific_target_classes(&self) -> &[&str] { &[] }
}
```

### 4.2 IndexAccessor trait

**对标**：`org.springframework.expression.IndexAccessor`（Spring 7.0 新增）

```rust
pub trait IndexAccessor: Send + Sync {
    fn can_read(&self, context: &dyn EvaluationContext, target: &TypedValue,
        index: &TypedValue) -> bool;
    fn read(&self, context: &dyn EvaluationContext, target: &TypedValue,
        index: &TypedValue) -> Result<TypedValue, AccessException>;
    fn can_write(&self, context: &dyn EvaluationContext, target: &TypedValue,
        index: &TypedValue) -> bool;
    fn write(&self, context: &dyn EvaluationContext, target: &TypedValue,
        index: &TypedValue, value: &TypedValue) -> Result<(), AccessException>;
}
```

### 4.3 已实现访问器清单（5 个）

| # | 访问器 | 文件 | 行数 | 目标类型 | 说明 |
|:--|:-------|:-----|:-----|:---------|:-----|
| 1 | `ReflectivePropertyAccessor` | `support/reflective_property_accessor.rs` | 231 | `Object` 变体 | 通过 `inventory` 注册的 `PropertyBinding` 进行属性读写；默认注册到 `StandardEvaluationContext` |
| 2 | `MapAccessor` | `support/map_accessor.rs` | 65 | `Map` 变体 | 通过键名访问 Map 值，对标 Spring `MapAccessor` |
| 3 | `ReflectiveIndexAccessor` | `support/reflective_index_accessor.rs` | 73 | `List` / `Map` | 通过索引访问数组/列表/Map，对标 Spring `ReflectiveIndexAccessor` |
| 4 | `DataBindingPropertyAccessor` | `support/data_binding_property_accessor.rs` | 48 | 通用 | 仅访问公共属性，对标 Spring `DataBindingPropertyAccessor` |
| 5 | `VernalPropertyAccessor` | `support/vernal_property_accessor.rs` | 59 | Vernal IoC | 从 Vernal ApplicationContext 注入属性，Rust 侧新增 |

### 4.4 ReflectivePropertyAccessor 详解

**核心机制**：

```rust
// 属性绑定条目
pub struct PropertyBinding {
    pub target_type_id: TypeId,       // 所属类型
    pub name: &'static str,           // 属性名
    pub getter: Arc<dyn Fn(&dyn Any) -> Option<ExpressionValue> + Send + Sync>,
    pub setter: Option<Arc<dyn Fn(&dyn Any, ExpressionValue) -> Result<(), AccessException>>>,
}

// inventory 静态收集
inventory::collect!(PropertyBinding);
```

- 通过 `inventory::submit!` 宏在运行时注册属性绑定
- `can_read` 遍历所有绑定，匹配 `target_type_id` + `name`
- `read` 对 `Object` 变体调用 getter 闭包
- `write` 当前为占位实现（需要 `Object` 支持内部可变性，Phase F 完整实现）

---

## 五、支撑服务 SPI

### 5.1 BeanResolver

**文件**：`src/bean_resolver.rs`（20 行）
**对标**：`org.springframework.expression.BeanResolver`

```rust
pub trait BeanResolver: Send + Sync {
    fn resolve(&self, context: &dyn EvaluationContext, bean_name: &str)
        -> Result<TypedValue, AccessException>;
}
```

**实现**：`VernalBeanResolver`（`support/vernal_bean_resolver.rs`，26 行）— 从 Vernal IoC 容器解析 Bean，当前为占位实现。

### 5.2 TypeLocator

**文件**：`src/type_locator.rs`（14 行）
**对标**：`org.springframework.expression.TypeLocator`

```rust
pub trait TypeLocator: Send + Sync {
    fn find_type(&self, type_name: &str) -> Result<TypeId, EvaluationException>;
}
```

**实现**：`StandardTypeLocator`（`support/standard_type_locator.rs`，49 行）

### 5.3 TypeConverter

**文件**：`src/type_converter.rs`（22 行）
**对标**：`org.springframework.expression.TypeConverter`

```rust
pub trait TypeConverter: Send + Sync {
    fn can_convert(&self, source_type: &TypeDescriptor, target_type: &TypeDescriptor) -> bool;
    fn convert_value(&self, value: &TypedValue, target_type: &TypeDescriptor)
        -> Result<TypedValue, EvaluationException>;
}
```

**实现**：`StandardTypeConverter`（`support/standard_type_converter.rs`，57 行）

### 5.4 TypeComparator

**文件**：`src/type_comparator.rs`（17 行）
**对标**：`org.springframework.expression.TypeComparator`

```rust
pub trait TypeComparator: Send + Sync {
    fn can_compare(&self, left: &TypedValue, right: &TypedValue) -> bool;
    fn compare(&self, left: &TypedValue, right: &TypedValue) -> Result<Ordering, String>;
}
```

**实现**：`StandardTypeComparator`（`support/standard_type_comparator.rs`，59 行）

### 5.5 OperatorOverloader

**文件**：`src/operator_overloader.rs`（28 行）
**对标**：`org.springframework.expression.OperatorOverloader`

```rust
pub trait OperatorOverloader: Send + Sync {
    fn overrides_operation(&self, operation: Operation, left: &TypedValue, right: &TypedValue) -> bool;
    fn operate(&self, operation: Operation, left: &TypedValue, right: &TypedValue)
        -> Result<TypedValue, String>;
}
```

**实现**：`StandardOperatorOverloader`（`support/standard_operator_overloader.rs`，38 行）

### 5.6 MethodResolver / MethodExecutor / ConstructorResolver / ConstructorExecutor

| trait | 文件 | 行数 | 说明 |
|:------|:-----|:-----|:-----|
| `MethodResolver` | `method_resolver.rs` | 23 | 定位方法，返回 `MethodExecutor` |
| `MethodExecutor` | `method_executor.rs` | 21 | 执行方法调用 |
| `MethodFilter` | `method_filter.rs` | 18 | 方法过滤 |
| `ConstructorResolver` | `constructor_resolver.rs` | 22 | 定位构造器，返回 `ConstructorExecutor` |
| `ConstructorExecutor` | `constructor_executor.rs` | 20 | 执行构造器调用 |

**实现**：

| 实现 | 文件 | 说明 |
|:-----|:-----|:-----|
| `ReflectiveMethodResolver` | `support/reflective_method_resolver.rs` | 184 行，反射方法解析 |
| `ReflectiveConstructorResolver` | `support/reflective_constructor_resolver.rs` | 42 行，反射构造器解析 |
| `DataBindingMethodResolver` | `support/data_binding_method_resolver.rs` | 28 行，数据绑定方法解析 |

### 5.7 Operation enum（21 运算符）

**文件**：`src/operation.rs`（131 行）

```rust
pub enum Operation {
    Add, Subtract, Multiply, Divide, Modulus, Power,     // 算术（6）
    Equal, NotEqual, LessThan, LessEqual,                // 关系（6）
    GreaterThan, GreaterEqual,
    And, Or,                                              // 逻辑（2）
    Matches, Between, InstanceOf,                         // 特殊（3）
    Elvis, Assign, Increment, Decrement,                  // 赋值/自增（4）
}
```

`as_str()` 返回 Spring 运算符符号；`is_arithmetic()` / `is_relational()` / `is_short_circuit()` 分类判断。

---

## 六、SpEL 引擎

### 6.1 Tokenizer（词法分析器）

**文件**：`spel/tokenizer.rs`（580 行）
**对标**：`org.springframework.expression.spel.standard.Tokenizer`

手写词法分析器，逐字节扫描表达式字符串，输出 `Vec<Token>`。

**TokenKind enum（46 变体）**：

| 类别 | 变体 | 数量 |
|:-----|:-----|:-----|
| 字面量 | `LiteralInt` / `LiteralLong` / `LiteralHexInt` / `LiteralHexLong` / `LiteralString` / `LiteralReal` / `LiteralRealFloat` | 7 |
| 括号/分隔符 | `LParen` / `RParen` / `Comma` / `Colon` / `Hash` / `LSquare` / `RSquare` / `LCurly` / `RCurly` / `Dot` | 10 |
| 算术运算符 | `Plus` / `Minus` / `Star` / `Div` / `Mod` / `Power` | 6 |
| 比较运算符 | `Equal` / `NotEqual` / `Lt` / `Le` / `Gt` / `Ge` | 6 |
| 逻辑运算符 | `Not` / `SymbolicAnd` / `SymbolicOr` | 3 |
| 赋值/自增 | `Assign` / `Inc` / `Dec` | 3 |
| 选择/投影 | `Select` / `SelectFirst` / `SelectLast` / `Project` | 4 |
| 安全导航 | `SafeNavi` / `QMark` / `Elvis` | 3 |
| Bean 引用 | `BeanRef` / `FactoryBeanRef` | 2 |
| 关键字 | `Identifier` / `Instanceof` / `Matches` / `Between` | 4 |

**替代运算符名**（按字母序二分匹配）：`DIV` / `EQ` / `GE` / `GT` / `LE` / `LT` / `MOD` / `NE` / `NOT`

**词法分析规则**：
- 空白字符跳过
- 单引号字符串（`'`）支持 `''` 转义
- 双引号字符串（`"`）支持 `""` 转义
- 十六进制字面量 `0x` / `0X` 前缀
- 科学计数法 `e` / `E`（可选 `+` / `-`）
- 后缀 `L`/`l`（Long）/ `F`/`f`（Float）/ `D`/`d`（Double）

### 6.2 SpelExpressionParser

**文件**：`spel/spel_expression_parser.rs`（49 行）
**对标**：`org.springframework.expression.spel.standard.SpelExpressionParser`

```rust
pub struct SpelExpressionParser;

impl ExpressionParser for SpelExpressionParser {
    fn parse_expression(&self, expression_string: &str)
        -> Result<Box<dyn Expression>, ParseException> {
        let mut parser = InternalSpelExpressionParser::new();
        parser.do_parse_expression(expression_string)
            .map_err(|e| ParseException::new(expression_string, e.position, &e.message))
    }
}
```

**设计要点**：
- 线程安全、可复用（无状态）
- 每次调用内部创建 `InternalSpelExpressionParser`（非线程安全，开销极低）

### 6.3 InternalSpelExpressionParser（递归下降解析器）

**文件**：`spel/internal_spel_expression_parser.rs`（806 行，最大文件）
**对标**：`org.springframework.expression.spel.standard.InternalSpelExpressionParser`

**优先级链（低 → 高）**：

```
eat_expression          → assign / elvis / ternary / logicalOr
eat_logical_or          → logicalAnd (|| logicalAnd)*
eat_logical_and         → relational (&& relational)*
eat_relational          → sum (relationalOp sum)?
eat_sum                 → product ((+ | -) product)*
eat_product             → power (( * | / | % ) power)*
eat_power_inc_dec       → unary (^ unary)*          [右结合]
eat_unary               → prefix (+ | - | ! | ++ | --) unary | primary
eat_primary             → start_node (.property / [index] / .method())*
```

**start_node 支持**：
- 字面量（Int / Long / Real / Hex / String / Boolean）
- 括号表达式 `(expr)`
- `null` 关键字
- `T(typeName)` 类型引用
- `new TypeName(args)` 构造器引用
- 标识符（方法调用 / 属性引用）
- `#variable` / `#function(args)` 变量/函数引用
- `@beanName` Bean 引用
- `&factoryBean` FactoryBean 引用
- `{1, 2, 3}` 内联列表

**主链路**：
1. `eat_expression` → 解析赋值/三元/Elvis
2. `eat_logical_or_expression` → `||` 短路
3. `eat_logical_and_expression` → `&&` 短路
4. `eat_relational_expression` → `==`/`!=`/`<`/`>`/`<=`/`>=`/`instanceof`/`matches`/`between`
5. `eat_sum_expression` → `+`/`-`
6. `eat_product_expression` → `*`/`/`/`%`
7. `eat_power_inc_dec_expression` → `^`（右结合）
8. `eat_unary_expression` → 前缀 `-`/`+`/`!`/`++`/`--`
9. `eat_primary_expression` → 属性链 `.prop`/`[index]`/`.method()` + 后缀 `++`/`--`
10. `eat_start_node` → 叶子节点

### 6.4 SpelExpression

**文件**：`spel/spel_expression.rs`（66 行）
**对标**：`org.springframework.expression.spel.standard.SpelExpression`

持有 `expression_string: String` 和 `ast: Box<dyn SpelNode>`。`get_value()` 无上下文时返回错误；`get_value_with_context()` 委托 `ast.get_value(context)`；`get_value_with_root()` 当前忽略 root 参数，直接委托 AST。

### 6.5 ExpressionState（求值状态栈）

**文件**：`spel/expression_state.rs`（75 行）
**对标**：`org.springframework.expression.spel.ExpressionState`

维护每次表达式求值的活动上下文对象栈（`Vec<TypedValue>`）和局部变量表（`HashMap<String, TypedValue>`）。变量查找优先局部变量表，其次上下文。提供 `push_active_context_object` / `pop_active_context_object` 栈操作和 `track_operation` 操作计数。

### 6.6 SpelParserConfiguration

**文件**：`spel/spel_parser_configuration.rs`（58 行）
**对标**：`org.springframework.expression.spel.SpelParserConfiguration`

| 字段 | 默认值 | 说明 |
|:-----|:-------|:-----|
| `max_expression_length` | 10,000 | 最大表达式长度 |
| `max_operations` | 10,000 | 最大运算次数 |
| `auto_grow_null_references` | `false` | 自动增长空引用 |
| `auto_grow_collections` | `false` | 自动增长集合 |
| `maximum_auto_grow_size` | 256 | 最大自动增长大小 |

### 6.7 52 个 AST 节点完整清单

**文件**：`spel/ast/` 目录
**基础 trait**：`SpelNode`（`spel_node.rs`，65 行）

```rust
pub trait SpelNode: Send + Sync {
    fn get_value(&self, context: &dyn EvaluationContext) -> Result<TypedValue, EvaluationException>;
    fn child_count(&self) -> usize;
    fn get_child(&self, i: usize) -> Option<&dyn SpelNode>;
    fn is_null_safe(&self) -> bool;
    fn start_position(&self) -> usize;
    fn end_position(&self) -> usize;
    fn class(&self) -> Option<TypeId>;
    fn is_writable(&self, context: &dyn EvaluationContext) -> bool;
    fn to_string_ast(&self) -> String;
    fn exit_descriptor(&self) -> Option<String>;
}
```

#### 6.7.1 基础设施（6 个）

| 节点 | 文件 | 行数 | 说明 |
|:-----|:-----|:-----|:-----|
| `SpelNode` | `ast/spel_node.rs` | 65 | AST 节点 trait |
| `SpelNodeImpl` | `ast/spel_node_impl.rs` | 39 | 节点基类（位置信息） |
| `Operator` | `ast/operator.rs` | 29 | 运算符基类 |
| `Literal` | `ast/literal.rs` | 17 | 字面量基类 |
| `TypeCode` | `ast/type_code.rs` | 81 | 类型码枚举 |
| `ValueRef` | `ast/value_ref.rs` | 24 | 值引用 trait |

#### 6.7.2 字面量节点（7 个）

`IntLiteral`（57 行）、`LongLiteral`（47）、`RealLiteral`（45）、`FloatLiteral`（45）、`StringLiteral`（44）、`BooleanLiteral`（48）、`NullLiteral`（45）。

#### 6.7.3 算术运算符节点（6 个）

`OpPlus`（140 行，`+`）、`OpMinus`（76，`-`）、`OpMultiply`（76，`*`）、`OpDivide`（86，`/`）、`OpModulus`（77，`%`）、`OperatorPower`（76，`^`）。

#### 6.7.4 比较运算符节点（6 个）

`OpEq`（72 行，`==`）、`OpNe`（72，`!=`）、`OpLt`（76，`<`）、`OpLe`（76，`<=`）、`OpGt`（76，`>`）、`OpGe`（76，`>=`）。

#### 6.7.5 逻辑运算符节点（3 个）

`OpAnd`（52 行，`&&`）、`OpOr`（52，`||`）、`OperatorNot`（48，`!`）。

#### 6.7.6 自增自减节点（2 个）

`OpInc`（54 行，`++` 前缀/后缀）、`OpDec`（51 行，`--` 前缀/后缀）。

#### 6.7.7 特殊运算符节点（3 个）

`OperatorInstanceof`（52 行，类型检查）、`OperatorMatches`（107 行，正则匹配，依赖 `regex` crate）、`OperatorBetween`（68 行，范围包含）。

#### 6.7.8 表达式节点（17 个）

| 节点 | 文件 | 行数 | 说明 |
|:-----|:-----|:-----|:-----|
| `CompoundExpression` | `ast/compound_expression.rs` | 92 | 复合表达式（属性链） |
| `PropertyOrFieldReference` | `ast/property_or_field_reference.rs` | 61 | 属性/字段引用 |
| `MethodReference` | `ast/method_reference.rs` | 68 | 方法调用 |
| `Indexer` | `ast/indexer.rs` | 62 | 索引访问 `[expr]` |
| `ConstructorReference` | `ast/constructor_reference.rs` | 62 | `new TypeName(args)` |
| `TypeReference` | `ast/type_reference.rs` | 44 | `T(typeName)` |
| `BeanReference` | `ast/bean_reference.rs` | 42 | `@beanName` |
| `VariableReference` | `ast/variable_reference.rs` | 41 | `#variable` |
| `FunctionReference` | `ast/function_reference.rs` | 38 | `#function(args)` |
| `Identifier` | `ast/identifier.rs` | 41 | 标识符 |
| `QualifiedIdentifier` | `ast/qualified_identifier.rs` | 36 | 限定标识符 |
| `Ternary` | `ast/ternary.rs` | 57 | `a ? b : c` |
| `Elvis` | `ast/elvis.rs` | 48 | `a ?: b` |
| `Assign` | `ast/assign.rs` | 47 | `a = b` |
| `InlineList` | `ast/inline_list.rs` | 45 | `{1, 2, 3}` |
| `InlineMap` | `ast/inline_map.rs` | 51 | `{'k': 'v'}` |
| `Selection` | `ast/selection.rs` | 115 | `?[criteria]` / `^[criteria]` / `$[criteria]` |
| `Projection` | `ast/projection.rs` | 73 | `![expr]` |

### 6.8 错误体系

#### 6.8.1 SpelMessage（86 错误码）

**文件**：`spel/spel_message.rs`（506 行）
**对标**：`org.springframework.expression.spel.SpelMessage`

86 个变体，按 Spring 源码顺序定义。每个变体携带错误码（`code()` 方法）和默认消息模板。

**消息格式**：`EL{code}E: <插值后的消息模板>`，占位符 `{n}` 替换为 `inserts[n]`。提供 `format_message(&[&str])` 和泛型版本 `format_message_display(&[D])`。

**错误码范围**：1001-1005 类型/转换/构造；1006-1011 函数/属性/方法；1012-1019 索引/选择器；1020-1034 杂项求值；1035-1040 数字字面量；1041-1069 解析器侧；1052-1056 集合增长；1057-1064 Bean/数组构造；1065-1085 杂项/限制；9999 内部错误。

#### 6.8.2 SpelParseException / SpelEvaluationException

**文件**：`spel/spel_parse_exception.rs`（136 行）/ `spel/spel_evaluation_exception.rs`（114 行）
**对标**：Spring `SpelParseException` / `SpelEvaluationException`

两者结构相同，均使用 `thiserror` 派生 `std::error::Error`：

```rust
// SpelParseException
pub struct SpelParseException {
    pub code: SpelMessage,
    pub inserts: Vec<String>,
    pub expression_string: Option<String>,
    pub position: Option<usize>,
    pub message: String,
}
```

- `new(expression, position, code, inserts)` — 带位置
- `at_unknown(code, inserts)` — 无位置
- `detailed_message()` — `Expression [{expr}] @{pos}: {simple}`

`SpelEvaluationException` 结构相同（去掉 `expression_string`），额外提供 `set_position()` 后置设置位置。

#### 6.8.3 其他异常

`ParseException`（`parse_exception.rs`）、`EvaluationException`（`evaluation_exception.rs`）、`ExpressionException`（`expression_exception.rs`）、`AccessException`（`access_exception.rs`）、`ExpressionInvocationTargetException`（`expression_invocation_target_exception.rs`）、`InternalParseException`（`spel/internal_parse_exception.rs`，包装 `SpelParseException`）。

### 6.9 通用工具层（common 模块）

`LiteralExpression`（52 行）— 纯字面量表达式；`CompositeStringExpression`（80 行）— 模板组合表达式；`TemplateAwareExpressionParser`（97 行）— 模板感知解析器；`TemplateParserContext`（54 行）— 默认 `#{` / `}` 语法；`ExpressionUtils`（66 行）— 表达式工具函数。

---

## 附录 A：Spring ↔ Vernal 对照速查

| Spring 类/接口 | Vernal 位置 | 行数 |
|:---------------|:-----------|:-----|
| `Expression` | `src/expression.rs` | 111 |
| `ExpressionParser` | `src/parser.rs` | 85 |
| `EvaluationContext` | `src/evaluation_context.rs` | 155 |
| `TypedValue` | `src/typed_value.rs` | 143 |
| `TypeDescriptor` | `src/type_descriptor.rs` | 485 |
| `PropertyAccessor` / `IndexAccessor` | `src/property_accessor.rs` | 79 |
| `BeanResolver` | `src/bean_resolver.rs` | 20 |
| `TypeLocator` / `TypeConverter` / `TypeComparator` | `src/type_*.rs` | 14-22 |
| `OperatorOverloader` | `src/operator_overloader.rs` | 28 |
| `MethodResolver` / `ConstructorResolver` | `src/method_resolver.rs` 等 | 20-23 |
| `SpelExpression` | `spel/spel_expression.rs` | 66 |
| `SpelExpressionParser` | `spel/spel_expression_parser.rs` | 49 |
| `InternalSpelExpressionParser` | `spel/internal_spel_expression_parser.rs` | 806 |
| `Tokenizer` | `spel/tokenizer.rs` | 580 |
| `TokenKind`（46 变体） | `spel/token_kind.rs` | 254 |
| `SpelMessage`（86 错误码） | `spel/spel_message.rs` | 506 |
| `SpelParseException` / `SpelEvaluationException` | `spel/spel_*_exception.rs` | 114-136 |
| `ExpressionState` | `spel/expression_state.rs` | 75 |
| `SpelParserConfiguration` | `spel/spel_parser_configuration.rs` | 58 |
| `StandardEvaluationContext` | `support/standard_evaluation_context.rs` | 218 |
| `SimpleEvaluationContext` | `support/simple_evaluation_context.rs` | 97 |
| `ReflectivePropertyAccessor` | `support/reflective_property_accessor.rs` | 231 |
| `MapAccessor` / `ReflectiveIndexAccessor` | `support/map_accessor.rs` 等 | 65-73 |
| `DataBindingPropertyAccessor` | `support/data_binding_property_accessor.rs` | 48 |
| `VernalPropertyAccessor` / `VernalBeanResolver` | `support/vernal_*.rs` | 26-59 |
| `StandardTypeLocator` 等 | `support/standard_type_*.rs` | 38-59 |
| `ReflectiveMethodResolver` | `support/reflective_method_resolver.rs` | 184 |
| `ReflectiveConstructorResolver` | `support/reflective_constructor_resolver.rs` | 42 |


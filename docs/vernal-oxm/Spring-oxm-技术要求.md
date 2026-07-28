# vernal-oxm 技术要求（对标 spring-oxm）

> **版本**：v1.0（2026-07-28）
> **对标**：`spring-oxm` 6.1 / Spring Framework 6.1
> **Rust 基线**：edition 2024 / rustc 1.88
> **crate 现状**：待建（规划中）
> **选型**：quick-xml 0.41 + serde
> **引用约定**：《Spring 组件替换约定》第 8.1 节（序列化）

---

## 一、概述与定位

### 1.1 crate 职责

`vernal-oxm` 是 Vernal Framework 的 **Object-XML Mapping 抽象层**，对标
Spring Framework 中的 `spring-oxm` 模块。它在 `quick-xml` + `serde` 之上，
提供 Spring OXM 语义的编程模型，包括 `Marshaller`、`Unmarshaller` 和
`Jaxb2Marshaller` 三大核心抽象。

在 Vernal 序列化架构中，`vernal-oxm` 与 JSON 序列化互补：

```
序列化层全景
  ┌─────────────────────────────────────────────────┐
  │  vernal-oxm (本 crate)                           │
  │    ├─ Marshaller         → XML 序列化            │
  │    ├─ Unmarshaller       → XML 反序列化          │
  │    └─ Jaxb2Marshaller    → 一站式编解码          │
  ├─────────────────────────────────────────────────┤
  │  serde + serde_json (JSON 序列化)                │
  │    └─ 已集成于 vernal-core                        │
  ├─────────────────────────────────────────────────┤
  │  quick-xml 0.41 (XML 底层引擎)                   │
  │    └─ vernal-core 已集成                          │
  ├─────────────────────────────────────────────────┤
  │  vernal-core (基础类型 + serde 属性宏)            │
  └─────────────────────────────────────────────────┘
```

### 1.2 与 spring-oxm 的对齐边界

| spring-oxm 概念 | vernal-oxm 对应 | 说明 |
|:---|:---|:---|
| `Marshaller` | `Marshaller` trait | XML 序列化抽象 |
| `Unmarshaller` | `Unmarshaller` trait | XML 反序列化抽象 |
| `Jaxb2Marshaller` | `XmlMapper` struct | 基于 quick-xml + serde 的实现 |
| `@XmlRootElement` | `#[serde(rename = "...")]` | 根元素命名 |
| `@XmlElement` | `#[serde(rename = "...")]` | 子元素命名 |
| `@XmlAttribute` | `#[serde(rename = "@...")]` | 属性映射（@前缀约定） |
| `@XmlValue` | `#[serde(rename = "$value")]` | 文本内容映射 |
| `@XmlAccessorType` | 不直接映射 | serde 默认按字段名映射 |
| `@XmlTransient` | `#[serde(skip)]` | 跳过字段 |
| `@XmlElementWrapper` | `#[serde(rename = "...")]` 包装层 | 集合包装元素 |

### 1.3 设计约束

1. **`#![forbid(unsafe_code)]`**：与 vernal-core 保持一致。
2. **quick-xml-first**：直接使用 quick-xml 的 XML 读写器。
3. **serde 驱动**：通过 serde 属性宏实现 JAXB 注解语义，无需自定义注解系统。
4. **IoC 桥接**：XmlMapper 通过 vernal-beans `Container` 解析，支持依赖注入。
5. **Send + Sync**：所有公共类型满足跨线程边界。
6. **命名空间支持**：完整支持 XML Namespace 声明和前缀映射。

### 1.4 JAXB 注解 → serde 属性映射表

这是 `vernal-oxm` 的核心设计决策：用 serde 属性宏替代 JAXB 注解。

| JAXB 注解 | serde 属性 | 说明 | 示例 |
|:---|:---|:---|:---|
| `@XmlRootElement(name="user")` | `#[serde(rename = "user")]` | 根元素名 | 结构体级别 |
| `@XmlElement(name="userName")` | `#[serde(rename = "userName")]` | 子元素名 | 字段级别 |
| `@XmlAttribute(name="id")` | `#[serde(rename = "@id")]` | 属性映射 | `@` 前缀约定 |
| `@XmlValue` | `#[serde(rename = "$value")]` | 文本内容 | 枚举或特殊字段 |
| `@XmlTransient` | `#[serde(skip)]` | 跳过 | 不参与序列化 |
| `@XmlElementWrapper(name="items")` | 嵌套结构体 + `#[serde(rename)]` | 包装层 | 集合包装 |
| `@XmlList` | `#[serde(with = "space_separated")]` | 空格分隔列表 | 自定义序列化 |
| `@XmlEnum` / `@XmlEnumValue` | `#[serde(rename = "...")]` | 枚举值映射 | 枚举变体 |
| `@XmlNs` | `#[serde(rename = "xmlns:prefix")]` | 命名空间声明 | 特殊字段 |
| `@XmlAttributeWrapper` | 嵌套结构体 + 属性前缀 | 属性集合 | 属性包装 |

---

## 二、待建 crate 规划

### 2.1 目录结构

```
crates/vernal-oxm/
├── Cargo.toml
└── src/
    ├── lib.rs                          # 入口，重导出公共 API
    ├── marshaller.rs                   # Marshaller trait
    ├── unmarshaller.rs                 # Unmarshaller trait
    ├── xml_mapper.rs                   # XmlMapper 实现
    ├── xml_value.rs                    # XmlValue 类型封装
    ├── xml_error.rs                    # XML 异常体系
    ├── xml_config.rs                   # XML 序列化配置
    ├── namespace.rs                    # 命名空间处理
    ├── cdata.rs                        # CDATA 区段处理
    ├── pretty_print.rs                # 格式化输出
    └── helpers/
        ├── mod.rs                      # 辅助模块
        ├── attr_serde.rs               # @属性 serde 辅助
        ├── value_serde.rs              # $value serde 辅助
        └── wrapper_serde.rs            # 包装层 serde 辅助
```

### 2.2 Cargo.toml 草案

```toml
[package]
name = "vernal-oxm"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Vernal OXM Object-XML Mapping（对标 spring-oxm）"
publish = false

[features]
default = ["pretty"]
pretty = []

[dependencies]
bytes.workspace = true
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true
vernal-core = { path = "../vernal-core" }
quick-xml = { workspace = true, features = ["serialize", "serde-types"] }
thiserror.workspace = true

[dev-dependencies]
tokio.workspace = true

[lints]
workspace = true
```

### 2.3 Feature 矩阵

| Feature | 说明 |
|:---|:---|
| `pretty` | 启用格式化 XML 输出（缩进、换行） |

---

## 三、核心 trait 设计

### 3.1 Marshaller

`Marshaller` 对标 Spring OXM 的 `Marshaller`，定义 XML 序列化抽象。

```rust
/// XML 序列化 trait，对标 spring-oxm Marshaller。
pub trait Marshaller: Send + Sync {
    /// 将对象序列化为 XML 字符串。
    fn marshal<T: serde::Serialize>(&self, value: &T) -> Result<String, XmlError>;

    /// 将对象序列化为 XML 字节流。
    fn marshal_bytes<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, XmlError>;

    /// 将对象序列化为 XML 并写入 writer。
    fn marshal_to_writer<T: serde::Serialize, W: std::io::Write>(
        &self,
        value: &T,
        writer: W,
    ) -> Result<(), XmlError>;

    /// 将对象序列化为 XML 并写入异步 writer。
    fn marshal_to_async_writer<T: serde::Serialize, W: tokio::io::AsyncWrite + Unpin>(
        &self,
        value: &T,
        writer: W,
    ) -> impl Future<Output = Result<(), XmlError>> + Send;
}
```

### 3.2 Unmarshaller

`Unmarshaller` 对标 Spring OXM 的 `Unmarshaller`，定义 XML 反序列化抽象。

```rust
/// XML 反序列化 trait，对标 spring-oxm Unmarshaller。
pub trait Unmarshaller: Send + Sync {
    /// 从 XML 字符串反序列化。
    fn unmarshal<T: serde::de::DeserializeOwned>(&self, xml: &str) -> Result<T, XmlError>;

    /// 从 XML 字节流反序列化。
    fn unmarshal_bytes<T: serde::de::DeserializeOwned>(
        &self,
        bytes: &[u8],
    ) -> Result<T, XmlError>;

    /// 从 reader 反序列化。
    fn unmarshal_from_reader<T: serde::de::DeserializeOwned, R: std::io::Read>(
        &self,
        reader: R,
    ) -> Result<T, XmlError>;

    /// 从异步 reader 反序列化。
    fn unmarshal_from_async_reader<
        T: serde::de::DeserializeOwned,
        R: tokio::io::AsyncRead + Unpin,
    >(
        &self,
        reader: R,
    ) -> impl Future<Output = Result<T, XmlError>> + Send;

    /// 判断是否支持指定类型的反序列化。
    fn supports(&self, clazz: &str) -> bool;
}
```

### 3.3 XmlMapper

`XmlMapper` 对标 Spring OXM 的 `Jaxb2Marshaller`，是 `Marshaller` 和
`Unmarshaller` 的一站式实现。

```rust
/// XML 映射器，对标 spring-oxm Jaxb2Marshaller。
/// 同时实现 Marshaller 和 Unmarshaller。
pub struct XmlMapper {
    config: XmlConfig,
}

impl XmlMapper {
    /// 创建默认配置的 XmlMapper。
    pub fn new() -> Self { ... }

    /// 使用自定义配置创建。
    pub fn with_config(config: XmlConfig) -> Self { ... }

    /// Builder 模式：设置是否格式化输出。
    pub fn pretty_print(mut self, pretty: bool) -> Self { ... }

    /// Builder 模式：设置 XML 声明版本。
    pub fn xml_version(mut self, version: XmlVersion) -> Self { ... }

    /// Builder 模式：设置编码。
    pub fn encoding(mut self, encoding: impl Into<String>) -> Self { ... }

    /// Builder 模式：设置是否省略 XML 声明。
    pub fn omit_xml_declaration(mut self, omit: bool) -> Self { ... }

    /// Builder 模式：设置根元素名称（覆盖 serde rename）。
    pub fn root_element(mut self, name: impl Into<String>) -> Self { ... }
}

impl Marshaller for XmlMapper {
    fn marshal<T: serde::Serialize>(&self, value: &T) -> Result<String, XmlError> { ... }
    fn marshal_bytes<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, XmlError> { ... }
    fn marshal_to_writer<T: serde::Serialize, W: std::io::Write>(
        &self, value: &T, writer: W,
    ) -> Result<(), XmlError> { ... }
    fn marshal_to_async_writer<T: serde::Serialize, W: tokio::io::AsyncWrite + Unpin>(
        &self, value: &T, writer: W,
    ) -> impl Future<Output = Result<(), XmlError>> + Send { ... }
}

impl Unmarshaller for XmlMapper {
    fn unmarshal<T: serde::de::DeserializeOwned>(&self, xml: &str) -> Result<T, XmlError> { ... }
    fn unmarshal_bytes<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, XmlError> { ... }
    fn unmarshal_from_reader<T: serde::de::DeserializeOwned, R: std::io::Read>(
        &self, reader: R,
    ) -> Result<T, XmlError> { ... }
    fn unmarshal_from_async_reader<T, R>(&self, reader: R) -> impl Future<Output = Result<T, XmlError>> + Send
    where T: serde::de::DeserializeOwned, R: tokio::io::AsyncRead + Unpin { ... }
    fn supports(&self, _clazz: &str) -> bool { true }
}
```

---

## 四、JAXB 注解语义详解

### 4.1 @XmlAttribute → `@` 前缀

JAXB 的 `@XmlAttribute` 将字段映射为 XML 属性。在 serde 中，quick-xml 支持
`@` 前缀约定：

```rust
// JAXB: @XmlAttribute(name = "id")
// Rust:
#[derive(Serialize, Deserialize)]
struct Element {
    #[serde(rename = "@id")]
    id: String,

    #[serde(rename = "@class")]
    class: Option<String>,

    #[serde(rename = "name")]
    name: String,
}

// 输出: <element id="123" class="primary"><name>foo</name></element>
```

### 4.2 @XmlElement → rename

JAXB 的 `@XmlElement` 将字段映射为 XML 子元素。在 serde 中直接使用 `rename`：

```rust
// JAXB: @XmlElement(name = "userName")
// Rust:
#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "userName")]
    user_name: String,

    #[serde(rename = "emailAddress")]
    email: String,
}

// 输出: <user><userName>alice</userName><emailAddress>alice@example.com</emailAddress></user>
```

### 4.3 @XmlValue → `$value`

JAXB 的 `@XmlValue` 将字段映射为元素的文本内容：

```rust
// JAXB: @XmlValue
// Rust:
#[derive(Serialize, Deserialize)]
struct TextContent {
    #[serde(rename = "@lang")]
    lang: String,

    #[serde(rename = "$value")]
    text: String,
}

// 输出: <textContent lang="en">Hello World</textContent>
```

### 4.4 @XmlElementWrapper → 包装层

JAXB 的 `@XmlElementWrapper` 为集合添加包装元素：

```rust
// JAXB: @XmlElementWrapper(name = "items")
//       @XmlElement(name = "item")
// Rust: 需要嵌套结构体
#[derive(Serialize, Deserialize)]
struct Order {
    #[serde(rename = "items")]
    items: ItemsWrapper,
}

#[derive(Serialize, Deserialize)]
struct ItemsWrapper {
    #[serde(rename = "item")]
    items: Vec<Item>,
}

// 输出: <order><items><item>...</item><item>...</item></items></order>
```

### 4.5 @XmlTransient → skip

```rust
// JAXB: @XmlTransient
// Rust:
#[derive(Serialize, Deserialize)]
struct User {
    name: String,

    #[serde(skip)]
    internal_id: u64,  // 不参与 XML 序列化
}
```

### 4.6 命名空间支持

```rust
// JAXB: @XmlNs(prefix = "soap", namespaceURI = "...")
// Rust: 使用 quick-xml 的命名空间功能
#[derive(Serialize, Deserialize)]
struct Envelope {
    #[serde(rename = "@xmlns:soap")]
    soap_ns: String,

    #[serde(rename = "soap:Body")]
    body: Body,
}
```

### 4.7 枚举映射

```rust
// JAXB: @XmlEnum + @XmlEnumValue
// Rust:
#[derive(Serialize, Deserialize)]
enum Status {
    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "INACTIVE")]
    Inactive,

    #[serde(rename = "PENDING")]
    Pending,
}
```

---

## 五、XML 配置与高级特性

### 5.1 XmlConfig

```rust
/// XML 序列化配置。
pub struct XmlConfig {
    /// XML 版本。
    pub version: XmlVersion,
    /// 编码。
    pub encoding: String,
    /// 是否省略 XML 声明。
    pub omit_xml_declaration: bool,
    /// 是否格式化输出。
    pub pretty_print: bool,
    /// 缩进字符。
    pub indent_char: char,
    /// 缩进级别。
    pub indent_size: usize,
    /// 是否输出自闭合标签。
    pub self_close_empty: bool,
    /// 根元素名称覆盖。
    pub root_element: Option<String>,
    /// 命名空间映射。
    pub namespaces: HashMap<String, String>,
}

/// XML 版本。
pub enum XmlVersion {
    V10,
    V11,
}
```

### 5.2 CDATA 区段处理

```rust
/// CDATA 区段包装器。
/// 在序列化时输出 `<![CDATA[...]]>` 包裹的内容。
#[derive(Debug, Clone)]
pub struct CData {
    content: String,
}

impl CData {
    pub fn new(content: impl Into<String>) -> Self {
        Self { content: content.into() }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Serialize for CData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // quick-xml 会自动处理 CDATA 区段
        serializer.serialize_str(&self.content)
    }
}
```

### 5.3 格式化输出

```rust
/// 格式化 XML 输出（pretty print）。
pub struct PrettyPrinter {
    indent: String,
    line_ending: &'static str,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self {
            indent: "  ".into(),
            line_ending: "\n",
        }
    }

    pub fn with_indent(indent: impl Into<String>) -> Self {
        Self {
            indent: indent.into(),
            line_ending: "\n",
        }
    }

    /// 格式化 XML 字符串。
    pub fn format(&self, xml: &str) -> Result<String, XmlError> { ... }
}
```

### 5.4 XML Schema 验证

```rust
/// XML Schema 验证器（可选扩展）。
pub struct SchemaValidator {
    schema: String,
}

impl SchemaValidator {
    /// 从 XSD 字符串创建验证器。
    pub fn from_xsd(xsd: &str) -> Result<Self, XmlError> { ... }

    /// 验证 XML 字符串是否符合 Schema。
    pub fn validate(&self, xml: &str) -> Result<(), XmlError> { ... }
}
```

---

## 六、异常体系与测试策略

### 6.1 XML 异常体系

```rust
#[derive(Debug, thiserror::Error)]
pub enum XmlError {
    /// XML 语法错误（格式不合法）。
    #[error("XML 语法错误: {message}")]
    SyntaxError { message: String, line: Option<usize>, column: Option<usize> },

    /// 序列化错误（对象无法转为 XML）。
    #[error("序列化错误: {message}")]
    SerializationError { message: String },

    /// 反序列化错误（XML 无法转为对象）。
    #[error("反序列化错误: {message}")]
    DeserializationError { message: String },

    /// 命名空间错误。
    #[error("命名空间错误: {message}")]
    NamespaceError { message: String },

    /// 编码错误（非 UTF-8 内容）。
    #[error("编码错误: {message}")]
    EncodingError { message: String },

    /// IO 错误。
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// Schema 验证失败。
    #[error("Schema 验证失败: {message}")]
    ValidationError { message: String },
}
```

### 6.2 quick-xml 错误映射

| quick-xml 错误 | XmlError | 说明 |
|:---|:---|:---|
| `quick_xml::Error::Syntax` | `SyntaxError` | XML 格式错误 |
| `quick_xml::Error::Io` | `IoError` | IO 错误 |
| `quick_xml::Error::Utf8` | `EncodingError` | UTF-8 编码错误 |
| `quick_xml::Error::InvalidAttr` | `SyntaxError` | 属性格式错误 |
| `quick_xml::de::Error` | `DeserializationError` | 反序列化错误 |
| `quick_xml::se::Error` | `SerializationError` | 序列化错误 |

### 6.3 测试策略

#### 单元测试

- `XmlMapper` 序列化/反序列化基本类型
- JAXB 注解语义映射测试（@属性、@元素、@值）
- 命名空间处理测试
- CDATA 区段测试
- 格式化输出测试
- 错误处理测试（格式错误、编码错误等）

#### 集成测试

- SOAP 消息序列化/反序列化
- 复杂嵌套结构体测试
- 大文件流式处理测试
- 与 spring-oxm 输出对比测试（兼容性验证）

#### 性能基准

- 序列化吞吐量（小文档 / 大文档）
- 反序列化吞吐量
- 内存占用对比（vs Java JAXB）
- 流式处理 vs 全量加载对比

### 6.4 实施路线图

| 阶段 | 内容 | 依赖 | 预计工作量 |
|:---|:---|:---|:---|
| P0 | Marshaller + Unmarshaller trait 定义 | quick-xml | 1 天 |
| P1 | XmlMapper 基本实现（序列化/反序列化） | quick-xml + serde | 3 天 |
| P2 | @属性 (@前缀) 和 @元素 (rename) 支持 | P1 | 1 天 |
| P3 | $value 文本内容支持 | P1 | 0.5 天 |
| P4 | 包装层 (wrapper) 支持 | P1 | 0.5 天 |
| P5 | 命名空间处理 | P1 | 1 天 |
| P6 | CDATA 区段支持 | P1 | 0.5 天 |
| P7 | 格式化输出 (pretty print) | P1 | 0.5 天 |
| P8 | XmlConfig 配置系统 | P0 | 0.5 天 |
| P9 | 异常体系 + quick-xml 错误映射 | P0 | 0.5 天 |
| P10 | 集成测试 + 性能基准 | P0-P9 | 2 天 |

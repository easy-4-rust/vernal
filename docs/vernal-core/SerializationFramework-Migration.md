# Spring 序列化与 XML 模块 → Rust serde/quick-xml 集成对照

> 版本：v1.0（2026-07-27）
> 基线：Spring Framework **7.0.8**
> - `org.springframework.core.serializer`(8 个 Java 类,序列化抽象)
> - `org.springframework.util.xml`(22 个 Java 类,XML 工具)
> Rust 替代栈:
> - **JSON**: [`serde`](https://crates.io/crates/serde) / [`serde_derive`](https://crates.io/crates/serde_derive) / [`serde_json`](https://crates.io/crates/serde_json)(Github: <https://github.com/serde-rs/serde>)
> - **XML**: [`quick-xml`](https://crates.io/crates/quick-xml) 0.41.0(Github: <https://github.com/tafia/quick-xml>)
>
> 状态:**全部 30 个 Java 类都不迁移到 vernal-core**,改为提供 Rust 生态等价物 + vernal-core 集成 trait 抽象。

---

## 一、设计原则

### 1.1 不直接迁移 Spring serializer 的理由

Spring `core/serializer` 的 8 个类全部基于 **JVM `java.io.Serializable`** 协议:

```java
// Spring Serializer.java
public interface Serializer<T> {
    byte[] serialize(T object) throws IOException;
}

// Spring DefaultSerializer.java
public class DefaultSerializer implements Serializer<Object> {
    public byte[] serialize(Object object) throws IOException {
        // 内部使用 ObjectOutputStream(JVM Serializable 协议)
        ObjectOutputStream oos = new ObjectOutputStream(...);
        oos.writeObject(object);
    }
}
```

**JVM Serializable 协议的局限**:
- 跨语言不可读(Java 私有二进制格式)
- 安全风险(`ObjectInputStream` 已知有 RCE 漏洞)
- 性能差(反射 + 类描述符开销)
- 字节码耦合(必须 `implements Serializable`)

**Rust 的方案**:`serde` trait 抽象 + 多种格式后端(JSON / XML / YAML / TOML / Bincode / Postcard / MessagePack),业务代码只写一次 `#[derive(Serialize, Deserialize)]`,运行时选择格式。

### 1.2 不直接迁移 Spring xml 的理由

Spring `util/xml` 的 22 个类全部是 **JDK XML API(JAXP / StAX / DOM / SAX)** 的包装:

| Spring 类 | 包装的 JDK API |
|---|---|
| `StaxUtils` / `StaxResult` / `StaxSource` | `javax.xml.stream`(StAX) |
| `DomUtils` | `org.w3c.dom`(DOM) |
| `SimpleSaxErrorHandler` | `org.xml.sax`(SAX) |
| `AbstractXMLReader` / `AbstractXMLEventReader` | JAXP Reader 抽象 |

**JDK XML API 在 Rust 中没有等价物**,且其设计(基于 abstract class + 巨型接口)不符合 Rust 习惯。Rust 的 `quick-xml` 提供**零拷贝事件模型**,性能与 API 现代化程度都远超 JAXP。

---

## 二、Spring serializer → Rust serde 集成映射

### 2.1 Spring serializer 接口 → Rust serde trait

| Spring 接口/类 | Rust 替代 | 说明 |
|---|---|---|
| `Serializer<T>` interface | `serde::Serialize` trait | Rust 用 derive 宏自动实现 |
| `Deserializer<T>` interface | `serde::Deserialize<'de>` trait | Rust 用 derive 宏自动实现 |
| `DefaultSerializer`(JVM 二进制) | `bincode::serialize()` 或 `postcard::to_vec()` | Rust 二进制格式 |
| `DefaultDeserializer`(JVM 二进制) | `bincode::deserialize()` 或 `postcard::from_bytes()` | 反序列化 |
| `SerializingConverter` | `serde_json::to_vec()` / `bincode::serialize()` | Converter 风格 |
| `DeserializingConverter` | `serde_json::from_slice()` / `bincode::deserialize()` | Converter 风格 |
| `SerializationDelegate` | 业务自定义 `SerializationDelegate` struct | 同时持 Serializer + Deserializer |
| `SerializationFailedException` | `bincode::Error` / `serde_json::Error` / `VernalError::Infrastructure` | 错误传播 |

### 2.2 完整 API 对照(DefaultSerializer 为例)

**Spring `DefaultSerializer`**:

```java
public class DefaultSerializer implements Serializer<Object> {
    public byte[] serialize(Object object) throws IOException {
        ByteArrayOutputStream baos = new ByteArrayOutputStream(128);
        try (ObjectOutputStream oos = ObjectOutputStream != null ?
             new ObjectOutputStream(baos) : new ConfigurableObjectInputStream(baos, ...)) {
            oos.writeObject(object);
        }
        return baos.toByteArray();
    }
}
```

**Rust 等价(bincode)**:

```rust
use serde::Serialize;

// 业务类型
#[derive(Serialize)]
struct MyData {
    name: String,
    value: i64,
}

let data = MyData { name: "hello".into(), value: 42 };
let bytes: Vec<u8> = bincode::serialize(&data).unwrap();
```

**Rust 等价(serde_json)**:

```rust
use serde::Serialize;

let data = MyData { name: "hello".into(), value: 42 };
let json: String = serde_json::to_string(&data).unwrap();
// 或:let bytes: Vec<u8> = serde_json::to_vec(&data).unwrap();
```

### 2.3 vernal-core 提供的集成 trait(可选,feature = "serde-bridge")

为了让 Spring 用户迁移更平滑,vernal-core 可在 feature = `"serde-bridge"` 下提供与 Spring 接口形态一致的 Rust trait:

```rust
// vernal-core/src/serde_bridge/mod.rs(feature = "serde-bridge")
use vernal_core::BoxError;

/// 对标 Spring `org.springframework.core.serializer.Serializer<T>`。
pub trait JavaStyleSerializer<T> {
    fn serialize(&self, object: &T) -> Result<Vec<u8>, BoxError>;
}

/// 对标 Spring `org.springframework.core.serializer.Deserializer<T>`。
pub trait JavaStyleDeserializer<T> {
    fn deserialize(&self, bytes: &[u8]) -> Result<T, BoxError>;
}

// bincode 实现
pub struct BincodeSerializer;
impl<T: serde::Serialize> JavaStyleSerializer<T> for BincodeSerializer {
    fn serialize(&self, object: &T) -> Result<Vec<u8>, BoxError> {
        bincode::serialize(object).map_err(|e| Box::new(e) as BoxError)
    }
}

// serde_json 实现
pub struct JsonSerializer;
impl<T: serde::Serialize> JavaStyleSerializer<T> for JsonSerializer {
    fn serialize(&self, object: &T) -> Result<Vec<u8>, BoxError> {
        serde_json::to_vec(object).map_err(|e| Box::new(e) as BoxError)
    }
}
```

> **决策**:此 trait **暂不实现**,因为 serde 已经是 Rust 事实标准,引入 Java 风格 trait 反而增加学习成本。文档化映射即可。

---

## 三、Spring xml → Rust quick-xml 集成映射

### 3.1 Spring xml 类 → Rust quick-xml 替代

| Spring 类 | Rust 替代 | 说明 |
|---|---|---|
| `DomUtils` | `quick_xml::Reader` + 事件循环 | DOM 风格读取(quick-xml 是事件流) |
| `StaxUtils` / `StaxResult` / `StaxSource` / `StaxEventHandler` / `StaxStreamHandler` / `StaxEventXMLReader` / `StaxStreamXMLReader` | `quick_xml::Writer` + `quick_xml::events::Event` | StAX 流式读写 |
| `SimpleSaxErrorHandler` | `quick_xml::Error` 直接处理 | SAX 错误处理 |
| `SimpleTransformErrorListener` | (不迁移,XSLT 转换特有) | 用 `xsltproc` 外部命令或不用 |
| `AbstractStaxHandler` / `AbstractStaxXMLReader` / `AbstractXMLReader` / `AbstractXMLEventReader` / `AbstractXMLStreamReader` | (不迁移,Rust 不用 abstract class 模式) | 直接用 quick-xml API |
| `DomContentHandler` | `quick_xml::events::BytesStart` / `BytesEnd` / `BytesText` | 事件模型替代 |
| `ListBasedXMLEventReader` | `Vec<quick_xml::events::Event>` 缓冲 | 自实现 |
| `SimpleNamespaceContext` | `quick_xml::NsReader` + `namespaces()` 方法 | 命名空间解析 |
| `XMLEventStreamReader` / `XMLEventStreamWriter` | `quick_xml::Reader` / `Writer` | 直接用 |
| `XmlValidationModeDetector` | 自实现(DTD vs XSD 检测) | 配合 `quick_xml::Reader::config_mut()` |
| `TransformerUtils` | (不迁移,XSLT 特有) | — |

### 3.2 quick-xml 0.41.0 核心 API

#### 3.2.1 Reader(对标 StAX XMLEventReader)

```rust
use quick_xml::Reader;
use quick_xml::events::Event;

let xml = "<message><greeting>Hello</greeting></message>";
let mut reader = Reader::from_str(xml);
reader.config_mut().trim_text(true);

let mut count = 0;
let mut buf = Vec::new();
loop {
    match reader.read_event_into(&mut buf) {
        Ok(Event::Start(_)) => count += 1,
        Ok(Event::Eof) => break,
        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        _ => (),
    }
    buf.clear();
}
```

#### 3.2.2 Writer(对标 StAX XMLEventWriter)

```rust
use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use std::io::Cursor;

let mut writer = Writer::new(Cursor::new(Vec::new()));
writer.write_event(Event::Decl(...))?;
writer.create_element("greeting").write_text_content(BytesText::new("Hello"))?;
let result = writer.into_inner().into_inner();
```

#### 3.2.3 NsReader(命名空间,对标 SimpleNamespaceContext)

```rust
use quick_xml::NsReader;

let xml = "<ns:greeting xmlns:ns=\"http://example.com\">Hi</ns:greeting>";
let mut reader = NsReader::from_str(xml);
reader.config_mut().trim_text(true);

loop {
    match reader.read_resolved_event() {
        Ok((namespace, Event::Start(e))) => {
            println!("namespace: {:?}", namespace);  // Some("http://example.com")
        }
        Ok((_, Event::Eof)) => break,
        _ => (),
    }
}
```

#### 3.2.4 serde 集成(feature = "serialize")

```rust
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Greeting {
    #[serde(rename = "@lang")]
    lang: String,
    #[serde(rename = "$text")]
    content: String,
}

let xml = "<greeting lang=\"en\">Hello</greeting>";
let greeting: Greeting = from_str(xml).unwrap();
```

### 3.3 Spring DomUtils → Rust quick-xml 完整对照

**Spring `DomUtils.findChildElementByName(Element, String)`**:

```java
// Spring 风格(DOM)
Element child = DomUtils.findChildElementByName(parentElem, "greeting");
String text = child.getTextContent();
```

**Rust quick-xml 等价(事件流)**:

```rust
use quick_xml::Reader;
use quick_xml::events::Event;

fn find_child_text(xml: &str, parent: &str, child: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut in_parent = false;
    let mut in_child = false;
    let mut result = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == parent { in_parent = true; }
                else if in_parent && name == child { in_child = true; }
            }
            Ok(Event::Text(e)) => {
                if in_child {
                    result = Some(String::from_utf8_lossy(e.as_ref()).to_string());
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == child { in_child = false; }
                else if name == parent { in_parent = false; }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    result
}
```

**更简洁:用 serde 集成**:

```rust
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize)]
struct Root {
    greeting: String,
}

let xml = "<root><greeting>Hello</greeting></root>";
let root: Root = from_str(xml).unwrap();
println!("{}", root.greeting);  // Hello
```

---

## 四、vernal-core 与 serde/quick-xml 的集成策略

### 4.1 不引入到 vernal-core 默认依赖

vernal-core 保持默认零依赖,serde/quick-xml 通过 feature flag 启用:

```toml
# vernal-core/Cargo.toml(S8 已配置,补充 quick-xml)
[dependencies]
serde = { version = "1", optional = true, default-features = false, features = ["std", "derive"] }
serde_json = { version = "1", optional = true, default-features = false, features = ["std"] }
quick-xml = { version = "0.41", optional = true, default-features = false, features = ["serialize"] }

[features]
# 已有(从 S8 继承)
serde = ["dep:serde"]

# 新增(v3.0 S21 阶段规划)
serde-json = ["serde", "dep:serde_json"]
xml = ["dep:quick-xml"]
```

### 4.2 vernal-core 类型对 serde 的实现

| vernal-core 类型 | serde 实现策略 | feature |
|---|---|---|
| `ObjectId` | `#[derive(Serialize, Deserialize)]`(S7 已实现) | `serde` |
| `SnowflakeId` | `#[derive(Serialize, Deserialize)]`(数值) | `serde` |
| `AppLifecyclePhase` | `#[derive(Serialize, Deserialize)]` | `serde` |
| `StopWatchUnit` | `#[derive(Serialize, Deserialize)]` | `serde` |
| `ErrorContext` | 自定义 `serialize`(只输出 entries 数量,不泄露内容) | `serde` |
| `ErrorReport` | `#[derive(Serialize, Deserialize)]` | `serde-json` |
| `VernalError` | 自定义 `serialize`(脱敏:Infrastructure 只输出 message) | `serde` |

### 4.3 Spring serializer/xml 用户迁移指南

#### 场景 1:Spring `DefaultSerializer` → Rust bincode

```java
// Spring
DefaultSerializer s = new DefaultSerializer();
byte[] bytes = s.serialize(myObject);
MyObject restored = new DeserializingConverter().convert(bytes);
```

```rust
// Rust(用 bincode,需在业务 Cargo.toml 添加 bincode 依赖)
#[derive(serde::Serialize, serde::Deserialize)]
struct MyObject { /* ... */ }

let bytes = bincode::serialize(&my_object).unwrap();
let restored: MyObject = bincode::deserialize(&bytes).unwrap();
```

#### 场景 2:Spring JSON(用 Jackson) → Rust serde_json

```java
// Spring + Jackson
ObjectMapper mapper = new ObjectMapper();
String json = mapper.writeValueAsString(myObject);
MyObject restored = mapper.readValue(json, MyObject.class);
```

```rust
// Rust(用 serde_json)
#[derive(serde::Serialize, serde::Deserialize)]
struct MyObject { /* ... */ }

let json = serde_json::to_string(&my_object).unwrap();
let restored: MyObject = serde_json::from_str(&json).unwrap();
```

#### 场景 3:Spring XML → Rust quick-xml

```java
// Spring + JAXB
JAXBContext ctx = JAXBContext.newInstance(MyObject.class);
Marshaller m = ctx.createMarshaller();
m.marshal(myObject, new File("data.xml"));

Unmarshaller u = ctx.createUnmarshaller();
MyObject restored = (MyObject) u.unmarshal(new File("data.xml"));
```

```rust
// Rust(用 quick-xml + serde)
use quick_xml::se::to_string;
use quick_xml::de::from_str;

#[derive(serde::Serialize, serde::Deserialize)]
struct MyObject { /* ... */ }

let xml = to_string(&my_object).unwrap();
let restored: MyObject = from_str(&xml).unwrap();
```

---

## 五、汇总

### 5.1 Spring 序列化模块处置一览

| Spring 模块 | Java 类数 | 处置 | Rust 替代 |
|---|---:|---|---|
| `core/serializer`(根 4) | 4 | 🚫 不迁移到 vernal-core | serde trait 抽象 |
| `core/serializer/support`(4) | 4 | 🚫 不迁移到 vernal-core | bincode / postcard 等具体格式 |
| `util/xml`(22) | 22 | 🚫 不迁移到 vernal-core | `quick-xml` crate |
| **合计** | **30** | **全部 🚫 不迁移** | serde + quick-xml |

### 5.2 vernal-core 在序列化层的角色

vernal-core **不重新实现序列化框架**,而是:
1. 提供与 serde 兼容的类型(`#[derive(Serialize, Deserialize)]` for ObjectId / SnowflakeId / AppLifecyclePhase 等)
2. 通过 feature flag 接入 serde / serde_json / quick-xml
3. 文档化 Spring serializer/xml 到 Rust serde/quick-xml 的迁移路径

### 5.3 推荐 Rust crate 版本(2026-07-27 实测)

| Crate | 版本 | 用途 | 许可证 | MSRV |
|---|---|---|---|---|
| [`serde`](https://crates.io/crates/serde) | 1.0.229 | 序列化 trait | MIT/Apache-2.0 | 1.56 |
| [`serde_derive`](https://crates.io/crates/serde_derive) | 1.0.229 | derive 宏(随 serde) | MIT/Apache-2.0 | 1.56 |
| [`serde_json`](https://crates.io/crates/serde_json) | 1.0.145+ | JSON 格式 | MIT/Apache-2.0 | 1.61 |
| [`quick-xml`](https://crates.io/crates/quick-xml) | 0.41.0 | XML 格式(零拷贝事件流 + serde 集成) | MIT | 1.56 |
| `bincode` | 2.0.x | Rust 二进制格式(替代 JVM Serializable) | MIT | 1.85 |
| `postcard` | 1.x | 紧凑二进制(嵌入式) | MIT/Apache-2.0 | 1.65 |

### 5.4 参考链接

- serde 官方:<https://crates.io/crates/serde>
- serde 使用指南:<https://docs.rs/serde/1.0.229/serde/>
- serde Github:<https://github.com/serde-rs/serde>
- serde_json 官方:<https://crates.io/crates/serde_json>
- serde_json 使用指南:<https://docs.rs/serde_json/>
- quick-xml 官方:<https://crates.io/crates/quick-xml>
- quick-xml 使用指南(0.41.0):<https://docs.rs/quick-xml/0.41.0/quick_xml/>
- quick-xml Github:<https://github.com/tafia/quick-xml>

---

## 六、与本系列其他文档的关系

| 文档 | 范围 | 关系 |
|---|---|---|
| `SkippedModules.md` | asm/cglib/aot/objenesis/lang/javapoet(245 类) | "JVM 工具特有,无 Rust 对应" |
| **本文档**(SerializationFramework-Migration.md) | serializer + xml(30 类) | "Rust 生态等价物:serde + quick-xml" |
| `vernal-core对象级对照表-v3.md` | 433 类完整清单 | 总索引,引用本文档作为序列化模块的详细说明 |
| `vernal-core迁移路线图-v3.md` | S11-S20 阶段计划 | 不包含序列化迁移(已划归"不迁移") |
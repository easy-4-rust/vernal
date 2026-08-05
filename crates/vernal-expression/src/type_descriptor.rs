//! 富类型描述符（对标 Spring `org.springframework.core.convert.TypeDescriptor`）。
//!
//! Spring 使用 `TypeDescriptor` 携带：基础类型 + 泛型参数 + 注解 + 集合元素类型 + Map K/V 类型。
//! Rust 中我们采用 enum 把这些信息集中表达，同时兼容 `TypeId` 做运行时反射基础。

use std::any::TypeId;
use std::fmt;

/// 基础类型枚举（对标 Spring 14 种 PrimitiveKind）。
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    /// 空类型。
    Null,
    /// 布尔。
    Boolean,
    /// 字节（对标 `byte`）。
    Byte,
    /// 短整型（对标 `short`）。
    Short,
    /// 整型（对标 `int`）。
    Int,
    /// 长整型（对标 `long`）。
    Long,
    /// 单精度浮点（对标 `float`）。
    Float,
    /// 双精度浮点（对标 `double`）。
    Double,
    /// 任意精度整数（对标 `BigInteger`）。
    BigInt,
    /// 任意精度小数（对标 `BigDecimal`）。
    BigDecimal,
    /// 字符（对标 `char`）。
    Char,
    /// 字符串（对标 `String`）。
    String,
    /// 日期时间（对标 `java.util.Date` / `Instant`）。
    DateTime,
    /// 时间间隔（对标 `java.time.Duration`）。
    Duration,
}

impl PrimitiveKind {
    /// 获取类型名称（与 Spring 一致，使用小写关键字形式）。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Byte => "byte",
            Self::Short => "short",
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::Double => "double",
            Self::BigInt => "java.math.BigInteger",
            Self::BigDecimal => "java.math.BigDecimal",
            Self::Char => "char",
            Self::String => "java.lang.String",
            Self::DateTime => "java.util.Date",
            Self::Duration => "java.time.Duration",
        }
    }

    /// Spring 大数字 widening 顺序中的"宽度"（越大越宽）。
    /// `BigDecimal(8) > Double(7) > Float(6) > BigInteger(5) > Long(4) > Integer(3) > Short(2) > Byte(1)`。
    #[must_use]
    pub const fn numeric_width(&self) -> u8 {
        match self {
            Self::Byte => 1,
            Self::Short => 2,
            Self::Int => 3,
            Self::Long => 4,
            Self::BigInt => 5,
            Self::Float => 6,
            Self::Double => 7,
            Self::BigDecimal => 8,
            _ => 0,
        }
    }

    /// 是否为整数类型（含 BigInt）。
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Byte | Self::Short | Self::Int | Self::Long | Self::BigInt
        )
    }

    /// 是否为浮点类型。
    #[must_use]
    pub const fn is_floating(&self) -> bool {
        matches!(self, Self::Float | Self::Double | Self::BigDecimal)
    }

    /// 是否为任意精度（`BigInt`/`BigDecimal`）。
    #[must_use]
    pub const fn is_big(&self) -> bool {
        matches!(self, Self::BigInt | Self::BigDecimal)
    }

    /// 类型放宽：两端中"较宽"的那个。
    /// 对应 Spring `org.springframework.core.convert.NumberUtils.getNumberTargetType`。
    #[must_use]
    pub const fn widen(self, other: Self) -> Self {
        if self.numeric_width() >= other.numeric_width() {
            self
        } else {
            other
        }
    }
}

impl fmt::Display for PrimitiveKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// 富类型描述符（对标 Spring `TypeDescriptor`）。
///
/// 携带：基础类型、数组维数、Map K/V 类型、泛型参数、注解、可选 `TypeId`。
#[derive(Debug, Clone)]
pub enum TypeDescriptor {
    /// 基础类型（包含 14 种 `PrimitiveKind`）。
    Primitive(PrimitiveKind),
    /// 数组：递归元素类型。
    Array(Box<TypeDescriptor>),
    /// Map：键类型 + 值类型。
    Map(Box<TypeDescriptor>, Box<TypeDescriptor>),
    /// 具名类型（用户类或 List<T>/Set<T> 等带泛型的容器）。
    Named {
        /// 运行时类型 ID（可为 None 表示未知）。
        type_id: Option<TypeId>,
        /// 简单类名（如 `String`、`List`）。
        name: String,
        /// 泛型实参。
        generics: Vec<TypeDescriptor>,
        /// 注解（占位，未来可用于标记 `@Nullable` 等）。
        annotations: Vec<String>,
    },
}

impl TypeDescriptor {
    // ──── Spring 风格常量 ─────────────────────────────────────────────
    /// 通用对象类型（用于兜底）。
    pub const OBJECT: Self = Self::Named {
        type_id: None,
        name: String::new(), // TODO: 需要 "object" 但 const 无法使用 String::from
        generics: Vec::new(),
        annotations: Vec::new(),
    };

    /// `int` 类型。
    pub const INT: Self = Self::Primitive(PrimitiveKind::Int);
    /// `long` 类型。
    pub const LONG: Self = Self::Primitive(PrimitiveKind::Long);
    /// `float` 类型。
    pub const FLOAT: Self = Self::Primitive(PrimitiveKind::Float);
    /// `double` 类型。
    pub const DOUBLE: Self = Self::Primitive(PrimitiveKind::Double);
    /// `boolean` 类型。
    pub const BOOLEAN: Self = Self::Primitive(PrimitiveKind::Boolean);
    /// `String` 类型。
    pub const STRING: Self = Self::Primitive(PrimitiveKind::String);
    /// `null` 类型。
    pub const NULL: Self = Self::Primitive(PrimitiveKind::Null);
    /// `value` 类型（对标 SpEL 的 `ValueRef` 静态类型）。
    pub const VALUE: Self = Self::Named {
        type_id: None,
        name: String::new(),
        generics: Vec::new(),
        annotations: Vec::new(),
    };

    /// 通过类型名构造具名描述符。
    #[must_use]
    pub fn from_type_name(name: impl Into<String>) -> Self {
        Self::Named {
            type_id: None,
            name: name.into(),
            generics: Vec::new(),
            annotations: Vec::new(),
        }
    }

    /// 通过运行时 `TypeId` 构造具名描述符。
    #[must_use]
    pub fn from_type_id<T: 'static + ?Sized>() -> Self {
        Self::Named {
            type_id: Some(TypeId::of::<T>()),
            name: std::any::type_name::<T>().to_string(),
            generics: Vec::new(),
            annotations: Vec::new(),
        }
    }

    /// 添加泛型参数。
    #[must_use]
    pub fn with_generic(mut self, g: TypeDescriptor) -> Self {
        if let Self::Named {
            ref mut generics, ..
        } = self
        {
            generics.push(g);
        }
        self
    }

    /// 添加注解（占位 API，对标 Spring 注解查找）。
    #[must_use]
    pub fn with_annotation(mut self, name: impl Into<String>) -> Self {
        if let Self::Named {
            ref mut annotations,
            ..
        } = self
        {
            annotations.push(name.into());
        }
        self
    }

    /// 获取类型名（Spring `getName()`）。
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Primitive(p) => p.name().to_string(),
            Self::Array(inner) => format!("{}[]", inner.name()),
            Self::Map(k, v) => format!("java.util.Map<{}, {}>", k.name(), v.name()),
            Self::Named { name, generics, .. } => {
                if generics.is_empty() {
                    name.clone()
                } else {
                    let parts: Vec<String> = generics.iter().map(Self::name).collect();
                    format!("{}<{}>", name, parts.join(","))
                }
            }
        }
    }

    /// 是否为基本类型（含 `int`/`long`/`boolean` 等）。
    #[must_use]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    /// 是否为可空基本类型（Spring `isPrimitiveOptional` 概念）。当前 Rust 侧映射为 `null` 类型。
    #[must_use]
    pub fn is_nullable_primitive(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveKind::Null))
    }

    /// 便捷工厂：按字符串名字构造（兼容旧 API 的 `TypeDescriptor::new(name)`）。
    ///
    /// 当名字匹配 Spring 已知基本类型（int/long/float/double/boolean/string/object/value）
    /// 时返回对应常量；否则退化为具名类型。
    #[must_use]
    pub fn new(name: &str) -> Self {
        match name {
            "int" | "Integer" => Self::INT,
            "long" | "Long" => Self::LONG,
            "float" | "Float" => Self::FLOAT,
            "double" | "Double" => Self::DOUBLE,
            "boolean" | "Boolean" => Self::BOOLEAN,
            "String" => Self::STRING,
            "object" => Self::OBJECT,
            "value" => Self::VALUE,
            "null" => Self::NULL,
            _ => Self::from_type_name(name),
        }
    }

    /// Spring `TypeDescriptor.isAssignableTo` 等价语义。
    #[must_use]
    pub fn is_assignable_from(&self, src: &TypeDescriptor) -> bool {
        // Object 兜底：任何类型可赋给 object
        // 对标 Spring: Object.class.isAssignableFrom(srcType) 总是 true
        // 注：const 中无法使用 String::from，所以 OBJECT.name 为空字符串
        if matches!(self, Self::Named { .. }) {
            let name = match self {
                Self::Named { name, .. } => name.as_str(),
                _ => "",
            };
            if name.is_empty() || name == "object" {
                return true;
            }
        }

        match (self, src) {
            // 同一类型
            (a, b) if a == b => true,
            // 任何类型可赋给 String（toString 转换）
            // 必须在 Primitive-Primitive 之前，否则 String 作为 Primitive 会先走数字 widening
            (Self::Primitive(PrimitiveKind::String), _) => true,
            // 数字 widening
            (Self::Primitive(dp), Self::Primitive(sp)) => {
                dp.numeric_width() >= sp.numeric_width()
                    || (*dp == PrimitiveKind::Double && sp.is_floating())
                    || (*dp == PrimitiveKind::Float && *sp == PrimitiveKind::Float)
            }
            // Map 兼容
            (Self::Map(ak, av), Self::Map(sk, sv)) => {
                ak.is_assignable_from(sk) && av.is_assignable_from(sv)
            }
            // Array 兼容
            (Self::Array(a), Self::Array(b)) => a.is_assignable_from(b),
            // 具名类型泛型匹配
            (Self::Named { name: na, .. }, Self::Named { name: nb, .. }) => na == nb,
            _ => false,
        }
    }

    /// Spring `TypeDescriptor.narrow(Object)` 语义——基于实际值缩窄类型。
    #[must_use]
    pub fn narrow(&self, value_kind: &str) -> TypeDescriptor {
        // 简化：基于值类型名缩窄
        match value_kind {
            "int" => Self::INT,
            "long" => Self::LONG,
            "boolean" => Self::BOOLEAN,
            "double" => Self::DOUBLE,
            "string" => Self::STRING,
            _ => self.clone(),
        }
    }

    /// 从 Map 描述符取键类型。
    #[must_use]
    pub fn get_map_key_type(&self) -> Option<&TypeDescriptor> {
        if let Self::Map(k, _) = self {
            Some(k)
        } else {
            None
        }
    }

    /// 从 Map 描述符取值类型。
    #[must_use]
    pub fn get_map_value_type(&self) -> Option<&TypeDescriptor> {
        if let Self::Map(_, v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// 从 Array 描述符取元素类型。
    #[must_use]
    pub fn get_element_type(&self) -> Option<&TypeDescriptor> {
        if let Self::Array(e) = self {
            Some(e)
        } else {
            None
        }
    }

    /// 是否为 Map 类型。
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_, _))
    }

    /// 是否为 Array 类型。
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// 获取 `TypeId`（如果存在）。
    #[must_use]
    pub fn type_id(&self) -> Option<TypeId> {
        if let Self::Named { type_id, .. } = self {
            *type_id
        } else {
            None
        }
    }

    /// 获取 primitive kind（仅当为 `Primitive` 时返回）。
    #[must_use]
    pub fn primitive_kind(&self) -> Option<PrimitiveKind> {
        if let Self::Primitive(p) = self {
            Some(*p)
        } else {
            None
        }
    }
}

impl PartialEq for TypeDescriptor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Primitive(a), Self::Primitive(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Map(ak, av), Self::Map(bk, bv)) => ak == bk && av == bv,
            (
                Self::Named {
                    type_id: t1,
                    name: n1,
                    generics: g1,
                    annotations: a1,
                },
                Self::Named {
                    type_id: t2,
                    name: n2,
                    generics: g2,
                    annotations: a2,
                },
            ) => t1 == t2 && n1 == n2 && g1 == g2 && a1 == a2,
            _ => false,
        }
    }
}

impl Eq for TypeDescriptor {}

impl std::hash::Hash for TypeDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Primitive(p) => {
                std::mem::discriminant(self).hash(state);
                p.hash(state);
            }
            Self::Array(a) => {
                std::mem::discriminant(self).hash(state);
                a.hash(state);
            }
            Self::Map(k, v) => {
                std::mem::discriminant(self).hash(state);
                k.hash(state);
                v.hash(state);
            }
            Self::Named {
                type_id,
                name,
                generics,
                annotations,
            } => {
                std::mem::discriminant(self).hash(state);
                type_id.hash(state);
                name.hash(state);
                generics.hash(state);
                annotations.hash(state);
            }
        }
    }
}

impl fmt::Display for TypeDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════════
    // PrimitiveKind
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn primitive_kind_name_all_variants() {
        assert_eq!(PrimitiveKind::Null.name(), "null");
        assert_eq!(PrimitiveKind::Boolean.name(), "boolean");
        assert_eq!(PrimitiveKind::Byte.name(), "byte");
        assert_eq!(PrimitiveKind::Short.name(), "short");
        assert_eq!(PrimitiveKind::Int.name(), "int");
        assert_eq!(PrimitiveKind::Long.name(), "long");
        assert_eq!(PrimitiveKind::Float.name(), "float");
        assert_eq!(PrimitiveKind::Double.name(), "double");
        assert_eq!(PrimitiveKind::BigInt.name(), "java.math.BigInteger");
        assert_eq!(PrimitiveKind::BigDecimal.name(), "java.math.BigDecimal");
        assert_eq!(PrimitiveKind::Char.name(), "char");
        assert_eq!(PrimitiveKind::String.name(), "java.lang.String");
        assert_eq!(PrimitiveKind::DateTime.name(), "java.util.Date");
        assert_eq!(PrimitiveKind::Duration.name(), "java.time.Duration");
    }

    #[test]
    fn primitive_kind_numeric_width() {
        assert_eq!(PrimitiveKind::Byte.numeric_width(), 1);
        assert_eq!(PrimitiveKind::Short.numeric_width(), 2);
        assert_eq!(PrimitiveKind::Int.numeric_width(), 3);
        assert_eq!(PrimitiveKind::Long.numeric_width(), 4);
        assert_eq!(PrimitiveKind::BigInt.numeric_width(), 5);
        assert_eq!(PrimitiveKind::Float.numeric_width(), 6);
        assert_eq!(PrimitiveKind::Double.numeric_width(), 7);
        assert_eq!(PrimitiveKind::BigDecimal.numeric_width(), 8);
        // Non-numeric types
        assert_eq!(PrimitiveKind::Null.numeric_width(), 0);
        assert_eq!(PrimitiveKind::Boolean.numeric_width(), 0);
        assert_eq!(PrimitiveKind::Char.numeric_width(), 0);
        assert_eq!(PrimitiveKind::String.numeric_width(), 0);
        assert_eq!(PrimitiveKind::DateTime.numeric_width(), 0);
        assert_eq!(PrimitiveKind::Duration.numeric_width(), 0);
    }

    #[test]
    fn primitive_kind_is_integer() {
        assert!(PrimitiveKind::Byte.is_integer());
        assert!(PrimitiveKind::Short.is_integer());
        assert!(PrimitiveKind::Int.is_integer());
        assert!(PrimitiveKind::Long.is_integer());
        assert!(PrimitiveKind::BigInt.is_integer());
        assert!(!PrimitiveKind::Float.is_integer());
        assert!(!PrimitiveKind::Double.is_integer());
        assert!(!PrimitiveKind::BigDecimal.is_integer());
        assert!(!PrimitiveKind::Boolean.is_integer());
        assert!(!PrimitiveKind::Null.is_integer());
    }

    #[test]
    fn primitive_kind_is_floating() {
        assert!(PrimitiveKind::Float.is_floating());
        assert!(PrimitiveKind::Double.is_floating());
        assert!(PrimitiveKind::BigDecimal.is_floating());
        assert!(!PrimitiveKind::Int.is_floating());
        assert!(!PrimitiveKind::Long.is_floating());
        assert!(!PrimitiveKind::BigInt.is_floating());
    }

    #[test]
    fn primitive_kind_is_big() {
        assert!(PrimitiveKind::BigInt.is_big());
        assert!(PrimitiveKind::BigDecimal.is_big());
        assert!(!PrimitiveKind::Int.is_big());
        assert!(!PrimitiveKind::Double.is_big());
    }

    #[test]
    fn primitive_kind_widen() {
        assert_eq!(
            PrimitiveKind::Byte.widen(PrimitiveKind::Int),
            PrimitiveKind::Int
        );
        assert_eq!(
            PrimitiveKind::Int.widen(PrimitiveKind::Byte),
            PrimitiveKind::Int
        );
        assert_eq!(
            PrimitiveKind::Long.widen(PrimitiveKind::Float),
            PrimitiveKind::Float
        );
        assert_eq!(
            PrimitiveKind::Float.widen(PrimitiveKind::Long),
            PrimitiveKind::Float
        );
        assert_eq!(
            PrimitiveKind::Double.widen(PrimitiveKind::BigDecimal),
            PrimitiveKind::BigDecimal
        );
        assert_eq!(
            PrimitiveKind::BigDecimal.widen(PrimitiveKind::Double),
            PrimitiveKind::BigDecimal
        );
        // Null and Boolean both have numeric_width=0, so widen returns self (left)
        assert_eq!(
            PrimitiveKind::Null.widen(PrimitiveKind::Boolean),
            PrimitiveKind::Null
        );
        assert_eq!(
            PrimitiveKind::Boolean.widen(PrimitiveKind::Null),
            PrimitiveKind::Boolean
        );
    }

    #[test]
    fn primitive_kind_display() {
        assert_eq!(format!("{}", PrimitiveKind::Int), "int");
        assert_eq!(format!("{}", PrimitiveKind::String), "java.lang.String");
    }

    #[test]
    fn primitive_kind_copy_clone_eq_hash() {
        let a = PrimitiveKind::Int;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(format!("{:?}", a), "Int");
    }

    // ════════════════════════════════════════════════════════════════════
    // TypeDescriptor constants
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn constants_are_correct() {
        assert!(matches!(
            TypeDescriptor::INT,
            TypeDescriptor::Primitive(PrimitiveKind::Int)
        ));
        assert!(matches!(
            TypeDescriptor::LONG,
            TypeDescriptor::Primitive(PrimitiveKind::Long)
        ));
        assert!(matches!(
            TypeDescriptor::FLOAT,
            TypeDescriptor::Primitive(PrimitiveKind::Float)
        ));
        assert!(matches!(
            TypeDescriptor::DOUBLE,
            TypeDescriptor::Primitive(PrimitiveKind::Double)
        ));
        assert!(matches!(
            TypeDescriptor::BOOLEAN,
            TypeDescriptor::Primitive(PrimitiveKind::Boolean)
        ));
        assert!(matches!(
            TypeDescriptor::STRING,
            TypeDescriptor::Primitive(PrimitiveKind::String)
        ));
        assert!(matches!(
            TypeDescriptor::NULL,
            TypeDescriptor::Primitive(PrimitiveKind::Null)
        ));
    }

    // ════════════════════════════════════════════════════════════════════
    // from_type_name / from_type_id / with_generic / with_annotation
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn from_type_name_basic() {
        let td = TypeDescriptor::from_type_name("MyClass");
        assert_eq!(td.name(), "MyClass");
        assert!(matches!(td, TypeDescriptor::Named { name, .. } if name == "MyClass"));
    }

    #[test]
    fn from_type_id_has_type_id() {
        let td = TypeDescriptor::from_type_id::<String>();
        assert!(td.type_id().is_some());
        assert!(td.name().contains("String"));
    }

    #[test]
    fn with_generic_adds_generic() {
        let td = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
        assert_eq!(td.name(), "List<int>");
    }

    #[test]
    fn with_multiple_generics() {
        let td = TypeDescriptor::from_type_name("Map")
            .with_generic(TypeDescriptor::STRING)
            .with_generic(TypeDescriptor::INT);
        assert_eq!(td.name(), "Map<java.lang.String,int>");
    }

    #[test]
    fn with_annotation_on_non_named_is_noop() {
        let td = TypeDescriptor::INT.with_annotation("Nullable");
        // Primitive variant ignores annotation
        assert_eq!(td, TypeDescriptor::INT);
    }

    #[test]
    fn with_annotation_on_named() {
        let td = TypeDescriptor::from_type_name("Foo").with_annotation("Nullable");
        if let TypeDescriptor::Named { annotations, .. } = &td {
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0], "Nullable");
        } else {
            panic!("expected Named");
        }
    }

    // ════════════════════════════════════════════════════════════════════
    // name()
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn name_primitive() {
        assert_eq!(TypeDescriptor::INT.name(), "int");
        assert_eq!(TypeDescriptor::BOOLEAN.name(), "boolean");
        assert_eq!(TypeDescriptor::STRING.name(), "java.lang.String");
    }

    #[test]
    fn name_array() {
        let td = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        assert_eq!(td.name(), "int[]");
    }

    #[test]
    fn name_map() {
        let td = TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        );
        assert_eq!(td.name(), "java.util.Map<java.lang.String, int>");
    }

    #[test]
    fn name_named_no_generics() {
        assert_eq!(TypeDescriptor::from_type_name("Foo").name(), "Foo");
    }

    #[test]
    fn name_named_with_generics() {
        let td = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
        assert_eq!(td.name(), "List<int>");
    }

    // ════════════════════════════════════════════════════════════════════
    // is_primitive / is_nullable_primitive
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn is_primitive() {
        assert!(TypeDescriptor::INT.is_primitive());
        assert!(TypeDescriptor::NULL.is_primitive());
        assert!(!TypeDescriptor::from_type_name("Foo").is_primitive());
        assert!(!TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).is_primitive());
        assert!(
            !TypeDescriptor::Map(Box::new(TypeDescriptor::INT), Box::new(TypeDescriptor::INT))
                .is_primitive()
        );
    }

    #[test]
    fn is_nullable_primitive() {
        assert!(TypeDescriptor::NULL.is_nullable_primitive());
        assert!(!TypeDescriptor::INT.is_nullable_primitive());
    }

    // ════════════════════════════════════════════════════════════════════
    // new() convenience factory
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn new_known_names() {
        assert_eq!(TypeDescriptor::new("int"), TypeDescriptor::INT);
        assert_eq!(TypeDescriptor::new("Integer"), TypeDescriptor::INT);
        assert_eq!(TypeDescriptor::new("long"), TypeDescriptor::LONG);
        assert_eq!(TypeDescriptor::new("Long"), TypeDescriptor::LONG);
        assert_eq!(TypeDescriptor::new("float"), TypeDescriptor::FLOAT);
        assert_eq!(TypeDescriptor::new("Float"), TypeDescriptor::FLOAT);
        assert_eq!(TypeDescriptor::new("double"), TypeDescriptor::DOUBLE);
        assert_eq!(TypeDescriptor::new("Double"), TypeDescriptor::DOUBLE);
        assert_eq!(TypeDescriptor::new("boolean"), TypeDescriptor::BOOLEAN);
        assert_eq!(TypeDescriptor::new("Boolean"), TypeDescriptor::BOOLEAN);
        assert_eq!(TypeDescriptor::new("String"), TypeDescriptor::STRING);
        assert_eq!(TypeDescriptor::new("null"), TypeDescriptor::NULL);
    }

    #[test]
    fn new_object_value() {
        let obj = TypeDescriptor::new("object");
        assert!(matches!(obj, TypeDescriptor::Named { .. }));
        let val = TypeDescriptor::new("value");
        assert!(matches!(val, TypeDescriptor::Named { .. }));
    }

    #[test]
    fn new_unknown_name() {
        let td = TypeDescriptor::new("com.example.Foo");
        assert_eq!(td.name(), "com.example.Foo");
        assert!(matches!(td, TypeDescriptor::Named { type_id: None, .. }));
    }

    // ════════════════════════════════════════════════════════════════════
    // is_assignable_from
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn assignable_object_accepts_all() {
        let obj = TypeDescriptor::OBJECT;
        assert!(obj.is_assignable_from(&TypeDescriptor::INT));
        assert!(obj.is_assignable_from(&TypeDescriptor::STRING));
        assert!(obj.is_assignable_from(&TypeDescriptor::BOOLEAN));
        assert!(obj.is_assignable_from(&TypeDescriptor::from_type_name("Foo")));
    }

    #[test]
    fn assignable_same_type() {
        assert!(TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::INT));
        assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::STRING));
    }

    #[test]
    fn assignable_numeric_widening() {
        // Long can accept Int (width 4 >= 3)
        assert!(TypeDescriptor::LONG.is_assignable_from(&TypeDescriptor::INT));
        // Double can accept Float
        assert!(TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::FLOAT));
        // BigDecimal can accept all
        let bd = TypeDescriptor::Primitive(PrimitiveKind::BigDecimal);
        assert!(bd.is_assignable_from(&TypeDescriptor::INT));
        assert!(bd.is_assignable_from(&TypeDescriptor::DOUBLE));
        // Int cannot accept Long
        assert!(!TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::LONG));
    }

    #[test]
    fn assignable_string_accepts_all() {
        assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::INT));
        assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::BOOLEAN));
        assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::from_type_name("Foo")));
    }

    #[test]
    fn assignable_map_compatible() {
        let m1 = TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        );
        let m2 = TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        );
        assert!(m1.is_assignable_from(&m2));
    }

    #[test]
    fn assignable_array_compatible() {
        let a1 = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        let a2 = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        assert!(a1.is_assignable_from(&a2));
    }

    #[test]
    fn assignable_named_same_name() {
        let n1 = TypeDescriptor::from_type_name("Foo");
        let n2 = TypeDescriptor::from_type_name("Foo");
        assert!(n1.is_assignable_from(&n2));
    }

    #[test]
    fn assignable_named_different_name() {
        let n1 = TypeDescriptor::from_type_name("Foo");
        let n2 = TypeDescriptor::from_type_name("Bar");
        assert!(!n1.is_assignable_from(&n2));
    }

    #[test]
    fn assignable_incompatible_types() {
        // Boolean has numeric_width=0, so numeric types accept it (width >= 0).
        // True incompatibility: Array vs Map, Primitive vs Named (different name).
        assert!(
            !TypeDescriptor::INT
                .is_assignable_from(&TypeDescriptor::Array(Box::new(TypeDescriptor::INT)))
        );
        assert!(
            !TypeDescriptor::BOOLEAN.is_assignable_from(&TypeDescriptor::from_type_name("Foo"))
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // narrow
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn narrow_to_int() {
        assert_eq!(TypeDescriptor::OBJECT.narrow("int"), TypeDescriptor::INT);
    }

    #[test]
    fn narrow_to_long() {
        assert_eq!(TypeDescriptor::OBJECT.narrow("long"), TypeDescriptor::LONG);
    }

    #[test]
    fn narrow_to_boolean() {
        assert_eq!(
            TypeDescriptor::OBJECT.narrow("boolean"),
            TypeDescriptor::BOOLEAN
        );
    }

    #[test]
    fn narrow_to_double() {
        assert_eq!(
            TypeDescriptor::OBJECT.narrow("double"),
            TypeDescriptor::DOUBLE
        );
    }

    #[test]
    fn narrow_to_string() {
        assert_eq!(
            TypeDescriptor::OBJECT.narrow("string"),
            TypeDescriptor::STRING
        );
    }

    #[test]
    fn narrow_unknown_returns_self() {
        let td = TypeDescriptor::from_type_name("Foo");
        assert_eq!(td.narrow("unknown"), td);
    }

    // ════════════════════════════════════════════════════════════════════
    // get_map_key_type / get_map_value_type / get_element_type
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn map_key_value_types() {
        let m = TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        );
        assert_eq!(m.get_map_key_type(), Some(&TypeDescriptor::STRING));
        assert_eq!(m.get_map_value_type(), Some(&TypeDescriptor::INT));
    }

    #[test]
    fn non_map_returns_none() {
        assert!(TypeDescriptor::INT.get_map_key_type().is_none());
        assert!(TypeDescriptor::INT.get_map_value_type().is_none());
    }

    #[test]
    fn array_element_type() {
        let a = TypeDescriptor::Array(Box::new(TypeDescriptor::STRING));
        assert_eq!(a.get_element_type(), Some(&TypeDescriptor::STRING));
    }

    #[test]
    fn non_array_returns_none() {
        assert!(TypeDescriptor::INT.get_element_type().is_none());
    }

    // ════════════════════════════════════════════════════════════════════
    // is_map / is_array
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn is_map_variants() {
        let m = TypeDescriptor::Map(Box::new(TypeDescriptor::INT), Box::new(TypeDescriptor::INT));
        assert!(m.is_map());
        assert!(!TypeDescriptor::INT.is_map());
        assert!(!TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).is_map());
        assert!(!TypeDescriptor::from_type_name("Foo").is_map());
    }

    #[test]
    fn is_array_variants() {
        let a = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        assert!(a.is_array());
        assert!(!TypeDescriptor::INT.is_array());
        assert!(
            !TypeDescriptor::Map(Box::new(TypeDescriptor::INT), Box::new(TypeDescriptor::INT))
                .is_array()
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // type_id / primitive_kind
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn type_id_on_named() {
        let td = TypeDescriptor::from_type_id::<String>();
        assert!(td.type_id().is_some());
    }

    #[test]
    fn type_id_on_primitive() {
        assert!(TypeDescriptor::INT.type_id().is_none());
    }

    #[test]
    fn type_id_on_array() {
        assert!(
            TypeDescriptor::Array(Box::new(TypeDescriptor::INT))
                .type_id()
                .is_none()
        );
    }

    #[test]
    fn primitive_kind_on_primitive() {
        assert_eq!(
            TypeDescriptor::INT.primitive_kind(),
            Some(PrimitiveKind::Int)
        );
        assert_eq!(
            TypeDescriptor::BOOLEAN.primitive_kind(),
            Some(PrimitiveKind::Boolean)
        );
    }

    #[test]
    fn primitive_kind_on_non_primitive() {
        assert!(
            TypeDescriptor::from_type_name("Foo")
                .primitive_kind()
                .is_none()
        );
        assert!(
            TypeDescriptor::Array(Box::new(TypeDescriptor::INT))
                .primitive_kind()
                .is_none()
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // PartialEq / Eq / Hash / Display / Clone / Debug
    // ════════════════════════════════════════════════════════════════════

    #[test]
    fn partial_eq_primitives() {
        assert_eq!(TypeDescriptor::INT, TypeDescriptor::INT);
        assert_ne!(TypeDescriptor::INT, TypeDescriptor::LONG);
    }

    #[test]
    fn partial_eq_arrays() {
        let a1 = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        let a2 = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        let a3 = TypeDescriptor::Array(Box::new(TypeDescriptor::STRING));
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }

    #[test]
    fn partial_eq_maps() {
        let m1 = TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        );
        let m2 = TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        );
        let m3 = TypeDescriptor::Map(
            Box::new(TypeDescriptor::INT),
            Box::new(TypeDescriptor::STRING),
        );
        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }

    #[test]
    fn partial_eq_named() {
        let n1 = TypeDescriptor::from_type_name("Foo");
        let n2 = TypeDescriptor::from_type_name("Foo");
        let n3 = TypeDescriptor::from_type_name("Bar");
        assert_eq!(n1, n2);
        assert_ne!(n1, n3);
    }

    #[test]
    fn partial_eq_cross_variant() {
        assert_ne!(TypeDescriptor::INT, TypeDescriptor::from_type_name("int"));
        assert_ne!(
            TypeDescriptor::Array(Box::new(TypeDescriptor::INT)),
            TypeDescriptor::Map(Box::new(TypeDescriptor::INT), Box::new(TypeDescriptor::INT))
        );
    }

    #[test]
    fn hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let h = |td: &TypeDescriptor| {
            let mut s = DefaultHasher::new();
            td.hash(&mut s);
            s.finish()
        };

        assert_eq!(h(&TypeDescriptor::INT), h(&TypeDescriptor::INT));
        assert_eq!(h(&TypeDescriptor::STRING), h(&TypeDescriptor::STRING));
    }

    #[test]
    fn display_impl() {
        assert_eq!(format!("{}", TypeDescriptor::INT), "int");
        assert_eq!(format!("{}", TypeDescriptor::STRING), "java.lang.String");
        let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        assert_eq!(format!("{}", arr), "int[]");
    }

    #[test]
    fn debug_impl() {
        assert!(format!("{:?}", TypeDescriptor::INT).contains("Primitive"));
        assert!(
            format!("{:?}", TypeDescriptor::Array(Box::new(TypeDescriptor::INT))).contains("Array")
        );
    }

    #[test]
    fn clone_independence() {
        let a = TypeDescriptor::from_type_name("Foo").with_generic(TypeDescriptor::INT);
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ── widening_order (original) ─────────────────────────────────────

    #[test]
    fn widening_order() {
        assert!(PrimitiveKind::BigDecimal.numeric_width() > PrimitiveKind::Double.numeric_width());
        assert_eq!(
            PrimitiveKind::Int.widen(PrimitiveKind::Long),
            PrimitiveKind::Long
        );
        assert_eq!(
            PrimitiveKind::Long.widen(PrimitiveKind::Int),
            PrimitiveKind::Long
        );
    }

    #[test]
    fn is_assignable_object() {
        let int_d = TypeDescriptor::INT;
        let obj = TypeDescriptor::OBJECT;
        assert!(obj.is_assignable_from(&int_d));
    }

    #[test]
    fn name_rendering() {
        assert_eq!(TypeDescriptor::INT.name(), "int");
        assert_eq!(TypeDescriptor::STRING.name(), "java.lang.String");
        let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        assert_eq!(arr.name(), "int[]");
    }
}

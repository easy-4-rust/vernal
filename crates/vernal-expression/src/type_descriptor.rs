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
        if let Self::Named { ref mut generics, .. } = self {
            generics.push(g);
        }
        self
    }

    /// 添加注解（占位 API，对标 Spring 注解查找）。
    #[must_use]
    pub fn with_annotation(mut self, name: impl Into<String>) -> Self {
        if let Self::Named { ref mut annotations, .. } = self {
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
            // 数字 widening
            (Self::Primitive(dp), Self::Primitive(sp)) => {
                dp.numeric_width() >= sp.numeric_width()
                    || (*dp == PrimitiveKind::Double && sp.is_floating())
                    || (*dp == PrimitiveKind::Float && *sp == PrimitiveKind::Float)
            }
            // 任何类型可赋给 String（toString 转换）
            (Self::Primitive(PrimitiveKind::String), _) => true,
            // Map 兼容
            (Self::Map(ak, av), Self::Map(sk, sv)) => {
                ak.is_assignable_from(sk) && av.is_assignable_from(sv)
            }
            // Array 兼容
            (Self::Array(a), Self::Array(b)) => a.is_assignable_from(b),
            // 具名类型泛型匹配
            (
                Self::Named { name: na, .. },
                Self::Named { name: nb, .. },
            ) => na == nb,
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

    #[test]
    fn widening_order() {
        assert!(PrimitiveKind::BigDecimal.numeric_width() > PrimitiveKind::Double.numeric_width());
        assert_eq!(PrimitiveKind::Int.widen(PrimitiveKind::Long), PrimitiveKind::Long);
        assert_eq!(PrimitiveKind::Long.widen(PrimitiveKind::Int), PrimitiveKind::Long);
    }

    #[test]
    fn is_assignable_object() {
        let int_d = TypeDescriptor::INT;
        let obj = TypeDescriptor::OBJECT;
        let result = obj.is_assignable_from(&int_d);
        eprintln!("obj.name() = {}", obj.name());
        eprintln!("int_d.name() = {}", int_d.name());
        eprintln!("is_assignable_from = {result}");
        // Debug: check the exact values
        if let TypeDescriptor::Named { name, .. } = &obj {
            eprintln!("obj.name == 'object' is {}", name == "object");
        } else {
            eprintln!("obj is NOT Named variant");
        }
        assert!(result);
    }

    #[test]
    fn name_rendering() {
        assert_eq!(TypeDescriptor::INT.name(), "int");
        assert_eq!(TypeDescriptor::STRING.name(), "java.lang.String");
        let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
        assert_eq!(arr.name(), "int[]");
    }
}

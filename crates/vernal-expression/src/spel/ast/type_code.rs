//! 类型代码枚举（对标 Spring `TypeCode`）。
//!
//! Spring `TypeCode` 是紧凑的类型码（用于 `OpPlus`/`OpMinus` 等优化路径）。
//! 我们扩展到包含 BigInteger/BigDecimal/String/array 以便在 AST 里复用。

/// 类型代码枚举。
///
/// 对标 Spring `org.springframework.expression.spel.ast.TypeCode`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCode {
    /// 对象类型。
    Object,
    /// 布尔类型。
    Boolean,
    /// 字符类型。
    Char,
    /// 字节类型。
    Byte,
    /// 短整型。
    Short,
    /// 整型。
    Int,
    /// 长整型。
    Long,
    /// 单精度浮点。
    Float,
    /// 双精度浮点。
    Double,
    /// 任意精度整数。
    BigInteger,
    /// 任意精度小数。
    BigDecimal,
    /// 字符串。
    String,
    /// 数组。
    Array,
}

impl TypeCode {
    /// 获取类型名称（与 Spring `TypeCode.getName()` 一致）。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Boolean => "boolean",
            Self::Char => "char",
            Self::Byte => "byte",
            Self::Short => "short",
            Self::Int => "int",
            Self::Long => "long",
            Self::Float => "float",
            Self::Double => "double",
            Self::BigInteger => "java.math.BigInteger",
            Self::BigDecimal => "java.math.BigDecimal",
            Self::String => "java.lang.String",
            Self::Array => "array",
        }
    }

    /// 是否为整数类型。
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Byte | Self::Short | Self::Int | Self::Long | Self::BigInteger)
    }

    /// 是否为数字类型。
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(
            self,
            Self::Byte
                | Self::Short
                | Self::Int
                | Self::Long
                | Self::Float
                | Self::Double
                | Self::BigInteger
                | Self::BigDecimal
        )
    }
}

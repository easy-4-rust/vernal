//! 类型代码枚举。
//!
//! 对标 Spring 的 `TypeCode`。

/// 类型代码枚举。
///
/// 捕获原始类型和引用类型。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.TypeCode`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCode {
    /// 对象类型
    Object,
    /// 布尔类型
    Boolean,
    /// 字符类型
    Char,
    /// 字节类型
    Byte,
    /// 短整型
    Short,
    /// 整型
    Int,
    /// 长整型
    Long,
    /// 单精度浮点
    Float,
    /// 双精度浮点
    Double,
}

impl TypeCode {
    /// 获取类型名称。
    #[must_use]
    pub fn name(&self) -> &'static str {
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
        }
    }
}

//! SpEL 错误消息枚举。
//!
//! 对标 Spring 的 `SpelMessage`：所有 SpEL 错误消息码。

/// SpEL 错误消息码。
///
/// 对标 Spring 的 `org.springframework.expression.spel.SpelMessage`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpelMessage {
    /// 类型转换错误
    TypeConversionError,
    /// 构造器未找到
    ConstructorNotFound,
    /// 方法未找到
    MethodNotFound,
    /// 类型未找到
    TypeNotFound,
    /// 属性或字段不可读
    PropertyOrFieldNotReadable,
    /// 属性或字段不可写
    PropertyOrFieldNotWritable,
    /// 运算符不支持
    OperatorNotSupported,
    /// 除零错误
    DivisionByZero,
    /// 表达式长度超限
    MaxExpressionLengthExceeded,
    /// 运算次数超限
    MaxOperationsExceeded,
    /// 内部错误
    InternalError,
}

impl SpelMessage {
    /// 获取错误码。
    #[must_use]
    pub fn code(&self) -> i32 {
        match self {
            Self::TypeConversionError => 1001,
            Self::ConstructorNotFound => 1002,
            Self::MethodNotFound => 1004,
            Self::TypeNotFound => 1005,
            Self::PropertyOrFieldNotReadable => 1008,
            Self::PropertyOrFieldNotWritable => 1009,
            Self::OperatorNotSupported => 1030,
            Self::DivisionByZero => 1040,
            Self::MaxExpressionLengthExceeded => 1079,
            Self::MaxOperationsExceeded => 1085,
            Self::InternalError => 9999,
        }
    }

    /// 获取默认消息。
    #[must_use]
    pub fn default_message(&self) -> &'static str {
        match self {
            Self::TypeConversionError => "类型转换错误",
            Self::ConstructorNotFound => "构造器未找到",
            Self::MethodNotFound => "方法未找到",
            Self::TypeNotFound => "类型未找到",
            Self::PropertyOrFieldNotReadable => "属性或字段不可读",
            Self::PropertyOrFieldNotWritable => "属性或字段不可写",
            Self::OperatorNotSupported => "运算符不支持",
            Self::DivisionByZero => "除零错误",
            Self::MaxExpressionLengthExceeded => "表达式长度超限",
            Self::MaxOperationsExceeded => "运算次数超限",
            Self::InternalError => "内部错误",
        }
    }
}

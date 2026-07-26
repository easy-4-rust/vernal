//! 词法单元类型枚举。
//!
//! 对标 Spring 的 `TokenKind`。

/// 词法单元类型。
///
/// 对标 Spring 的 `org.springframework.expression.spel.standard.TokenKind`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // 字面量
    LiteralInt,
    LiteralLong,
    LiteralHexInt,
    LiteralHexLong,
    LiteralString,
    LiteralReal,
    LiteralRealFloat,

    // 运算符
    Plus,
    Minus,
    Star,
    Divide,
    Modulus,
    Power,
    Inc,
    Dec,
    PlusAssign,
    MinusAssign,
    StarAssign,
    DivAssign,
    ModAssign,

    // 比较
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // 逻辑
    And,
    Or,
    Not,

    // 分隔符
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Comma,
    Dot,
    Colon,
    Semi,
    Hash,
    At,
    QMark,
    Elvis,
    SafeNavi,
    BeanRef,
    FactoryBeanRef,

    // 标识符和变量
    Identifier,
    Variable,

    // 特殊
    Assign,
    Project,
    Selection,
    SymbolicOr,
    SymbolicAnd,
    Pipe,
    Question,
}

impl TokenKind {
    /// 判断是否为字面量。
    #[must_use]
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::LiteralInt
                | Self::LiteralLong
                | Self::LiteralHexInt
                | Self::LiteralHexLong
                | Self::LiteralString
                | Self::LiteralReal
                | Self::LiteralRealFloat
        )
    }

    /// 判断是否为运算符。
    #[must_use]
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Star
                | Self::Divide
                | Self::Modulus
                | Self::Power
                | Self::Inc
                | Self::Dec
        )
    }
}

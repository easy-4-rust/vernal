//! TokenKind 枚举（对标 Spring `org.springframework.expression.spel.standard.TokenKind`）。
//!
//! 46 个变体，按 Spring 顺序定义。命名与 SpEL 文档完全一致。

/// 词法单元类型（对标 Spring `TokenKind`）。
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ── 字面量（无 tokenChars payload，data 携带值）──
    /// 整数字面量（`42`）。
    LiteralInt,
    /// 长整型字面量（`42L`）。
    LiteralLong,
    /// 十六进制整数字面量（`0x1A`）。
    LiteralHexInt,
    /// 十六进制长整型字面量（`0x1AL`）。
    LiteralHexLong,
    /// 字符串字面量（`'foo'` / `"foo"`）。
    LiteralString,
    /// 实数字面量（`3.14`）。
    LiteralReal,
    /// 单精度实数字面量（`3.14F`）。
    LiteralRealFloat,

    // ── 括号 / 分隔符 ──
    /// 左圆括号 `(`。
    LParen,
    /// 右圆括号 `)`。
    RParen,
    /// 逗号 `,`。
    Comma,
    /// 标识符（变量名、关键字等）。data 携带字符串。
    Identifier,
    /// 冒号 `:`。
    Colon,
    /// 井号 `#`。
    Hash,
    /// 右方括号 `]`。
    RSquare,
    /// 左方括号 `[`。
    LSquare,
    /// 左花括号 `{`。
    LCurly,
    /// 右花括号 `}`。
    RCurly,
    /// 点 `.`。
    Dot,
    /// 加号 `+`。
    Plus,
    /// 星号 `*`。
    Star,
    /// 减号 `-`。
    Minus,

    // ── 特殊 token ──
    /// 选择首元素 `^[`。
    SelectFirst,
    /// 选择末元素 `$[`。
    SelectLast,
    /// 问号 `?`（三元条件）。
    QMark,
    /// 投影 `![`。
    Project,
    /// 除号 `/`。
    Div,
    /// 大于等于 `>=`。
    Ge,
    /// 大于 `>`。
    Gt,
    /// 小于等于 `<=`。
    Le,
    /// 小于 `<`。
    Lt,
    /// 等于 `==`。
    Equal,
    /// 不等于 `!=`。
    NotEqual,
    /// 取模 `%`。
    Mod,
    /// 逻辑非 `!`。
    Not,
    /// 赋值 `=`。
    Assign,
    /// `instanceof` 关键字（由 IDENTIFIER 转换）。
    Instanceof,
    /// `matches` 关键字。
    Matches,
    /// `between` 关键字。
    Between,
    /// 选择全部 `?[`。
    Select,
    /// 幂 `^`。
    Power,
    /// Elvis `?:`。
    Elvis,
    /// 安全导航 `?.`。
    SafeNavi,
    /// Bean 引用 `@`。
    BeanRef,
    /// FactoryBean 引用 `&`。
    FactoryBeanRef,
    /// 逻辑或 `||`。
    SymbolicOr,
    /// 逻辑与 `&&`。
    SymbolicAnd,
    /// 自增 `++`。
    Inc,
    /// 自减 `--`。
    Dec,
}

impl TokenKind {
    /// 此 token 是否携带字符串 payload（数据 = `data` 字段）。
    ///
    /// Spring: 空 `tokenChars` ↔ `hasPayload`。
    /// 我们用枚举显式区分：标识符/字面量有 payload，运算符没有。
    #[must_use]
    pub const fn has_payload(self) -> bool {
        matches!(
            self,
            Self::Identifier
                | Self::LiteralInt
                | Self::LiteralLong
                | Self::LiteralHexInt
                | Self::LiteralHexLong
                | Self::LiteralString
                | Self::LiteralReal
                | Self::LiteralRealFloat
                | Self::Instanceof
                | Self::Matches
                | Self::Between
        )
    }

    /// Token 字符长度（单字符/双字符/三字符）。
    #[must_use]
    pub const fn token_chars(self) -> &'static str {
        match self {
            Self::LiteralInt
            | Self::LiteralLong
            | Self::LiteralHexInt
            | Self::LiteralHexLong
            | Self::LiteralString
            | Self::LiteralReal
            | Self::LiteralRealFloat
            | Self::Identifier => "",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::Comma => ",",
            Self::Colon => ":",
            Self::Hash => "#",
            Self::RSquare => "]",
            Self::LSquare => "[",
            Self::LCurly => "{",
            Self::RCurly => "}",
            Self::Dot => ".",
            Self::Plus => "+",
            Self::Star => "*",
            Self::Minus => "-",
            Self::SelectFirst => "^[",
            Self::SelectLast => "$[",
            Self::QMark => "?",
            Self::Project => "![",
            Self::Div => "/",
            Self::Ge => ">=",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Lt => "<",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Mod => "%",
            Self::Not => "!",
            Self::Assign => "=",
            Self::Instanceof => "instanceof",
            Self::Matches => "matches",
            Self::Between => "between",
            Self::Select => "?[",
            Self::Power => "^",
            Self::Elvis => "?:",
            Self::SafeNavi => "?.",
            Self::BeanRef => "@",
            Self::FactoryBeanRef => "&",
            Self::SymbolicOr => "||",
            Self::SymbolicAnd => "&&",
            Self::Inc => "++",
            Self::Dec => "--",
        }
    }

    /// Token 字符长度。
    #[must_use]
    pub const fn length(self) -> usize {
        self.token_chars().len()
    }

    /// 是否为数字关系运算符（用于 SpEL relational 节点判定）。
    #[must_use]
    pub const fn is_numeric_relational(self) -> bool {
        matches!(
            self,
            Self::Gt | Self::Ge | Self::Lt | Self::Le | Self::Equal | Self::NotEqual
        )
    }

    /// 是否为字面量。
    #[must_use]
    pub const fn is_literal(self) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_has_payload() {
        assert!(TokenKind::Identifier.has_payload());
    }

    #[test]
    fn plus_no_payload() {
        assert!(!TokenKind::Plus.has_payload());
        assert_eq!(TokenKind::Plus.token_chars(), "+");
        assert_eq!(TokenKind::Plus.length(), 1);
    }

    #[test]
    fn safe_navi_two_chars() {
        assert_eq!(TokenKind::SafeNavi.length(), 2);
        assert_eq!(TokenKind::SafeNavi.token_chars(), "?.");
    }

    #[test]
    fn numeric_relational() {
        assert!(TokenKind::Equal.is_numeric_relational());
        assert!(TokenKind::Lt.is_numeric_relational());
        assert!(!TokenKind::And.is_numeric_relational());
    }

    #[test]
    fn instanceof_keyword_payload() {
        assert!(TokenKind::Instanceof.has_payload());
        assert_eq!(TokenKind::Instanceof.token_chars(), "instanceof");
    }
}

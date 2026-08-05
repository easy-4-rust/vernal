//! 操作枚举（对标 Spring `Operation` 体系）。
//!
//! Spring 的 `SpelNodeImpl` 子类以 `getValueInternal(ExpressionState)` 直接实现求值；
//! 我们把所有操作统一收集到 `Operation` enum，让 `ExpressionState.operate` 集中调度，
//! 以便实现 `OperatorOverloader` SPI 与统一错误码路径。

/// 表达式求值时的运算符枚举。
///
/// 对应 Spring 中的所有二元/关系/逻辑/特殊运算符。
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    /// 加法（数值 + 字符串连接）。
    Add,
    /// 减法。
    Subtract,
    /// 乘法。
    Multiply,
    /// 除法。
    Divide,
    /// 取模。
    Modulus,
    /// 幂（Spring 6+ 的 `^` 运算符）。
    Power,
    /// 等于（`==`、`eq`）。
    Equal,
    /// 不等于（`!=`、`ne`）。
    NotEqual,
    /// 小于（`<`、`lt`）。
    LessThan,
    /// 小于等于（`<=`、`le`）。
    LessEqual,
    /// 大于（`>`、`gt`）。
    GreaterThan,
    /// 大于等于（`>=`、`ge`）。
    GreaterEqual,
    /// 逻辑与（`and`、`&&`）。
    And,
    /// 逻辑或（`or`、`||`）。
    Or,
    /// 正则匹配（`matches 'regex'`）。
    Matches,
    /// 范围包含（`a between {lo, hi}`）。
    Between,
    /// 类型检查（`obj instanceof T`）。
    InstanceOf,
    /// Elvis 运算符（`a ?: b`）。
    Elvis,
    /// 赋值（`a = b`）。
    Assign,
    /// 自增（`++`，prefix/postfix）。
    Increment,
    /// 自减（`--`，prefix/postfix）。
    Decrement,
}

impl Operation {
    /// Spring 操作名称（用于 `BinaryOperator.operator_name`）。
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulus => "%",
            Self::Power => "^",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterEqual => ">=",
            Self::And => "&&",
            Self::Or => "||",
            Self::Matches => "matches",
            Self::Between => "between",
            Self::InstanceOf => "instanceof",
            Self::Elvis => "?:",
            Self::Assign => "=",
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }

    /// 是否为算术运算符。
    #[must_use]
    pub const fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Subtract
                | Self::Multiply
                | Self::Divide
                | Self::Modulus
                | Self::Power
        )
    }

    /// 是否为关系运算符。
    #[must_use]
    pub const fn is_relational(&self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::LessThan
                | Self::LessEqual
                | Self::GreaterThan
                | Self::GreaterEqual
        )
    }

    /// 是否为短路运算符（`and`/`or`）。
    #[must_use]
    pub const fn is_short_circuit(&self) -> bool {
        matches!(self, Self::And | Self::Or)
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_round_trip() {
        assert_eq!(Operation::Add.as_str(), "+");
        assert_eq!(Operation::Power.as_str(), "^");
        assert_eq!(Operation::NotEqual.as_str(), "!=");
    }
}

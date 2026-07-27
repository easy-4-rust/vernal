//! Autowire — Spring 风格的自动装配模式枚举。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.Autowire`。
//!
//! 定义自动装配的策略：不装配、按名称、按类型、按构造器。

/// Spring 风格的自动装配模式枚举。
///
/// 对应 Spring 的 `Autowire` 枚举。
///
/// 定义容器在创建 Bean 时的自动装配策略。
///
/// ## 常量
///
/// - `AUTOWIRE_NO = 0` — 不自动装配（默认）
/// - `AUTOWIRE_BY_NAME = 1` — 按名称自动装配
/// - `AUTOWIRE_BY_TYPE = 2` — 按类型自动装配
/// - `AUTOWIRE_CONSTRUCTOR = 3` — 按构造器自动装配
/// - `AUTOWIRE_AUTODETECT = 4` — 已废弃，不使用
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Autowire {
    /// 不自动装配。
    ///
    /// 对应 Spring 的 `AUTOWIRE_NO = 0`。
    /// 需要显式通过 `@Autowired` 或构造器参数注入。
    No = 0,

    /// 按名称自动装配。
    ///
    /// 对应 Spring 的 `AUTOWIRE_BY_NAME = 1`。
    /// 将属性名与 Bean 名称匹配。
    ByName = 1,

    /// 按类型自动装配。
    ///
    /// 对应 Spring 的 `AUTOWIRE_BY_TYPE = 2`。
    /// 将属性类型与 Bean 类型匹配。
    ByType = 2,

    /// 按构造器自动装配。
    ///
    /// 对应 Spring 的 `AUTOWIRE_CONSTRUCTOR = 3`。
    /// 选择参数类型匹配最多的构造器。
    Constructor = 3,
}

impl Autowire {
    /// 从整数值创建。
    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::No),
            1 => Some(Self::ByName),
            2 => Some(Self::ByType),
            3 => Some(Self::Constructor),
            _ => None,
        }
    }

    /// 转换为整数值。
    pub fn as_int(&self) -> i32 {
        *self as i32
    }
}

impl Default for Autowire {
    fn default() -> Self {
        Self::No
    }
}

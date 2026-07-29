//! MethodOverride — Spring 风格的方法覆盖 trait 与基础实现。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MethodOverride`。
//!
//! 抽象基类，描述对 Bean 方法行为的覆盖（如 lookup 方法注入、方法替换）。
//! 子类型 `LookupOverride`、`ReplaceOverride` 提供具体语义。

/// 方法覆盖 trait。
///
/// 对应 Spring 的 `MethodOverride`（抽象类）。
///
/// 每个覆盖点至少记录被覆盖的方法名，并声明是否可能存在重载、
/// 以及对该覆盖的合法性校验。
pub trait MethodOverride: Send + Sync + std::fmt::Debug {
    /// 被覆盖的方法名。
    ///
    /// 对应 Spring 的 `getMethodName()`。
    fn get_method_name(&self) -> &str;

    /// 该方法是否可能存在重载（同名不同参）。
    ///
    /// 对应 Spring 的 `isOverloaded()`。
    /// 当方法存在重载时，解析阶段需要额外的参数匹配逻辑。
    fn is_overloaded(&self) -> bool;

    /// 校验覆盖配置是否合法。
    ///
    /// 对应 Spring 的 `validate(Method method)` / `matches(Method method)` 前置检查。
    ///
    /// # 返回
    ///
    /// 合法返回 `Ok(())`，否则返回错误信息。
    fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 是否启用方法匹配（标记位）。
    ///
    /// 对应 Spring 的 `overloaded` 字段的反向使用：默认返回 `true`。
    fn matches(&self) -> bool {
        true
    }
}

/// 基础方法覆盖实现。
///
/// 封装所有覆盖类型共享的字段：方法名、重载标记、来源描述。
/// 子结构体通过组合持有 `MethodOverrideBase` 并转发 trait 方法。
#[derive(Debug, Clone)]
pub struct MethodOverrideBase {
    /// 被覆盖的方法名。
    method_name: String,
    /// 是否可能重载。
    overloaded: bool,
    /// 来源描述（资源/文件名）。
    source_name: Option<String>,
}

impl MethodOverrideBase {
    /// 创建基础方法覆盖。
    pub fn new(method_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
            overloaded: false,
            source_name: None,
        }
    }

    /// 设置重载标记。
    pub fn with_overloaded(mut self, overloaded: bool) -> Self {
        self.overloaded = overloaded;
        self
    }

    /// 设置来源描述。
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// 获取方法名。
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// 是否重载。
    pub fn overloaded(&self) -> bool {
        self.overloaded
    }

    /// 获取来源描述。
    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }
}

/// 通用的、可直接使用的 `MethodOverride` 实现。
///
/// 仅基于方法名与重载标记进行校验，适用于无特殊语义的自定义覆盖。
#[derive(Debug, Clone)]
pub struct GenericMethodOverride {
    base: MethodOverrideBase,
}

impl GenericMethodOverride {
    /// 创建通用方法覆盖。
    pub fn new(method_name: impl Into<String>) -> Self {
        Self {
            base: MethodOverrideBase::new(method_name),
        }
    }

    /// 设置重载标记。
    pub fn with_overloaded(mut self, overloaded: bool) -> Self {
        self.base = self.base.with_overloaded(overloaded);
        self
    }
}

impl MethodOverride for GenericMethodOverride {
    fn get_method_name(&self) -> &str {
        self.base.method_name()
    }

    fn is_overloaded(&self) -> bool {
        self.base.overloaded()
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.base.method_name().is_empty() {
            return Err("Method override name must not be empty".into());
        }
        Ok(())
    }
}

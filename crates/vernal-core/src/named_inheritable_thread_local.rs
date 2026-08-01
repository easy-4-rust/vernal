//! 命名可继承线程局部变量。
//!
//! 对标 Spring `org.springframework.core.NamedInheritableThreadLocal`。

/// 命名可继承线程局部变量。
///
/// 对应 Java: org.springframework.core.NamedInheritableThreadLocal
///
/// Spring 语义：带名称的 `InheritableThreadLocal`（子线程继承值）；
/// Rust 无线程继承语义，此对象保留名称与类型标签（对标 Spring 的诊断用途）。
#[derive(Debug, Clone, Copy)]
pub struct NamedInheritableThreadLocal<T> {
    name: &'static str,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NamedInheritableThreadLocal<T> {
    /// 创建命名可继承线程局部变量。
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: std::marker::PhantomData,
        }
    }

    /// 返回变量名称。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// 返回 `toString` 描述。
    #[must_use]
    pub fn description(&self) -> String {
        format!("NamedInheritableThreadLocal [{}]", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_configured_name() {
        // A 类（合同对齐）：对标 Spring 名称访问
        let local = NamedInheritableThreadLocal::<String>::new("trace-id");
        assert_eq!(local.name(), "trace-id");
    }

    #[test]
    fn description_includes_name() {
        // B 类（边界行为）
        let local = NamedInheritableThreadLocal::<String>::new("trace-id");
        assert_eq!(
            local.description(),
            "NamedInheritableThreadLocal [trace-id]"
        );
    }
}

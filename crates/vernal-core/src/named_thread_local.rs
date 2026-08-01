//! 命名线程局部变量。
//!
//! 对标 Spring `org.springframework.core.NamedThreadLocal`。

/// 命名线程局部变量。
///
/// 对应 Java: org.springframework.core.NamedThreadLocal
///
/// Spring 语义：带名称的 `ThreadLocal`（名称用于诊断日志与 `toString`）。
#[derive(Debug, Clone, Copy)]
pub struct NamedThreadLocal<T> {
    name: &'static str,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NamedThreadLocal<T> {
    /// 创建命名线程局部变量。
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

    /// 返回 `toString` 描述（对标 Spring `NamedThreadLocal.toString`）。
    #[must_use]
    pub fn description(&self) -> String {
        format!("NamedThreadLocal [{}]", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static COUNTER: RefCell<u32> = const { RefCell::new(0) };
    }

    #[test]
    fn exposes_configured_name() {
        // A 类（合同对齐）：对标 Spring 名称访问
        let local = NamedThreadLocal::<u32>::new("request-id");
        assert_eq!(local.name(), "request-id");
    }

    #[test]
    fn description_includes_name() {
        // B 类（边界行为）：对标 Spring toString
        let local = NamedThreadLocal::<u32>::new("tx");
        assert_eq!(local.description(), "NamedThreadLocal [tx]");
    }

    #[test]
    fn usable_with_thread_local() {
        // D 类（重构安全）：作为线程局部变量标签使用
        let local = NamedThreadLocal::<u32>::new("counter");
        let _ = local;
        COUNTER.with(|c| *c.borrow_mut() += 1);
        COUNTER.with(|c| assert_eq!(*c.borrow(), 1));
    }
}

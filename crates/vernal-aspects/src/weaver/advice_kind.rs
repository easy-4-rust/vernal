//! 对标 AspectJ advice 类型枚举。
//!
//! 定义切面织入的 4 种 advice 类型。

/// Advice 类型枚举。
///
/// 对标 AspectJ 的 4 种 advice 类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdviceKind {
    /// 前置 advice：在目标方法执行前执行。
    Before,
    /// 后置 advice：在目标方法成功执行后执行。
    After,
    /// 环绕 advice：包装整个目标方法执行。
    Around,
    /// 异常 advice：在目标方法抛出异常时执行。
    AfterError,
}

impl AdviceKind {
    /// 返回 advice 类型的名称。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Before => "Before",
            Self::After => "After",
            Self::Around => "Around",
            Self::AfterError => "AfterError",
        }
    }

    /// 返回 advice 类型的中文描述。
    pub fn as_chinese(&self) -> &'static str {
        match self {
            Self::Before => "前置",
            Self::After => "后置",
            Self::Around => "环绕",
            Self::AfterError => "异常",
        }
    }
}

impl std::fmt::Display for AdviceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advice_kind_variants() {
        assert_eq!(AdviceKind::Before.as_str(), "Before");
        assert_eq!(AdviceKind::After.as_str(), "After");
        assert_eq!(AdviceKind::Around.as_str(), "Around");
        assert_eq!(AdviceKind::AfterError.as_str(), "AfterError");
    }

    #[test]
    fn test_advice_kind_chinese() {
        assert_eq!(AdviceKind::Before.as_chinese(), "前置");
        assert_eq!(AdviceKind::After.as_chinese(), "后置");
        assert_eq!(AdviceKind::Around.as_chinese(), "环绕");
        assert_eq!(AdviceKind::AfterError.as_chinese(), "异常");
    }

    #[test]
    fn test_advice_kind_display() {
        assert_eq!(format!("{}", AdviceKind::Before), "Before");
    }

    #[test]
    fn test_advice_kind_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AdviceKind>();
        assert_sync::<AdviceKind>();
    }

    #[test]
    fn test_advice_kind_all_variants() {
        let variants = [
            AdviceKind::Before,
            AdviceKind::After,
            AdviceKind::Around,
            AdviceKind::AfterError,
        ];
        assert_eq!(variants.len(), 4);
    }

    #[test]
    fn test_advice_kind_debug() {
        assert_eq!(format!("{:?}", AdviceKind::Before), "Before");
        assert_eq!(format!("{:?}", AdviceKind::After), "After");
        assert_eq!(format!("{:?}", AdviceKind::Around), "Around");
        assert_eq!(format!("{:?}", AdviceKind::AfterError), "AfterError");
    }
}

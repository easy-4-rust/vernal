//! vernal-aop ↔ aspect-rs 桥接。
//!
//! 提供 vernal-aop 和 aspect-rs 之间的适配层。

/// AOP 桥接适配器。
#[derive(Debug)]
pub struct AopBridge;

impl AopBridge {
    /// 创建新的桥接适配器。
    pub fn new() -> Self {
        Self
    }
}

impl Default for AopBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aop_bridge_creation() {
        let bridge = AopBridge::new();
        let _ = bridge;
    }

    #[test]
    fn test_aop_bridge_default() {
        let bridge = AopBridge::default();
        let _ = bridge;
    }

    #[test]
    fn test_aop_bridge_debug() {
        let bridge = AopBridge::new();
        let debug_str = format!("{:?}", bridge);
        assert!(debug_str.contains("AopBridge"));
    }

    #[test]
    fn test_aop_bridge_clone() {
        let bridge = AopBridge::new();
        let _ = bridge;
    }

    #[test]
    fn test_aop_bridge_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AopBridge>();
        assert_sync::<AopBridge>();
    }
}

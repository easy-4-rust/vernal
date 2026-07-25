//! Option 构造注入测试组件对象。

use std::sync::Arc;

use super::{OptionalDerivedPort, OptionalNativeClient};

/// 使用 Rust 原生 Option 类型表达零或一个立即依赖的派生组件。
#[derive(vernal_macros::Component)]
pub(crate) struct OptionalDerivedService {
    client: Option<Arc<OptionalNativeClient>>,
    port: Option<Arc<dyn OptionalDerivedPort>>,
    missing_number: Option<Arc<u64>>,
    #[component(qualifier = "blue")]
    named_number: Option<Arc<u32>>,
    #[component(qualifier = "missing")]
    missing_named_port: Option<Arc<dyn OptionalDerivedPort>>,
}

impl OptionalDerivedService {
    /// 返回可选具体客户端。
    pub(crate) const fn client(&self) -> Option<&Arc<OptionalNativeClient>> {
        self.client.as_ref()
    }

    /// 返回可选 Trait 端口。
    pub(crate) const fn port(&self) -> Option<&Arc<dyn OptionalDerivedPort>> {
        self.port.as_ref()
    }

    /// 返回未安装的具体类型依赖。
    pub(crate) const fn missing_number(&self) -> Option<&Arc<u64>> {
        self.missing_number.as_ref()
    }

    /// 返回命名数值依赖。
    pub(crate) const fn named_number(&self) -> Option<&Arc<u32>> {
        self.named_number.as_ref()
    }

    /// 返回未安装的命名 Trait 端口。
    pub(crate) const fn missing_named_port(&self) -> Option<&Arc<dyn OptionalDerivedPort>> {
        self.missing_named_port.as_ref()
    }
}

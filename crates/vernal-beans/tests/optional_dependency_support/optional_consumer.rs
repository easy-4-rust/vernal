//! 立即可选依赖测试消费组件对象。

use std::sync::Arc;

use vernal_core::BoxError;
use vernal_beans::{ComponentDefinition, Qualifier};

use super::{OptionalPort, OptionalService};

/// 同时消费具体类型、Trait Object 与命名候选的可选组件。
pub(crate) struct OptionalConsumer {
    service: Option<Arc<OptionalService>>,
    port: Option<Arc<dyn OptionalPort>>,
    named_number: Option<Arc<u32>>,
    named_port: Option<Arc<dyn OptionalPort>>,
}

impl OptionalConsumer {
    /// 生成与工厂访问完全一致的可选依赖元数据。
    pub(crate) fn definition() -> ComponentDefinition {
        let blue = Qualifier::new("blue").expect("测试 qualifier 应合法");
        let factory_blue = blue.clone();
        let named = Qualifier::new("named").expect("测试 qualifier 应合法");
        let factory_named = named.clone();

        ComponentDefinition::try_singleton::<Self, _>(move |resolver| -> Result<Self, BoxError> {
            Ok(Self {
                service: resolver.resolve_optional::<OptionalService>()?,
                port: resolver.resolve_optional_trait::<dyn OptionalPort>()?,
                named_number: resolver.resolve_optional_qualified::<u32>(&factory_blue)?,
                named_port: resolver
                    .resolve_optional_qualified_trait::<dyn OptionalPort>(&factory_named)?,
            })
        })
        .depends_on_optional::<OptionalService>()
        .depends_on_optional_trait::<dyn OptionalPort>()
        .depends_on_optional_qualified::<u32>(blue)
        .depends_on_optional_qualified_trait::<dyn OptionalPort>(named)
    }

    /// 返回可选具体服务。
    pub(crate) const fn service(&self) -> Option<&Arc<OptionalService>> {
        self.service.as_ref()
    }

    /// 返回可选 Trait 端口。
    pub(crate) const fn port(&self) -> Option<&Arc<dyn OptionalPort>> {
        self.port.as_ref()
    }

    /// 返回可选命名数值组件。
    pub(crate) const fn named_number(&self) -> Option<&Arc<u32>> {
        self.named_number.as_ref()
    }

    /// 返回可选命名 Trait 端口。
    pub(crate) const fn named_port(&self) -> Option<&Arc<dyn OptionalPort>> {
        self.named_port.as_ref()
    }
}

//! ResourceLoaderAware — Spring 风格的 ResourceLoader 感知接口。
//!
//! 对应 Java 类：`org.springframework.context.ResourceLoaderAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_resource_loader`，
//! 将当前 `ResourceLoader` 注入 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 ResourceLoader 感知接口。
///
/// 对应 Spring 的 `ResourceLoaderAware.setResourceLoader(ResourceLoader resourceLoader)`。
///
/// 容器在实例化 Bean 后，调用此方法将当前 ResourceLoader 注入。
///
/// 注意：由于 `dyn ResourceLoader` 不是 dyn-compatible 的（泛型方法），
/// 此接口使用 `Arc<dyn Any>` 传递引用。实现方需要 downcast 到具体类型。
pub trait ResourceLoaderAware: Aware {
    /// 将当前 ResourceLoader 注入 Bean。
    ///
    /// 对应 Spring 的 `ResourceLoaderAware.setResourceLoader(ResourceLoader resourceLoader)`。
    ///
    /// `loader` 是一个类型擦除的资源加载器引用。实现方可以通过
    /// `Arc::downcast_ref::<ConcreteResourceLoader>()` 获取具体类型。
    fn set_resource_loader(&mut self, loader: Arc<dyn Any + Send + Sync>);
}

//! BeanClassLoaderAware — Spring 风格的 Bean ClassLoader 感知接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.BeanClassLoaderAware`。
//!
//! Bean 实现此接口后，容器在创建 Bean 时回调 `set_bean_class_loader`，
//! 将用于加载 Bean 类的 ClassLoader 注入 Bean。

use std::any::Any;
use std::sync::Arc;

use crate::aware::Aware;

/// Spring 风格的 Bean ClassLoader 感知接口。
///
/// 对应 Spring 的 `BeanClassLoaderAware.setBeanClassLoader(ClassLoader classLoader)`。
///
/// 容器在实例化 Bean 后、初始化阶段之前，调用此方法将当前 ClassLoader 注入。
/// 实现方可以使用此 ClassLoader 动态加载类或资源。
///
/// 注意：Rust 没有 Java 的 ClassLoader 概念，此接口使用 `Arc<dyn Any>`
/// 传递抽象的类加载器引用。具体实现可以在 `vernal-context` 或
/// 自定义环境中定义实际的类加载机制。
pub trait BeanClassLoaderAware: Aware {
    /// 将 Bean ClassLoader 注入 Bean。
    ///
    /// 对应 Spring 的 `BeanClassLoaderAware.setBeanClassLoader(ClassLoader classLoader)`。
    ///
    /// `class_loader` 是一个类型擦除的类加载器引用。实现方可以通过
    /// `Arc::downcast_ref::<ConcreteClassLoader>()` 获取具体类型。
    fn set_bean_class_loader(&mut self, class_loader: Arc<dyn Any + Send + Sync>);
}

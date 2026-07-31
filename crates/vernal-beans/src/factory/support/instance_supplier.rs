//! InstanceSupplier — Spring 风格的实例供应器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.InstanceSupplier`。
//!
//! 在 Spring 中，`InstanceSupplier` 是一个函数式接口，用于提供 Bean 实例。
//! 它替代了传统的反射构造，允许使用 lambda 或方法引用来创建 Bean。
//! 该接口在 Spring 6.x 的 AOT（Ahead-of-Time）处理中被广泛使用。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`InstanceSupplier` 使用闭包 trait 实现，
//! 与 Rust 的工厂模式自然契合。

use std::any::Any;
use std::sync::Arc;

/// 实例供应器接口。
///
/// 对应 Spring 的 `InstanceSupplier<T>`。
///
/// 提供 Bean 实例的抽象，支持延迟创建和依赖注入。
///
/// # 类型参数
///
/// 由于 Rust 的 trait 对象限制，使用 `Arc<dyn Any + Send + Sync>` 作为通用返回类型。
pub trait InstanceSupplier: Send + Sync {
    /// 获取 Bean 实例。
    ///
    /// 对应 Spring 的 `T get(BeanFactory, String, RootBeanDefinition)`。
    ///
    /// # 参数
    /// - `bean_name` — Bean 名称（用于日志和错误报告）
    ///
    /// # 返回
    /// Bean 实例，或错误。
    fn get(
        &self,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取实例的暴露类型。
    ///
    /// 对应 Spring 中 `InstanceSupplier` 可声明的暴露类型，
    /// 用于类型检查和自动装配匹配。
    fn exposed_type_name(&self) -> Option<&str> {
        None
    }
}

/// 基于闭包的实例供应器。
///
/// 最简单的 `InstanceSupplier` 实现，包装一个工厂闭包。
pub struct ClosureInstanceSupplier {
    factory: Box<dyn Fn(&str) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
    exposed_type: Option<String>,
}

impl ClosureInstanceSupplier {
    /// 创建闭包实例供应器。
    ///
    /// # 参数
    /// - `factory` — 工厂闭包，接收 Bean 名称，返回实例
    pub fn new(
        factory: impl Fn(&str) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            factory: Box::new(factory),
            exposed_type: None,
        }
    }

    /// 设置暴露类型名。
    pub fn with_exposed_type(mut self, type_name: impl Into<String>) -> Self {
        self.exposed_type = Some(type_name.into());
        self
    }
}

impl InstanceSupplier for ClosureInstanceSupplier {
    fn get(
        &self,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        (self.factory)(bean_name)
    }

    fn exposed_type_name(&self) -> Option<&str> {
        self.exposed_type.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_supplier_returns_instance() {
        let supplier = ClosureInstanceSupplier::new(|_name| Ok(Arc::new(42_i32)));
        let result = supplier.get("testBean").unwrap();
        assert_eq!(result.downcast_ref::<i32>(), Some(&42));
    }

    #[test]
    fn closure_supplier_with_exposed_type() {
        let supplier = ClosureInstanceSupplier::new(|_| Ok(Arc::new("hello")))
            .with_exposed_type("java.lang.String");
        assert_eq!(supplier.exposed_type_name(), Some("java.lang.String"));
    }

    #[test]
    fn closure_supplier_error_propagation() {
        let supplier = ClosureInstanceSupplier::new(|name| {
            Err(format!("Bean '{}' not found", name).into())
        });
        let result = supplier.get("missingBean");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missingBean"));
    }

    #[test]
    fn default_exposed_type_is_none() {
        let supplier = ClosureInstanceSupplier::new(|_| Ok(Arc::new(1_u8)));
        assert!(supplier.exposed_type_name().is_none());
    }
}

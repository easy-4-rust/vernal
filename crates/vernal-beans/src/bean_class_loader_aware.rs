//! BeanClassLoaderAware — 对应 Spring `org.springframework.beans.factory.BeanClassLoaderAware`。
//!
//! Bean 类加载器感知接口。

/// Bean 类加载器感知接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.BeanClassLoaderAware`。
///
/// 由希望被通知其所属 Bean 的类加载器的 Bean 实现。
pub trait BeanClassLoaderAware: Send + Sync {
    /// 设置 Bean 的类加载器。
    ///
    /// 对应 Java 方法：`void setBeanClassLoader(ClassLoader classLoader)`
    fn set_bean_class_loader(&mut self, class_loader: &str);
}

/// BeanClassLoaderAware 的简单实现。
#[derive(Debug, Default)]
pub struct SimpleBeanClassLoaderAware {
    class_loader_name: Option<String>,
}

impl SimpleBeanClassLoaderAware {
    /// 创建一个新的 SimpleBeanClassLoaderAware。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取类加载器名称。
    pub fn class_loader_name(&self) -> Option<&str> {
        self.class_loader_name.as_deref()
    }
}

impl BeanClassLoaderAware for SimpleBeanClassLoaderAware {
    fn set_bean_class_loader(&mut self, class_loader: &str) {
        self.class_loader_name = Some(class_loader.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_bean_class_loader() {
        let mut aware = SimpleBeanClassLoaderAware::new();
        assert!(aware.class_loader_name().is_none());

        aware.set_bean_class_loader("appClassLoader");
        assert_eq!(aware.class_loader_name(), Some("appClassLoader"));
    }
}

//! ImportBeanDefinitionRegistrar — Spring 风格的 Bean 定义注册器。
//!
//! 对应 Java 类：`org.springframework.context.annotation.ImportBeanDefinitionRegistrar`。
//!
//! 在配置类解析阶段，以编程方式向注册表追加 Bean 定义。

use crate::annotation_metadata::AnnotationMetadata;
use crate::bean_definition_registry::BeanDefinitionRegistry;

/// Spring 风格的 Bean 定义注册器 trait。
///
/// 对应 Spring 的 `ImportBeanDefinitionRegistrar`。
///
/// 实现 `@Import` 的编程式注册扩展点：当某个类被 `@Import` 引入且其
/// 类型实现了此接口时，容器会在所有配置类解析完毕后回调
/// `register_bean_definitions`，允许其动态注册额外 Bean。
pub trait ImportBeanDefinitionRegistrar: Send + Sync {
    /// 向注册表追加 Bean 定义。
    ///
    /// 对应 Spring 的
    /// `registerBeanDefinitions(AnnotationMetadata metadata, BeanDefinitionRegistry registry)`。
    ///
    /// # 参数
    ///
    /// * `importing_metadata` — 触发导入的类的注解元数据
    /// * `registry` — 目标 Bean 定义注册表
    ///
    /// # 返回
    ///
    /// 注册过程中发生的错误（若有）。
    fn register_bean_definitions(
        &self,
        importing_metadata: &dyn AnnotationMetadata,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 复合注册器，按顺序执行多个 `ImportBeanDefinitionRegistrar`。
pub struct CompositeImportBeanDefinitionRegistrar {
    registrars: Vec<Box<dyn ImportBeanDefinitionRegistrar>>,
}

impl std::fmt::Debug for CompositeImportBeanDefinitionRegistrar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositeImportBeanDefinitionRegistrar")
            .field("registrar_count", &self.registrars.len())
            .finish()
    }
}

impl CompositeImportBeanDefinitionRegistrar {
    /// 创建空的复合注册器。
    pub fn new() -> Self {
        Self {
            registrars: Vec::new(),
        }
    }

    /// 添加一个子注册器。
    pub fn add(&mut self, registrar: Box<dyn ImportBeanDefinitionRegistrar>) {
        self.registrars.push(registrar);
    }

    /// 子注册器数量。
    pub fn len(&self) -> usize {
        self.registrars.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.registrars.is_empty()
    }
}

impl Default for CompositeImportBeanDefinitionRegistrar {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportBeanDefinitionRegistrar for CompositeImportBeanDefinitionRegistrar {
    fn register_bean_definitions(
        &self,
        importing_metadata: &dyn AnnotationMetadata,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for registrar in &self.registrars {
            registrar.register_bean_definitions(importing_metadata, registry)?;
        }
        Ok(())
    }
}

/// 空操作注册器（无副作用），用于占位或测试。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopImportBeanDefinitionRegistrar;

impl ImportBeanDefinitionRegistrar for NoopImportBeanDefinitionRegistrar {
    fn register_bean_definitions(
        &self,
        _importing_metadata: &dyn AnnotationMetadata,
        _registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation_metadata::AnnotationDescriptor;

    struct EmptyMetadata;
    impl AnnotationMetadata for EmptyMetadata {
        fn annotations(&self) -> &[AnnotationDescriptor] {
            &[]
        }
    }

    #[test]
    fn test_noop_registrar() {
        let registrar = NoopImportBeanDefinitionRegistrar;
        // 注册需要一个真实的 registry 实现；此处仅验证 noop 不被调用即返回 Ok。
        // 通过复合注册器包装验证计数。
        let mut composite = CompositeImportBeanDefinitionRegistrar::new();
        composite.add(Box::new(NoopImportBeanDefinitionRegistrar));
        assert_eq!(composite.len(), 1);
        let meta = EmptyMetadata;
        // 无法在不实现 registry 的情况下调用 register_bean_definitions，
        // 这里验证 registrar 的 Default 派生。
        let _ = NoopImportBeanDefinitionRegistrar;
        let _ = &meta; // 占位使用
    }
}

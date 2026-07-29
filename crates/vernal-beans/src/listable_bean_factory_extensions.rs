//! ListableBeanFactoryExtensions — `ListableBeanFactory` 的扩展方法。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.DefaultListableBeanFactory`
//! 中的 `findAnnotationOnBean` / `getBeanNamesForAnnotation` 等方法。
//!
//! 由于 Rust 没有运行时注解反射，这里采用显式的注解注册映射。

use std::collections::HashMap;

use crate::listable_bean_factory::ListableBeanFactory;

/// Bean 名称到其注解类型名集合的映射。
///
/// 用于在无反射环境下显式声明每个 Bean 携带的注解。
pub type AnnotationRegistry = HashMap<String, Vec<String>>;

/// `ListableBeanFactory` 的扩展方法集合。
///
/// 对应 Spring 中 `ListableBeanFactory` 的注解相关查询能力。
///
/// 由于 Rust 没有运行时注解反射，本扩展依赖一个显式构造的
/// `AnnotationRegistry`，将 Bean 名称映射到其携带的注解类型名列表。
pub struct ListableBeanFactoryExtensions<'a> {
    bean_factory: &'a dyn ListableBeanFactory,
    annotations: &'a AnnotationRegistry,
}

impl<'a> std::fmt::Debug for ListableBeanFactoryExtensions<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListableBeanFactoryExtensions")
            .field("annotation_entries", &self.annotations.len())
            .finish()
    }
}

impl<'a> ListableBeanFactoryExtensions<'a> {
    /// 创建扩展方法集合。
    pub fn new(
        bean_factory: &'a dyn ListableBeanFactory,
        annotations: &'a AnnotationRegistry,
    ) -> Self {
        Self {
            bean_factory,
            annotations,
        }
    }

    /// 返回携带指定注解的所有 Bean 名称。
    ///
    /// 对应 Spring 的 `getBeanNamesForAnnotation(Class<? extends Annotation>)`。
    pub fn get_bean_names_for_annotation(&self, annotation_type: &str) -> Vec<String> {
        self.bean_factory
            .bean_definition_names()
            .into_iter()
            .filter(|name| self.bean_has_annotation(name, annotation_type))
            .collect()
    }

    /// 返回携带指定注解的所有 Bean 名称（含类型过滤的可选变体）。
    ///
    /// 仅返回同时存在于 `bean_factory` 的 Bean 定义集合中的名称。
    pub fn get_bean_names_for_annotation_filtered(&self, annotation_type: &str) -> Vec<String> {
        self.bean_factory
            .bean_definition_names()
            .into_iter()
            .filter(|name| {
                self.bean_factory.contains_bean_definition(name)
                    && self.bean_has_annotation(name, annotation_type)
            })
            .collect()
    }

    /// 查询指定 Bean 上是否存在给定注解。
    ///
    /// 对应 Spring 的 `findAnnotationOnBean(String, Class<? extends Annotation>)`。
    pub fn find_annotation_on_bean(&self, bean_name: &str, annotation_type: &str) -> bool {
        self.bean_has_annotation(bean_name, annotation_type)
    }

    /// 获取指定 Bean 携带的所有注解类型名。
    pub fn get_annotations_on_bean(&self, bean_name: &str) -> Vec<String> {
        self.annotations.get(bean_name).cloned().unwrap_or_default()
    }

    /// 统计携带指定注解的 Bean 数量。
    pub fn count_beans_with_annotation(&self, annotation_type: &str) -> usize {
        self.get_bean_names_for_annotation(annotation_type).len()
    }

    fn bean_has_annotation(&self, bean_name: &str, annotation_type: &str) -> bool {
        self.annotations
            .get(bean_name)
            .is_some_and(|list| list.iter().any(|a| a == annotation_type))
    }
}

/// 为注解注册表提供的构建器便利方法。
pub trait AnnotationRegistryExt {
    /// 为指定 Bean 注册一个注解。
    fn register(&mut self, bean_name: impl Into<String>, annotation_type: impl Into<String>);
}

impl AnnotationRegistryExt for AnnotationRegistry {
    fn register(&mut self, bean_name: impl Into<String>, annotation_type: impl Into<String>) {
        let bean_name = bean_name.into();
        let annotation_type = annotation_type.into();
        self.entry(bean_name).or_default().push(annotation_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};
    use std::collections::HashMap as StdMap;
    use std::sync::Arc;

    use crate::bean_factory::BeanFactory;
    use crate::component_key::ComponentKey;
    use crate::object_provider::ObjectProvider;

    struct StubListable {
        names: Vec<String>,
    }

    impl BeanFactory for StubListable {
        fn get_bean_by_key(
            &self,
            _key: &ComponentKey,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Err("not supported".into())
        }
        fn get_bean_by_type_id(
            &self,
            _type_id: TypeId,
        ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            Err("not supported".into())
        }
        fn contains_bean(&self, _key: &ComponentKey) -> bool {
            false
        }
        fn is_singleton(
            &self,
            _key: &ComponentKey,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(true)
        }
        fn is_prototype(
            &self,
            _key: &ComponentKey,
        ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            Ok(false)
        }
        fn get_type(
            &self,
            _key: &ComponentKey,
        ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
        }
        fn get_aliases(&self, _key: &ComponentKey) -> Vec<ComponentKey> {
            Vec::new()
        }
        fn get_bean_provider_by_type_id(
            &self,
            _type_id: TypeId,
        ) -> Result<
            Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Err("not supported".into())
        }
        fn is_type_match(&self, _key: &ComponentKey, _type_id: TypeId) -> bool {
            false
        }
    }

    impl ListableBeanFactory for StubListable {
        fn contains_bean_definition(&self, bean_name: &str) -> bool {
            self.names.iter().any(|n| n == bean_name)
        }
        fn bean_definition_count(&self) -> usize {
            self.names.len()
        }
        fn bean_definition_names(&self) -> Vec<String> {
            self.names.clone()
        }
        fn bean_names_for_type_id(
            &self,
            _type_id: TypeId,
            _include_non_singletons: bool,
            _allow_eager_init: bool,
        ) -> Vec<String> {
            Vec::new()
        }
        fn beans_of_type_id(
            &self,
            _type_id: TypeId,
            _include_non_singletons: bool,
            _allow_eager_init: bool,
        ) -> Result<
            StdMap<String, Arc<dyn Any + Send + Sync>>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            Ok(StdMap::new())
        }
        fn bean_post_processor_count(&self) -> usize {
            0
        }
        fn contains_non_singleton_bean(&self) -> bool {
            false
        }
        fn contains_singleton_bean(&self) -> bool {
            true
        }
        fn bean_names_iterator(&self) -> Box<dyn Iterator<Item = String> + '_> {
            Box::new(self.names.clone().into_iter())
        }
    }

    #[test]
    fn test_find_and_list_annotations() {
        let factory = StubListable {
            names: vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
        };
        let mut annotations: AnnotationRegistry = AnnotationRegistry::new();
        AnnotationRegistryExt::register(&mut annotations, "foo", "Component");
        AnnotationRegistryExt::register(&mut annotations, "bar", "Component");
        AnnotationRegistryExt::register(&mut annotations, "bar", "Primary");

        let ext = ListableBeanFactoryExtensions::new(&factory, &annotations);
        assert!(ext.find_annotation_on_bean("foo", "Component"));
        assert!(!ext.find_annotation_on_bean("baz", "Component"));
        let names = ext.get_bean_names_for_annotation("Component");
        assert_eq!(names, vec!["foo".to_owned(), "bar".to_owned()]);
        assert_eq!(ext.count_beans_with_annotation("Component"), 2);
        assert_eq!(
            ext.get_annotations_on_bean("bar"),
            vec!["Component".to_owned(), "Primary".to_owned()]
        );
    }
}

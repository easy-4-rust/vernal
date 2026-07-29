use crate::{annotation_metadata::AnnotationDescriptor, bean_factory::BeanFactory};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
pub trait BeanAnnotationRegistry: Send + Sync {
    fn annotations_for_bean(&self, bean_name: &str) -> Vec<AnnotationDescriptor>;
}
#[derive(Default)]
pub struct SimpleBeanAnnotationRegistry {
    annotations: RwLock<HashMap<String, Vec<AnnotationDescriptor>>>,
}
impl SimpleBeanAnnotationRegistry {
    pub fn register(&self, bean: impl Into<String>, annotation: AnnotationDescriptor) {
        self.annotations
            .write()
            .expect("annotation lock poisoned")
            .entry(bean.into())
            .or_default()
            .push(annotation);
    }
}
impl BeanAnnotationRegistry for SimpleBeanAnnotationRegistry {
    fn annotations_for_bean(&self, name: &str) -> Vec<AnnotationDescriptor> {
        self.annotations
            .read()
            .expect("annotation lock poisoned")
            .get(name)
            .cloned()
            .unwrap_or_default()
    }
}
pub trait BeanFactoryExtensions: BeanFactory {
    fn find_annotation_on_bean(
        &self,
        registry: &dyn BeanAnnotationRegistry,
        bean_name: &str,
        annotation_type: &str,
    ) -> Option<AnnotationDescriptor> {
        registry
            .annotations_for_bean(bean_name)
            .into_iter()
            .find(|a| a.annotation_type() == annotation_type)
    }
    fn get_bean_names_for_annotation(
        &self,
        registry: &dyn BeanAnnotationRegistry,
        bean_names: &[String],
        annotation_type: &str,
    ) -> Vec<String> {
        let mut seen = HashSet::new();
        bean_names
            .iter()
            .filter(|n| {
                self.find_annotation_on_bean(registry, n, annotation_type)
                    .is_some()
                    && seen.insert((*n).clone())
            })
            .cloned()
            .collect()
    }
}
impl<T: BeanFactory + ?Sized> BeanFactoryExtensions for T {}

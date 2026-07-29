use crate::{
    abstract_application_context::AbstractApplicationContext,
    application_context::ApplicationContext, application_event::ApplicationEvent,
    application_event_publisher::ApplicationEventPublisher, bean_factory::BeanFactory,
    bean_factory_post_processor::BeanFactoryPostProcessor, component_key::ComponentKey,
    configurable_application_context::ConfigurableApplicationContext,
    default_resource_loader::ResourceLoader, environment::Environment,
    object_provider::ObjectProvider, property_resolver::PropertyResolver, resource::Resource,
    static_listable_bean_factory::StaticListableBeanFactory,
};
use std::any::{Any, TypeId};
use std::sync::Arc;
struct ContextObjectProvider {
    value: Option<Arc<dyn Any + Send + Sync>>,
}
impl ObjectProvider<dyn Any + Send + Sync> for ContextObjectProvider {
    fn get(
        &self,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.value.clone().ok_or_else(|| "no matching bean".into())
    }
    fn if_available(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.value.clone()
    }
    fn get_if_unique(
        &self,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        match &self.value {
            Some(v) => Ok(v.clone()),
            None => Err("no matching bean".into()),
        }
    }
    fn stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.value.clone().into_iter().collect()
    }
    fn ordered_stream(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.value.clone().into_iter().collect()
    }
}
pub struct GenericApplicationContext {
    base: AbstractApplicationContext,
    beans: StaticListableBeanFactory,
}
impl GenericApplicationContext {
    pub fn new() -> Self {
        Self::with_id("genericApplicationContext")
    }
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            base: AbstractApplicationContext::new(id),
            beans: StaticListableBeanFactory::new(),
        }
    }
    pub fn register_bean<T: Any + Send + Sync>(&mut self, name: impl Into<String>, bean: T) {
        self.beans.register_singleton(name, bean);
    }
    pub fn bean_names(&self) -> Vec<String> {
        self.beans.bean_names()
    }
    pub fn base(&self) -> &AbstractApplicationContext {
        &self.base
    }
    pub fn base_mut(&mut self) -> &mut AbstractApplicationContext {
        &mut self.base
    }
}
impl Default for GenericApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}
impl BeanFactory for GenericApplicationContext {
    fn get_bean_by_key(
        &self,
        k: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.get_bean_by_type_id(k.type_id)
    }
    fn get_bean_by_type_id(
        &self,
        t: TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let names = self.beans.bean_names_for_type(t);
        if names.len() != 1 {
            return Err(format!("expected one bean for type, found {}", names.len()).into());
        }
        self.beans.get_bean(&names[0])
    }
    fn contains_bean(&self, k: &ComponentKey) -> bool {
        !self.beans.bean_names_for_type(k.type_id).is_empty()
    }
    fn is_singleton(
        &self,
        k: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.contains_bean(k))
    }
    fn is_prototype(
        &self,
        _: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
    fn get_type(
        &self,
        k: &ComponentKey,
    ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .beans
            .bean_names_for_type(k.type_id)
            .first()
            .and_then(|n| self.beans.get_type(n)))
    }
    fn get_aliases(&self, _: &ComponentKey) -> Vec<ComponentKey> {
        Vec::new()
    }
    fn get_bean_provider_by_type_id(
        &self,
        t: TypeId,
    ) -> Result<
        Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(Box::new(ContextObjectProvider {
            value: self.get_bean_by_type_id(t).ok(),
        }))
    }
    fn is_type_match(&self, k: &ComponentKey, t: TypeId) -> bool {
        k.type_id == t && self.contains_bean(k)
    }
}
impl PropertyResolver for GenericApplicationContext {
    fn contains_property(&self, k: &str) -> bool {
        self.base.contains_property(k)
    }
    fn get_property(&self, k: &str) -> Option<String> {
        self.base.get_property(k)
    }
    fn resolve_placeholders(&self, t: &str) -> String {
        self.base.resolve_placeholders(t)
    }
    fn resolve_required_placeholders(
        &self,
        t: &str,
    ) -> Result<String, crate::property_resolver::UnresolvedPlaceholderError> {
        self.base.resolve_required_placeholders(t)
    }
}
impl Environment for GenericApplicationContext {
    fn get_active_profiles(&self) -> Vec<String> {
        self.base.get_active_profiles()
    }
    fn get_default_profiles(&self) -> Vec<String> {
        self.base.get_default_profiles()
    }
}
impl ResourceLoader for GenericApplicationContext {
    fn get_resource(&self, l: &str) -> Arc<dyn Resource> {
        self.base.get_resource(l)
    }
    fn class_loader_name(&self) -> Option<&str> {
        self.base.class_loader_name()
    }
}
impl ApplicationEventPublisher for GenericApplicationContext {
    fn publish_event(&self, e: &dyn ApplicationEvent) {
        self.base.publish_event(e)
    }
}
impl ApplicationContext for GenericApplicationContext {
    fn id(&self) -> &str {
        self.base.id()
    }
    fn application_name(&self) -> &str {
        self.base.application_name()
    }
    fn display_name(&self) -> &str {
        self.base.display_name()
    }
    fn startup_timestamp(&self) -> u128 {
        self.base.startup_timestamp()
    }
    fn is_active(&self) -> bool {
        self.base.is_active()
    }
}
impl ConfigurableApplicationContext for GenericApplicationContext {
    fn add_bean_factory_post_processor(&mut self, p: Arc<dyn BeanFactoryPostProcessor>) {
        self.base.add_post_processor(p)
    }
    fn remove_bean_factory_post_processor(
        &mut self,
        i: usize,
    ) -> Option<Arc<dyn BeanFactoryPostProcessor>> {
        self.base.remove_post_processor(i)
    }
    fn bean_factory_post_processor_count(&self) -> usize {
        self.base.post_processors().len()
    }
    fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.base.activate();
        Ok(())
    }
    fn close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.beans.clear();
        self.base.deactivate();
        Ok(())
    }
}

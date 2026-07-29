use crate::{
    application_event::ApplicationEvent,
    application_event_multicaster::ApplicationEventMulticaster,
    application_event_publisher::ApplicationEventPublisher,
    bean_factory_post_processor::BeanFactoryPostProcessor,
    default_resource_loader::{DefaultResourceLoader, ResourceLoader},
    environment::{Environment, SimpleEnvironment},
    property_resolver::PropertyResolver,
    resource::Resource,
    simple_application_event_multicaster::SimpleApplicationEventMulticaster,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

pub struct AbstractApplicationContext {
    id: String,
    application_name: String,
    display_name: String,
    startup_timestamp: u128,
    active: AtomicBool,
    environment: SimpleEnvironment,
    resource_loader: DefaultResourceLoader,
    multicaster: Arc<SimpleApplicationEventMulticaster>,
    post_processors: Vec<Arc<dyn BeanFactoryPostProcessor>>,
}
impl AbstractApplicationContext {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            application_name: String::new(),
            display_name: id,
            startup_timestamp: 0,
            active: AtomicBool::new(false),
            environment: SimpleEnvironment::new(),
            resource_loader: DefaultResourceLoader::new(),
            multicaster: Arc::new(SimpleApplicationEventMulticaster::new()),
            post_processors: Vec::new(),
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn application_name(&self) -> &str {
        &self.application_name
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    pub fn set_application_name(&mut self, v: impl Into<String>) {
        self.application_name = v.into();
    }
    pub fn set_display_name(&mut self, v: impl Into<String>) {
        self.display_name = v.into();
    }
    pub fn startup_timestamp(&self) -> u128 {
        self.startup_timestamp
    }
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    pub fn activate(&mut self) {
        self.startup_timestamp = crate::application_event::current_timestamp_millis();
        self.active.store(true, Ordering::Release);
    }
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
    }
    pub fn environment(&self) -> &SimpleEnvironment {
        &self.environment
    }
    pub fn environment_mut(&mut self) -> &mut SimpleEnvironment {
        &mut self.environment
    }
    pub fn resource_loader(&self) -> &DefaultResourceLoader {
        &self.resource_loader
    }
    pub fn resource_loader_mut(&mut self) -> &mut DefaultResourceLoader {
        &mut self.resource_loader
    }
    pub fn multicaster(&self) -> &Arc<SimpleApplicationEventMulticaster> {
        &self.multicaster
    }
    pub fn add_post_processor(&mut self, p: Arc<dyn BeanFactoryPostProcessor>) {
        self.post_processors.push(p);
    }
    pub fn remove_post_processor(&mut self, i: usize) -> Option<Arc<dyn BeanFactoryPostProcessor>> {
        if i < self.post_processors.len() {
            Some(self.post_processors.remove(i))
        } else {
            None
        }
    }
    pub fn post_processors(&self) -> &[Arc<dyn BeanFactoryPostProcessor>] {
        &self.post_processors
    }
}
impl PropertyResolver for AbstractApplicationContext {
    fn contains_property(&self, k: &str) -> bool {
        self.environment.contains_property(k)
    }
    fn get_property(&self, k: &str) -> Option<String> {
        self.environment.get_property(k)
    }
    fn resolve_placeholders(&self, t: &str) -> String {
        self.environment.resolve_placeholders(t)
    }
    fn resolve_required_placeholders(
        &self,
        t: &str,
    ) -> Result<String, crate::property_resolver::UnresolvedPlaceholderError> {
        self.environment.resolve_required_placeholders(t)
    }
}
impl Environment for AbstractApplicationContext {
    fn get_active_profiles(&self) -> Vec<String> {
        self.environment.get_active_profiles()
    }
    fn get_default_profiles(&self) -> Vec<String> {
        self.environment.get_default_profiles()
    }
}
impl ResourceLoader for AbstractApplicationContext {
    fn get_resource(&self, l: &str) -> Arc<dyn Resource> {
        self.resource_loader.get_resource(l)
    }
    fn class_loader_name(&self) -> Option<&str> {
        self.resource_loader.class_loader_name()
    }
}
impl ApplicationEventPublisher for AbstractApplicationContext {
    fn publish_event(&self, e: &dyn ApplicationEvent) {
        self.multicaster.multicast_event(e);
    }
}
impl std::fmt::Debug for AbstractApplicationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbstractApplicationContext")
            .field("id", &self.id)
            .field("active", &self.is_active())
            .finish()
    }
}
pub type SharedApplicationContext = Arc<RwLock<AbstractApplicationContext>>;

use crate::{
    application_event_publisher::ApplicationEventPublisher, bean_factory::BeanFactory,
    default_resource_loader::ResourceLoader, environment::Environment,
};

pub trait ApplicationContext:
    BeanFactory + Environment + ResourceLoader + ApplicationEventPublisher
{
    fn id(&self) -> &str;
    fn application_name(&self) -> &str {
        ""
    }
    fn display_name(&self) -> &str {
        self.id()
    }
    fn startup_timestamp(&self) -> u128;
    fn is_active(&self) -> bool;
}

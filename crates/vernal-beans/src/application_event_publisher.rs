use crate::application_event::ApplicationEvent;

pub trait ApplicationEventPublisher: Send + Sync {
    fn publish_event(&self, event: &dyn ApplicationEvent);
}

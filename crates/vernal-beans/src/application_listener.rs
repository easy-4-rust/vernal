use crate::application_event::ApplicationEvent;

pub trait ApplicationListener: Send + Sync {
    fn on_application_event(&self, event: &dyn ApplicationEvent);
}

impl<F> ApplicationListener for F
where
    F: Fn(&dyn ApplicationEvent) + Send + Sync,
{
    fn on_application_event(&self, event: &dyn ApplicationEvent) {
        self(event);
    }
}

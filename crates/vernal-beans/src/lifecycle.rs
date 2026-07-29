pub trait Lifecycle: Send + Sync {
    fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn is_running(&self) -> bool;
}

pub trait LifecycleProcessor: Lifecycle {
    fn on_refresh(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.start()
    }
    fn on_close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stop()
    }
}

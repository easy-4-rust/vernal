use crate::{default_resource_loader::ResourceLoader, resource::Resource};
use std::sync::Arc;
pub trait ResourceLoaderExtensions: ResourceLoader {
    fn get_resources<'a, I>(&self, locations: I) -> Vec<Arc<dyn Resource>>
    where
        I: IntoIterator<Item = &'a str>,
        Self: Sized,
    {
        locations
            .into_iter()
            .map(|l| self.get_resource(l))
            .collect()
    }
    fn resource_exists(&self, location: &str) -> bool {
        self.get_resource(location).exists()
    }
    fn read_resource_to_string(
        &self,
        location: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut r = self.get_resource(location);
        let r = Arc::get_mut(&mut r).ok_or_else(|| "resource is shared".to_string())?;
        r.read_to_string()
    }
}
impl<T: ResourceLoader + ?Sized> ResourceLoaderExtensions for T {}

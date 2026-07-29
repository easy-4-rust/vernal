use crate::{
    placeholder_resolver::{PlaceholderResolver, UnresolvablePlaceholderError},
    property_sources::PropertySources,
    string_value_resolver::StringValueResolver,
};
pub struct EmbeddedValueResolverImpl<'a> {
    resolver: PlaceholderResolver<'a>,
    required: bool,
}
impl<'a> EmbeddedValueResolverImpl<'a> {
    pub fn new(sources: &'a PropertySources) -> Self {
        Self {
            resolver: PlaceholderResolver::new(sources),
            required: false,
        }
    }
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    pub fn resolve_required(&self, value: &str) -> Result<String, UnresolvablePlaceholderError> {
        self.resolver.resolve_required_placeholders(value)
    }
}
impl StringValueResolver for EmbeddedValueResolverImpl<'_> {
    fn resolve_string_value(&self, value: &str) -> String {
        if self.required {
            self.resolver
                .resolve_required_placeholders(value)
                .unwrap_or_else(|_| value.to_owned())
        } else {
            self.resolver.resolve_placeholders(value)
        }
    }
}

use std::sync::Arc;

#[derive(vernal_macros::Component)]
struct RedundantOptionalAttribute {
    #[component(optional)]
    value: Option<Arc<String>>,
}

fn main() {}

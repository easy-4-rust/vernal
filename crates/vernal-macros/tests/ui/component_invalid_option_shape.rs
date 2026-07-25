use std::sync::Arc;

#[derive(vernal_macros::Component)]
struct InvalidOptionShape {
    value: Option<String>,
    #[component(default)]
    marker: Option<Arc<String>>,
}

fn main() {}

use std::sync::Arc;

#[derive(vernal_macros::Component)]
struct InvalidOptionalComponent {
    #[component(optional)]
    value: Arc<String>,
}

fn main() {}

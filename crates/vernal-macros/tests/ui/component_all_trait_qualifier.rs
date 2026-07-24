use std::sync::Arc;

trait Port: Send + Sync + 'static {}

#[derive(vernal_macros::Component)]
struct InvalidComponent {
    #[component(qualifier = "named")]
    ports: Vec<Arc<dyn Port>>,
}

fn main() {}

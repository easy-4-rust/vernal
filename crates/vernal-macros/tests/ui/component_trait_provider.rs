use vernal_ioc::ComponentProvider;

trait Extension: Send + Sync {}

#[derive(vernal_macros::Component)]
struct InvalidTraitProvider {
    extension: ComponentProvider<dyn Extension>,
}

fn main() {}

use vernal_beans::ComponentProvider;

trait Extension: Send + Sync {}

#[derive(vernal_macros::Component)]
struct InvalidTraitProvider {
    extension: ComponentProvider<dyn Extension>,
}

fn main() {}

use vernal_beans::TraitProvider;

struct ConcreteExtension;

#[derive(vernal_macros::Component)]
struct InvalidConcreteTraitProvider {
    extension: TraitProvider<ConcreteExtension>,
}

fn main() {}

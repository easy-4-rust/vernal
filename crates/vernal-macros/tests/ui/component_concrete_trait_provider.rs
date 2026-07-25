use vernal_ioc::TraitProvider;

struct ConcreteExtension;

#[derive(vernal_macros::Component)]
struct InvalidConcreteTraitProvider {
    extension: TraitProvider<ConcreteExtension>,
}

fn main() {}

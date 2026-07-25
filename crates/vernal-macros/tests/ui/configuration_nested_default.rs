#[derive(vernal_macros::ConfigurationProperties)]
#[configuration(prefix = "service")]
struct NestedDefault {
    #[configuration(nested, default)]
    child: Child,
}

struct Child;

fn main() {}

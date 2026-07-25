#[derive(vernal_macros::ConfigurationProperties)]
#[configuration(prefix = "service")]
struct OptionalDefault {
    #[configuration(default)]
    port: Option<u16>,
}

fn main() {}

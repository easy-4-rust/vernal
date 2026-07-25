#[derive(vernal_macros::ConfigurationProperties)]
#[configuration(prefix = "service")]
struct InvalidRename {
    #[configuration(rename = "nested.port")]
    port: u16,
}

fn main() {}

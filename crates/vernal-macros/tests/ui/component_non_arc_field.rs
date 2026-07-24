//! 未标记 default 的非 Arc 字段必须在编译期被拒绝。

#[derive(vernal_macros::Component)]
struct InvalidComponent {
    value: usize,
}

fn main() {}

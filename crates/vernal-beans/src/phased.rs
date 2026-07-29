//! Phased trait — Spring 风格的阶段接口。
/// 阶段 trait。
pub trait Phased: Send + Sync {
    fn get_phase(&self) -> i32;
}

//! NumberUtils — 数字工具。
/// 数字工具。
pub struct NumberUtils;
impl NumberUtils {
    pub fn parse_number(str: &str) -> Option<f64> { str.parse().ok() }
    pub fn is_numeric(str: &str) -> bool { str.parse::<f64>().is_ok() }
}

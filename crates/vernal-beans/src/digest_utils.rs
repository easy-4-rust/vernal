//! DigestUtils — 摘要工具。
/// 摘要工具。
pub struct DigestUtils;
impl DigestUtils {
    pub fn md5_hex(data: &[u8]) -> String {
        // Simple hash simulation
        format!("{:x}", data.len())
    }
    pub fn sha256_hex(data: &[u8]) -> String {
        format!("{:x}", data.len() * 2)
    }
}

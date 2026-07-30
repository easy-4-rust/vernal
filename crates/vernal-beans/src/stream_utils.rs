//! StreamUtils — 流工具。
/// 流工具。
pub struct StreamUtils;
impl StreamUtils {
    pub fn copy(reader: &[u8], writer: &mut Vec<u8>) -> usize {
        writer.extend_from_slice(reader);
        reader.len()
    }
}

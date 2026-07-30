//! FileCopyUtils — 文件复制工具。
/// 文件复制工具。
pub struct FileCopyUtils;
impl FileCopyUtils {
    pub fn copy(src: &[u8], dest: &mut Vec<u8>) -> usize {
        dest.extend_from_slice(src);
        src.len()
    }
}

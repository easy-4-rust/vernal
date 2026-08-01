//! 资源区域。
//!
//! 对标 Spring `org.springframework.core.io.support.ResourceRegion`。

use crate::io::Resource;

/// 资源区域（资源的字节区间）。
///
/// 对应 Java: org.springframework.core.io.support.ResourceRegion
///
/// Spring 语义：`Resource + position + count`，用于 HTTP Range 请求等
/// 区间读取场景。
pub struct ResourceRegion {
    resource: Box<dyn Resource>,
    position: u64,
    count: u64,
}

impl ResourceRegion {
    /// 创建资源区域。
    #[must_use]
    pub fn new(resource: Box<dyn Resource>, position: u64, count: u64) -> Self {
        Self {
            resource,
            position,
            count,
        }
    }

    /// 返回底层资源。
    #[must_use]
    pub fn resource(&self) -> &dyn Resource {
        self.resource.as_ref()
    }

    /// 返回起始位置。
    #[must_use]
    pub fn position(&self) -> u64 {
        self.position
    }

    /// 返回区间长度。
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// 读取区间内容。
    ///
    /// # 错误
    ///
    /// 读取失败时返回 [`std::io::Error`]。
    pub fn read_region(&self) -> std::io::Result<Vec<u8>> {
        let bytes = self.resource.read_bytes()?;
        let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let start = usize::try_from(self.position.min(length)).unwrap_or(bytes.len());
        let end = usize::try_from((self.position + self.count).min(length)).unwrap_or(bytes.len());
        Ok(bytes[start..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn reads_sub_range() {
        // A 类（合同对齐）：对标 Spring 区间读取
        let resource = ByteArrayResource::new(b"0123456789".to_vec());
        let region = ResourceRegion::new(Box::new(resource), 2, 4);
        assert_eq!(region.read_region().unwrap(), b"2345");
        assert_eq!(region.position(), 2);
        assert_eq!(region.count(), 4);
    }

    #[test]
    fn range_beyond_length_is_clamped() {
        // B 类（边界行为）：区间越界时截断
        let resource = ByteArrayResource::new(b"01234".to_vec());
        let region = ResourceRegion::new(Box::new(resource), 3, 100);
        assert_eq!(region.read_region().unwrap(), b"34");
    }
}

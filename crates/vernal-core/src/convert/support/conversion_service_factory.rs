//! 转换服务工厂。
//!
//! 对标 Spring `org.springframework.core.convert.support.ConversionServiceFactory`。

use super::DefaultConversionService;

/// 转换服务工厂。
///
/// 对应 Java: org.springframework.core.convert.support.ConversionServiceFactory
///
/// Spring 语义：静态工厂，创建带全部默认转换器的转换服务
/// （对标 `ConversionServiceFactory.createDefaultConversionService`）。
pub struct ConversionServiceFactory;

impl ConversionServiceFactory {
    /// 创建带默认转换器的转换服务。
    #[must_use]
    pub fn create_default_conversion_service() -> DefaultConversionService {
        DefaultConversionService::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_default_service() {
        // A 类（合同对齐）：对标 Spring 工厂创建
        let service = ConversionServiceFactory::create_default_conversion_service();
        assert_eq!(service.convert::<i32>("7").unwrap(), 7);
    }

    #[test]
    #[allow(clippy::borrow_as_ptr)] // 测试地址独立性（对标 Spring `new` 语义）
    fn instances_are_independent() {
        // D 类（生命周期）：每次创建独立实例（对标 Spring `new` 语义）
        let a = ConversionServiceFactory::create_default_conversion_service();
        let b = ConversionServiceFactory::create_default_conversion_service();
        let pa = std::ptr::from_ref(&a);
        let pb = std::ptr::from_ref(&b);
        assert_ne!(pa, pb);
    }
}

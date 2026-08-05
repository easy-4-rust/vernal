//! 目标源创建器。
//!
//! 对应 spring-aop `TargetSourceCreator`。
//! 创建目标源实例。

use crate::target_source::TargetSource;
use crate::target_source_error::TargetSourceError;

/// 目标源创建器接口。
///
/// 对应 spring-aop `TargetSourceCreator`。
pub trait TargetSourceCreator: Send + Sync + 'static {
    /// 获取目标源。
    fn get_target_source(
        &self,
        target_class: &str,
    ) -> Result<Box<dyn TargetSource>, TargetSourceError>;
}

/// 基于闭包的目标源创建器。
pub struct FnTargetSourceCreator<F>
where
    F: Fn(&str) -> Result<Box<dyn TargetSource>, TargetSourceError> + Send + Sync + 'static,
{
    factory: F,
}

impl<F> FnTargetSourceCreator<F>
where
    F: Fn(&str) -> Result<Box<dyn TargetSource>, TargetSourceError> + Send + Sync + 'static,
{
    /// 创建基于闭包的目标源创建器。
    pub fn new(factory: F) -> Self {
        Self { factory }
    }
}

impl<F> TargetSourceCreator for FnTargetSourceCreator<F>
where
    F: Fn(&str) -> Result<Box<dyn TargetSource>, TargetSourceError> + Send + Sync + 'static,
{
    fn get_target_source(
        &self,
        target_class: &str,
    ) -> Result<Box<dyn TargetSource>, TargetSourceError> {
        (self.factory)(target_class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_source::LazyTargetSource;

    #[test]
    fn fn_target_source_creator() {
        let creator = FnTargetSourceCreator::new(|_class_name| {
            Ok(Box::new(LazyTargetSource::new(|| Ok(Box::new(42i32)))))
        });

        let target_source = creator.get_target_source("TestClass").unwrap();
        assert!(!target_source.is_static());
    }
}

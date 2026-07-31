//! MethodReplacer — Spring 风格的方法替换器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MethodReplacer`。
//!
//! 在 Spring 中，`MethodReplacer` 用于运行时替换 Bean 的方法实现。
//! 配合 XML 中的 `<replaced-method>` 元素使用。
//!
//! 例如：
//! ```xml
//! <bean id="myBean" class="...">
//!     <replaced-method name="compute" replacer="myReplacer"/>
//! </bean>
//! ```
//!
//! ## 设计说明
//!
//! 在 vernal 中，方法替换通过闭包实现。
//! `MethodReplacer` trait 定义了替换方法的接口。

use std::any::Any;
use std::sync::Arc;

/// 方法替换器接口。
///
/// 对应 Spring 的 `MethodReplacer`。
///
/// 替换 Bean 的指定方法实现。
/// `reimplement` 方法接收原始方法的参数，返回替换后的结果。
pub trait MethodReplacer: Send + Sync {
    /// 重新实现方法。
    ///
    /// 对应 Spring 的 `Object reimplement(Object obj, Method method, Object[] args)`。
    ///
    /// # 参数
    /// - `args` — 原始方法的调用参数
    ///
    /// # 返回
    /// 替换后的返回值。
    fn reimplement(
        &self,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 基于闭包的方法替换器。
///
/// 将闭包包装为 `MethodReplacer`，方便使用。
pub struct ClosureMethodReplacer {
    closure: Box<dyn Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
    description: String,
}

impl ClosureMethodReplacer {
    /// 创建闭包方法替换器。
    ///
    /// # 参数
    /// - `description` — 替换描述（用于日志）
    /// - `closure` — 替换闭包
    pub fn new(
        description: impl Into<String>,
        closure: impl Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            closure: Box::new(closure),
            description: description.into(),
        }
    }

    /// 获取替换描述。
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl MethodReplacer for ClosureMethodReplacer {
    fn reimplement(
        &self,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        (self.closure)(args)
    }
}

/// 返回固定值的方法替换器。
///
/// 忽略所有参数，始终返回固定的替换值。
pub struct FixedValueMethodReplacer {
    value: Arc<dyn Any + Send + Sync>,
}

impl FixedValueMethodReplacer {
    /// 创建固定值方法替换器。
    pub fn new(value: Arc<dyn Any + Send + Sync>) -> Self {
        Self { value }
    }
}

impl MethodReplacer for FixedValueMethodReplacer {
    fn reimplement(
        &self,
        _args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::clone(&self.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_replacer_executes() {
        let replacer = ClosureMethodReplacer::new("double", |args| {
            if let Some(val) = args.first().and_then(|a| a.downcast_ref::<i32>()) {
                Ok(Arc::new(val * 2))
            } else {
                Err("expected i32 argument".into())
            }
        });

        assert_eq!(replacer.description(), "double");
        let result = replacer.reimplement(&[Arc::new(5_i32)]).unwrap();
        assert_eq!(result.downcast_ref::<i32>(), Some(&10));
    }

    #[test]
    fn fixed_value_replacer_returns_same_value() {
        let replacer = FixedValueMethodReplacer::new(Arc::new("hello".to_string()));
        let r1 = replacer.reimplement(&[]).unwrap();
        let r2 = replacer.reimplement(&[Arc::new(42)]).unwrap();
        assert_eq!(r1.downcast_ref::<String>(), Some(&"hello".to_string()));
        assert_eq!(r2.downcast_ref::<String>(), Some(&"hello".to_string()));
    }

    #[test]
    fn closure_replacer_error() {
        let replacer = ClosureMethodReplacer::new("fail", |_args| {
            Err("always fails".into())
        });
        let result = replacer.reimplement(&[]);
        assert!(result.is_err());
    }
}

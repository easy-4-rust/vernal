//! 校验切面。
//!
//! 对应 aspect-rs：aspect-std/src/validation.rs。
//! spring-aop 无直接对应。
//!
//! 移植自 aspect-rs 的 `ValidationRule` trait + `ValidationAspect` + 内置校验器，
//! 改造为 Tokio-first 异步。`ValidationRule::validate` 接收 `&Operation` 替代 `&JoinPoint`。

use std::sync::Arc;

use crate::{Interceptor, Invocation, InvocationError, InvocationFuture, InvocationResult, Next};

/// 校验规则 trait。
///
/// 对应 aspect-rs `ValidationRule`。
/// 实现此 trait 创建自定义校验规则，可组合并应用于函数。
pub trait ValidationRule: Send + Sync {
    /// 校验输入。
    ///
    /// 返回 `Ok(())` 校验通过，`Err(message)` 校验失败。
    fn validate(&self, operation: &crate::Operation) -> Result<(), String>;

    /// 获取此校验规则的描述。
    fn description(&self) -> &str {
        "validation rule"
    }
}

/// 校验切面。
///
/// 允许组合多个校验规则，在函数执行前检查。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::{ValidationAspect, ValidationRule};
///
/// struct AgeValidator;
/// impl ValidationRule for AgeValidator {
///     fn validate(&self, _op: &Operation) -> Result<(), String> {
///         Ok(())
///     }
/// }
///
/// let validator = ValidationAspect::new()
///     .add_rule(Box::new(AgeValidator));
/// ```
pub struct ValidationAspect {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ValidationAspect {
    /// 创建新的校验切面。
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加校验规则。
    #[must_use]
    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// 执行所有校验规则。
    fn validate(&self, operation: &crate::Operation) -> Result<(), InvocationError> {
        for rule in self.rules.iter() {
            if let Err(msg) = rule.validate(operation) {
                return Err(InvocationError::Cancelled);
            }
        }
        Ok(())
    }
}

impl Default for ValidationAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Interceptor for ValidationAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            // 执行前校验
            self.validate(invocation.operation())?;

            // 执行目标函数
            next.run(invocation).await
        })
    }
}

/// 非空校验器。
///
/// 对应 aspect-rs `NotEmptyValidator`。
pub struct NotEmptyValidator {
    field_name: String,
    getter: Arc<dyn Fn(&crate::Operation) -> Option<String> + Send + Sync>,
}

impl NotEmptyValidator {
    /// 创建新的非空校验器。
    ///
    /// # 参数
    /// * `field_name` - 被校验字段的名称
    /// * `getter` - 从 Operation 提取值的函数
    #[must_use]
    pub fn new<F>(field_name: &str, getter: F) -> Self
    where
        F: Fn(&crate::Operation) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            field_name: field_name.to_string(),
            getter: Arc::new(getter),
        }
    }
}

impl ValidationRule for NotEmptyValidator {
    fn validate(&self, operation: &crate::Operation) -> Result<(), String> {
        if let Some(value) = (self.getter)(operation) {
            if value.is_empty() {
                return Err(format!("{} cannot be empty", self.field_name));
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "not empty"
    }
}

/// 范围校验器。
///
/// 对应 aspect-rs `RangeValidator`。
pub struct RangeValidator {
    field_name: String,
    min: i64,
    max: i64,
    getter: Arc<dyn Fn(&crate::Operation) -> Option<i64> + Send + Sync>,
}

impl RangeValidator {
    /// 创建新的范围校验器。
    ///
    /// # 参数
    /// * `field_name` - 被校验字段的名称
    /// * `min` - 最小值（包含）
    /// * `max` - 最大值（包含）
    /// * `getter` - 从 Operation 提取值的函数
    #[must_use]
    pub fn new<F>(field_name: &str, min: i64, max: i64, getter: F) -> Self
    where
        F: Fn(&crate::Operation) -> Option<i64> + Send + Sync + 'static,
    {
        Self {
            field_name: field_name.to_string(),
            min,
            max,
            getter: Arc::new(getter),
        }
    }
}

impl ValidationRule for RangeValidator {
    fn validate(&self, operation: &crate::Operation) -> Result<(), String> {
        if let Some(value) = (self.getter)(operation) {
            if value < self.min || value > self.max {
                return Err(format!(
                    "{} must be between {} and {}, got {}",
                    self.field_name, self.min, self.max, value
                ));
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "range check"
    }
}

/// 自定义校验器（闭包）。
///
/// 对应 aspect-rs `CustomValidator`。
pub struct CustomValidator {
    description: String,
    validator: Arc<dyn Fn(&crate::Operation) -> Result<(), String> + Send + Sync>,
}

impl CustomValidator {
    /// 从闭包创建自定义校验器。
    ///
    /// # 参数
    /// * `description` - 校验描述
    /// * `validator` - 校验函数
    #[must_use]
    pub fn new<F>(description: &str, validator: F) -> Self
    where
        F: Fn(&crate::Operation) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            description: description.to_string(),
            validator: Arc::new(validator),
        }
    }
}

impl ValidationRule for CustomValidator {
    fn validate(&self, operation: &crate::Operation) -> Result<(), String> {
        (self.validator)(operation)
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn validation_aspect_creation() {
        let validator = ValidationAspect::new();
        assert_eq!(validator.rules.len(), 0);
    }

    #[test]
    fn custom_validator_success() {
        let validator = CustomValidator::new("test", |_op| Ok(()));
        let op = Operation::new("test", "test");
        assert!(validator.validate(&op).is_ok());
    }

    #[test]
    fn custom_validator_failure() {
        let validator =
            CustomValidator::new("test", |_op| Err("validation failed".to_string()));
        let op = Operation::new("test", "test");
        assert!(validator.validate(&op).is_err());
    }

    #[test]
    fn not_empty_validator_success() {
        let validator = NotEmptyValidator::new("method", |op| Some(op.method().to_string()));
        let op = Operation::new("Svc", "create");
        assert!(validator.validate(&op).is_ok());
    }

    #[test]
    fn not_empty_validator_failure() {
        let validator = NotEmptyValidator::new("method", |_op| Some("".to_string()));
        let op = Operation::new("Svc", "");
        let result = validator.validate(&op);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn range_validator_success() {
        let validator = RangeValidator::new("id", 1, 100, |_op| Some(42));
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_ok());
    }

    #[test]
    fn range_validator_failure() {
        let validator = RangeValidator::new("id", 1, 100, |_op| Some(200));
        let op = Operation::new("Svc", "m");
        let result = validator.validate(&op);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be between"));
    }

    #[test]
    fn range_validator_boundary() {
        let validator = RangeValidator::new("id", 0, 120, |_op| Some(0));
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_ok());

        let validator = RangeValidator::new("id", 0, 120, |_op| Some(120));
        assert!(validator.validate(&op).is_ok());
    }

    // --- 分支覆盖补充测试 ---

    #[test]
    fn not_empty_validator_getter_returns_none() {
        let validator = NotEmptyValidator::new("method", |_op| None);
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_ok());
    }

    #[test]
    fn range_validator_getter_returns_none() {
        let validator = RangeValidator::new("id", 1, 100, |_op| None);
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_ok());
    }

    #[test]
    fn not_empty_validator_description() {
        let validator = NotEmptyValidator::new("method", |_op| None);
        assert_eq!(validator.description(), "not empty");
    }

    #[test]
    fn range_validator_description() {
        let validator = RangeValidator::new("id", 1, 100, |_op| None);
        assert_eq!(validator.description(), "range check");
    }

    #[test]
    fn custom_validator_description() {
        let validator = CustomValidator::new("my custom rule", |_op| Ok(()));
        assert_eq!(validator.description(), "my custom rule");
    }

    #[test]
    fn validation_aspect_with_multiple_rules() {
        let _validator = ValidationAspect::new()
            .add_rule(Box::new(CustomValidator::new("rule1", |_op| Ok(()))))
            .add_rule(Box::new(CustomValidator::new("rule2", |_op| Ok(()))));
    }

    #[test]
    fn validation_aspect_validate_passes() {
        let validator = ValidationAspect::new()
            .add_rule(Box::new(CustomValidator::new("test", |_op| Ok(()))));
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_ok());
    }

    #[test]
    fn validation_aspect_validate_fails() {
        let validator = ValidationAspect::new()
            .add_rule(Box::new(CustomValidator::new("test", |_op| {
                Err("failed".to_string())
            })));
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_err());
    }

    #[test]
    fn validation_aspect_validate_first_rule_fails_stops() {
        let validator = ValidationAspect::new()
            .add_rule(Box::new(CustomValidator::new("rule1", |_op| {
                Err("first failed".to_string())
            })))
            .add_rule(Box::new(CustomValidator::new("rule2", |_op| Ok(()))));
        let op = Operation::new("Svc", "m");
        assert!(validator.validate(&op).is_err());
    }

    #[test]
    fn validation_aspect_default() {
        let _validator = ValidationAspect::default();
    }
}

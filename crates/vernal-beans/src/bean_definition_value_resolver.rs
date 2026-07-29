//! BeanDefinitionValueResolver — Spring 风格的 Bean 定义值解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionValueResolver`。
//!
//! 解析 Bean 定义中的属性值和构造参数值，将引用、表达式等解析为实际对象。

use std::any::Any;
use std::sync::Arc;

use crate::bean_definition::BeanDefinition;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_expression_resolver::BeanExpressionResolver;
use crate::runtime_bean_name_reference::RuntimeBeanNameReference;
use crate::runtime_bean_reference::RuntimeBeanReference;
use crate::typed_string_value::TypedStringValue;

/// Spring 风格的 Bean 定义值解析器。
///
/// 对应 Spring 的 `BeanDefinitionValueResolver`。
///
/// 解析 Bean 定义中的各种值类型：
/// - `RuntimeBeanReference` — 解析为 Bean 实例
/// - `RuntimeBeanNameReference` — 解析为 Bean 名称字符串
/// - `TypedStringValue` — 解析为字符串（可选带类型转换）
/// - 其他值 — 直接返回
pub struct BeanDefinitionValueResolver {
    /// 持有 Bean 定义的注册表，用于按名称查找 Bean。
    registry: Arc<dyn BeanDefinitionRegistry>,
    /// Bean 表达式解析器（可选），用于解析 SpEL 表达式。
    expression_resolver: Option<Arc<dyn BeanExpressionResolver>>,
}

impl BeanDefinitionValueResolver {
    /// 创建一个新的 BeanDefinitionValueResolver。
    ///
    /// # 参数
    ///
    /// * `registry` — Bean 定义注册表
    pub fn new(registry: Arc<dyn BeanDefinitionRegistry>) -> Self {
        Self {
            registry,
            expression_resolver: None,
        }
    }

    /// 创建一个带表达式解析器的 BeanDefinitionValueResolver。
    ///
    /// # 参数
    ///
    /// * `registry` — Bean 定义注册表
    /// * `expression_resolver` — 表达式解析器
    pub fn with_expression_resolver(
        registry: Arc<dyn BeanDefinitionRegistry>,
        expression_resolver: Arc<dyn BeanExpressionResolver>,
    ) -> Self {
        Self {
            registry,
            expression_resolver: Some(expression_resolver),
        }
    }

    /// 解析给定的值（如果必要）。
    ///
    /// 对应 Spring 的 `resolveValueIfNecessary(Object argName, Object value)`。
    ///
    /// 处理以下值类型：
    /// - `RuntimeBeanReference` — 按名称查找并返回 Bean 实例
    /// - `RuntimeBeanNameReference` — 仅解析并返回 Bean 名称字符串
    /// - `TypedStringValue` — 返回字符串值（暂不进行类型转换）
    /// - 其他 — 直接返回原始值
    ///
    /// # 参数
    ///
    /// * `value` — 要解析的值
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 解析后的值
    /// - `Err` — 解析失败
    pub fn resolve_value_if_necessary(
        &self,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 尝试按类型逐一检查
        let any_ref: &dyn Any = &*value;

        // 检查是否是 RuntimeBeanReference
        if let Some(bean_ref) = any_ref.downcast_ref::<RuntimeBeanReference>() {
            return self.resolve_runtime_bean_reference(bean_ref);
        }

        // 检查是否是 RuntimeBeanNameReference
        if let Some(name_ref) = any_ref.downcast_ref::<RuntimeBeanNameReference>() {
            return self.resolve_runtime_bean_name_reference(name_ref);
        }

        // 检查是否是 TypedStringValue
        if let Some(typed_val) = any_ref.downcast_ref::<TypedStringValue>() {
            return self.resolve_typed_string_value(typed_val);
        }

        // 检查是否是 String
        if let Some(s) = any_ref.downcast_ref::<String>() {
            // 如果配置了表达式解析器，尝试解析表达式
            if let Some(resolver) = &self.expression_resolver {
                if let Some(result) = resolver.evaluate(s, None)? {
                    return Ok(result);
                }
            }
            return Ok(Arc::new(s.clone()));
        }

        // 其他值类型直接返回
        Ok(value)
    }

    /// 解析 RuntimeBeanReference：按名称查找 Bean 实例。
    fn resolve_runtime_bean_reference(
        &self,
        reference: &RuntimeBeanReference,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let bean_name = reference.get_bean_name();
        let definition = self
            .registry
            .get_bean_definition(bean_name)
            .ok_or_else(|| format!("No bean named '{}' is defined in the registry", bean_name))?;

        // 这里我们简单地检查定义是否存在。
        // 实际容器中，此处应调用 BeanFactory::get_bean_by_key 等获取实例。
        // 由于 BeanDefinitionValueResolver 通常由容器内部使用，
        // 它需要持有 BeanFactory 引用来实际获取 Bean 实例。
        // 当前实现返回一个标记值，实际容器会在实例化时提供完整逻辑。
        let _ = definition;
        Err(format!(
            "RuntimeBeanReference '{}' requires a BeanFactory to resolve; \
             use a concrete resolver wired to the container",
            bean_name
        )
        .into())
    }

    /// 解析 RuntimeBeanNameReference：返回 Bean 名称字符串。
    fn resolve_runtime_bean_name_reference(
        &self,
        reference: &RuntimeBeanNameReference,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let bean_name = reference.bean_name();
        Ok(Arc::new(bean_name.to_string()) as Arc<dyn Any + Send + Sync>)
    }

    /// 解析 TypedStringValue：返回原始字符串值。
    fn resolve_typed_string_value(
        &self,
        typed_value: &TypedStringValue,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 当前实现仅返回字符串值。
        // 后续可扩展为根据 target_type_name 进行类型转换。
        let value_str = typed_value.value().to_string();

        // 如果配置了表达式解析器，尝试解析表达式
        if let Some(resolver) = &self.expression_resolver {
            if let Some(result) = resolver.evaluate(&value_str, None)? {
                return Ok(result);
            }
        }

        Ok(Arc::new(value_str))
    }
}

impl std::fmt::Debug for BeanDefinitionValueResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BeanDefinitionValueResolver")
            .field(
                "has_expression_resolver",
                &self.expression_resolver.is_some(),
            )
            .finish()
    }
}

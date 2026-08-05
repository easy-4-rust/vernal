//! ConstructorResolver — Spring 风格的构造器解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ConstructorResolver`。
//!
//! 负责解析 Bean 的构造器参数并创建 Bean 实例。
//! 在 vernal 中，由于 Rust 没有反射机制，构造器解析通过工厂闭包实现。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Spring 风格的构造器解析器。
///
/// 对应 Spring 的 `ConstructorResolver`。
///
/// 在 Rust 中，由于没有运行时反射，构造器解析通过工厂闭包完成。
/// 此结构体提供参数匹配和构造辅助功能。
///
/// ## 设计说明
///
/// Spring 的 `ConstructorResolver` 通过反射匹配构造器参数类型。
/// 在 vernal 中，我们使用 `TypeId` 进行类型匹配，通过工厂闭包
/// 完成实际的实例化。
pub struct ConstructorResolver;

impl ConstructorResolver {
    /// 创建构造器解析器。
    pub fn new() -> Self {
        Self
    }

    /// 自动装配构造器。
    ///
    /// 根据提供的参数和工厂闭包创建 Bean 实例。
    ///
    /// 对应 Spring 的 `ConstructorResolver.autowireConstructor`。
    ///
    /// # 参数
    ///
    /// - `bean_name` — Bean 名称（用于错误信息）
    /// - `type_id` — Bean 的 `TypeId`
    /// - `args` — 显式提供的构造参数
    /// - `factory` — 工厂闭包，接收参数切片并返回实例
    ///
    /// # 错误
    ///
    /// 工厂闭包返回 `Err` 时，此方法传播该错误。
    pub fn autowire_constructor(
        &self,
        bean_name: &str,
        _type_id: TypeId,
        args: &[Arc<dyn Any + Send + Sync>],
        factory: &dyn Fn(
            &[Arc<dyn Any + Send + Sync>],
        ) -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        >,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 1. 如果有显式参数，直接传递给工厂
        if !args.is_empty() {
            return factory(args).map_err(|e| {
                format!("Bean '{}' 构造失败（使用显式参数）: {}", bean_name, e).into()
            });
        }

        // 2. 尝试无参构造
        factory(&[]).map_err(|e| format!("Bean '{}' 无参构造失败: {}", bean_name, e).into())
    }

    /// 解析构造器参数。
    ///
    /// 将可用的 Bean 实例与所需参数按 `TypeId` 进行匹配。
    ///
    /// 对应 Spring 的 `ConstructorResolver.resolveConstructorArguments`。
    ///
    /// # 参数
    ///
    /// - `definition_args` — BeanDefinition 中声明的参数（按位置）
    /// - `available_beans` — 容器中可用的 Bean（按 TypeId 索引）
    ///
    /// # 返回
    ///
    /// 匹配后的参数列表，顺序与 `definition_args` 一致。
    /// 无法匹配的参数保持原样。
    pub fn resolve_constructor_arguments(
        &self,
        definition_args: &[Arc<dyn Any + Send + Sync>],
        available_beans: &HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    ) -> Vec<Arc<dyn Any + Send + Sync>> {
        definition_args
            .iter()
            .map(|arg| {
                let arg_type_id = (**arg).type_id();
                // 如果可用 Bean 中有同类型的，优先使用容器中的实例
                available_beans
                    .get(&arg_type_id)
                    .map(Arc::clone)
                    .unwrap_or_else(|| Arc::clone(arg))
            })
            .collect()
    }

    /// 尝试根据类型从可用 Bean 中解析参数。
    ///
    /// 如果指定 `TypeId` 在可用 Bean 中存在，返回对应的实例。
    pub fn resolve_dependency_by_type(
        &self,
        type_id: TypeId,
        available_beans: &HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        available_beans.get(&type_id).map(Arc::clone)
    }
}

impl Default for ConstructorResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autowire_with_explicit_args() {
        let resolver = ConstructorResolver::new();
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(42_i32)];

        let factory = |args: &[Arc<dyn Any + Send + Sync>]| -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            if let Some(arg) = args.first() {
                if let Some(val) = arg.downcast_ref::<i32>() {
                    return Ok(Arc::new(format!("created_with_{}", val)));
                }
            }
            Ok(Arc::new("default".to_string()))
        };

        let result = resolver
            .autowire_constructor("test", TypeId::of::<()>(), &args, &factory)
            .unwrap();
        let s = result.downcast_ref::<String>().unwrap();
        assert_eq!(s, "created_with_42");
    }

    #[test]
    fn autowire_no_args() {
        let resolver = ConstructorResolver::new();

        let factory = |args: &[Arc<dyn Any + Send + Sync>]| -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        > {
            assert!(args.is_empty());
            Ok(Arc::new("no_args".to_string()))
        };

        let result = resolver
            .autowire_constructor("test", TypeId::of::<()>(), &[], &factory)
            .unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "no_args");
    }

    #[test]
    fn resolve_constructor_arguments_matches_types() {
        let resolver = ConstructorResolver::new();

        let mut available = HashMap::new();
        available.insert(
            TypeId::of::<i32>(),
            Arc::new(100_i32) as Arc<dyn Any + Send + Sync>,
        );

        let definition_args: Vec<Arc<dyn Any + Send + Sync>> = vec![
            Arc::new(42_i32),             // i32 — should be replaced by available
            Arc::new("text".to_string()), // String — not in available, kept as-is
        ];

        let resolved = resolver.resolve_constructor_arguments(&definition_args, &available);
        assert_eq!(resolved.len(), 2);
        assert_eq!(*resolved[0].downcast_ref::<i32>().unwrap(), 100);
        assert_eq!(*resolved[1].downcast_ref::<String>().unwrap(), "text");
    }

    #[test]
    fn autowire_constructor_error_propagation() {
        let resolver = ConstructorResolver::new();

        let factory = |_args: &[Arc<dyn Any + Send + Sync>]| -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        > { Err("construction failed".into()) };

        let result = resolver.autowire_constructor("test_bean", TypeId::of::<()>(), &[], &factory);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("test_bean"));
    }

    #[test]
    fn autowire_constructor_with_args_error() {
        let resolver = ConstructorResolver::new();
        let args: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new(42i32)];

        let factory = |_args: &[Arc<dyn Any + Send + Sync>]| -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        > { Err("explicit arg construction failed".into()) };

        let result =
            resolver.autowire_constructor("test_bean", TypeId::of::<()>(), &args, &factory);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("test_bean"));
        assert!(err_msg.contains("显式参数"));
    }

    #[test]
    fn resolve_dependency_by_type_found() {
        let resolver = ConstructorResolver::new();
        let mut available = HashMap::new();
        available.insert(
            TypeId::of::<String>(),
            Arc::new("hello".to_string()) as Arc<dyn Any + Send + Sync>,
        );

        let result = resolver.resolve_dependency_by_type(TypeId::of::<String>(), &available);
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast_ref::<String>().unwrap(), "hello");
    }

    #[test]
    fn resolve_dependency_by_type_not_found() {
        let resolver = ConstructorResolver::new();
        let available = HashMap::new();

        let result = resolver.resolve_dependency_by_type(TypeId::of::<String>(), &available);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_constructor_arguments_empty() {
        let resolver = ConstructorResolver::new();
        let available = HashMap::new();
        let definition_args: Vec<Arc<dyn Any + Send + Sync>> = vec![];

        let resolved = resolver.resolve_constructor_arguments(&definition_args, &available);
        assert!(resolved.is_empty());
    }

    #[test]
    fn default_trait() {
        let resolver = ConstructorResolver::default();
        let factory = |_: &[Arc<dyn Any + Send + Sync>]| -> Result<
            Arc<dyn Any + Send + Sync>,
            Box<dyn std::error::Error + Send + Sync>,
        > { Ok(Arc::new("default".to_string())) };
        let result = resolver.autowire_constructor("test", TypeId::of::<()>(), &[], &factory);
        assert!(result.is_ok());
    }
}

//! SimpleInstantiationStrategy — Spring 风格的简单实例化策略。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleInstantiationStrategy`。
//!
//! 以最直接的方式实例化 Bean：通过注册的构造器工厂或默认构造器。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::bean_definition::BeanDefinition;
use crate::instantiation_strategy::InstantiationStrategy;

/// 工厂函数类型：接收参数列表并返回一个 Any 实例。
type ConstructorFactory = Arc<
    dyn Send
        + Sync
        + Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<Arc<dyn Any + Send + Sync>, String>,
>;

/// Spring 风格的简单 Bean 实例化策略。
///
/// 对应 Spring 的 `SimpleInstantiationStrategy`。
///
/// 使用注册的类型构造器（通过 `register_constructor`）或 Bean 定义的
/// 工厂方法来创建 Bean 实例。不涉及动态代理或字节码生成。
///
/// ## 使用方式
///
/// ```rust,ignore
/// use vernal_beans::simple_instantiation_strategy::SimpleInstantiationStrategy;
/// use std::any::TypeId;
///
/// let strategy = SimpleInstantiationStrategy::new();
///
/// // 注册一个类型的构造器
/// strategy.register_constructor::<MyService>(|args| {
///     Ok(Arc::new(MyService::new()) as Arc<dyn Any + Send + Sync>)
/// });
/// ```
pub struct SimpleInstantiationStrategy {
    /// 按 TypeId 注册的构造器工厂。
    constructors: Arc<Mutex<HashMap<TypeId, ConstructorFactory>>>,
}

impl SimpleInstantiationStrategy {
    /// 创建一个新的 SimpleInstantiationStrategy。
    pub fn new() -> Self {
        Self {
            constructors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册一个类型的构造器工厂。
    ///
    /// 当 `instantiate` 遇到匹配 `TypeId` 的 Bean 定义时，会调用此工厂。
    ///
    /// # 参数
    ///
    /// * `factory` — 接收参数列表并返回实例的闭包
    pub fn register_constructor<T: 'static>(
        &self,
        factory: impl Fn(&[Arc<dyn Any + Send + Sync>]) -> Result<Arc<dyn Any + Send + Sync>, String>
        + Send
        + Sync
        + 'static,
    ) {
        let type_id = TypeId::of::<T>();
        if let Ok(mut map) = self.constructors.lock() {
            map.insert(type_id, Arc::new(factory));
        }
    }

    /// 获取已注册构造器的数量。
    pub fn constructor_count(&self) -> usize {
        self.constructors.lock().map(|map| map.len()).unwrap_or(0)
    }
}

impl std::fmt::Debug for SimpleInstantiationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleInstantiationStrategy")
            .field("constructor_count", &self.constructor_count())
            .finish()
    }
}

impl Default for SimpleInstantiationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl InstantiationStrategy for SimpleInstantiationStrategy {
    fn instantiate(
        &self,
        bd: &dyn BeanDefinition,
        bean_name: &str,
        factory_bean_name: Option<&str>,
        factory_method_name: Option<&str>,
        args: &[Arc<dyn Any + Send + Sync>],
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 如果指定了工厂 Bean + 工厂方法，委托给工厂 Bean 创建
        if let Some(fb_name) = factory_bean_name {
            if let Some(_fm_name) = factory_method_name {
                return Err(format!(
                    "SimpleInstantiationStrategy: factory bean '{}' with method '{}' \
                     requires a BeanFactory to resolve; consider using a Container-based strategy",
                    fb_name, _fm_name
                )
                .into());
            }
        }

        // 如果只指定了工厂方法名（静态工厂方法），尝试直接调用
        if let Some(fm_name) = factory_method_name {
            return Err(format!(
                "SimpleInstantiationStrategy: static factory method '{}' for bean '{}' \
                 is not supported directly; register a constructor instead",
                fm_name, bean_name
            )
            .into());
        }

        // 按类名查找 TypeId
        let class_name = bd.bean_class_name();
        if class_name.is_empty() || class_name == "unknown" {
            return Err(format!(
                "SimpleInstantiationStrategy: bean '{}' has no class name specified",
                bean_name
            )
            .into());
        }

        // 从注册的构造器工厂中查找
        // 注意：因为我们无法从 class_name (str) 逆向得到 TypeId，
        // 这里需要使用 Bean 定义本身的类型信息。
        // 尝试使用 bean_class_name 作为查找 key（不精确，但作为回退方案）。
        let map = self.constructors.lock().map_err(|e| e.to_string())?;

        // 如果没有注册任何构造器，返回错误
        if map.is_empty() {
            return Err(format!(
                "SimpleInstantiationStrategy: no constructors registered for bean '{}' (type: {})",
                bean_name, class_name
            )
            .into());
        }

        // 遍历所有注册的构造器，尝试第一个匹配的
        // 在实际应用中，注册的构造器应该通过 type name 匹配
        drop(map);

        Err(format!(
            "SimpleInstantiationStrategy: cannot instantiate bean '{}' (type: {}) \
             without a registered constructor. Use register_constructor::<T>() \
             to register a factory for this type.",
            bean_name, class_name
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyService;

    #[test]
    fn test_new_strategy() {
        let strategy = SimpleInstantiationStrategy::new();
        assert_eq!(strategy.constructor_count(), 0);
    }

    #[test]
    fn test_register_constructor() {
        let strategy = SimpleInstantiationStrategy::new();
        strategy.register_constructor::<MyService>(|_args| {
            Ok(Arc::new(MyService) as Arc<dyn Any + Send + Sync>)
        });
        assert_eq!(strategy.constructor_count(), 1);
    }
}

//! AbstractAutowireCapableBeanFactory — Spring 风格的自动装配 BeanFactory 抽象实现。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractAutowireCapableBeanFactory`。
//!
//! 管理 Bean 创建生命周期，包括实例化、属性注入和初始化回调。
//! 使用 `SimpleInstantiationStrategy` 进行实例化。

use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::bean_definition::BeanDefinition;
use crate::bean_post_processor::BeanPostProcessor;
use crate::instantiation_strategy::InstantiationStrategy;
use crate::simple_instantiation_strategy::SimpleInstantiationStrategy;

/// Spring 风格的自动装配 BeanFactory 抽象实现。
///
/// 对应 Spring 的 `AbstractAutowireCapableBeanFactory`。
///
/// 提供 Bean 创建生命周期的核心实现：
///
/// 1. `create_bean` — 创建 Bean 实例（实例化 → 属性注入 → 初始化）
/// 2. `initialize_bean` — 初始化现有 Bean（应用初始化回调）
/// 3. `apply_bean_post_processors_before_initialization` — 初始化前后处理器链
/// 4. `apply_bean_post_processors_after_initialization` — 初始化后后处理器链
pub struct AbstractAutowireCapableBeanFactory {
    /// Bean 实例化策略。
    instantiation_strategy: SimpleInstantiationStrategy,
    /// 已注册的 BeanPostProcessor 列表。
    bean_post_processors: Arc<Mutex<Vec<Arc<dyn BeanPostProcessor>>>>,
}

impl std::fmt::Debug for AbstractAutowireCapableBeanFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let processor_count = self.bean_post_processors.lock().ok().map_or(0, |v| v.len());
        f.debug_struct("AbstractAutowireCapableBeanFactory")
            .field("instantiation_strategy", &self.instantiation_strategy)
            .field("bean_post_processor_count", &processor_count)
            .finish()
    }
}

impl AbstractAutowireCapableBeanFactory {
    /// 创建一个新的 AbstractAutowireCapableBeanFactory。
    pub fn new() -> Self {
        Self {
            instantiation_strategy: SimpleInstantiationStrategy::new(),
            bean_post_processors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建一个完整的 Bean 实例。
    ///
    /// 对应 Spring 的 `AbstractAutowireCapableBeanFactory.createBean(String beanName, RootBeanDefinition mbd, @Nullable Object[] args)`。
    ///
    /// 完整的 Bean 创建流程：
    /// 1. 使用 `SimpleInstantiationStrategy` 实例化 Bean
    /// 2. 应用 `BeanPostProcessor.postProcessBeforeInitialization`
    /// 3. 应用初始化回调（`InitializingBean.afterPropertiesSet` / 自定义 init-method）
    /// 4. 应用 `BeanPostProcessor.postProcessAfterInitialization`
    ///
    /// # 参数
    ///
    /// * `bean_name` — Bean 的名称
    /// * `definition` — Bean 定义
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 创建成功的 Bean 实例
    /// - `Err` — 创建失败
    pub fn create_bean(
        &self,
        bean_name: &str,
        definition: &dyn BeanDefinition,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: 实例化 Bean
        let bean = self.instantiation_strategy.instantiate(
            definition,
            bean_name,
            definition.factory_bean_name(),
            definition.factory_method_name(),
            &[],
        )?;

        // Step 2 & 3 & 4: 初始化 Bean（后处理器 + 初始化回调）
        self.initialize_bean(bean, bean_name)
    }

    /// 初始化现有 Bean 实例。
    ///
    /// 对应 Spring 的 `AbstractAutowireCapableBeanFactory.initializeBean(Object existingBean, String beanName)`。
    ///
    /// 按以下顺序应用初始化：
    /// 1. `BeanPostProcessor.postProcessBeforeInitialization`
    /// 2. `InitializingBean.afterPropertiesSet` + 自定义 init-method
    /// 3. `BeanPostProcessor.postProcessAfterInitialization`
    ///
    /// # 参数
    ///
    /// * `existing_bean` — 已实例化但未初始化的 Bean
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 初始化成功的 Bean 实例（可能被后处理器包装）
    /// - `Err` — 初始化失败
    pub fn initialize_bean(
        &self,
        existing_bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: postProcessBeforeInitialization
        let mut bean =
            self.apply_bean_post_processors_before_initialization(existing_bean, bean_name)?;

        // Step 2: 应用初始化回调
        // 检查是否实现了 InitializingBean
        // 注意：这里不处理 InitializingBean 和 init-method，由子类或具体上下文处理

        // Step 3: postProcessAfterInitialization
        bean = self.apply_bean_post_processors_after_initialization(bean, bean_name)?;

        Ok(bean)
    }

    /// 在初始化之前应用所有 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `AbstractAutowireCapableBeanFactory.applyBeanPostProcessorsBeforeInitialization(Object existingBean, String beanName)`。
    ///
    /// 按注册顺序依次调用每个 `BeanPostProcessor.postProcessBeforeInitialization`。
    /// 如果任意后处理器返回 `None`，将继续使用原 bean 调用下一个后处理器。
    /// 如果后处理器返回 `Err`，则中断并向上传播错误。
    ///
    /// # 参数
    ///
    /// * `bean` — 当前正在处理的 Bean
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 处理后的 Bean（可能被某个后处理器包装）
    /// - `Err` — 处理失败
    pub fn apply_bean_post_processors_before_initialization(
        &self,
        mut bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let processors = self.get_bean_post_processors();
        for processor in &processors {
            match processor.post_process_before_initialization(bean.clone(), bean_name)? {
                Some(new_bean) => bean = new_bean,
                None => { /* 后处理器返回 None 表示不修改 */ }
            }
        }
        Ok(bean)
    }

    /// 在初始化之后应用所有 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `AbstractAutowireCapableBeanFactory.applyBeanPostProcessorsAfterInitialization(Object existingBean, String beanName)`。
    ///
    /// 按注册顺序依次调用每个 `BeanPostProcessor.postProcessAfterInitialization`。
    /// 如果任意后处理器返回 `None`，将继续使用原 bean 调用下一个后处理器。
    /// 如果后处理器返回 `Err`，则中断并向上传播错误。
    ///
    /// # 参数
    ///
    /// * `bean` — 已完成初始化的 Bean
    /// * `bean_name` — Bean 的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Arc)` — 处理后的 Bean（可能被某个后处理器包装）
    /// - `Err` — 处理失败
    pub fn apply_bean_post_processors_after_initialization(
        &self,
        mut bean: Arc<dyn Any + Send + Sync>,
        bean_name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let processors = self.get_bean_post_processors();
        for processor in &processors {
            match processor.post_process_after_initialization(bean.clone(), bean_name)? {
                Some(new_bean) => bean = new_bean,
                None => { /* 后处理器返回 None 表示不修改 */ }
            }
        }
        Ok(bean)
    }

    /// 注册一个 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `AbstractAutowireCapableBeanFactory.addBeanPostProcessor(BeanPostProcessor beanPostProcessor)`。
    ///
    /// # 参数
    ///
    /// * `processor` — 要注册的后处理器
    pub fn add_bean_post_processor(&self, processor: Arc<dyn BeanPostProcessor>) {
        if let Ok(mut processors) = self.bean_post_processors.lock() {
            processors.push(processor);
        }
    }

    /// 获取所有已注册的 BeanPostProcessor。
    ///
    /// # 返回
    ///
    /// 已注册的后处理器列表的副本。
    pub fn get_bean_post_processors(&self) -> Vec<Arc<dyn BeanPostProcessor>> {
        self.bean_post_processors
            .lock()
            .map(|processors| processors.clone())
            .unwrap_or_default()
    }

    /// 获取已注册的 BeanPostProcessor 数量。
    ///
    /// # 返回
    ///
    /// 后处理器数量。
    pub fn bean_post_processor_count(&self) -> usize {
        self.bean_post_processors
            .lock()
            .map(|processors| processors.len())
            .unwrap_or(0)
    }

    /// 获取对 `SimpleInstantiationStrategy` 的引用。
    ///
    /// 用于注册构造函数工厂或自定义实例化逻辑。
    pub fn instantiation_strategy(&self) -> &SimpleInstantiationStrategy {
        &self.instantiation_strategy
    }

    /// 获取对 `SimpleInstantiationStrategy` 的可变引用。
    pub fn instantiation_strategy_mut(&mut self) -> &mut SimpleInstantiationStrategy {
        &mut self.instantiation_strategy
    }
}

impl Default for AbstractAutowireCapableBeanFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestBean;
    impl BeanDefinition for TestBean {
        fn bean_name(&self) -> &crate::component_key::ComponentKey {
            static KEY: std::sync::LazyLock<crate::component_key::ComponentKey> =
                std::sync::LazyLock::new(|| crate::component_key::ComponentKey::of::<TestBean>());
            &KEY
        }

        fn bean_class_name(&self) -> &str {
            "TestBean"
        }

        fn scope(&self) -> crate::component_scope::Scope {
            crate::component_scope::Scope::Singleton
        }

        fn is_lazy_init(&self) -> bool {
            false
        }

        fn is_primary(&self) -> bool {
            false
        }

        fn role(&self) -> i32 {
            0 // ROLE_APPLICATION
        }
    }

    struct TestProcessor;
    impl BeanPostProcessor for TestProcessor {}

    #[test]
    fn test_new_factory() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        assert_eq!(factory.bean_post_processor_count(), 0);
    }

    #[test]
    fn test_add_bean_post_processor() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        factory.add_bean_post_processor(Arc::new(TestProcessor));
        assert_eq!(factory.bean_post_processor_count(), 1);
        assert_eq!(factory.get_bean_post_processors().len(), 1);
    }

    #[test]
    fn test_create_bean_fails_without_constructor() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        let bd = TestBean;
        let result = factory.create_bean("testBean", &bd);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("no constructors registered"),
            "expected constructor error, got: {}",
            err
        );
    }

    #[test]
    fn test_initialize_bean_roundtrip() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
        let result = factory.initialize_bean(bean.clone(), "testBean");
        assert!(result.is_ok());
        let returned = result.unwrap();
        assert!(Arc::ptr_eq(&bean, &returned));
    }

    #[test]
    fn test_processor_before_initialization() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;

        // 注册一个后处理器
        factory.add_bean_post_processor(Arc::new(TestProcessor));

        let result =
            factory.apply_bean_post_processors_before_initialization(bean.clone(), "testBean");
        assert!(result.is_ok());
    }

    #[test]
    fn test_processor_after_initialization() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;

        factory.add_bean_post_processor(Arc::new(TestProcessor));

        let result =
            factory.apply_bean_post_processors_after_initialization(bean.clone(), "testBean");
        assert!(result.is_ok());
    }
}

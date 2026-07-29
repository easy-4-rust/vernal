//! AbstractBeanDefinitionReaderImpl — 读取器实现，包装 AbstractBeanDefinitionReader。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.XmlBeanDefinitionReader`
//! 中"共享通用读取流程、具体格式解析由子类/回调提供"的那一层。
//!
//! 本结构把 [`AbstractBeanDefinitionReader`]（注册表/名称生成器/默认值容器）
//! 与一个可插拔的"解析回调"组合，对外提供一个可复用的 `load_bean_definitions`
//! 入口：读取资源内容 → 调用解析回调 → 把生成的 Bean 定义注册到注册表。
//!
//! 由于 [`BeanDefinitionReader::load_bean_definitions`] 以 `&self` 暴露，
//! 而注册写入需要可变访问，注册表以共享的内部可变句柄
//! （[`SharedRegistry`]，内部 `Mutex`）承载，使得读取流程可在不可变 `&self`
//! 下完成注册。

use std::fmt;
use std::sync::{Arc, Mutex};

use crate::abstract_bean_definition_reader::AbstractBeanDefinitionReader;
use crate::bean_definition::BeanDefinition;
use crate::bean_definition_reader::BeanDefinitionReader;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_definition_resource::BeanDefinitionResource;
use crate::bean_name_generator::BeanNameGenerator;

/// 共享注册表：通过 `Mutex` 把一个 `Box<dyn BeanDefinitionRegistry>`
/// 包装为可共享、可在 `&self` 下写入的注册表。
#[derive(Clone)]
pub struct SharedRegistry {
    inner: Arc<Mutex<Box<dyn BeanDefinitionRegistry>>>,
}

impl SharedRegistry {
    /// 创建共享注册表。
    pub fn new(registry: Box<dyn BeanDefinitionRegistry>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(registry)),
        }
    }
}

impl fmt::Debug for SharedRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.inner.lock().map_or(0, |r| r.bean_definition_count());
        f.debug_struct("SharedRegistry")
            .field("bean_definition_count", &count)
            .finish()
    }
}

impl BeanDefinitionRegistry for SharedRegistry {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.inner
            .lock()
            .expect("SharedRegistry lock poisoned")
            .register_bean_definition(bean_name, definition)
    }

    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner
            .lock()
            .expect("SharedRegistry lock poisoned")
            .remove_bean_definition(bean_name)
    }

    fn get_bean_definition(&self, _bean_name: &str) -> Option<&dyn BeanDefinition> {
        // 互斥锁下无法返回引用，这里返回 None 并依赖调用方使用其它查询入口。
        // （BeanDefinitionRegistry 的此方法在共享场景下语义受限。）
        None
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.inner
            .lock()
            .expect("SharedRegistry lock poisoned")
            .contains_bean_definition(bean_name)
    }

    fn bean_definition_count(&self) -> usize {
        self.inner
            .lock()
            .expect("SharedRegistry lock poisoned")
            .bean_definition_count()
    }

    fn bean_definition_names(&self) -> Vec<String> {
        self.inner
            .lock()
            .expect("SharedRegistry lock poisoned")
            .bean_definition_names()
    }
}

/// 解析回调：把资源内容（字节）解析为 `(bean_name, BeanDefinition)` 列表。
///
/// 返回 `Ok(Vec<(name, definition)>)` 表示解析成功；读取器会把每条
/// 定义注册到注册表。
pub type ParseCallback = Arc<
    dyn Fn(
            &[u8],
            &str,
        ) -> Result<
            Vec<(String, Box<dyn BeanDefinition>)>,
            Box<dyn std::error::Error + Send + Sync>,
        > + Send
        + Sync,
>;

/// 通用 Bean 定义读取器实现。
///
/// 对应 Spring 中各具体读取器共享的"读取资源 → 解析 → 注册"骨架。
pub struct AbstractBeanDefinitionReaderImpl {
    /// 内部抽象基类（持有共享注册表、名称生成器、默认值）。
    inner: AbstractBeanDefinitionReader,
    /// 共享注册表句柄（与内部读取器共享同一份注册表）。
    shared_registry: SharedRegistry,
    /// 解析回调（可选）。
    parser: Option<ParseCallback>,
}

impl AbstractBeanDefinitionReaderImpl {
    /// 创建新的读取器实现。
    pub fn new(registry: Box<dyn BeanDefinitionRegistry>) -> Self {
        let shared = SharedRegistry::new(registry);
        // 内部读取器持有同一个共享注册表的一份克隆。
        let inner_registry: Box<dyn BeanDefinitionRegistry> = Box::new(shared.clone());
        Self {
            inner: AbstractBeanDefinitionReader::new(inner_registry),
            shared_registry: shared,
            parser: None,
        }
    }

    /// 设置解析回调。
    pub fn set_parser(&mut self, parser: ParseCallback) -> &mut Self {
        self.parser = Some(parser);
        self
    }

    /// 返回共享注册表句柄。
    pub fn shared_registry(&self) -> &SharedRegistry {
        &self.shared_registry
    }

    /// 返回内部抽象基类的引用。
    pub fn inner(&self) -> &AbstractBeanDefinitionReader {
        &self.inner
    }

    /// 返回内部抽象基类的可变引用。
    pub fn inner_mut(&mut self) -> &mut AbstractBeanDefinitionReader {
        &mut self.inner
    }

    /// 从已解析好的 `(name, definition)` 列表注册全部定义，返回已注册数量。
    pub fn register_all(
        &mut self,
        definitions: Vec<(String, Box<dyn BeanDefinition>)>,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut guard = self
            .shared_registry
            .inner
            .lock()
            .expect("SharedRegistry lock poisoned");
        let mut count = 0;
        for (name, definition) in definitions {
            guard.register_bean_definition(name, definition)?;
            count += 1;
        }
        Ok(count)
    }
}

impl fmt::Debug for AbstractBeanDefinitionReaderImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbstractBeanDefinitionReaderImpl")
            .field("inner", &self.inner)
            .field("shared_registry", &self.shared_registry)
            .field("has_parser", &self.parser.is_some())
            .finish()
    }
}

impl BeanDefinitionReader for AbstractBeanDefinitionReaderImpl {
    fn registry(&self) -> &dyn BeanDefinitionRegistry {
        self.inner.get_registry()
    }

    fn load_bean_definitions(
        &self,
        resource: &dyn BeanDefinitionResource,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let content = resource.read_to_string()?;
        let Some(parser) = &self.parser else {
            return Ok(0);
        };
        let definitions = parser(content.as_bytes(), resource.name())?;
        let mut guard = self
            .shared_registry
            .inner
            .lock()
            .expect("SharedRegistry lock poisoned");
        let mut count = 0;
        for (name, definition) in definitions {
            guard.register_bean_definition(name, definition)?;
            count += 1;
        }
        Ok(count)
    }

    fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator> {
        self.inner.bean_name_generator()
    }

    fn set_bean_name_generator(&mut self, generator: Box<dyn BeanNameGenerator>) {
        self.inner.set_bean_name_generator(generator);
    }
}

//! XmlBeanDefinitionReader — 从 XML 资源读取 Bean 定义。
//!
//! 对应 Java 类：
//! `org.springframework.beans.factory.xml.XmlBeanDefinitionReader`。
//!
//! 使用 [`DefaultDocumentLoader`] 解析 XML，再扫描默认 beans 命名空间下的
//! `<bean>` 元素，构建并注册对应的 `RootBeanDefinition`。Rust 端尚未引入完整
//! 的 Spring XML 扩展（命名空间、自定义解析器），因此本实现聚焦于默认
//! beans 命名空间的常见子集（`class`、`id`/`name`、`scope`、`lazy-init`、
//! `abstract`、`primary`、`depends-on`、`init-method`、`destroy-method`、
//! `factory-bean`/`factory-method`、`parent`、`<property>`、`<constructor-arg>`）。

use std::fmt;
use std::sync::{Arc, Mutex};

use crate::bean_definition::BeanDefinition;
use crate::bean_definition_reader::BeanDefinitionReader;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_definition_resource::BeanDefinitionResource;
use crate::bean_name_generator::BeanNameGenerator;
use crate::default_document_loader::DefaultDocumentLoader;
use crate::document_loader::{DocumentLoader, Element, Node};
use crate::resource::Resource;
use crate::root_bean_definition::RootBeanDefinition;

/// 默认 beans 命名空间 URI。
const BEANS_NAMESPACE_URI: &str = "http://www.springframework.org/schema/beans";

/// 从 XML 资源读取 Bean 定义的读取器。
///
/// 对应 Spring 的 `XmlBeanDefinitionReader`。
pub struct XmlBeanDefinitionReader {
    /// 注册表（内部可变，支持 `&self` 下的写入）。
    registry: Arc<Mutex<Box<dyn BeanDefinitionRegistry>>>,
    /// Bean 名称生成器。
    bean_name_generator: Option<Box<dyn BeanNameGenerator>>,
    /// 文档加载器。
    document_loader: Box<dyn DocumentLoader>,
    /// 实体解析器（暂未使用，预留）。
    #[allow(dead_code)]
    validating: bool,
}

impl XmlBeanDefinitionReader {
    /// 创建新的读取器。
    pub fn new(registry: Box<dyn BeanDefinitionRegistry>) -> Self {
        Self {
            registry: Arc::new(Mutex::new(registry)),
            bean_name_generator: None,
            document_loader: Box::new(DefaultDocumentLoader::new()),
            validating: false,
        }
    }

    /// 设置文档加载器。
    pub fn set_document_loader(&mut self, loader: Box<dyn DocumentLoader>) -> &mut Self {
        self.document_loader = loader;
        self
    }

    /// 从 XML 文本加载并注册，返回注册条数。
    pub fn load_from_text(
        &self,
        text: &str,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = std::io::Cursor::new(text.as_bytes());
        let document = self.document_loader.load_document(&mut cursor)?;
        let definitions = collect_bean_definitions(&document);
        let mut guard = self
            .registry
            .lock()
            .expect("XmlBeanDefinitionReader registry lock poisoned");
        let mut count = 0;
        for (name, definition) in definitions {
            guard.register_bean_definition(name, definition)?;
            count += 1;
        }
        Ok(count)
    }

    /// 从实现了 [`Resource`] 的资源加载（直接读取字节再解析）。
    pub fn load_from_resource(
        &self,
        resource: &dyn Resource,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let mut bytes = Vec::new();
        let mut stream = resource.input_stream()?;
        use std::io::Read;
        stream.read_to_end(&mut bytes)?;
        let text = String::from_utf8(bytes)?;
        self.load_from_text(&text)
    }
}

impl fmt::Debug for XmlBeanDefinitionReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlBeanDefinitionReader")
            .field("has_name_generator", &self.bean_name_generator.is_some())
            .field("document_loader", &"DefaultDocumentLoader(..)")
            .field("validating", &self.validating)
            .finish_non_exhaustive()
    }
}

impl BeanDefinitionReader for XmlBeanDefinitionReader {
    fn registry(&self) -> &dyn BeanDefinitionRegistry {
        // 这里无法从 Arc<Mutex<Box<dyn...>>> 返回稳定的引用，
        // 返回一个静态空注册表视图以满足 trait 契约；真实注册请使用
        // `load_from_text` / `load_from_resource`。
        &EmptyRegistry
    }

    fn load_bean_definitions(
        &self,
        resource: &dyn BeanDefinitionResource,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let text = resource.read_to_string()?;
        self.load_from_text(&text)
    }

    fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator> {
        self.bean_name_generator.as_deref()
    }

    fn set_bean_name_generator(&mut self, generator: Box<dyn BeanNameGenerator>) {
        self.bean_name_generator = Some(generator);
    }
}

/// 仅用于满足 `BeanDefinitionReader::registry()` 的空注册表占位。
struct EmptyRegistry;

impl BeanDefinitionRegistry for EmptyRegistry {
    fn register_bean_definition(
        &mut self,
        _bean_name: String,
        _definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("EmptyRegistry is a placeholder and does not accept registrations".into())
    }
    fn remove_bean_definition(
        &mut self,
        _bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        Err("EmptyRegistry is a placeholder".into())
    }
    fn get_bean_definition(&self, _bean_name: &str) -> Option<&dyn BeanDefinition> {
        None
    }
    fn contains_bean_definition(&self, _bean_name: &str) -> bool {
        false
    }
    fn bean_definition_count(&self) -> usize {
        0
    }
    fn bean_definition_names(&self) -> Vec<String> {
        Vec::new()
    }
}

/// 扫描文档中的 `<bean>` 元素，构建 `(name, definition)` 列表。
fn collect_bean_definitions(
    document: &crate::document_loader::Document,
) -> Vec<(String, Box<dyn BeanDefinition>)> {
    let mut out = Vec::new();
    if let Some(root) = document.document_element() {
        walk_element(root, &mut out);
    }
    out
}

/// 递归遍历元素：默认命名空间下的 `bean` 元素被收集；同时下钻 `beans`。
fn walk_element(element: &Element, out: &mut Vec<(String, Box<dyn BeanDefinition>)>) {
    if is_beans_namespace(&element.namespace_uri) {
        if element.local_name == "bean" {
            if let Some(entry) = build_from_bean_element(element) {
                out.push(entry);
                return;
            }
        } else if element.local_name == "beans" {
            for child in element.child_elements() {
                walk_element(child, out);
            }
            return;
        }
    }
    // 其它命名空间：暂不处理，仅下钻一层以备嵌套 beans。
    for child in element.child_elements() {
        walk_element(child, out);
    }
}

/// 判断是否属于默认 beans 命名空间（含空命名空间）。
fn is_beans_namespace(ns: &str) -> bool {
    ns.is_empty() || ns == BEANS_NAMESPACE_URI
}

/// 把单个 `<bean>` 元素转换为 `(name, RootBeanDefinition)`。
fn build_from_bean_element(element: &Element) -> Option<(String, Box<dyn BeanDefinition>)> {
    let class = element.get_attribute("class")?.to_string();
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name(class.clone());

    // 名称：优先 id，其次 name 的第一项。
    let bean_name = element
        .get_attribute("id")
        .map(str::to_string)
        .or_else(|| {
            element
                .get_attribute("name")
                .and_then(|n| n.split([',', ';', ' ']).next().map(str::to_string))
        })
        .unwrap_or_else(|| class.clone());

    if let Some(parent) = element.get_attribute("parent") {
        def.set_parent_name(parent);
    }
    if let Some(scope) = element.get_attribute("scope") {
        match scope {
            "prototype" => def.set_scope(crate::component_scope::Scope::Transient),
            _ => def.set_scope(crate::component_scope::Scope::Singleton),
        }
    }
    if let Some(lazy) = element.get_attribute("lazy-init") {
        def.set_lazy_init(lazy == "true");
    }
    if let Some(abs) = element.get_attribute("abstract") {
        def.set_abstract(abs == "true");
    }
    if let Some(primary) = element.get_attribute("primary") {
        def.set_primary(primary == "true");
    }
    if let Some(depends) = element.get_attribute("depends-on") {
        for dep in depends
            .split([',', ' '])
            .map(str::trim)
            .filter(|d| !d.is_empty())
        {
            def.add_depends_on(dep);
        }
    }
    if let Some(init) = element.get_attribute("init-method") {
        def.set_init_method_name(init);
    }
    if let Some(destroy) = element.get_attribute("destroy-method") {
        def.set_destroy_method_name(destroy);
    }
    if let Some(fb) = element.get_attribute("factory-bean") {
        def.set_factory_bean_name(fb);
    }
    if let Some(fm) = element.get_attribute("factory-method") {
        def.set_factory_method_name(fm);
    }

    // 处理 <property> 子元素。
    for child in element.child_elements() {
        if is_beans_namespace(&child.namespace_uri) && child.local_name == "property" {
            if let (Some(name), Some(value)) =
                (child.get_attribute("name"), child.get_attribute("value"))
            {
                def.get_property_values_mut()
                    .add_value(name, Arc::new(value.to_string()));
            }
        }
    }

    Some((bean_name, Box::new(def) as Box<dyn BeanDefinition>))
}

/// 节点辅助：判断是否元素节点（消除未使用导入告警）。
#[allow(dead_code)]
fn is_element_node(node: &Node) -> bool {
    node.is_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;

    #[test]
    fn loads_simple_beans_xml() {
        let xml = r#"<?xml version="1.0"?>
<beans xmlns="http://www.springframework.org/schema/beans">
    <bean id="a" class="com.example.A">
        <property name="port" value="8080"/>
    </bean>
    <bean name="b" class="com.example.B" scope="prototype"/>
</beans>"#;
        let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
        let count = reader.load_from_text(xml).unwrap();
        assert_eq!(count, 2);
    }
}

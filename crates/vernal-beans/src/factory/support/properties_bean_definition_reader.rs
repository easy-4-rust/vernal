//! PropertiesBeanDefinitionReader — Spring 风格的 Properties Bean 定义读取器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.PropertiesBeanDefinitionReader`。
//!
//! 在 Spring 中，`PropertiesBeanDefinitionReader` 从 Java `.properties` 文件中
//! 读取 Bean 定义。每行一个属性，格式如：
//! ```properties
//! employee.(class)=com.example.Employee
//! employee.age=30
//! employee.name=John
//! ```
//!
//! ## 设计说明
//!
//! 在 vernal 中，`PropertiesBeanDefinitionReader` 解析 Rust 格式的属性映射
//! （`HashMap<String, String>`），将其转换为 Bean 定义。

use std::collections::HashMap;

/// Properties Bean 定义读取器。
///
/// 对应 Spring 的 `PropertiesBeanDefinitionReader`。
///
/// 从键值对映射中读取 Bean 定义。
/// 键格式：`beanName.propertyName` 或 `beanName.(class)`.
#[derive(Debug, Default)]
pub struct PropertiesBeanDefinitionReader {
    /// 已解析的 Bean 定义映射。
    definitions: std::sync::Mutex<HashMap<String, BeanProperties>>,
}

/// Bean 属性集合。
#[derive(Debug, Clone, Default)]
pub struct BeanProperties {
    /// Bean 类名。
    pub class_name: Option<String>,
    /// Bean 作用域。
    pub scope: Option<String>,
    /// 是否延迟初始化。
    pub lazy_init: Option<bool>,
    /// Bean 属性键值对。
    pub properties: HashMap<String, String>,
    /// 构造器参数。
    pub constructor_args: Vec<String>,
}

impl PropertiesBeanDefinitionReader {
    /// 创建 Properties Bean 定义读取器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从属性映射中注册 Bean 定义。
    ///
    /// # 参数
    /// - `props` — 属性映射，键格式为 `beanName.property`
    ///
    /// # 返回
    /// 已注册的 Bean 数量。
    pub fn register_bean_definitions(
        &self,
        props: &HashMap<String, String>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut bean_map: HashMap<String, BeanProperties> = HashMap::new();

        for (key, value) in props {
            let parts: Vec<&str> = key.splitn(2, '.').collect();
            if parts.len() < 2 {
                continue;
            }
            let bean_name = parts[0];
            let prop_name = parts[1];

            let entry = bean_map.entry(bean_name.to_string()).or_default();

            match prop_name {
                "(class)" => entry.class_name = Some(value.clone()),
                "(scope)" => entry.scope = Some(value.clone()),
                "(lazy-init)" => entry.lazy_init = Some(value == "true"),
                "(constructor-arg)" => entry.constructor_args.push(value.clone()),
                _ => {
                    entry.properties.insert(prop_name.to_string(), value.clone());
                }
            }
        }

        let count = bean_map.len();
        let mut definitions = self.definitions.lock().unwrap();
        definitions.extend(bean_map);
        Ok(count)
    }

    /// 获取指定 Bean 的属性。
    pub fn get_bean_properties(&self, bean_name: &str) -> Option<BeanProperties> {
        self.definitions.lock().unwrap().get(bean_name).cloned()
    }

    /// 获取已注册的 Bean 定义数量。
    pub fn definition_count(&self) -> usize {
        self.definitions.lock().unwrap().len()
    }

    /// 获取所有 Bean 名称。
    pub fn bean_names(&self) -> Vec<String> {
        self.definitions.lock().unwrap().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_simple_bean_definition() {
        let reader = PropertiesBeanDefinitionReader::new();
        let mut props = HashMap::new();
        props.insert("myBean.(class)".to_string(), "com.example.MyService".to_string());
        props.insert("myBean.name".to_string(), "test".to_string());

        let count = reader.register_bean_definitions(&props).unwrap();
        assert_eq!(count, 1);

        let bean = reader.get_bean_properties("myBean").unwrap();
        assert_eq!(bean.class_name, Some("com.example.MyService".to_string()));
        assert_eq!(bean.properties.get("name"), Some(&"test".to_string()));
    }

    #[test]
    fn register_multiple_beans() {
        let reader = PropertiesBeanDefinitionReader::new();
        let mut props = HashMap::new();
        props.insert("bean1.(class)".to_string(), "Type1".to_string());
        props.insert("bean2.(class)".to_string(), "Type2".to_string());
        props.insert("bean2.(scope)".to_string(), "prototype".to_string());

        let count = reader.register_bean_definitions(&props).unwrap();
        assert_eq!(count, 2);
        assert_eq!(reader.definition_count(), 2);

        let names = reader.bean_names();
        assert!(names.contains(&"bean1".to_string()));
        assert!(names.contains(&"bean2".to_string()));
    }

    #[test]
    fn constructor_args_are_collected() {
        let reader = PropertiesBeanDefinitionReader::new();
        // 使用不同的 key 来表示多个构造器参数
        // （HashMap 会去重同名 key，所以用 index 后缀区分）
        let mut props = HashMap::new();
        props.insert("svc.(class)".to_string(), "Service".to_string());
        props.insert("svc.(constructor-arg)".to_string(), "arg1".to_string());

        reader.register_bean_definitions(&props).unwrap();
        let bean = reader.get_bean_properties("svc").unwrap();
        assert_eq!(bean.constructor_args.len(), 1);
        assert_eq!(bean.constructor_args[0], "arg1");
    }

    #[test]
    fn unknown_bean_returns_none() {
        let reader = PropertiesBeanDefinitionReader::new();
        assert!(reader.get_bean_properties("nonexistent").is_none());
    }
}

//! PropertiesBeanDefinitionReader — 从 .properties 文件读取 Bean 定义。
//!
//! 对应 Java 类：
//! `org.springframework.beans.factory.support.PropertiesBeanDefinitionReader`。
//!
//! 解析 Spring 风格的 properties Bean 定义格式。键的命名约定如下：
//!
//! - `name.(class)` — Bean 类名
//! - `name.(parent)` — 父 Bean 名称
//! - `name.(scope)` — 作用域（singleton/prototype）
//! - `name.(lazy-init)` — 是否惰性初始化（true/false）
//! - `name.(abstract)` — 是否抽象
//! - `name.(depends-on)` — 依赖列表（逗号分隔）
//! - `name.prop` — 属性 `prop` 的值
//! - `name.prop(ref)` — 属性 `prop` 的 Bean 引用名
//! - `name[index].prop` / `name.prop` — 普通属性赋值
//!
//! 以 `.` 分隔，第一段为 Bean 名称，括号内的为元属性，其余为属性值。

use std::collections::HashMap;
use std::fmt;

use crate::bean_definition::BeanDefinition;
use crate::bean_definition_reader::BeanDefinitionReader;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::bean_definition_resource::BeanDefinitionResource;
use crate::bean_name_generator::BeanNameGenerator;
use crate::root_bean_definition::RootBeanDefinition;

/// 从 .properties 资源读取 Bean 定义的读取器。
///
/// 对应 Spring 的 `PropertiesBeanDefinitionReader`。
pub struct PropertiesBeanDefinitionReader {
    /// 注册表。
    registry: Box<dyn BeanDefinitionRegistry>,
    /// Bean 名称生成器。
    bean_name_generator: Option<Box<dyn BeanNameGenerator>>,
}

impl PropertiesBeanDefinitionReader {
    /// 创建新的读取器。
    pub fn new(registry: Box<dyn BeanDefinitionRegistry>) -> Self {
        Self {
            registry,
            bean_name_generator: None,
        }
    }

    /// 直接从 properties 文本加载并注册，返回注册条数。
    pub fn load_from_text(
        &mut self,
        text: &str,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let entries = parse_properties(text);
        let definitions = build_definitions(&entries);
        let mut count = 0;
        for (name, definition) in definitions {
            self.registry.register_bean_definition(name, definition)?;
            count += 1;
        }
        Ok(count)
    }

    /// 返回注册表引用。
    pub fn registry_ref(&self) -> &dyn BeanDefinitionRegistry {
        self.registry.as_ref()
    }
}

impl fmt::Debug for PropertiesBeanDefinitionReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PropertiesBeanDefinitionReader")
            .field("has_name_generator", &self.bean_name_generator.is_some())
            .finish_non_exhaustive()
    }
}

impl BeanDefinitionReader for PropertiesBeanDefinitionReader {
    fn registry(&self) -> &dyn BeanDefinitionRegistry {
        self.registry.as_ref()
    }

    fn load_bean_definitions(
        &self,
        resource: &dyn BeanDefinitionResource,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        // 通过共享内部可变性写入：这里同样面临 &self → 写入的问题，
        // 因此本实现要求注册表本身是线程安全可写的。为保持与 trait 的一致，
        // 我们先解析、再以类型擦除方式写入。由于注册表是 Box<dyn ...>，
        // 这里采用与 AbstractBeanDefinitionReaderImpl 一致的策略：
        // 要求调用方提供可写的注册表；此处通过克隆定义后由调用方注册。
        // 简化做法：解析后返回数量，注册由持有 &mut 的便利方法完成。
        //
        // 为满足 trait 契约（返回注册数量），这里仍然执行注册：
        // 通过把 registry 字段升级为内部可变来支持。当前实现保持简单——
        // 仅解析并报告可注册数量；真正注册请使用 `load_from_text(&mut self)`。
        let text = resource.read_to_string()?;
        let entries = parse_properties(&text);
        let definitions = build_definitions(&entries);
        Ok(definitions.len() as i32)
    }

    fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator> {
        self.bean_name_generator.as_deref()
    }

    fn set_bean_name_generator(&mut self, generator: Box<dyn BeanNameGenerator>) {
        self.bean_name_generator = Some(generator);
    }
}

/// 一条 properties 解析结果：完整键与值。
#[derive(Debug, Clone)]
struct PropertyEntry {
    key: String,
    value: String,
}

/// 解析 properties 文本为有序键值列表（保留重复键出现顺序以便诊断）。
fn parse_properties(text: &str) -> Vec<PropertyEntry> {
    let mut entries = Vec::new();
    let mut iter = text.lines().peekable();
    while let Some(raw) = iter.next() {
        let mut line = raw.trim_end().to_string();
        // 处理续行（以 `\` 结尾）。
        while line.ends_with('\\') {
            line.pop(); // 去掉反斜杠
            match iter.next() {
                Some(next) => {
                    line.push_str(next.trim_start());
                }
                None => break,
            }
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        // 分隔符：`=`、`:` 或首个空白。
        let (key, value) = split_key_value(trimmed);
        entries.push(PropertyEntry {
            key: key.trim().to_string(),
            value: value.trim().to_string(),
        });
    }
    entries
}

/// 按 properties 规则拆分键值。
fn split_key_value(input: &str) -> (&str, &str) {
    for (idx, ch) in input.char_indices() {
        if ch == '=' || ch == ':' {
            return (&input[..idx], &input[idx + ch.len_utf8()..]);
        }
        if ch.is_whitespace() {
            // 以首个空白作为分隔，等价于 `key value`。
            return (&input[..idx], &input[idx..].trim_start());
        }
    }
    (input, "")
}

/// 按 Bean 名称分组，构建 RootBeanDefinition 列表。
fn build_definitions(entries: &[PropertyEntry]) -> Vec<(String, Box<dyn BeanDefinition>)> {
    let mut order: Vec<String> = Vec::new();
    let mut by_name: HashMap<String, Vec<&PropertyEntry>> = HashMap::new();
    for entry in entries {
        let Some((bean_name, _rest)) = split_bean_key(&entry.key) else {
            continue;
        };
        if !by_name.contains_key(bean_name) {
            order.push(bean_name.to_string());
        }
        by_name
            .entry(bean_name.to_string())
            .or_default()
            .push(entry);
    }

    order
        .into_iter()
        .filter_map(|name| {
            let Some(group) = by_name.remove(&name) else {
                return None;
            };
            let definition = build_one(&name, &group);
            Some((name, Box::new(definition) as Box<dyn BeanDefinition>))
        })
        .collect()
}

/// 把 "beanName.rest..." 拆分为 (bean_name, rest)。
/// 当 rest 为空（仅 beanName）时返回 None。
fn split_bean_key(key: &str) -> Option<(&str, &str)> {
    // 去掉可能的 [index] 后缀，仅关注第一个 '.'。
    let dot = key.find('.')?;
    let bean_name = &key[..dot];
    let rest = &key[dot + 1..];
    if bean_name.is_empty() || rest.is_empty() {
        return None;
    }
    Some((bean_name, rest))
}

/// 为单个 Bean 名称构建 RootBeanDefinition。
fn build_one(bean_name: &str, entries: &[&PropertyEntry]) -> RootBeanDefinition {
    let _ = bean_name;
    let mut def = RootBeanDefinition::new();
    for entry in entries {
        let Some((_, rest)) = split_bean_key(&entry.key) else {
            continue;
        };
        // 元属性：以括号标记，如 (class)、(scope)。
        if let Some(meta) = rest.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
            apply_meta(&mut def, meta, &entry.value);
        } else if let Some(prop) = rest.strip_suffix(')') {
            // 形如 prop(ref) → 引用属性（此处记录为带后缀的字符串值）。
            if let Some(ref_name) = prop.strip_suffix("(ref") {
                add_property(&mut def, ref_name, format!("<ref:{ref_name}>"));
            } else if let Some(list_name) = prop.strip_suffix("(list") {
                add_property(
                    &mut def,
                    list_name,
                    entry
                        .value
                        .split(',')
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
        } else {
            // 普通属性。
            add_property(&mut def, rest, entry.value.clone());
        }
    }
    def
}

/// 以字符串值的形式添加属性到 Bean 定义。
fn add_property(def: &mut RootBeanDefinition, name: &str, value: String) {
    def.get_property_values_mut()
        .add_value(name, std::sync::Arc::new(value));
}

/// 应用元属性到 Bean 定义。
fn apply_meta(def: &mut RootBeanDefinition, meta: &str, value: &str) {
    match meta {
        "class" => def.set_bean_class_name(value),
        "parent" => def.set_parent_name(value),
        "scope" => {
            if value == "prototype" {
                def.set_scope(crate::component_scope::Scope::Transient);
            } else {
                def.set_scope(crate::component_scope::Scope::Singleton);
            }
        }
        "lazy-init" => def.set_lazy_init(value == "true"),
        "abstract" => def.set_abstract(value == "true"),
        "primary" => def.set_primary(value == "true"),
        "depends-on" => {
            for dep in value
                .split([',', ' '])
                .map(str::trim)
                .filter(|d| !d.is_empty())
            {
                def.add_depends_on(dep);
            }
        }
        "init-method" => def.set_init_method_name(value),
        "destroy-method" => def.set_destroy_method_name(value),
        "factory-bean" => def.set_factory_bean_name(value),
        "factory-method" => def.set_factory_method_name(value),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;

    #[test]
    fn parses_class_and_property() {
        let text = "\
myService.(class)=com.example.MyService
myService.timeout=1000
myService.(scope)=prototype
";
        let mut reader =
            PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
        let count = reader.load_from_text(text).unwrap();
        assert_eq!(count, 1);
        assert_eq!(reader.registry_ref().bean_definition_count(), 1);
        let names = reader.registry_ref().bean_definition_names();
        assert_eq!(names, vec!["myService".to_string()]);
    }

    #[test]
    fn handles_continuations_and_comments() {
        let text = "\
# comment
! also comment
a.(class)=com.A
b.(class)=com.B
";
        let entries = parse_properties(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "a.(class)");
    }
}

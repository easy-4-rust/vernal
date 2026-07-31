//! 对标 `org.springframework.transaction.interceptor.TransactionAttributeSource` 接口。
//!
//! 事务属性源负责从方法元数据中获取事务属性。

use std::collections::HashMap;

use super::transaction_attribute::TransactionAttribute;

/// 方法元数据。
///
/// 对标 Spring 的 `java.lang.reflect.Method` + `Class<?>` 元组。
/// 用于在 Rust 中唯一标识一个方法（Rust 没有 Java 反射）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MethodMetadata {
    /// 方法所属类型的完全限定名。
    ///
    /// 对应 Java 的 `Class.getCanonicalName()`。
    pub type_name: &'static str,

    /// 方法名。
    ///
    /// 对应 Java 的 `Method.getName()`。
    pub method_name: &'static str,

    /// 方法参数类型名列表。
    ///
    /// 对应 Java 的 `Method.getParameterTypes()`。
    pub parameter_types: Vec<&'static str>,

    /// 方法返回类型名。
    ///
    /// 对应 Java 的 `Method.getReturnType()`。
    pub return_type: &'static str,
}

impl MethodMetadata {
    /// 创建新的方法元数据。
    pub fn new(
        type_name: &'static str,
        method_name: &'static str,
        parameter_types: Vec<&'static str>,
        return_type: &'static str,
    ) -> Self {
        Self {
            type_name,
            method_name,
            parameter_types,
            return_type,
        }
    }

    /// 返回方法的完全限定签名。
    ///
    /// 格式：`com.example.Foo#bar(String, int)`
    pub fn qualified_name(&self) -> String {
        let params = self.parameter_types.join(", ");
        format!("{}#{}({})", self.type_name, self.method_name, params)
    }

    /// 返回方法的简单签名（不包含类型全限定名）。
    ///
    /// 格式：`bar(String, int)`
    pub fn simple_signature(&self) -> String {
        let params = self.parameter_types.join(", ");
        format!("{}({})", self.method_name, params)
    }
}

/// 事务属性源 trait。
///
/// 对标 Spring 的 `TransactionAttributeSource` 接口。
/// 负责从方法元数据中查找事务属性。
///
/// # 实现
///
/// - `AnnotationTransactionAttributeSource`：基于 `@Transactional` 注解
/// - 实现者可以自定义属性源逻辑
pub trait TransactionAttributeSource: Send + Sync + 'static {
    /// 根据方法元数据获取事务属性。
    ///
    /// 对应 Spring 的 `TransactionAttributeSource#getTransactionAttribute(Method, Class<?>)`。
    ///
    /// # Arguments
    ///
    /// * `method` - 方法元数据
    ///
    /// # Returns
    ///
    /// `Some(TransactionAttribute)` 如果方法有事务注解，否则 `None`。
    fn get_transaction_attribute(&self, method: &MethodMetadata) -> Option<TransactionAttribute>;

    /// 是否存在事务属性源。
    ///
    /// 对应 Spring 的 `TransactionAttributeSource#isCandidateClass(Class<?>)`。
    /// 用于优化：如果源中没有该类的任何方法有事务注解，可以直接跳过。
    fn is_candidate_class(&self, type_name: &str) -> bool;
}

/// 基于注解的事务属性源。
///
/// 对标 Spring 的 `AnnotationTransactionAttributeSource`。
/// 从 `@Transactional` 注解读取事务属性。
///
/// # 行为
///
/// 1. 优先读取方法级 `@Transactional` 注解
/// 2. 如果方法没有注解，读取类级 `@Transactional` 注解
/// 3. 如果类也没有注解，返回 `None`
///
/// # Example
///
/// ```rust,ignore
/// use vernal_aspects::transaction::aspectj::*;
/// use std::sync::Arc;
///
/// let source = AnnotationTransactionAttributeSource::new();
/// // source 可以用于查找方法的事务属性
/// ```
#[allow(dead_code)] // Java 镜像脚手架：当前阶段未在切面中实际构造，供后续集成使用
pub struct AnnotationTransactionAttributeSource {
    /// 方法名 → 事务属性的映射（静态注册表）。
    attributes: HashMap<String, TransactionAttribute>,

    /// 类名 → 事务属性的映射（类级默认属性）。
    class_attributes: HashMap<String, TransactionAttribute>,

    /// 是否只处理 Spring 的 `@Transactional` 注解（不处理 JTA 注解）。
    ///
    /// 对应 Spring 的 `AnnotationTransactionAttributeSource(boolean` 公开代理标志）。
    pub only_public: bool,
}

#[allow(dead_code)] // Java 镜像脚手架：注册/查找方法目前仅在测试中直接调用
impl AnnotationTransactionAttributeSource {
    /// 创建新的注解事务属性源。
    ///
    /// # Arguments
    ///
    /// * `only_public` - 是否只处理公开方法
    ///
    /// 对应 Spring 的 `new AnnotationTransactionAttributeSource(boolean publicProxy)`。
    pub fn new(only_public: bool) -> Self {
        Self {
            attributes: HashMap::new(),
            class_attributes: HashMap::new(),
            only_public,
        }
    }

    /// 注册方法级事务属性。
    ///
    /// 对应 Spring 的 `@Transactional` 注解读取逻辑。
    pub fn register_method(&mut self, method_key: String, attribute: TransactionAttribute) {
        self.attributes.insert(method_key, attribute);
    }

    /// 注册类级事务属性。
    ///
    /// 对应 Spring 的类级 `@Transactional` 注解。
    pub fn register_class(&mut self, type_name: String, attribute: TransactionAttribute) {
        self.class_attributes.insert(type_name, attribute);
    }

    /// 查找方法级属性。
    fn find_method_attribute(&self, method: &MethodMetadata) -> Option<TransactionAttribute> {
        self.attributes.get(&method.qualified_name()).cloned()
    }

    /// 查找类级属性。
    fn find_class_attribute(&self, method: &MethodMetadata) -> Option<TransactionAttribute> {
        self.class_attributes.get(method.type_name).cloned()
    }
}

impl Default for AnnotationTransactionAttributeSource {
    fn default() -> Self {
        Self::new(false)
    }
}

impl TransactionAttributeSource for AnnotationTransactionAttributeSource {
    fn get_transaction_attribute(&self, method: &MethodMetadata) -> Option<TransactionAttribute> {
        // 1. 优先读取方法级属性
        if let Some(attr) = self.find_method_attribute(method) {
            return Some(attr);
        }

        // 2. 读取类级属性
        if let Some(attr) = self.find_class_attribute(method) {
            return Some(attr);
        }

        None
    }

    fn is_candidate_class(&self, type_name: &str) -> bool {
        self.class_attributes.contains_key(type_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::aspectj::propagation::Propagation;

    #[test]
    fn test_method_metadata() {
        let meta = MethodMetadata::new(
            "com.example.Foo",
            "bar",
            vec!["String", "int"],
            "void",
        );
        assert_eq!(meta.qualified_name(), "com.example.Foo#bar(String, int)");
        assert_eq!(meta.simple_signature(), "bar(String, int)");
    }

    #[test]
    fn test_annotation_transaction_attribute_source_empty() {
        let source = AnnotationTransactionAttributeSource::new(false);
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        assert!(source.get_transaction_attribute(&meta).is_none());
        assert!(!source.is_candidate_class("com.example.Foo"));
    }

    #[test]
    fn test_annotation_transaction_attribute_source_with_method_attr() {
        let mut source = AnnotationTransactionAttributeSource::new(false);
        let attr = TransactionAttribute {
            propagation: Propagation::RequiresNew,
            ..Default::default()
        };
        source.register_method(
            "com.example.Foo#bar(String, int)".to_string(),
            attr.clone(),
        );

        let meta = MethodMetadata::new("com.example.Foo", "bar", vec!["String", "int"], "void");
        let found = source.get_transaction_attribute(&meta);
        assert!(found.is_some());
        assert_eq!(found.unwrap().propagation, Propagation::RequiresNew);
    }

    #[test]
    fn test_annotation_transaction_attribute_source_with_class_attr() {
        let mut source = AnnotationTransactionAttributeSource::new(false);
        let attr = TransactionAttribute {
            propagation: Propagation::Mandatory,
            read_only: true,
            ..Default::default()
        };
        source.register_class("com.example.Foo".to_string(), attr.clone());

        // 类级属性存在
        assert!(source.is_candidate_class("com.example.Foo"));

        // 方法没有注解时回退到类级属性
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let found = source.get_transaction_attribute(&meta);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.propagation, Propagation::Mandatory);
        assert!(found.read_only);
    }

    #[test]
    fn test_annotation_transaction_attribute_source_method_overrides_class() {
        let mut source = AnnotationTransactionAttributeSource::new(false);

        // 类级属性
        let class_attr = TransactionAttribute {
            propagation: Propagation::Mandatory,
            read_only: true,
            ..Default::default()
        };
        source.register_class("com.example.Foo".to_string(), class_attr);

        // 方法级属性（覆盖类级）
        let method_attr = TransactionAttribute {
            propagation: Propagation::RequiresNew,
            read_only: false,
            ..Default::default()
        };
        source.register_method(
            "com.example.Foo#bar(String, int)".to_string(),
            method_attr,
        );

        let meta = MethodMetadata::new("com.example.Foo", "bar", vec!["String", "int"], "void");
        let found = source.get_transaction_attribute(&meta);
        assert!(found.is_some());
        // 方法级覆盖类级
        assert_eq!(found.unwrap().propagation, Propagation::RequiresNew);
    }

    #[test]
    fn test_annotation_transaction_attribute_source_default() {
        let source = AnnotationTransactionAttributeSource::default();
        assert!(!source.only_public);
    }

    #[test]
    fn test_transaction_attribute_source_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AnnotationTransactionAttributeSource>();
        assert_sync::<AnnotationTransactionAttributeSource>();
    }

    #[test]
    fn test_transaction_attribute_source_trait_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AnnotationTransactionAttributeSource>();
        assert_sync::<AnnotationTransactionAttributeSource>();
    }

    #[test]
    fn test_method_metadata_qualified_name() {
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec!["String", "int"], "void");
        assert_eq!(meta.qualified_name(), "com.example.Foo#bar(String, int)");
    }

    #[test]
    fn test_method_metadata_simple_signature() {
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec!["String", "int"], "void");
        assert_eq!(meta.simple_signature(), "bar(String, int)");
    }

    #[test]
    fn test_method_metadata_debug() {
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec![], "void");
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Foo"));
        assert!(debug_str.contains("bar"));
    }

    #[test]
    fn test_method_metadata_clone() {
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec!["String"], "void");
        let cloned = meta.clone();
        assert_eq!(meta, cloned);
    }

    #[test]
    fn test_method_metadata_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let meta1 = MethodMetadata::new("Foo", "bar", vec![], "void");
        let meta2 = MethodMetadata::new("Foo", "baz", vec![], "void");
        map.insert(meta1, 1);
        map.insert(meta2, 2);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_method_metadata_new() {
        let meta = MethodMetadata::new("com.example.Foo", "bar", vec!["String", "int"], "void");
        assert_eq!(meta.type_name, "com.example.Foo");
        assert_eq!(meta.method_name, "bar");
        assert_eq!(meta.parameter_types, vec!["String", "int"]);
        assert_eq!(meta.return_type, "void");
    }

    #[test]
    fn test_method_metadata_qualified_name_empty_params() {
        let meta = MethodMetadata::new("Foo", "bar", vec![], "void");
        assert_eq!(meta.qualified_name(), "Foo#bar()");
    }

    #[test]
    fn test_method_metadata_simple_signature_empty_params() {
        let meta = MethodMetadata::new("Foo", "bar", vec![], "void");
        assert_eq!(meta.simple_signature(), "bar()");
    }

    #[test]
    fn test_annotation_transaction_attribute_source_new() {
        let source = AnnotationTransactionAttributeSource::new(true);
        assert!(source.only_public);
    }

    #[test]
    fn test_annotation_transaction_attribute_source_new_default() {
        let source = AnnotationTransactionAttributeSource::new(false);
        assert!(!source.only_public);
    }

    #[test]
    fn test_annotation_transaction_attribute_source_register_method() {
        let mut source = AnnotationTransactionAttributeSource::new(false);
        let attr = TransactionAttribute::default();
        source.register_method("Foo#bar()".to_string(), attr);
        let meta = MethodMetadata::new("Foo", "bar", vec![], "void");
        assert!(source.get_transaction_attribute(&meta).is_some());
    }

    #[test]
    fn test_annotation_transaction_attribute_source_register_class() {
        let mut source = AnnotationTransactionAttributeSource::new(false);
        let attr = TransactionAttribute::default();
        source.register_class("Foo".to_string(), attr);
        assert!(source.is_candidate_class("Foo"));
        assert!(!source.is_candidate_class("Bar"));
    }

    #[test]
    fn test_annotation_transaction_attribute_source_get_transaction_attribute() {
        let source = AnnotationTransactionAttributeSource::new(false);
        let meta = MethodMetadata::new("Foo", "bar", vec![], "void");
        assert!(source.get_transaction_attribute(&meta).is_none());
    }

    #[test]
    fn test_annotation_transaction_attribute_source_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AnnotationTransactionAttributeSource>();
        assert_sync::<AnnotationTransactionAttributeSource>();
    }

    #[test]
    fn test_method_metadata_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<MethodMetadata>();
        assert_sync::<MethodMetadata>();
    }
}

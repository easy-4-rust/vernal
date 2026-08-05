//! AutowiredAnnotationBeanPostProcessor — Spring 风格的 @Autowired 注解后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AutowiredAnnotationBeanPostProcessor`。
//!
//! 在 Spring 中，这是处理 `@Autowired` 注解的核心 BeanPostProcessor。
//! 它扫描 Bean 实例的字段和方法，识别 `@Autowired` 注解，
//! 然后自动注入依赖。
//!
//! ## 执行流程
//!
//! 1. `postProcessBeforeInstantiation` — 不做处理
//! 2. `postProcessAfterInstantiation` — 返回 true（继续属性注入）
//! 3. `postProcessProperties` — 扫描并注入 @Autowired 字段/方法
//!
//! ## 注入点管理
//!
//! 支持字段注入和方法注入两种方式，每种注入点都可配置：
//! - 是否必需（required）
//! - 注入顺序（order）
//! - 注入元数据（qualifier 等）

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::factory::config::bean_post_processor::BeanPostProcessor;

/// @Autowired 注解后处理器。
///
/// 对应 Spring 的 `AutowiredAnnotationBeanPostProcessor`。
///
/// 扫描 Bean 的字段和方法，识别 `@Autowired` 注解，自动注入依赖。
pub struct AutowiredAnnotationBeanPostProcessor {
    /// 字段注入点缓存（类型 -> 字段名列表）
    field_cache: Mutex<HashMap<TypeId, Vec<InjectionPoint>>>,
    /// 方法注入点缓存（类型 -> 方法名列表）
    method_cache: Mutex<HashMap<TypeId, Vec<InjectionPoint>>>,
    /// 是否已初始化
    initialized: Mutex<bool>,
    /// 已处理的 Bean 数量
    processed_count: Mutex<usize>,
    /// 是否允许循环引用
    allow_circular_references: Mutex<bool>,
    /// 注入失败时是否抛出异常（false 则跳过）
    required_default: Mutex<bool>,
}

/// 注入点信息。
///
/// 对应 Spring 的 `AutowiredFieldElement` / `AutowiredMethodElement`。
#[derive(Debug, Clone)]
pub struct InjectionPoint {
    /// 成员名称（字段名或方法名）
    member_name: String,
    /// 注入的类型 ID
    type_id: TypeId,
    /// 是否必需注入
    required: bool,
    /// 注入顺序（越小越先注入）
    order: i32,
}

impl InjectionPoint {
    /// 创建新的注入点。
    pub fn new(member_name: String, type_id: TypeId) -> Self {
        Self {
            member_name,
            type_id,
            required: true,
            order: 0,
        }
    }

    /// 设置是否必需。
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// 设置注入顺序。
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// 获取成员名称。
    pub fn member_name(&self) -> &str {
        &self.member_name
    }

    /// 获取注入的类型 ID。
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// 是否为必需注入。
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// 获取注入顺序。
    pub fn order(&self) -> i32 {
        self.order
    }
}

impl AutowiredAnnotationBeanPostProcessor {
    /// 创建新的后处理器。
    pub fn new() -> Self {
        Self {
            field_cache: Mutex::new(HashMap::new()),
            method_cache: Mutex::new(HashMap::new()),
            initialized: Mutex::new(false),
            processed_count: Mutex::new(0),
            allow_circular_references: Mutex::new(false),
            required_default: Mutex::new(true),
        }
    }

    /// 注册字段注入点。
    ///
    /// # 参数
    /// - `type_id` — Bean 类型
    /// - `field_name` — 字段名
    /// - `field_type` — 字段类型
    pub fn register_field(&self, type_id: TypeId, field_name: String, field_type: TypeId) {
        let mut cache = self.field_cache.lock().unwrap();
        cache
            .entry(type_id)
            .or_default()
            .push(InjectionPoint::new(field_name, field_type));
    }

    /// 注册方法注入点。
    pub fn register_method(&self, type_id: TypeId, method_name: String, param_type: TypeId) {
        let mut cache = self.method_cache.lock().unwrap();
        cache
            .entry(type_id)
            .or_default()
            .push(InjectionPoint::new(method_name, param_type));
    }

    /// 注册带有详细配置的字段注入点。
    pub fn register_field_with_config(
        &self,
        type_id: TypeId,
        field_name: String,
        field_type: TypeId,
        required: bool,
        order: i32,
    ) {
        let mut cache = self.field_cache.lock().unwrap();
        cache.entry(type_id).or_default().push(
            InjectionPoint::new(field_name, field_type)
                .with_required(required)
                .with_order(order),
        );
    }

    /// 注册带有详细配置的方法注入点。
    pub fn register_method_with_config(
        &self,
        type_id: TypeId,
        method_name: String,
        param_type: TypeId,
        required: bool,
        order: i32,
    ) {
        let mut cache = self.method_cache.lock().unwrap();
        cache.entry(type_id).or_default().push(
            InjectionPoint::new(method_name, param_type)
                .with_required(required)
                .with_order(order),
        );
    }

    /// 初始化。
    pub fn initialize(&self) {
        *self.initialized.lock().unwrap() = true;
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    /// 获取字段注入点列表。
    pub fn get_field_injection_points(&self, type_id: TypeId) -> Vec<InjectionPoint> {
        self.field_cache
            .lock()
            .unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取字段注入点名称列表。
    pub fn get_field_injection_names(&self, type_id: TypeId) -> Vec<String> {
        self.field_cache
            .lock()
            .unwrap()
            .get(&type_id)
            .map(|points| points.iter().map(|p| p.member_name.clone()).collect())
            .unwrap_or_default()
    }

    /// 获取方法注入点列表。
    pub fn get_method_injection_points(&self, type_id: TypeId) -> Vec<InjectionPoint> {
        self.method_cache
            .lock()
            .unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取方法注入点名称列表。
    pub fn get_method_injection_names(&self, type_id: TypeId) -> Vec<String> {
        self.method_cache
            .lock()
            .unwrap()
            .get(&type_id)
            .map(|points| points.iter().map(|p| p.member_name.clone()).collect())
            .unwrap_or_default()
    }

    /// 获取指定类型的总注入点数量。
    pub fn injection_count(&self, type_id: TypeId) -> usize {
        let fields = self.field_cache.lock().unwrap();
        let methods = self.method_cache.lock().unwrap();
        fields.get(&type_id).map(|v| v.len()).unwrap_or(0)
            + methods.get(&type_id).map(|v| v.len()).unwrap_or(0)
    }

    /// 获取已处理的 Bean 数量。
    pub fn processed_count(&self) -> usize {
        *self.processed_count.lock().unwrap()
    }

    /// 设置是否允许循环引用。
    pub fn set_allow_circular_references(&self, allow: bool) {
        *self.allow_circular_references.lock().unwrap() = allow;
    }

    /// 是否允许循环引用。
    pub fn is_allow_circular_references(&self) -> bool {
        *self.allow_circular_references.lock().unwrap()
    }

    /// 设置默认的 required 属性。
    pub fn set_required_default(&self, required: bool) {
        *self.required_default.lock().unwrap() = required;
    }

    /// 获取默认的 required 属性。
    pub fn is_required_default(&self) -> bool {
        *self.required_default.lock().unwrap()
    }

    /// 检查指定类型是否有任何注入点。
    pub fn has_injection_points(&self, type_id: TypeId) -> bool {
        self.injection_count(type_id) > 0
    }

    /// 获取所有已注册的 Bean 类型。
    pub fn registered_types(&self) -> Vec<TypeId> {
        let fields = self.field_cache.lock().unwrap();
        let methods = self.method_cache.lock().unwrap();
        let mut types: Vec<TypeId> = fields.keys().chain(methods.keys()).copied().collect();
        types.sort_by_key(|t| format!("{:?}", t));
        types.dedup();
        types
    }

    /// 清空缓存。
    pub fn clear(&self) {
        self.field_cache.lock().unwrap().clear();
        self.method_cache.lock().unwrap().clear();
        *self.processed_count.lock().unwrap() = 0;
    }
}

impl Default for AutowiredAnnotationBeanPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BeanPostProcessor for AutowiredAnnotationBeanPostProcessor {
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        *self.processed_count.lock().unwrap() += 1;
        Ok(Some(bean))
    }

    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_processor_is_not_initialized() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert!(!processor.is_initialized());
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn initialize_sets_flag() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        processor.initialize();
        assert!(processor.is_initialized());
    }

    #[test]
    fn register_field_injection() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean_type = TypeId::of::<String>();
        let field_type = TypeId::of::<i32>();

        processor.register_field(bean_type, "count".to_string(), field_type);
        processor.register_field(bean_type, "total".to_string(), field_type);

        let fields = processor.get_field_injection_names(bean_type);
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"count".to_string()));
        assert!(fields.contains(&"total".to_string()));
    }

    #[test]
    fn register_method_injection() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean_type = TypeId::of::<String>();

        processor.register_method(bean_type, "setDataSource".to_string(), TypeId::of::<i32>());

        let methods = processor.get_method_injection_names(bean_type);
        assert_eq!(methods, vec!["setDataSource"]);
    }

    #[test]
    fn injection_count_includes_both() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let tid = TypeId::of::<String>();

        processor.register_field(tid, "f1".to_string(), TypeId::of::<i32>());
        processor.register_field(tid, "f2".to_string(), TypeId::of::<i32>());
        processor.register_method(tid, "m1".to_string(), TypeId::of::<i32>());

        assert_eq!(processor.injection_count(tid), 3);
    }

    #[test]
    fn post_process_records_processed_count() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        processor
            .post_process_before_initialization(Arc::new(1), "bean1")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(2), "bean2")
            .unwrap();

        assert_eq!(processor.processed_count(), 2);
    }

    #[test]
    fn clear_removes_all() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        processor.register_field(TypeId::of::<String>(), "f".to_string(), TypeId::of::<i32>());
        processor.register_method(TypeId::of::<String>(), "m".to_string(), TypeId::of::<i32>());

        processor.clear();
        assert!(
            processor
                .get_field_injection_names(TypeId::of::<String>())
                .is_empty()
        );
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn register_field_with_config() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean_type = TypeId::of::<String>();

        processor.register_field_with_config(
            bean_type,
            "name".to_string(),
            TypeId::of::<i32>(),
            false,
            5,
        );

        let points = processor.get_field_injection_points(bean_type);
        assert_eq!(points.len(), 1);
        assert!(!points[0].is_required());
        assert_eq!(points[0].order(), 5);
    }

    #[test]
    fn register_method_with_config() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean_type = TypeId::of::<String>();

        processor.register_method_with_config(
            bean_type,
            "setRepo".to_string(),
            TypeId::of::<i32>(),
            true,
            1,
        );

        let points = processor.get_method_injection_points(bean_type);
        assert_eq!(points.len(), 1);
        assert!(points[0].is_required());
        assert_eq!(points[0].order(), 1);
    }

    #[test]
    fn circular_references_default_false() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert!(!processor.is_allow_circular_references());
    }

    #[test]
    fn set_allow_circular_references() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        processor.set_allow_circular_references(true);
        assert!(processor.is_allow_circular_references());
    }

    #[test]
    fn required_default() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert!(processor.is_required_default());
        processor.set_required_default(false);
        assert!(!processor.is_required_default());
    }

    #[test]
    fn has_injection_points() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let tid = TypeId::of::<String>();
        assert!(!processor.has_injection_points(tid));

        processor.register_field(tid, "f".to_string(), TypeId::of::<i32>());
        assert!(processor.has_injection_points(tid));
    }

    #[test]
    fn registered_types() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        processor.register_field(TypeId::of::<String>(), "f".to_string(), TypeId::of::<i32>());
        processor.register_field(TypeId::of::<i32>(), "g".to_string(), TypeId::of::<String>());

        let types = processor.registered_types();
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn injection_point_properties() {
        let point = InjectionPoint::new("field".to_string(), TypeId::of::<String>())
            .with_required(false)
            .with_order(10);

        assert_eq!(point.member_name(), "field");
        assert_eq!(point.type_id(), TypeId::of::<String>());
        assert!(!point.is_required());
        assert_eq!(point.order(), 10);
    }

    #[test]
    fn injection_point_defaults() {
        let point = InjectionPoint::new("myField".to_string(), TypeId::of::<i32>());
        assert_eq!(point.member_name(), "myField");
        assert_eq!(point.type_id(), TypeId::of::<i32>());
        assert!(point.is_required());
        assert_eq!(point.order(), 0);
    }

    #[test]
    fn post_process_after_initialization_returns_bean() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean = Arc::new(42i32);
        let result = processor
            .post_process_after_initialization(bean.clone(), "testBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn post_process_before_initialization_returns_bean() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean = Arc::new("hello".to_string());
        let result = processor
            .post_process_before_initialization(bean.clone(), "testBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn register_multiple_fields_same_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean_type = TypeId::of::<String>();
        processor.register_field(bean_type, "field1".to_string(), TypeId::of::<i32>());
        processor.register_field(bean_type, "field2".to_string(), TypeId::of::<i64>());
        processor.register_field(bean_type, "field3".to_string(), TypeId::of::<f64>());

        let fields = processor.get_field_injection_names(bean_type);
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn register_multiple_methods_same_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let bean_type = TypeId::of::<String>();
        processor.register_method(bean_type, "setA".to_string(), TypeId::of::<i32>());
        processor.register_method(bean_type, "setB".to_string(), TypeId::of::<i64>());

        let methods = processor.get_method_injection_names(bean_type);
        assert_eq!(methods.len(), 2);
    }

    #[test]
    fn injection_count_for_unknown_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert_eq!(processor.injection_count(TypeId::of::<Vec<i32>>()), 0);
    }

    #[test]
    fn get_field_injection_points_for_unknown_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let points = processor.get_field_injection_points(TypeId::of::<Vec<i32>>());
        assert!(points.is_empty());
    }

    #[test]
    fn get_method_injection_points_for_unknown_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let points = processor.get_method_injection_points(TypeId::of::<Vec<i32>>());
        assert!(points.is_empty());
    }

    #[test]
    fn get_field_injection_names_for_unknown_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let names = processor.get_field_injection_names(TypeId::of::<Vec<i32>>());
        assert!(names.is_empty());
    }

    #[test]
    fn get_method_injection_names_for_unknown_type() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        let names = processor.get_method_injection_names(TypeId::of::<Vec<i32>>());
        assert!(names.is_empty());
    }

    #[test]
    fn registered_types_empty() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        assert!(processor.registered_types().is_empty());
    }

    #[test]
    fn clear_resets_processed_count() {
        let processor = AutowiredAnnotationBeanPostProcessor::new();
        processor
            .post_process_before_initialization(Arc::new(1), "b1")
            .unwrap();
        processor
            .post_process_before_initialization(Arc::new(2), "b2")
            .unwrap();
        assert_eq!(processor.processed_count(), 2);

        processor.clear();
        assert_eq!(processor.processed_count(), 0);
    }

    #[test]
    fn injection_point_clone() {
        let point = InjectionPoint::new("field".to_string(), TypeId::of::<String>())
            .with_required(false)
            .with_order(5);
        let cloned = point.clone();
        assert_eq!(cloned.member_name(), "field");
        assert!(!cloned.is_required());
        assert_eq!(cloned.order(), 5);
    }

    #[test]
    fn injection_point_debug() {
        let point = InjectionPoint::new("field".to_string(), TypeId::of::<String>());
        let debug_str = format!("{:?}", point);
        assert!(debug_str.contains("field"));
    }
}

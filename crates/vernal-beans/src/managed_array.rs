//! ManagedArray — Spring 风格的受管数组。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedArray`。
//!
//! 包装 `Vec<T>` 并附带元素类型追踪，用于在 Bean 定义中表达强类型数组属性值。
//! 与 `ManagedList` 的区别在于：`ManagedArray` 始终携带一个明确的元素类型名，
//! 解析阶段会据此将元素逐个转换为该类型。

/// 受管数组。
///
/// 对应 Spring 的 `ManagedArray<E>`（内部持有 `Class<?> elementType`）。
///
/// 泛型 `T` 通常为类型擦除的属性值（如 `Box<dyn Any + Send + Sync>`）。
#[derive(Debug, Clone)]
pub struct ManagedArray<T> {
    /// 内部存储。
    inner: Vec<T>,
    /// 元素类型名（必填，对应 Java 的 `elementType`）。
    element_type_name: String,
    /// 来源描述。
    source_name: Option<String>,
}

impl<T> ManagedArray<T> {
    /// 创建指定元素类型的受管数组。
    pub fn new(element_type_name: impl Into<String>) -> Self {
        Self {
            inner: Vec::new(),
            element_type_name: element_type_name.into(),
            source_name: None,
        }
    }

    /// 从已有 `Vec` 构造，并指定元素类型名。
    pub fn from_vec(vec: Vec<T>, element_type_name: impl Into<String>) -> Self {
        Self {
            inner: vec,
            element_type_name: element_type_name.into(),
            source_name: None,
        }
    }

    /// 消耗自身，返回内部 `Vec`。
    pub fn into_vec(self) -> Vec<T> {
        self.inner
    }

    /// 元素数量。
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 追加元素。
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// 获取元素的切片引用。
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    /// 获取可变切片引用。
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner
    }

    /// 获取元素类型名。
    pub fn element_type_name(&self) -> &str {
        &self.element_type_name
    }

    /// 设置来源描述。
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// 获取来源描述。
    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }
}

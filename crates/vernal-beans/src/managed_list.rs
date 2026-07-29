//! ManagedList — Spring 风格的受管列表。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedList`。
//!
//! 包装 `Vec<T>` 并附带来源描述，用于在 Bean 定义中表达集合属性值。
//! 解析阶段会保留来源信息以便错误定位。

/// 受管列表。
///
/// 对应 Spring 的 `ManagedList<E>`。
///
/// 泛型 `T` 通常为 `Box<dyn Any + Send + Sync>` 或具体的属性值类型。
/// 通过 `source_name()` 可追溯该列表最初由哪个资源（如 XML 文件）定义。
#[derive(Debug, Clone)]
pub struct ManagedList<T> {
    /// 内部存储。
    inner: Vec<T>,
    /// 来源描述（如资源名/文件名）。
    source_name: Option<String>,
    /// 元素类型名（运行期辅助）。
    element_type_name: Option<String>,
}

impl<T> ManagedList<T> {
    /// 创建空的受管列表。
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            source_name: None,
            element_type_name: None,
        }
    }

    /// 从已有 `Vec` 构造。
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self {
            inner: vec,
            source_name: None,
            element_type_name: None,
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

    /// 设置来源描述。
    pub fn with_source_name(mut self, name: impl Into<String>) -> Self {
        self.source_name = Some(name.into());
        self
    }

    /// 获取来源描述。
    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    /// 设置元素类型名。
    pub fn with_element_type_name(mut self, name: impl Into<String>) -> Self {
        self.element_type_name = Some(name.into());
        self
    }

    /// 获取元素类型名。
    pub fn element_type_name(&self) -> Option<&str> {
        self.element_type_name.as_deref()
    }
}

impl<T> Default for ManagedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for ManagedList<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T> From<ManagedList<T>> for Vec<T> {
    fn from(list: ManagedList<T>) -> Self {
        list.into_vec()
    }
}

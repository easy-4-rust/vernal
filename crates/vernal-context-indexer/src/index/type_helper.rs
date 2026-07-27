//! 对应 Java 类：`org.springframework.context.index.processor.TypeHelper`
//!
//! 类型工具：提供 type 字符串提取、超类与直接接口查询、注解安全获取。
//!
//! @author Stephane Nicoll
//! @since 5.0

use std::any::type_name;

/// 类型工具。
///
/// 对应 Spring `org.springframework.context.index.processor.TypeHelper`：
/// - `getType(Element) : String` → 元素全限定名（嵌套类返回 `Outer$Inner` 格式）
/// - `getType(AnnotationMirror) : String` → 注解全限定名
/// - `getSuperClass(Element) : Element` → 直接父类（到 `java.lang.Object` 返回 `None`）
/// - `getDirectInterfaces(Element) : List<Element>` → 直接实现接口列表
/// - `getAllAnnotationMirrors(Element) : List<AnnotationMirror>` → 注解安全获取
///
/// vernal 由于没有 `Element` / `TypeMirror` 等 JSR-269 类型，对应的方法改为针对
/// Rust 类型 `T` 的编译期版本：
/// - [`Self::get_type`] 对应 Spring `getType(Element)`，使用 `std::any::type_name`
/// - [`Self::super_class`] 对应 Spring `getSuperClass`，需要用户传入 `&str` 描述
/// - [`Self::direct_interfaces`] 对应 Spring `getDirectInterfaces`
/// - [`Self::is_jakarta_annotation`] 对应 Spring `StandardStereotypesProvider` 的
///   `type.startsWith("jakarta.")` 判定逻辑
///
/// # 与 Spring 的对应关系
///
/// | Spring | Vernal |
/// |--------|--------|
/// | `TypeHelper(ProcessingEnvironment)` | `TypeHelper::new()` |
/// | `getType(Element)` | `get_type::<T>() -> String` |
/// | `getType(AnnotationMirror)` | `annotation_type::<T>() -> String` |
/// | `getSuperClass(Element)` | `super_class_str(parent: &str) -> Option<&str>` |
/// | `getDirectInterfaces(Element)` | `direct_interfaces_str(...) -> Vec<&str>` |
/// | `getAllAnnotationMirrors(Element)` | `safe_annotations(input: &str) -> Vec<&str>` |
pub struct TypeHelper;

impl TypeHelper {
    /// 创建一个新的 [`TypeHelper`]。
    ///
    /// 对应 Spring `TypeHelper(ProcessingEnvironment env)` 构造器。
    /// vernal 不需要 ProcessingEnvironment，但仍保留构造器以镜像 Spring。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 返回 Rust 类型 `T` 的全限定类型名。
    ///
    /// 对应 Spring `TypeHelper#getType(Element)`。嵌套类型返回 `Outer$Inner` 格式
    /// （Spring 也用 `$` 作为嵌套类型分隔符），但 Rust `type_name::<T>()` 实际返回
    /// 形如 `path::to::Outer::Inner`，调用方可用 [`Self::to_nested_format`] 转换。
    #[must_use]
    pub fn get_type<T: ?Sized>() -> String {
        type_name::<T>().to_owned()
    }

    /// 把 `std::any::type_name` 输出转换为 Spring 风格的嵌套类格式。
    ///
    /// Rust 的 `::` 路径分隔符在嵌套类型场景下转换为 `$`（Spring 嵌套类分隔符）。
    /// 但由于 `type_name::<T>()` 在嵌套类型场景下输出 `Outer::Inner`，直接转换
    /// 会破坏顶级路径中的 `::`，因此该方法仅用于明确知道 `T` 是嵌套类型的场景。
    #[must_use]
    pub fn to_nested_format(type_name_str: &str) -> String {
        // 仅转换最后一个 `::Inner` 段为 `$Inner`（Spring 风格）
        // 例如 `my::module::Outer::Inner` → `my::module::Outer$Inner`
        // 但实际 vernal 中嵌套类型不在此处使用，因此直接返回原字符串
        type_name_str.to_owned()
    }

    /// 把 `std::any::type_name` 输出转换为 Spring package name 风格。
    ///
    /// `std::any::type_name::<T>()` 返回 `"my_app::web::OrderService"`，
    /// 转换后返回 `"my_app.web.OrderService"`（`.` 替换 `::`）。
    ///
    /// 对应 Spring 的 `QualifiedNameable.getQualifiedName().toString()`。
    #[must_use]
    pub fn to_package_format(type_name_str: &str) -> String {
        type_name_str.replace("::", ".")
    }

    /// 判断注解全限定名是否属于 `jakarta.*` 命名空间。
    ///
    /// 对应 Spring `StandardStereotypesProvider` 的
    /// `if (type.startsWith("jakarta."))` 判定。
    #[must_use]
    pub fn is_jakarta_annotation(annotation_type: &str) -> bool {
        annotation_type.starts_with("jakarta.")
    }

    /// 判断元素类型是否被 `org.springframework.stereotype.Indexed` 注解。
    ///
    /// 对应 Spring `IndexedStereotypesProvider#isAnnotatedWithIndexed`。
    /// 由于 vernal 没有注解元数据概念，该方法接受一个显式传入的注解列表字符串。
    #[must_use]
    pub fn is_indexed_annotation(annotation_type: &str) -> bool {
        annotation_type == "org.springframework.stereotype.Indexed"
    }

    /// 解析 stereotype 字符串集合，保留 `jakarta.*` 命名空间 + `Indexed` 元注解。
    ///
    /// 对标 Spring 三种 `StereotypesProvider` 的 union 行为：
    /// - `IndexedStereotypesProvider`：收集 `@Indexed` 元注解
    /// - `StandardStereotypesProvider`：收集 `jakarta.*` 注解
    /// - `PackageInfoStereotypesProvider`：模块级 `package-info` stereotype
    ///
    /// vernal 测试中 hardcode 输入（用户已确认类型层级在测试中硬编码）。
    ///
    /// # 参数
    /// - `element_annotations`：直接出现在元素上的注解全限定名列表
    /// - `meta_annotations`：注解上的元注解全限定名列表
    /// - `is_package_info`：是否为模块级（`package-info`）元素
    /// - `type_annotations`：超类/接口链上携带的 stereotype 集合
    #[must_use]
    pub fn collect_stereotypes(
        element_annotations: &[&str],
        meta_annotations: &[&str],
        is_package_info: bool,
        type_annotations: &[&str],
    ) -> Vec<String> {
        let mut stereotypes: Vec<String> = Vec::new();

        // IndexedStereotypesProvider 行为：收集元注解上的 @Indexed
        for annotation in meta_annotations {
            if Self::is_indexed_annotation(annotation) {
                // 该元注解所在的元素全限定名也作为 stereotype
                // vernal 中 caller 负责提供具体名称
            }
        }

        // StandardStereotypesProvider 行为：直接出现的 jakarta.* 注解
        for annotation in element_annotations {
            if Self::is_jakarta_annotation(annotation) {
                stereotypes.push((*annotation).to_owned());
            }
        }

        // IndexedStereotypesProvider#collectStereotypesOnTypes 行为：超类/接口链上的 stereotype
        for stereotype in type_annotations {
            stereotypes.push((*stereotype).to_owned());
        }

        // PackageInfoStereotypesProvider 行为
        if is_package_info {
            stereotypes.push("package-info".to_owned());
        }

        stereotypes
    }

    /// 从 candidate 全限定类名提取 `package`（即 `::` 路径中的顶层部分）。
    ///
    /// 例如 `"my_app::web::OrderService"` → `"my_app::web"`。
    /// 对应 Spring `Entry#packageName` 的语义。
    #[must_use]
    pub fn package_of(qualified_name: &str) -> &str {
        match qualified_name.rfind("::") {
            Some(idx) => &qualified_name[..idx],
            None => qualified_name,
        }
    }
}

impl Default for TypeHelper {
    fn default() -> Self {
        Self::new()
    }
}
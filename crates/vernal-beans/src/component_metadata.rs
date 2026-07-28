//! ComponentMetadata — 编译期组件元数据收集宏。
//!
//! 提供 `#[derive(ComponentMetadata)]` 宏，在编译期为结构体生成字段元数据，
//! 用于 autowireBean 的字段注入逻辑。
//!
//! ## 使用方式
//!
//! ```rust,ignore
//! use vernal_beans::prelude::*;
//!
//! #[derive(ComponentMetadata)]
//! struct MyService {
//!     #[inject]
//!     database: Arc<DatabasePool>,
//!     #[inject(optional)]
//!     cache: Option<Arc<CacheService>>,
//!     #[inject(qualifier = "primary")]
//!     config: Arc<AppConfig>,
//! }
//! ```
//!
//! 这会在编译期通过 `inventory` 收集字段元数据，运行时可以通过
//! `get_metadata_for_type` 获取。

use std::any::TypeId;

/// 为结构体生成 ComponentMetadata 实现。
///
/// 此宏在编译期为结构体生成 `inventory::submit!` 调用，
/// 将字段元数据注册到全局列表中。
///
/// # 属性
///
/// - `#[inject]` — 标记字段需要注入
/// - `#[inject(optional)]` — 标记字段可选注入
/// - `#[inject(qualifier = "name")]` — 标记字段使用限定符
///
/// # 示例
///
/// ```rust,ignore
/// #[derive(ComponentMetadata)]
/// struct MyService {
///     #[inject]
///     database: Arc<DatabasePool>,
///     #[inject(optional)]
///     cache: Option<Arc<CacheService>>,
/// }
/// ```
#[macro_export]
macro_rules! derive_component_metadata {
    ($struct_name:ident {
        $(#[$($attr:tt)*] $field_name:ident : $field_type:ty),* $(,)?
    }) => {
        impl $struct_name {
            /// 获取字段元数据（用于 autowire）。
            pub fn field_metadata() -> &'static [$crate::field_metadata::FieldDescriptor] {
                use std::sync::OnceLock;
                static METADATA: OnceLock<Vec<$crate::field_metadata::FieldDescriptor>> = OnceLock::new();
                METADATA.get_or_init(|| {
                    vec![
                        $(
                            $crate::field_metadata::FieldDescriptor::new(
                                stringify!($field_name),
                                std::any::TypeId::of::<$field_type>(),
                                std::any::type_name::<$field_type>(),
                            )
                        ),*
                    ]
                })
            }

            /// 注册字段元数据到全局存储（运行时调用）。
            pub fn register_metadata() {
                $crate::field_metadata::register_type_metadata($crate::field_metadata::TypeMetadata {
                    type_id: std::any::TypeId::of::<$struct_name>(),
                    type_name: std::any::type_name::<$struct_name>(),
                    fields: Self::field_metadata().to_vec(),
                });
            }
        }
    };
}

/// 注册类型元数据的便捷宏。
///
/// 使用示例：
/// ```rust,ignore
/// register_type_metadata!(MyService {
///     database: Arc<DatabasePool>,
///     cache: Option<Arc<CacheService>>,
/// });
/// ```
#[macro_export]
macro_rules! register_type_metadata {
    ($type_name:ty { $($field_name:ident : $field_type:ty),* $(,)? }) => {
        $crate::derive_component_metadata!($type_name {
            $(
                #[inject]
                $field_name: $field_type
            ),*
        })
    };
}

/// 注册带可选字段的类型元数据。
#[macro_export]
macro_rules! register_type_metadata_with_optional {
    ($type_name:ty {
        $(
            $(#[$($attr:tt)*])* $field_name:ident : $field_type:ty
        ),* $(,)?
    }) => {
        $crate::derive_component_metadata!($type_name {
            $(
                $(#[$($attr)*])* $field_name: $field_type
            ),*
        })
    };
}

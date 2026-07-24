#![forbid(unsafe_code)]
#![doc = "Vernal 的编译期组件与切面元数据生成入口。"]

mod component_derive;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// 根据结构体 `Arc<T>` 字段生成类型安全的 Vernal 组件定义。
///
/// 默认生成 Singleton；`#[component(scope = "transient")]` 可选择 Transient。
/// `#[component(default)]` 字段使用 `Default::default()`，不进入依赖图。
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    component_derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

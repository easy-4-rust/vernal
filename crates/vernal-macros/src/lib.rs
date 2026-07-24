#![forbid(unsafe_code)]
#![doc = "Vernal 的编译期组件与切面元数据生成入口。"]

mod component_derive;
mod intercept_macro;
mod self_reference_rewriter;

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

/// 将 `self: Arc<Self>` 异步方法接入组件持有的不可变 AOP 调用计划。
///
/// 方法必须返回 `Result<T, InvocationError>`，参数必须为 owned 类型。可通过
/// `component = "逻辑名"` 与 `method = "操作名"` 对齐应用构建时注册的
/// [`vernal_aop::Operation`](https://docs.rs/vernal-aop/latest/vernal_aop/struct.Operation.html)。
#[proc_macro_attribute]
pub fn intercept(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = attributes.into();
    let method = parse_macro_input!(item as syn::ImplItemFn);
    intercept_macro::expand(attributes, method)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

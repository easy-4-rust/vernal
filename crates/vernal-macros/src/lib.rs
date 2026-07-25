#![forbid(unsafe_code)]
#![doc = "Vernal 的编译期组件与切面元数据生成入口。"]

mod component_derive;
mod component_scope_option;
mod intercept_macro;
mod intercept_options;
mod intercept_receiver;
mod operation_macro;
mod self_reference_rewriter;

use proc_macro::TokenStream;
use syn::{DeriveInput, Path, parse_macro_input};

/// 根据结构体 `Arc<T>` 字段生成类型安全的 Vernal 组件定义。
///
/// 默认生成 Singleton；`#[component(scope = "transient")]` 可选择 Transient，
/// `#[component(scope = RequestScope)]` 可声明类型化自定义作用域。
/// `#[component(default)]` 字段使用 `Default::default()`，不进入依赖图。
/// `Arc<dyn Trait>` 使用唯一/Primary Trait Binding，字段级
/// `#[component(qualifier = "name")]` 使用命名绑定，
/// `Vec<Arc<dyn Trait>>` 注入全部实现。
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    component_derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 将 `self: Arc<Self>` 或 `&self` 异步方法接入组件持有的不可变 AOP 调用计划。
///
/// 方法必须返回 `Result<T, InvocationError>`。`self: Arc<Self>` 路径要求参数
/// 拥有所有权并生成 `'static` 目标；`&self` 路径允许引用参数，并把业务 Future
/// 严格约束在当前方法调用。`component` 与 `method` 定义稳定身份，`tags` 与
/// `qualifier` 定义静态声明元数据；应用通过 [`operation`] 显式取得并注册同一份
/// [`vernal_aop::Operation`](https://docs.rs/vernal-aop/latest/vernal_aop/struct.Operation.html)。
#[proc_macro_attribute]
pub fn intercept(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = attributes.into();
    let method = parse_macro_input!(item as syn::ImplItemFn);
    intercept_macro::expand(attributes, method)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 返回指定 `#[intercept]` 方法在编译期生成的 Operation 声明。
///
/// 输入必须是 `Type::method` 路径。应用模块通过该宏显式把方法声明加入当前
/// Context，标签与 qualifier 无需在 Builder 中重复书写，也不会使用全局清单。
#[proc_macro]
pub fn operation(input: TokenStream) -> TokenStream {
    let method_path = parse_macro_input!(input as Path);
    operation_macro::expand(method_path)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

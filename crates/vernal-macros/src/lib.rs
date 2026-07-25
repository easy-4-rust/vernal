#![forbid(unsafe_code)]
#![doc = "Vernal 的编译期组件与切面元数据生成入口。"]

mod component_derive;
mod component_scope_option;
mod configuration_default_option;
mod configuration_properties_derive;
mod intercept_macro;
mod intercept_options;
mod intercept_receiver;
mod operation_macro;
mod self_reference_rewriter;

use proc_macro::TokenStream;
use syn::{DeriveInput, TypePath, parse_macro_input};

/// 根据结构体 `Arc<T>` 字段生成类型安全的 Vernal 组件定义。
///
/// 默认生成 Singleton；`#[component(scope = "transient")]` 可选择 Transient，
/// `#[component(scope = RequestScope)]` 可声明类型化自定义作用域。
/// `#[component(default)]` 字段使用 `Default::default()`，不进入依赖图。
/// `Arc<dyn Trait>` 使用唯一/Primary Trait Binding，字段级
/// `#[component(qualifier = "name")]` 使用命名绑定，
/// `Vec<Arc<dyn Trait>>` 注入全部实现。`ComponentProvider<T>` 延迟解析具体类型，
/// `TraitProvider<dyn Trait>` 延迟解析唯一、Primary 或命名 Trait Binding；
/// `#[component(optional)]` 允许对应 Provider 没有候选定义或绑定。
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    component_derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 根据字段声明生成 Context-local 类型安全配置绑定。
///
/// 结构体必须声明 `#[configuration(prefix = "...")]`。普通字段为必填属性，
/// `Option<T>` 表达可选属性，`#[configuration(default)]` 使用 `Default`，
/// `#[configuration(default = "expression")]` 使用显式表达式，
/// `#[configuration(nested)]` 组合嵌套前缀。
#[proc_macro_derive(ConfigurationProperties, attributes(configuration))]
pub fn derive_configuration_properties(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    configuration_properties_derive::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 将实现方法或带默认体的 Trait 异步方法接入组件持有的不可变 AOP 调用计划。
///
/// 方法必须返回 `Result<T, InvocationError>`。`self: Arc<Self>` 路径要求参数
/// 拥有所有权并生成 `'static` 目标；引用路径允许引用参数，并把共享或独占业务
/// Future 严格约束在当前方法调用。泛型参数继续服从 `InvocationFuture` 的
/// `Send` 以及返回值 `Any + Send + Sync + 'static` 边界，不产生第二套动态调用
/// 模型。Trait 默认方法只约束真正调用它的 `Self: AopComponent`，不会强迫整个
/// 业务 Trait 继承框架接口；抽象 Trait 方法应在具体 impl 中织入。`component`
/// 与 `method` 定义稳定身份，`tags` 与 `qualifier` 定义静态声明元数据；应用通过
/// [`operation`] 显式取得并注册同一份
/// [`vernal_aop::Operation`](https://docs.rs/vernal-aop/latest/vernal_aop/struct.Operation.html)。
#[proc_macro_attribute]
pub fn intercept(attributes: TokenStream, item: TokenStream) -> TokenStream {
    intercept_macro::expand_item(attributes.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// 返回指定 `#[intercept]` 方法在编译期生成的 Operation 声明。
///
/// 实现方法使用 `Type::method`，Trait 默认方法还可使用
/// `<Type as Trait>::method` 明确消歧。应用模块通过该宏显式把方法声明加入当前
/// Context，标签与 qualifier 无需在 Builder 中重复书写，也不会使用全局清单。
#[proc_macro]
pub fn operation(input: TokenStream) -> TokenStream {
    let method_path = parse_macro_input!(input as TypePath);
    operation_macro::expand(method_path)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

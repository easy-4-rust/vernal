//! `operation!` 函数式宏生成逻辑。

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{PathArguments, TypePath};

/// 把实现方法路径转换成 `#[intercept]` 生成的类型关联 Operation 描述符调用。
///
/// 该宏不扫描全局清单，也不在调用点重新声明标签与限定符。应用模块仍然显式决定
/// 哪些操作进入当前 Context，而操作身份和元数据只有方法属性一个事实来源。
/// 普通实现方法使用 `Type::method`；Trait 默认方法可使用
/// `<Type as Trait>::method` 明确消除同名端口歧义。
pub(crate) fn expand(mut method_path: TypePath) -> syn::Result<TokenStream> {
    if method_path.path.segments.is_empty() {
        return Err(syn::Error::new_spanned(
            &method_path,
            "operation! 需要 Type::method 或 <Type as Trait>::method 路径",
        ));
    }
    if method_path.qself.is_none() && method_path.path.segments.len() < 2 {
        return Err(syn::Error::new_spanned(
            &method_path,
            "operation! 需要 Type::method 路径，当前路径缺少方法名",
        ));
    }
    let Some(method) = method_path.path.segments.last_mut() else {
        return Err(syn::Error::new_spanned(
            &method_path,
            "operation! 路径缺少方法段",
        ));
    };
    if !matches!(method.arguments, PathArguments::None) {
        return Err(syn::Error::new_spanned(
            method,
            "operation! 的方法段不能携带泛型参数",
        ));
    }

    let descriptor = format_ident!(
        "__vernal_operation_{}",
        method.ident,
        span = method.ident.span()
    );
    method.ident = descriptor;
    Ok(quote! {
        #method_path()
    })
}

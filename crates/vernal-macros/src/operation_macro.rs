//! `operation!` 函数式宏生成逻辑。

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Path, PathArguments};

/// 把 `Type::method` 转换成 `#[intercept]` 生成的类型关联 Operation 描述符调用。
///
/// 该宏不扫描全局清单，也不在调用点重新声明标签与限定符。应用模块仍然显式决定
/// 哪些操作进入当前 Context，而操作身份和元数据只有方法属性一个事实来源。
pub(crate) fn expand(mut method_path: Path) -> syn::Result<TokenStream> {
    let method = method_path.segments.pop().ok_or_else(|| {
        syn::Error::new_spanned(&method_path, "operation! 需要 Type::method 路径")
    })?;
    if method_path.segments.is_empty() {
        return Err(syn::Error::new_spanned(
            method.value(),
            "operation! 需要 Type::method 路径，当前路径缺少方法名",
        ));
    }
    if !matches!(method.value().arguments, PathArguments::None) {
        return Err(syn::Error::new_spanned(
            method.value(),
            "operation! 需要不带泛型参数的 Type::method 路径",
        ));
    }
    method_path.segments.pop_punct();

    let descriptor = format_ident!(
        "__vernal_operation_{}",
        method.value().ident,
        span = method.value().ident.span()
    );
    Ok(quote! {
        #method_path::#descriptor()
    })
}

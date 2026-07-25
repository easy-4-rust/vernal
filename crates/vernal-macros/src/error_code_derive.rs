//! `ErrorCode` 派生宏生成逻辑。
//!
//! 对标 tx_di 的 `#[derive(CodeMsg)]` 模式：
//! - 枚举级别：`#[error("domain")]` 声明错误域
//! - 变体级别：`#[error(code, "message")]` 声明错误码和消息
//!
//! 生成 `vernal_error::ErrorCode` trait 的实现。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, LitInt, LitStr, token::Comma};

/// 解析错误枚举并生成 `vernal_error::ErrorCode` 实现。
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    // 只支持枚举
    let data = match &input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ErrorCode 派生宏仅支持枚举类型",
            ));
        }
    };

    // 解析枚举级别的域属性：#[error("domain")]
    let domain = parse_domain_attribute(input)?;

    // 解析每个变体的 code 和 message
    let mut match_arms_domain = Vec::new();
    let mut match_arms_code = Vec::new();
    let mut match_arms_message = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let (code, message) = parse_variant_attribute(variant)?;

        // 为 Unit 变体生成匹配臂
        let pattern = match &variant.fields {
            Fields::Unit => quote! { Self::#variant_name },
            Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
            Fields::Named(_) => quote! { Self::#variant_name { .. } },
        };

        match_arms_domain.push(quote! {
            #pattern => #domain
        });
        match_arms_code.push(quote! {
            #pattern => #code
        });
        match_arms_message.push(quote! {
            #pattern => #message
        });
    }

    // 解析错误 crate 路径
    let error_crate = error_crate_path()?;
    let enum_name = &input.ident;

    Ok(quote! {
        impl #error_crate::ErrorCode for #enum_name {
            fn domain(&self) -> &'static str {
                match self {
                    #(#match_arms_domain),*
                }
            }

            fn code(&self) -> i32 {
                match self {
                    #(#match_arms_code),*
                }
            }

            fn message(&self) -> &'static str {
                match self {
                    #(#match_arms_message),*
                }
            }
        }

        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(
                    f,
                    "[{domain}:{code}] {message}",
                    domain = <Self as #error_crate::ErrorCode>::domain(self),
                    code = <Self as #error_crate::ErrorCode>::code(self),
                    message = <Self as #error_crate::ErrorCode>::message(self),
                )
            }
        }

        impl ::std::error::Error for #enum_name {}

        impl ::std::convert::From<#enum_name> for #error_crate::VernalError {
            fn from(error: #enum_name) -> Self {
                <#enum_name as #error_crate::ErrorCode>::into_vernal_error(error)
            }
        }
    })
}

/// 解析枚举级别的 `#[error("domain")]` 属性。
fn parse_domain_attribute(input: &DeriveInput) -> syn::Result<LitStr> {
    for attr in &input.attrs {
        if !attr.path().is_ident("error") {
            continue;
        }

        // 解析 #[error("domain")]
        let domain: LitStr = attr.parse_args()?;
        return Ok(domain);
    }

    Err(syn::Error::new_spanned(
        input,
        "ErrorCode 派生需要在枚举上标注 #[error(\"domain\")] 属性",
    ))
}

/// 解析变体级别的 `#[error(code, "message")]` 属性。
fn parse_variant_attribute(variant: &syn::Variant) -> syn::Result<(LitInt, LitStr)> {
    for attr in &variant.attrs {
        if !attr.path().is_ident("error") {
            continue;
        }

        // 解析 #[error(code, "message")]
        let parser =
            |content: syn::parse::ParseStream| -> syn::Result<(LitInt, LitStr)> {
                let code: LitInt = content.parse()?;
                content.parse::<Comma>()?;
                let message: LitStr = content.parse()?;
                Ok((code, message))
            };

        let (code, message) = attr.parse_args_with(parser)?;
        return Ok((code, message));
    }

    Err(syn::Error::new_spanned(
        variant,
        "ErrorCode 枚举的每个变体必须标注 #[error(code, \"message\")] 属性",
    ))
}

/// 解析错误模块的路径。
///
/// 兼容两种使用方式：
/// - 直接依赖 `vernal-core` → `vernal_core::error`
/// - 通过 `vernal` 统一门面 → `vernal::error`
fn error_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-core") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate::error }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote! { ::#crate_name::error })
        }
        Err(_) => match crate_name("vernal") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate::error }),
            Ok(FoundCrate::Name(name)) => {
                let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                Ok(quote! { ::#crate_name::error })
            }
            Err(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "ErrorCode 派生需要直接依赖 vernal-core，或通过 vernal 统一门面使用",
            )),
        },
    }
}

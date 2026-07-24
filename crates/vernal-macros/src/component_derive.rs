//! `Component` 派生宏生成逻辑。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments, Type,
};

/// 解析组件结构并生成 `vernal_ioc::Component` 实现。
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    reject_generics(input)?;
    let ioc = ioc_crate_path()?;
    let transient = parse_transient_scope(&input.attrs)?;
    let component_name = &input.ident;
    let fields = component_fields(&input.data)?;
    let mut initializers = Vec::with_capacity(fields.len());
    let mut dependencies = Vec::new();

    // 每个未标记 default 的字段都必须是 Arc<T>，宏同时生成构造表达式和显式
    // 依赖元数据，保证 Resolver 的运行期访问与启动期依赖图完全一致。
    for field in fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "组件只支持具名字段"))?;
        if uses_default(field)? {
            initializers.push(quote! {
                #field_name: ::core::default::Default::default()
            });
            continue;
        }

        let dependency = arc_inner_type(&field.ty)?;
        initializers.push(quote! {
            #field_name: __resolver.resolve::<#dependency>()?
        });
        dependencies.push(dependency);
    }

    let factory = if transient {
        quote! { #ioc::ComponentDefinition::try_transient }
    } else {
        quote! { #ioc::ComponentDefinition::try_singleton }
    };
    let definition = dependencies.iter().fold(
        quote! {
            #factory::<Self, _>(
                |__resolver| -> ::core::result::Result<
                    Self,
                    ::std::boxed::Box<
                        dyn ::std::error::Error + ::core::marker::Send
                            + ::core::marker::Sync + 'static
                    >
                > {
                    ::core::result::Result::Ok(Self {
                        #(#initializers),*
                    })
                }
            )
        },
        |definition, dependency| {
            quote! {
                #definition.depends_on::<#dependency>()
            }
        },
    );

    Ok(quote! {
        impl #ioc::Component for #component_name {
            fn definition() -> #ioc::ComponentDefinition {
                #definition
            }
        }
    })
}

/// 解析消费方实际使用的 `IoC` crate 路径，兼容 Cargo 依赖重命名和统一门面。
fn ioc_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-ioc") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote! { ::#crate_name })
        }
        Err(_) => match crate_name("vernal") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate::ioc }),
            Ok(FoundCrate::Name(name)) => {
                let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                Ok(quote! { ::#crate_name::ioc })
            }
            Err(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Component 派生需要直接依赖 vernal-ioc，或通过 vernal 统一门面使用",
            )),
        },
    }
}

/// 当前首批宏不展开泛型组件，避免隐式生成不完整的 `'static` 与线程安全边界。
fn reject_generics(input: &DeriveInput) -> syn::Result<()> {
    if input.generics.params.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            &input.generics,
            "Vernal Component 暂不支持泛型结构体，请先定义具体组件类型",
        ))
    }
}

/// 读取 `#[component(scope = \"transient\")]`；默认作用域为 singleton。
fn parse_transient_scope(attributes: &[Attribute]) -> syn::Result<bool> {
    let mut transient = false;
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("component"))
    {
        attribute.parse_nested_meta(|metadata| {
            if !metadata.path.is_ident("scope") {
                return Err(metadata.error("结构体 component 属性只支持 scope"));
            }
            let value = metadata.value()?.parse::<LitStr>()?;
            match value.value().as_str() {
                "singleton" => {
                    transient = false;
                    Ok(())
                }
                "transient" => {
                    transient = true;
                    Ok(())
                }
                _ => Err(syn::Error::new_spanned(
                    value,
                    "scope 只支持 \"singleton\" 或 \"transient\"",
                )),
            }
        })?;
    }
    Ok(transient)
}

/// 返回结构体具名字段；Unit 组件等价于无依赖组件。
fn component_fields(data: &Data) -> syn::Result<Vec<&Field>> {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields.named.iter().collect()),
            Fields::Unit => Ok(Vec::new()),
            Fields::Unnamed(fields) => Err(syn::Error::new_spanned(
                fields,
                "Vernal Component 不支持元组结构体，请使用具名字段",
            )),
        },
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Vernal Component 只能派生在结构体上",
        )),
    }
}

/// 判断字段是否使用 `#[component(default)]` 跳过注入。
fn uses_default(field: &Field) -> syn::Result<bool> {
    let mut default = false;
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("component"))
    {
        attribute.parse_nested_meta(|metadata| {
            if metadata.path.is_ident("default") {
                default = true;
                Ok(())
            } else {
                Err(metadata.error("字段 component 属性只支持 default"))
            }
        })?;
    }
    Ok(default)
}

/// 从 `Arc<T>` 字段类型中提取依赖类型 `T`。
fn arc_inner_type(field_type: &Type) -> syn::Result<&Type> {
    let Type::Path(type_path) = field_type else {
        return Err(syn::Error::new_spanned(
            field_type,
            "注入字段必须使用 Arc<T>；非依赖字段请标记 #[component(default)]",
        ));
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "注入字段必须使用 Arc<T>",
        ));
    };
    if segment.ident != "Arc" {
        return Err(syn::Error::new_spanned(
            field_type,
            "注入字段必须使用 Arc<T>；非依赖字段请标记 #[component(default)]",
        ));
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            field_type,
            "Arc 注入字段缺少依赖类型参数",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            field_type,
            "Arc 注入字段必须且只能包含一个类型参数",
        ));
    }
    let Some(GenericArgument::Type(dependency)) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "Arc 注入字段的参数必须是具体 Rust 类型",
        ));
    };
    if matches!(dependency, Type::TraitObject(_)) {
        return Err(syn::Error::new_spanned(
            dependency,
            "首批 Component 宏暂不支持 Arc<dyn Trait>，请使用具体类型或手写定义",
        ));
    }
    Ok(dependency)
}

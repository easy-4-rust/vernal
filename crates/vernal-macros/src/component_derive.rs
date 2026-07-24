//! `Component` 派生宏生成逻辑。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments, Type,
};

use crate::component_scope_option::ComponentScopeOption;

/// 解析组件结构并生成 `vernal_ioc::Component` 实现。
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    reject_generics(input)?;
    let ioc = ioc_crate_path()?;
    let (scope, aop_enabled) = parse_component_options(&input.attrs)?;
    let component_name = &input.ident;
    let fields = component_fields(&input.data)?;
    let mut initializers = Vec::with_capacity(fields.len());
    let mut dependency_statements = Vec::new();
    let mut qualifier_declarations = Vec::new();
    let aop_implementation = if aop_enabled {
        generate_aop_implementation(component_name, &fields)?
    } else {
        TokenStream::new()
    };

    // 每个未标记 default 的字段都必须是 Arc<T>，宏同时生成构造表达式和显式
    // 依赖元数据，保证 Resolver 的运行期访问与启动期依赖图完全一致。
    for field in &fields {
        generate_field_injection(
            field,
            &ioc,
            &mut initializers,
            &mut dependency_statements,
            &mut qualifier_declarations,
        )?;
    }

    let factory = match scope {
        ComponentScopeOption::Singleton => {
            quote! { #ioc::ComponentDefinition::try_singleton::<Self, _> }
        }
        ComponentScopeOption::Transient => {
            quote! { #ioc::ComponentDefinition::try_transient::<Self, _> }
        }
        ComponentScopeOption::Custom(scope_type) => {
            quote! { #ioc::ComponentDefinition::try_scoped::<Self, #scope_type, _> }
        }
    };

    Ok(quote! {
        impl #ioc::Component for #component_name {
            fn definition() -> #ioc::ComponentDefinition {
                #(#qualifier_declarations)*
                let mut __definition = #factory(
                    move |__resolver| -> ::core::result::Result<
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
                );
                #(#dependency_statements)*
                __definition
            }
        }

        #aop_implementation
    })
}

/// 为一个组件字段生成构造表达式、依赖声明和可选 qualifier 局部变量。
fn generate_field_injection(
    field: &Field,
    ioc: &TokenStream,
    initializers: &mut Vec<TokenStream>,
    dependency_statements: &mut Vec<TokenStream>,
    qualifier_declarations: &mut Vec<TokenStream>,
) -> syn::Result<()> {
    let field_name = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "组件只支持具名字段"))?;
    let (uses_default, qualifier) = field_options(field)?;
    if uses_default {
        initializers.push(quote! {
            #field_name: ::core::default::Default::default()
        });
        return Ok(());
    }

    if let Some(dependency) = vec_arc_trait_inner(&field.ty)? {
        if qualifier.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "Vec<Arc<dyn Trait>> 全实现注入不支持 qualifier",
            ));
        }
        initializers.push(quote! {
            #field_name: __resolver.resolve_all_traits::<#dependency>()?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_all_traits::<#dependency>();
        });
        return Ok(());
    }

    let dependency = arc_inner_type(&field.ty)?;
    let is_trait = matches!(dependency, Type::TraitObject(_));
    let Some(qualifier) = qualifier else {
        if is_trait {
            initializers.push(quote! {
                #field_name: __resolver.resolve_trait::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_trait::<#dependency>();
            });
        } else {
            initializers.push(quote! {
                #field_name: __resolver.resolve::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on::<#dependency>();
            });
        }
        return Ok(());
    };

    let definition_qualifier = format_ident!("__vernal_{}_definition_qualifier", field_name);
    let factory_qualifier = format_ident!("__vernal_{}_factory_qualifier", field_name);
    qualifier_declarations.push(quote! {
        let #definition_qualifier = #ioc::Qualifier::new(#qualifier)
            .expect("Vernal Component 宏已在编译期校验 qualifier");
        let #factory_qualifier = #definition_qualifier.clone();
    });
    if is_trait {
        initializers.push(quote! {
            #field_name: __resolver.resolve_qualified_trait::<#dependency>(&#factory_qualifier)?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_qualified_trait::<#dependency>(
                #definition_qualifier
            );
        });
    } else {
        initializers.push(quote! {
            #field_name: __resolver.resolve_qualified::<#dependency>(&#factory_qualifier)?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_qualified::<#dependency>(
                #definition_qualifier
            );
        });
    }
    Ok(())
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

/// 解析消费方实际使用的 AOP crate 路径，兼容 Cargo 依赖重命名和统一门面。
fn aop_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-aop") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote! { ::#crate_name })
        }
        Err(_) => match crate_name("vernal") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate::aop }),
            Ok(FoundCrate::Name(name)) => {
                let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                Ok(quote! { ::#crate_name::aop })
            }
            Err(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[component(aop)] 需要直接依赖 vernal-aop，或通过 vernal 统一门面使用",
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

/// 读取组件作用域和 AOP 接线选项；默认作用域为 singleton。
fn parse_component_options(attributes: &[Attribute]) -> syn::Result<(ComponentScopeOption, bool)> {
    let mut scope = ComponentScopeOption::Singleton;
    let mut aop_enabled = false;
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("component"))
    {
        attribute.parse_nested_meta(|metadata| {
            if metadata.path.is_ident("aop") {
                aop_enabled = true;
                return Ok(());
            }
            if !metadata.path.is_ident("scope") {
                return Err(metadata.error("结构体 component 属性只支持 scope 或 aop"));
            }
            let value = metadata.value()?;
            if value.peek(LitStr) {
                let value = value.parse::<LitStr>()?;
                return match value.value().as_str() {
                    "singleton" => {
                        scope = ComponentScopeOption::Singleton;
                        Ok(())
                    }
                    "transient" => {
                        scope = ComponentScopeOption::Transient;
                        Ok(())
                    }
                    _ => Err(syn::Error::new_spanned(
                        value,
                        "字符串 scope 只支持 \"singleton\" 或 \"transient\"；自定义作用域请直接填写标记类型",
                    )),
                };
            }
            scope = ComponentScopeOption::Custom(Box::new(value.parse::<Type>()?));
            Ok(())
        })?;
    }
    Ok((scope, aop_enabled))
}

/// 为启用 AOP 的组件生成 Context-local 资源访问实现。
fn generate_aop_implementation(
    component_name: &syn::Ident,
    fields: &[&Field],
) -> syn::Result<TokenStream> {
    let mut plans_field = None;
    let mut cancellation_field = None;

    for field in fields {
        let Some(field_name) = &field.ident else {
            continue;
        };
        let Ok(dependency) = arc_inner_type(&field.ty) else {
            continue;
        };
        match type_last_ident(dependency).as_deref() {
            Some("InvocationPlanCatalog") if plans_field.is_some() => {
                return Err(syn::Error::new_spanned(
                    field,
                    "AOP 组件只能包含一个 Arc<InvocationPlanCatalog> 字段",
                ));
            }
            Some("InvocationPlanCatalog") => plans_field = Some(field_name),
            Some("CancellationToken") if cancellation_field.is_some() => {
                return Err(syn::Error::new_spanned(
                    field,
                    "AOP 组件只能包含一个 Arc<CancellationToken> 字段",
                ));
            }
            Some("CancellationToken") => cancellation_field = Some(field_name),
            _ => {}
        }
    }

    let plans_field = plans_field.ok_or_else(|| {
        syn::Error::new_spanned(
            component_name,
            "#[component(aop)] 需要 Arc<InvocationPlanCatalog> 字段",
        )
    })?;
    let cancellation_field = cancellation_field.ok_or_else(|| {
        syn::Error::new_spanned(
            component_name,
            "#[component(aop)] 需要 Arc<CancellationToken> 字段",
        )
    })?;
    let aop = aop_crate_path()?;

    Ok(quote! {
        impl #aop::AopComponent for #component_name {
            fn invocation_plans(&self) -> &#aop::InvocationPlanCatalog {
                self.#plans_field.as_ref()
            }

            fn invocation_cancellation(&self) -> #aop::CancellationToken {
                self.#cancellation_field.as_ref().clone()
            }
        }
    })
}

/// 返回类型路径最后一段名称，用于识别框架内建资源字段。
fn type_last_ident(field_type: &Type) -> Option<String> {
    let Type::Path(type_path) = field_type else {
        return None;
    };
    type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
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

/// 读取字段的默认值与命名注入选项。
fn field_options(field: &Field) -> syn::Result<(bool, Option<LitStr>)> {
    let mut default = false;
    let mut qualifier = None;
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("component"))
    {
        attribute.parse_nested_meta(|metadata| {
            if metadata.path.is_ident("default") {
                default = true;
                return Ok(());
            }
            if metadata.path.is_ident("qualifier") {
                if qualifier.is_some() {
                    return Err(metadata.error("同一字段只能声明一个 qualifier"));
                }
                let value = metadata.value()?.parse::<LitStr>()?;
                let text = value.value();
                if text.is_empty() || text.trim() != text {
                    return Err(syn::Error::new_spanned(
                        value,
                        "qualifier 不能为空且不能包含首尾空白",
                    ));
                }
                qualifier = Some(value);
                return Ok(());
            }
            Err(metadata.error("字段 component 属性只支持 default 或 qualifier"))
        })?;
    }
    if default && qualifier.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "default 字段不能同时声明 qualifier",
        ));
    }
    Ok((default, qualifier))
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
    Ok(dependency)
}

/// 尝试从 `Vec<Arc<dyn Trait>>` 字段中提取 Trait Object 类型。
fn vec_arc_trait_inner(field_type: &Type) -> syn::Result<Option<&Type>> {
    let Type::Path(type_path) = field_type else {
        return Ok(None);
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(None);
    };
    if segment.ident != "Vec" {
        return Ok(None);
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            field_type,
            "Vec 注入字段缺少类型参数",
        ));
    };
    let Some(GenericArgument::Type(item_type)) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "Vec 注入字段的参数必须是 Arc<dyn Trait>",
        ));
    };
    let dependency = arc_inner_type(item_type)?;
    if !matches!(dependency, Type::TraitObject(_)) {
        return Err(syn::Error::new_spanned(
            dependency,
            "Vec 注入目前只支持 Vec<Arc<dyn Trait>>",
        ));
    }
    Ok(Some(dependency))
}

//! `ConfigurationProperties` 派生宏生成逻辑。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments, Type,
};

use crate::configuration_default_option::ConfigurationDefaultOption;

/// 解析具名配置结构体并生成精确键绑定实现。
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    reject_generics(input)?;
    let context = context_crate_path()?;
    let (prefix, kebab_case) = struct_options(&input.attrs)?;
    let fields = named_fields(&input.data)?;
    let mut initializers = Vec::with_capacity(fields.len());

    for field in fields {
        initializers.push(field_initializer(field, kebab_case, &context)?);
    }

    let configuration_type = &input.ident;
    Ok(quote! {
        impl #context::ConfigurationProperties for #configuration_type {
            const PREFIX: &'static str = #prefix;

            fn bind_with_prefix(
                __environment: &#context::ApplicationEnvironment,
                __prefix: &str,
            ) -> ::core::result::Result<
                Self,
                #context::ConfigurationPropertiesError
            > {
                ::core::result::Result::Ok(Self {
                    #(#initializers),*
                })
            }
        }
    })
}

/// 生成单个字段的属性键、读取策略和脱敏错误映射。
fn field_initializer(
    field: &Field,
    kebab_case: bool,
    context: &TokenStream,
) -> syn::Result<TokenStream> {
    let field_ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "配置对象只支持具名字段"))?;
    let (rename, default, nested) = field_options(field)?;
    let field_name = field_ident.to_string();
    let property_name = rename.unwrap_or_else(|| {
        if kebab_case {
            field_name.replace('_', "-")
        } else {
            field_name.clone()
        }
    });
    validate_property_segment(&property_name, field)?;
    let field_name_literal = LitStr::new(&field_name, field_ident.span());
    let property_name_literal = LitStr::new(&property_name, field_ident.span());
    let field_type = &field.ty;

    let read = if nested {
        if option_inner(field_type).is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "nested 配置字段暂不支持 Option；请让嵌套对象字段自行声明默认值",
            ));
        }
        if !matches!(default, ConfigurationDefaultOption::Required) {
            return Err(syn::Error::new_spanned(
                field,
                "nested 配置字段不能同时声明 default",
            ));
        }
        quote! {
            <#field_type as #context::ConfigurationProperties>::bind_with_prefix(
                __environment,
                &__property_key,
            )
            .map_err(|__source| {
                #context::ConfigurationPropertiesError::nested::<Self>(
                    #field_name_literal,
                    __property_key.clone(),
                    __source,
                )
            })?
        }
    } else {
        ordinary_field_read(field, field_type, default, context, &field_name_literal)?
    };

    Ok(quote! {
        #field_ident: {
            let __property_key = if __prefix.is_empty() {
                ::std::string::String::from(#property_name_literal)
            } else {
                ::std::format!("{}.{name}", __prefix, name = #property_name_literal)
            };
            #read
        }
    })
}

/// 生成非嵌套字段的可选、必填或默认值读取表达式。
fn ordinary_field_read(
    field: &Field,
    field_type: &Type,
    default: ConfigurationDefaultOption,
    context: &TokenStream,
    field_name: &LitStr,
) -> syn::Result<TokenStream> {
    if let Some(inner) = option_inner(field_type) {
        if !matches!(default, ConfigurationDefaultOption::Required) {
            return Err(syn::Error::new_spanned(
                field,
                "Option 配置字段已经表达缺失语义，不能再声明 default",
            ));
        }
        Ok(quote! {
            __environment
                .get::<#inner>(&__property_key)
                .map_err(|__source| {
                    #context::ConfigurationPropertiesError::environment::<Self>(
                        #field_name,
                        __property_key.clone(),
                        __source,
                    )
                })?
        })
    } else {
        Ok(match default {
            ConfigurationDefaultOption::Required => quote! {
                __environment
                    .require::<#field_type>(&__property_key)
                    .map_err(|__source| {
                        #context::ConfigurationPropertiesError::environment::<Self>(
                            #field_name,
                            __property_key.clone(),
                            __source,
                        )
                    })?
            },
            ConfigurationDefaultOption::Default => quote! {
                __environment
                    .get::<#field_type>(&__property_key)
                    .map_err(|__source| {
                        #context::ConfigurationPropertiesError::environment::<Self>(
                            #field_name,
                            __property_key.clone(),
                            __source,
                        )
                    })?
                    .unwrap_or_default()
            },
            ConfigurationDefaultOption::Expression(expression) => quote! {
                __environment
                    .get::<#field_type>(&__property_key)
                    .map_err(|__source| {
                        #context::ConfigurationPropertiesError::environment::<Self>(
                            #field_name,
                            __property_key.clone(),
                            __source,
                        )
                    })?
                    .unwrap_or_else(|| #expression)
            },
        })
    }
}

/// 读取结构体级 prefix 和字段命名策略。
fn struct_options(attributes: &[Attribute]) -> syn::Result<(LitStr, bool)> {
    let mut prefix = None;
    let mut kebab_case = false;
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("configuration"))
    {
        attribute.parse_nested_meta(|metadata| {
            if metadata.path.is_ident("prefix") {
                if prefix.is_some() {
                    return Err(metadata.error("configuration prefix 只能声明一次"));
                }
                let value = metadata.value()?.parse::<LitStr>()?;
                validate_prefix(&value)?;
                prefix = Some(value);
                return Ok(());
            }
            if metadata.path.is_ident("rename_all") {
                let value = metadata.value()?.parse::<LitStr>()?;
                return match value.value().as_str() {
                    "snake_case" => {
                        kebab_case = false;
                        Ok(())
                    }
                    "kebab-case" => {
                        kebab_case = true;
                        Ok(())
                    }
                    _ => Err(syn::Error::new_spanned(
                        value,
                        "configuration rename_all 只支持 \"snake_case\" 或 \"kebab-case\"",
                    )),
                };
            }
            Err(metadata.error("结构体 configuration 属性只支持 prefix 或 rename_all"))
        })?;
    }
    let prefix = prefix.ok_or_else(|| {
        syn::Error::new_spanned(
            attributes.first().map_or_else(
                proc_macro2::TokenStream::new,
                quote::ToTokens::to_token_stream,
            ),
            "ConfigurationProperties 必须声明 #[configuration(prefix = \"...\")]",
        )
    })?;
    Ok((prefix, kebab_case))
}

/// 读取字段级 rename、default 和 nested 选项。
fn field_options(field: &Field) -> syn::Result<(Option<String>, ConfigurationDefaultOption, bool)> {
    let mut rename = None;
    let mut default = ConfigurationDefaultOption::Required;
    let mut default_seen = false;
    let mut nested = false;
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("configuration"))
    {
        attribute.parse_nested_meta(|metadata| {
            if metadata.path.is_ident("rename") {
                if rename.is_some() {
                    return Err(metadata.error("configuration rename 只能声明一次"));
                }
                rename = Some(metadata.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if metadata.path.is_ident("default") {
                if default_seen {
                    return Err(metadata.error("configuration default 只能声明一次"));
                }
                default_seen = true;
                if metadata.input.peek(syn::Token![=]) {
                    let expression = metadata.value()?.parse::<LitStr>()?;
                    default = ConfigurationDefaultOption::Expression(
                        syn::parse_str(&expression.value()).map_err(|error| {
                            syn::Error::new_spanned(
                                expression,
                                format!("default 必须是有效 Rust 表达式: {error}"),
                            )
                        })?,
                    );
                } else {
                    default = ConfigurationDefaultOption::Default;
                }
                return Ok(());
            }
            if metadata.path.is_ident("nested") {
                if nested {
                    return Err(metadata.error("configuration nested 只能声明一次"));
                }
                nested = true;
                return Ok(());
            }
            Err(metadata.error("字段 configuration 属性只支持 rename、default 或 nested"))
        })?;
    }
    Ok((rename, default, nested))
}

/// 返回 `Option<T>` 的内部类型。
fn option_inner(field_type: &Type) -> Option<&Type> {
    let Type::Path(path) = field_type else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    match arguments.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// 要求配置对象是具名、非空结构体。
fn named_fields(data: &Data) -> syn::Result<Vec<&Field>> {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) if !fields.named.is_empty() => Ok(fields.named.iter().collect()),
            Fields::Named(_) => Err(syn::Error::new_spanned(
                &data.fields,
                "配置对象至少需要一个具名字段",
            )),
            _ => Err(syn::Error::new_spanned(
                &data.fields,
                "配置对象只支持具名字段结构体",
            )),
        },
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "ConfigurationProperties 只能派生到结构体",
        )),
    }
}

/// 拒绝尚未建立完整线程安全和生命周期合同的泛型配置类型。
fn reject_generics(input: &DeriveInput) -> syn::Result<()> {
    if input.generics.params.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            &input.generics,
            "Vernal ConfigurationProperties 暂不支持泛型结构体",
        ))
    }
}

/// 校验根前缀，空前缀允许显式绑定顶层键。
fn validate_prefix(prefix: &LitStr) -> syn::Result<()> {
    if prefix.value().chars().any(char::is_whitespace) {
        Err(syn::Error::new_spanned(
            prefix,
            "configuration prefix 不能包含空白字符",
        ))
    } else {
        Ok(())
    }
}

/// 校验字段映射到的单段属性名称。
fn validate_property_segment(name: &str, field: &Field) -> syn::Result<()> {
    if name.is_empty()
        || name.contains('.')
        || name
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        Err(syn::Error::new_spanned(
            field,
            "configuration 字段属性名不能为空、包含点号、空白或控制字符",
        ))
    } else {
        Ok(())
    }
}

/// 解析消费方实际使用的 Context crate 路径。
fn context_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-context") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote! { ::#crate_name })
        }
        Err(_) => match crate_name("vernal") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate::context }),
            Ok(FoundCrate::Name(name)) => {
                let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                Ok(quote! { ::#crate_name::context })
            }
            Err(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "ConfigurationProperties 派生需要直接依赖 vernal-context，或通过 vernal 统一门面使用",
            )),
        },
    }
}

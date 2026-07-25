//! `Component` 派生宏生成逻辑。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Field, Fields, GenericArgument, LitStr, Path, PathArguments, Type};

use crate::{
    component_options::{ComponentOptions, ComponentOptionsParts},
    component_scope_option::ComponentScopeOption,
};

/// 解析组件结构并生成 `vernal_beans::Component` 实现。
///
/// 根据 `#[component(...)]` 属性中的选项，可能额外生成：
/// - `Lifecycle` trait 实现（当指定了生命周期钩子时）
/// - Trait 绑定注册代码（当指定了 `as_trait` 时）
/// - 链接期自动发现注册代码（所有 `#[derive(Component)]` 自动注册到 linkme）
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    reject_generics(input)?;
    let ioc = ioc_crate_path()?;
    let options = ComponentOptions::parse(&input.attrs)?.into_parts();
    let component_name = &input.ident;
    let fields = component_fields(&input.data)?;
    let mut initializers = Vec::with_capacity(fields.len());
    let mut dependency_statements = Vec::new();
    let mut qualifier_declarations = Vec::new();
    let aop_implementation = if options.aop_enabled {
        generate_aop_implementation(component_name, &fields)?
    } else {
        TokenStream::new()
    };
    let discovery_registration = generate_discovery_registration(
        component_name,
        &ioc,
    )?;

    // 生命周期钩子生成
    let lifecycle_implementation = generate_lifecycle_implementation(
        component_name,
        &options,
    )?;

    // Trait 绑定注册
    let trait_binding_registration = generate_trait_binding_registration(
        component_name,
        options.as_trait.as_ref(),
        &ioc,
    )?;

    // 初始化排序：当指定了 init_order 时，生成 with_init_order() 调用
    let init_order_statement = if let Some(ref order) = options.init_order {
        quote! {
            let __definition = __definition.with_init_order(#order);
        }
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

    let factory = match options.scope {
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
                // 当指定了 init_order 时，设置同层初始化排序值
                #init_order_statement
                __definition
            }
        }

        #aop_implementation
        #lifecycle_implementation
        #trait_binding_registration
        #discovery_registration
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
    let (uses_default, qualifier, optional) = field_options(field)?;
    if uses_default {
        initializers.push(quote! {
            #field_name: ::core::default::Default::default()
        });
        return Ok(());
    }

    if let Some(dependency) = trait_provider_inner(&field.ty)? {
        return generate_trait_provider_injection(
            field,
            field_name,
            dependency,
            qualifier,
            optional,
            ioc,
            initializers,
            dependency_statements,
            qualifier_declarations,
        );
    }
    if let Some(dependency) = component_provider_inner(&field.ty)? {
        return generate_provider_injection(
            field,
            field_name,
            dependency,
            qualifier,
            optional,
            ioc,
            initializers,
            dependency_statements,
            qualifier_declarations,
        );
    }
    if let Some(dependency) = option_arc_inner(&field.ty)? {
        if optional {
            return Err(syn::Error::new_spanned(
                field,
                "Option<Arc<T>> 已直接表达可选注入，请移除 #[component(optional)]",
            ));
        }
        generate_optional_eager_injection(
            field_name,
            dependency,
            qualifier,
            ioc,
            initializers,
            dependency_statements,
            qualifier_declarations,
        );
        return Ok(());
    }
    if optional {
        return Err(syn::Error::new_spanned(
            field,
            "#[component(optional)] 只支持 ComponentProvider<T> 或 TraitProvider<dyn Trait>；立即可选依赖请使用 Option<Arc<T>>",
        ));
    }

    generate_eager_injection(
        field,
        field_name,
        qualifier,
        ioc,
        (initializers, dependency_statements, qualifier_declarations),
    )
}

/// 为 `Arc<T>`、`Arc<dyn Trait>` 或 Trait 实现集合生成立即依赖注入代码。
fn generate_eager_injection(
    field: &Field,
    field_name: &syn::Ident,
    qualifier: Option<LitStr>,
    ioc: &TokenStream,
    outputs: (
        &mut Vec<TokenStream>,
        &mut Vec<TokenStream>,
        &mut Vec<TokenStream>,
    ),
) -> syn::Result<()> {
    let (initializers, dependency_statements, qualifier_declarations) = outputs;
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

/// 为 `Option<Arc<T>>` 或 `Option<Arc<dyn Trait>>` 生成立即可选依赖注入。
///
/// Option 的类型本身就是可选性事实来源，不需要再增加属性开关。目标存在时仍通过
/// 普通 eager 图边参与拓扑规划，只有零候选转换为 `None`；宏不会生成吞掉其他错误
/// 的 `.ok()` 调用。
#[allow(clippy::too_many_arguments)]
fn generate_optional_eager_injection(
    field_name: &syn::Ident,
    dependency: &Type,
    qualifier: Option<LitStr>,
    ioc: &TokenStream,
    initializers: &mut Vec<TokenStream>,
    dependency_statements: &mut Vec<TokenStream>,
    qualifier_declarations: &mut Vec<TokenStream>,
) {
    let is_trait = matches!(dependency, Type::TraitObject(_));
    let Some(qualifier) = qualifier else {
        if is_trait {
            initializers.push(quote! {
                #field_name: __resolver.resolve_optional_trait::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_optional_trait::<#dependency>();
            });
        } else {
            initializers.push(quote! {
                #field_name: __resolver.resolve_optional::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_optional::<#dependency>();
            });
        }
        return;
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
            #field_name: __resolver.resolve_optional_qualified_trait::<#dependency>(
                &#factory_qualifier
            )?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_optional_qualified_trait::<#dependency>(
                #definition_qualifier
            );
        });
    } else {
        initializers.push(quote! {
            #field_name: __resolver.resolve_optional_qualified::<#dependency>(
                &#factory_qualifier
            )?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_optional_qualified::<#dependency>(
                #definition_qualifier
            );
        });
    }
}

/// 为 `ComponentProvider<T>` 字段生成受限 Provider 和延迟依赖元数据。
#[allow(clippy::too_many_arguments)]
fn generate_provider_injection(
    field: &Field,
    field_name: &syn::Ident,
    dependency: &Type,
    qualifier: Option<LitStr>,
    optional: bool,
    ioc: &TokenStream,
    initializers: &mut Vec<TokenStream>,
    dependency_statements: &mut Vec<TokenStream>,
    qualifier_declarations: &mut Vec<TokenStream>,
) -> syn::Result<()> {
    if matches!(dependency, Type::TraitObject(_)) {
        return Err(syn::Error::new_spanned(
            field,
            "ComponentProvider 只支持具体类型；延迟 Trait Object 请使用 TraitProvider<dyn Trait>",
        ));
    }

    let Some(qualifier) = qualifier else {
        if optional {
            initializers.push(quote! {
                #field_name: __resolver.optional_provider::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_optional_provider::<#dependency>();
            });
        } else {
            initializers.push(quote! {
                #field_name: __resolver.provider::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_provider::<#dependency>();
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
    if optional {
        initializers.push(quote! {
            #field_name: __resolver.optional_qualified_provider::<#dependency>(
                &#factory_qualifier
            )?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_optional_qualified_provider::<#dependency>(
                #definition_qualifier
            );
        });
    } else {
        initializers.push(quote! {
            #field_name: __resolver.qualified_provider::<#dependency>(&#factory_qualifier)?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_qualified_provider::<#dependency>(
                #definition_qualifier
            );
        });
    }
    Ok(())
}

/// 为 `TraitProvider<dyn Trait>` 字段生成 Trait Binding 延迟依赖元数据。
#[allow(clippy::too_many_arguments)]
fn generate_trait_provider_injection(
    field: &Field,
    field_name: &syn::Ident,
    dependency: &Type,
    qualifier: Option<LitStr>,
    optional: bool,
    ioc: &TokenStream,
    initializers: &mut Vec<TokenStream>,
    dependency_statements: &mut Vec<TokenStream>,
    qualifier_declarations: &mut Vec<TokenStream>,
) -> syn::Result<()> {
    if !matches!(dependency, Type::TraitObject(_)) {
        return Err(syn::Error::new_spanned(
            field,
            "TraitProvider 只支持 dyn Trait；具体类型请使用 ComponentProvider<T>",
        ));
    }

    let Some(qualifier) = qualifier else {
        if optional {
            initializers.push(quote! {
                #field_name: __resolver.optional_trait_provider::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_optional_trait_provider::<#dependency>();
            });
        } else {
            initializers.push(quote! {
                #field_name: __resolver.trait_provider::<#dependency>()?
            });
            dependency_statements.push(quote! {
                __definition = __definition.depends_on_trait_provider::<#dependency>();
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
    if optional {
        initializers.push(quote! {
            #field_name: __resolver.optional_qualified_trait_provider::<#dependency>(
                &#factory_qualifier
            )?
        });
        dependency_statements.push(quote! {
            __definition = __definition
                .depends_on_optional_qualified_trait_provider::<#dependency>(
                    #definition_qualifier
                );
        });
    } else {
        initializers.push(quote! {
            #field_name: __resolver.qualified_trait_provider::<#dependency>(
                &#factory_qualifier
            )?
        });
        dependency_statements.push(quote! {
            __definition = __definition.depends_on_qualified_trait_provider::<#dependency>(
                #definition_qualifier
            );
        });
    }
    Ok(())
}

/// 解析消费方实际使用的 Beans crate 路径，兼容 Cargo 依赖重命名和统一门面。
fn ioc_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-beans") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote! { ::#crate_name })
        }
        Err(_) => match crate_name("vernal") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate::beans }),
            Ok(FoundCrate::Name(name)) => {
                let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                Ok(quote! { ::#crate_name::beans })
            }
            Err(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Component 派生需要直接依赖 vernal-beans，或通过 vernal 统一门面使用",
            )),
        },
    }
}

/// 解析消费方实际使用的 Context crate 路径，兼容 Cargo 依赖重命名和统一门面。
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
                "生命周期钩子需要直接依赖 vernal-context，或通过 vernal 统一门面使用",
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

/// 解析消费方显式依赖的可选链接期发现 crate 路径。
fn discovery_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-discovery") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            Ok(quote! { ::#crate_name })
        }
        Err(_) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Component 派生的自动发现需要直接依赖 vernal-discovery",
        )),
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

/// 为所有 `#[derive(Component)]` 的组件生成只读链接期定义入口。
///
/// 对标 Spring 的 `@Component` 自动进入 classpath 扫描范围：
/// 每个 `#[derive(Component)]` 的结构体自动注册到 linkme 分布式切片，
/// 携带模块路径（`module_path!()`）和定义工厂。消费方通过
/// `ComponentScanModule::base_packages()` 按模块路径过滤。
///
/// 静态名称包含模块路径和类型名；分布式切片只携带定义工厂，不创建组件实例，也
/// 不触碰任何全局 Registry。
///
/// 如果消费方没有依赖 `vernal-discovery`，此函数静默返回空 TokenStream，
/// 不影响 `#[derive(Component)]` 的其他功能。
fn generate_discovery_registration(
    component_name: &syn::Ident,
    ioc: &TokenStream,
) -> syn::Result<TokenStream> {
    // 尝试解析 discovery crate 路径；如果不可用则静默跳过
    let discovery = match discovery_crate_path() {
        Ok(path) => path,
        Err(_) => return Ok(TokenStream::new()),
    };
    let registration_name =
        format_ident!("__VERNAL_LINKED_COMPONENT_REGISTRATION_{}", component_name);

    Ok(quote! {
        #[#discovery::linkme::distributed_slice(
            #discovery::LINKED_COMPONENT_REGISTRATIONS
        )]
        #[linkme(crate = #discovery::linkme)]
        #[allow(non_upper_case_globals)]
        static #registration_name: #discovery::LinkedComponentRegistration =
            #discovery::LinkedComponentRegistration::new(
                ::core::module_path!(),
                ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#component_name)),
                <#component_name as #ioc::Component>::definition,
            );
    })
}

/// 为指定了生命周期钩子的组件生成 `Lifecycle` trait 实现。
///
/// 生成的实现将用户定义的方法委托给 vernal-context 的 Lifecycle trait：
/// - `init` → 同步初始化（在 `initialize()` 中调用）
/// - `async_init` → 异步初始化（在 `initialize()` 中调用）
/// - `async_run` → 异步后台运行（在 `start()` 中调用）
/// - `shutdown` → 关闭钩子（在 `stop()` 中调用）
///
/// 对标 tx_di 的 `#[component(init, app_async_init, app_async_run, shutdown)]`。
fn generate_lifecycle_implementation(
    component_name: &syn::Ident,
    options: &ComponentOptionsParts,
) -> syn::Result<TokenStream> {
    // 如果没有指定任何生命周期钩子，不生成 Lifecycle 实现
    let has_hooks = options.init_hook.is_some()
        || options.async_init_hook.is_some()
        || options.async_run_hook.is_some()
        || options.shutdown_hook.is_some();

    if !has_hooks {
        return Ok(TokenStream::new());
    }

    // Lifecycle trait 在 vernal-context 中，需要解析 context crate 路径
    let ctx = context_crate_path()?;

    // 生成 initialize 方法体
    // 当同时指定 init 和 async_init 时，先同步后异步
    let initialize_body = match (&options.init_hook, &options.async_init_hook) {
        (Some(init_fn), Some(async_init_fn)) => {
            let init_ident = syn::Ident::new(&init_fn.value(), init_fn.span());
            let async_init_ident = syn::Ident::new(&async_init_fn.value(), async_init_fn.span());
            quote! {
                self.#init_ident();
                self.#async_init_ident().await
            }
        }
        (Some(init_fn), None) => {
            let init_ident = syn::Ident::new(&init_fn.value(), init_fn.span());
            quote! {
                self.#init_ident();
                ::core::result::Result::Ok(())
            }
        }
        (None, Some(async_init_fn)) => {
            let async_init_ident = syn::Ident::new(&async_init_fn.value(), async_init_fn.span());
            quote! {
                self.#async_init_ident().await
            }
        }
        (None, None) => {
            quote! { ::core::result::Result::Ok(()) }
        }
    };

    // 生成 start 方法体（async_run）
    let start_body = if let Some(async_run_fn) = &options.async_run_hook {
        let async_run_ident = syn::Ident::new(&async_run_fn.value(), async_run_fn.span());
        quote! { self.#async_run_ident(_cancellation).await }
    } else {
        quote! { ::core::result::Result::Ok(()) }
    };

    // 生成 stop 方法体（shutdown）
    let stop_body = if let Some(shutdown_fn) = &options.shutdown_hook {
        let shutdown_ident = syn::Ident::new(&shutdown_fn.value(), shutdown_fn.span());
        quote! {
            self.#shutdown_ident();
            ::core::result::Result::Ok(())
        }
    } else {
        quote! { ::core::result::Result::Ok(()) }
    };

    Ok(quote! {
        impl #ctx::Lifecycle for #component_name {
            fn initialize(&self) -> #ctx::LifecycleFuture<'_> {
                ::std::boxed::Box::pin(async move {
                    #initialize_body
                })
            }

            fn start(
                &self,
                _cancellation: ::tokio_util::sync::CancellationToken,
            ) -> #ctx::LifecycleFuture<'_> {
                ::std::boxed::Box::pin(async move {
                    #start_body
                })
            }

            fn stop(&self) -> #ctx::LifecycleFuture<'_> {
                ::std::boxed::Box::pin(async move {
                    #stop_body
                })
            }
        }
    })
}

/// 为指定了 `as_trait` 的组件生成 Trait 绑定注册代码。
///
/// 在组件定义上自动调用 `with_trait_binding()`，将组件注册为指定 trait 的实现。
/// 对标 tx_di 的 `#[component(as_trait = dyn Trait)]`。
fn generate_trait_binding_registration(
    component_name: &syn::Ident,
    as_trait: Option<&Path>,
    _ioc: &TokenStream,
) -> syn::Result<TokenStream> {
    let Some(trait_path) = as_trait else {
        return Ok(TokenStream::new());
    };

    Ok(quote! {
        // 编译期校验：确保类型确实实现了指定的 trait
        const _: () = {
            fn _assert_trait_impl<T: #trait_path>() {}
            fn _check() { _assert_trait_impl::<#component_name>(); }
        };
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
fn field_options(field: &Field) -> syn::Result<(bool, Option<LitStr>, bool)> {
    let mut default = false;
    let mut qualifier = None;
    let mut optional = false;
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
            if metadata.path.is_ident("optional") {
                optional = true;
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
            Err(metadata.error("字段 component 属性只支持 default、optional 或 qualifier"))
        })?;
    }
    if default && (qualifier.is_some() || optional) {
        return Err(syn::Error::new_spanned(
            field,
            "default 字段不能同时声明 qualifier 或 optional",
        ));
    }
    Ok((default, qualifier, optional))
}

/// 尝试从 `ComponentProvider<T>` 字段提取延迟依赖类型。
fn component_provider_inner(field_type: &Type) -> syn::Result<Option<&Type>> {
    let Type::Path(type_path) = field_type else {
        return Ok(None);
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(None);
    };
    if segment.ident != "ComponentProvider" {
        return Ok(None);
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            field_type,
            "ComponentProvider 注入字段缺少依赖类型参数",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            field_type,
            "ComponentProvider 注入字段必须且只能包含一个类型参数",
        ));
    }
    let Some(GenericArgument::Type(dependency)) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "ComponentProvider 参数必须是具体 Rust 类型",
        ));
    };
    Ok(Some(dependency))
}

/// 尝试从 `TraitProvider<dyn Trait>` 字段提取 Trait Object 类型。
fn trait_provider_inner(field_type: &Type) -> syn::Result<Option<&Type>> {
    let Type::Path(type_path) = field_type else {
        return Ok(None);
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(None);
    };
    if segment.ident != "TraitProvider" {
        return Ok(None);
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            field_type,
            "TraitProvider 注入字段缺少 dyn Trait 类型参数",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            field_type,
            "TraitProvider 注入字段必须且只能包含一个类型参数",
        ));
    }
    let Some(GenericArgument::Type(dependency)) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "TraitProvider 参数必须是 dyn Trait",
        ));
    };
    Ok(Some(dependency))
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

/// 尝试从 `Option<Arc<T>>` 字段中提取立即可选依赖类型 `T`。
///
/// 一旦识别到外层 `Option`，内部形状错误会返回针对可选注入的诊断，避免继续落入
/// 普通 `Arc<T>` 检查而只报告模糊的字段类型错误。
fn option_arc_inner(field_type: &Type) -> syn::Result<Option<&Type>> {
    let Type::Path(type_path) = field_type else {
        return Ok(None);
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Ok(None);
    };
    if segment.ident != "Option" {
        return Ok(None);
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            field_type,
            "Option 可选注入字段缺少 Arc<T> 类型参数",
        ));
    };
    if arguments.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            field_type,
            "Option 可选注入字段必须且只能包含一个 Arc<T> 类型参数",
        ));
    }
    let Some(GenericArgument::Type(item_type)) = arguments.args.first() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "Option 可选注入字段的参数必须是 Arc<T>",
        ));
    };
    arc_inner_type(item_type).map(Some).map_err(|_| {
        syn::Error::new_spanned(
            item_type,
            "立即可选注入只支持 Option<Arc<T>> 或 Option<Arc<dyn Trait>>",
        )
    })
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

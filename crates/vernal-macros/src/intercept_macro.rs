//! `intercept` 属性宏生成逻辑。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Block, FnArg, GenericArgument, Ident, ImplItemFn, LitStr, Pat, PatIdent, PathArguments,
    ReturnType, TraitItemFn, Type, WherePredicate, visit_mut::VisitMut,
};

use crate::{
    intercept_options::InterceptOptions, intercept_receiver::InterceptReceiver,
    self_reference_rewriter::SelfReferenceRewriter,
};

/// 汇集一次方法展开所需的已校验语法片段。
///
/// 解析、校验与代码渲染分离后，签名错误可以在生成大段 `TokenStream` 前返回，
/// 同时避免把所有宏阶段堆积在入口函数中。
struct InterceptCodegen<'a> {
    aop: &'a TokenStream,
    component: &'a TokenStream,
    method_name: &'a LitStr,
    return_type: &'a Type,
    argument_patterns: &'a [PatIdent],
    argument_idents: &'a [Ident],
    business_body: &'a Block,
    self_replacement: &'a Ident,
    receiver: InterceptReceiver,
}

/// 解析实现方法或 Trait 方法，并要求 Trait 方法提供可织入的默认实现。
///
/// 带方法体的 Trait Item 与 Impl Item 共享 Rust 方法语法，因此统一进入
/// [`expand`]，不会复制第二套代码生成器。只有分号结尾的抽象 Trait 方法需要单独
/// 识别并返回面向使用者的诊断；Vernal 不能在没有业务方法体时构造最终调用目标。
pub(crate) fn expand_item(attributes: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    match syn::parse2::<ImplItemFn>(item.clone()) {
        Ok(method) => expand(attributes, method),
        Err(implementation_error) => {
            if let Ok(trait_method) = syn::parse2::<TraitItemFn>(item) {
                if trait_method.default.is_none() {
                    return Err(syn::Error::new_spanned(
                        trait_method.sig,
                        "Trait 方法使用 #[intercept] 必须提供默认方法体；抽象方法请在具体 impl 中织入",
                    ));
                }
            }
            Err(implementation_error)
        }
    }
}

/// 解析拦截选项、校验方法签名并生成异步 AOP 调用包装。
pub(crate) fn expand(attributes: TokenStream, mut method: ImplItemFn) -> syn::Result<TokenStream> {
    let options = InterceptOptions::parse(attributes)?;
    let receiver = validate_method(&method)?;
    let aop = aop_crate_path()?;
    add_aop_component_bound(&mut method, &aop)?;
    let (return_type, _) = result_types(&method.sig.output)?;
    let function_name = &method.sig.ident;
    let method_name = options
        .method()
        .cloned()
        .unwrap_or_else(|| LitStr::new(&function_name.to_string(), function_name.span()));
    let component = options.component().map_or_else(
        || quote! { ::core::any::type_name::<Self>() },
        |component| quote! { #component },
    );
    let descriptor_name = format_ident!(
        "__vernal_operation_{}",
        function_name,
        span = function_name.span()
    );
    let visibility = &method.vis;
    let tags = options.tags();
    let qualifier = options.qualifier();

    let mut argument_patterns = Vec::new();
    let mut argument_idents = Vec::new();
    for argument in method.sig.inputs.iter().skip(1) {
        let FnArg::Typed(argument) = argument else {
            continue;
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "被拦截方法只支持简单标识符参数，不支持解构模式",
            ));
        };
        argument_patterns.push(pattern.clone());
        argument_idents.push(pattern.ident.clone());
    }

    // 方法体中的 self 统一改写为 __vernal_self。Arc 路径让目标拥有 receiver 与
    // 参数；借用路径则把它们约束在当前方法 Future，二者都不要求隐式 Clone。
    let replacement = syn::Ident::new("__vernal_self", Span::mixed_site());
    let mut business_body = method.block.clone();
    SelfReferenceRewriter::new(replacement.clone()).visit_block_mut(&mut business_body);

    method.block = InterceptCodegen {
        aop: &aop,
        component: &component,
        method_name: &method_name,
        return_type,
        argument_patterns: &argument_patterns,
        argument_idents: &argument_idents,
        business_body: &business_body,
        self_replacement: &replacement,
        receiver,
    }
    .render()?;

    // 描述符与业务方法处于同一个 impl，能够使用 `Self` 计算默认组件名。应用模块
    // 通过 `operation!(Type::method)` 显式调用它，避免全局扫描和元数据重复声明。
    let qualifier_statement = qualifier.map(|qualifier| {
        quote! {
            __vernal_operation = __vernal_operation
                .with_qualifier(#qualifier)
                .expect("vernal-macros 已在编译期校验 intercept qualifier");
        }
    });
    let descriptor = quote! {
        #[doc(hidden)]
        #[doc = "返回宏在编译期校验过的 AOP Operation 声明。"]
        #[allow(dead_code)]
        #visibility fn #descriptor_name() -> #aop::Operation
        where
            Self: ::core::marker::Sized,
        {
            let mut __vernal_operation = #aop::Operation::new(#component, #method_name);
            #(
                __vernal_operation = __vernal_operation
                    .with_tag(#tags)
                    .expect("vernal-macros 已在编译期校验 intercept tag");
            )*
            #qualifier_statement
            __vernal_operation
        }
    };

    Ok(quote! {
        #method
        #descriptor
    })
}

impl InterceptCodegen<'_> {
    /// 根据接收器所有权模型选择静态目标或借用目标代码生成路径。
    fn render(&self) -> syn::Result<Block> {
        match self.receiver {
            InterceptReceiver::OwnedArc => self.render_owned(),
            InterceptReceiver::SharedReference | InterceptReceiver::MutableReference => {
                self.render_borrowed()
            }
        }
    }

    /// 渲染 `self: Arc<Self>` 方法的计划查找、owned 目标与返回类型恢复。
    fn render_owned(&self) -> syn::Result<Block> {
        let Self {
            aop,
            component,
            method_name,
            return_type,
            argument_patterns,
            argument_idents,
            business_body,
            self_replacement,
            receiver: _,
        } = self;

        syn::parse2(quote! {{
            let __vernal_operation = #aop::Operation::new(#component, #method_name);
            let __vernal_plan = match #aop::AopComponent::invocation_plans(self.as_ref())
                .get(&__vernal_operation)
                .cloned()
            {
                ::core::option::Option::Some(plan) => plan,
                ::core::option::Option::None => {
                    return ::core::result::Result::Err(#aop::InvocationError::PlanNotFound {
                        operation: __vernal_operation,
                    });
                }
            };
            let __vernal_cancellation =
                #aop::AopComponent::invocation_cancellation(self.as_ref());
            let __vernal_invocation = #aop::Invocation::new(__vernal_operation.clone())
                .with_cancellation(__vernal_cancellation)
                .shared();
            let __vernal_target_self = ::std::sync::Arc::clone(&self);
            let __vernal_arguments = ::std::sync::Arc::new(::std::sync::Mutex::new(
                ::core::option::Option::Some((#(#argument_idents,)*))
            ));
            let __vernal_target_operation = __vernal_operation.clone();
            let __vernal_target: ::std::sync::Arc<#aop::InvocationTarget> =
                ::std::sync::Arc::new(move |_invocation| {
                    let __vernal_target_self =
                        ::std::sync::Arc::clone(&__vernal_target_self);
                    let __vernal_arguments =
                        ::std::sync::Arc::clone(&__vernal_arguments);
                    let __vernal_target_operation = __vernal_target_operation.clone();
                    ::std::boxed::Box::pin(async move {
                        let (#(#argument_patterns,)*) = {
                            let mut __vernal_guard = __vernal_arguments
                                .lock()
                                .unwrap_or_else(::std::sync::PoisonError::into_inner);
                            match __vernal_guard.take() {
                                ::core::option::Option::Some(arguments) => arguments,
                                ::core::option::Option::None => {
                                    return ::core::result::Result::Err(
                                        #aop::InvocationError::TargetAlreadyInvoked {
                                            operation: __vernal_target_operation,
                                        }
                                    );
                                }
                            }
                        };
                        let #self_replacement = __vernal_target_self;
                        let __vernal_result: ::core::result::Result<
                            #return_type,
                            #aop::InvocationError
                        > = (async move #business_body).await;
                        __vernal_result.map(|value| {
                            ::std::boxed::Box::new(value) as #aop::InvocationValue
                        })
                    })
                });

            let __vernal_value = match __vernal_plan
                .invoke(__vernal_invocation, __vernal_target)
                .await
            {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(error) => {
                    return ::core::result::Result::Err(error);
                }
            };
            match __vernal_value.downcast::<#return_type>() {
                ::core::result::Result::Ok(value) => ::core::result::Result::Ok(*value),
                ::core::result::Result::Err(_) => {
                    ::core::result::Result::Err(
                        #aop::InvocationError::ReturnTypeMismatch {
                            expected: ::core::any::type_name::<#return_type>(),
                        }
                    )
                }
            }
        }})
    }

    /// 渲染 `&self` 或 `&mut self` 方法的借用型目标调用与返回类型恢复。
    ///
    /// 业务 Future 在进入拦截器链前创建，并由
    /// `BorrowedInvocationFutureTarget` 持有。目标、接收器和引用参数都不会逃出
    /// 当前 `.await`，同时仍复用 Send-AOP 的顺序、取消和错误语义。
    fn render_borrowed(&self) -> syn::Result<Block> {
        let Self {
            aop,
            component,
            method_name,
            return_type,
            argument_patterns: _,
            argument_idents: _,
            business_body,
            self_replacement,
            receiver: _,
        } = self;

        syn::parse2(quote! {{
            let __vernal_operation = #aop::Operation::new(#component, #method_name);
            let __vernal_plan = match #aop::AopComponent::invocation_plans(self)
                .get(&__vernal_operation)
                .cloned()
            {
                ::core::option::Option::Some(plan) => plan,
                ::core::option::Option::None => {
                    return ::core::result::Result::Err(#aop::InvocationError::PlanNotFound {
                        operation: __vernal_operation,
                    });
                }
            };
            let __vernal_cancellation =
                #aop::AopComponent::invocation_cancellation(self);
            let __vernal_invocation = #aop::Invocation::new(__vernal_operation.clone())
                .with_cancellation(__vernal_cancellation)
                .shared();

            // 引用接收器与可能存在的引用参数只进入当前调用期 Future，不会被擦除
            // 为 `'static`。拦截器短路时该 Future 未被轮询，并在方法返回前安全释放。
            let __vernal_target_self = self;
            let __vernal_target_future: #aop::InvocationFuture<'_> =
                ::std::boxed::Box::pin(async move {
                    let #self_replacement = __vernal_target_self;
                    let __vernal_result: ::core::result::Result<
                        #return_type,
                        #aop::InvocationError
                    > = (async move #business_body).await;
                    __vernal_result.map(|value| {
                        ::std::boxed::Box::new(value) as #aop::InvocationValue
                    })
                });
            let mut __vernal_target = #aop::BorrowedInvocationFutureTarget::new(
                __vernal_operation.clone(),
                __vernal_target_future,
            );

            let __vernal_value = match __vernal_plan
                .invoke_borrowed(__vernal_invocation, &mut __vernal_target)
                .await
            {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(error) => {
                    return ::core::result::Result::Err(error);
                }
            };
            match __vernal_value.downcast::<#return_type>() {
                ::core::result::Result::Ok(value) => ::core::result::Result::Ok(*value),
                ::core::result::Result::Err(_) => {
                    ::core::result::Result::Err(
                        #aop::InvocationError::ReturnTypeMismatch {
                            expected: ::core::any::type_name::<#return_type>(),
                        }
                    )
                }
            }
        }})
    }
}

/// 为实现方法或 Trait 默认方法补充运行资源访问边界。
///
/// Trait 本身不必继承 `AopComponent`，只有真正调用被拦截默认方法的实现类型需要
/// 满足该约束。这样领域端口不会因为一个横切关注点被迫把 Vernal 设为 supertrait；
/// 普通 inherent impl 也得到同一条显式、可诊断的编译期边界。
fn add_aop_component_bound(method: &mut ImplItemFn, aop: &TokenStream) -> syn::Result<()> {
    let predicate: WherePredicate = syn::parse2(quote! {
        Self: #aop::AopComponent
    })?;
    method
        .sig
        .generics
        .make_where_clause()
        .predicates
        .push(predicate);
    Ok(())
}

/// 校验方法并返回其安全异步目标所有权模型。
fn validate_method(method: &ImplItemFn) -> syn::Result<InterceptReceiver> {
    if method.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            method.sig.fn_token,
            "#[intercept] 只支持 async fn",
        ));
    }
    if method.sig.constness.is_some() || method.sig.unsafety.is_some() || method.sig.abi.is_some() {
        return Err(syn::Error::new_spanned(
            &method.sig,
            "#[intercept] 不支持 const、unsafe 或 extern 方法",
        ));
    }
    let receiver = InterceptReceiver::parse(method)?;
    if !receiver.allows_borrowed_arguments() {
        for argument in method.sig.inputs.iter().skip(1) {
            let FnArg::Typed(argument) = argument else {
                continue;
            };
            if matches!(argument.ty.as_ref(), Type::Reference(_)) {
                return Err(syn::Error::new_spanned(
                    &argument.ty,
                    "self: Arc<Self> 被拦截方法的参数必须拥有所有权；如需引用参数请使用 &self",
                ));
            }
        }
    }
    let (_, error_type) = result_types(&method.sig.output)?;
    if type_last_ident(error_type).as_deref() != Some("InvocationError") {
        return Err(syn::Error::new_spanned(
            error_type,
            "被拦截方法必须返回 Result<T, InvocationError>",
        ));
    }
    Ok(receiver)
}

/// 从 `Result<T, E>` 返回类型中提取成功与错误类型。
fn result_types(output: &ReturnType) -> syn::Result<(&Type, &Type)> {
    let ReturnType::Type(_, output_type) = output else {
        return Err(syn::Error::new_spanned(
            output,
            "被拦截方法必须返回 Result<T, InvocationError>",
        ));
    };
    let Type::Path(type_path) = output_type.as_ref() else {
        return Err(syn::Error::new_spanned(
            output_type,
            "被拦截方法必须返回 Result<T, InvocationError>",
        ));
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Err(syn::Error::new_spanned(
            output_type,
            "被拦截方法必须返回 Result<T, InvocationError>",
        ));
    };
    if segment.ident != "Result" {
        return Err(syn::Error::new_spanned(
            output_type,
            "被拦截方法必须返回 Result<T, InvocationError>",
        ));
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            output_type,
            "Result 返回类型缺少成功与错误类型参数",
        ));
    };
    let mut types = arguments.args.iter().filter_map(|argument| {
        if let GenericArgument::Type(value) = argument {
            Some(value)
        } else {
            None
        }
    });
    let Some(success) = types.next() else {
        return Err(syn::Error::new_spanned(output_type, "Result 缺少成功类型"));
    };
    let Some(error) = types.next() else {
        return Err(syn::Error::new_spanned(output_type, "Result 缺少错误类型"));
    };
    if types.next().is_some() {
        return Err(syn::Error::new_spanned(
            output_type,
            "Result 只能包含成功与错误两个类型参数",
        ));
    }
    Ok((success, error))
}

/// 返回类型路径最后一段名称。
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

/// 解析消费方实际使用的 AOP crate 路径。
fn aop_crate_path() -> syn::Result<TokenStream> {
    match crate_name("vernal-aop") {
        Ok(FoundCrate::Itself) => Ok(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let crate_name = syn::Ident::new(&name, Span::call_site());
            Ok(quote! { ::#crate_name })
        }
        Err(_) => match crate_name("vernal") {
            Ok(FoundCrate::Itself) => Ok(quote! { crate::aop }),
            Ok(FoundCrate::Name(name)) => {
                let crate_name = syn::Ident::new(&name, Span::call_site());
                Ok(quote! { ::#crate_name::aop })
            }
            Err(_) => Err(syn::Error::new(
                Span::call_site(),
                "#[intercept] 需要直接依赖 vernal-aop，或通过 vernal 统一门面使用",
            )),
        },
    }
}

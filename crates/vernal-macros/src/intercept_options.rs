//! `intercept` 属性的已校验选项对象。

use proc_macro2::TokenStream;
use syn::{LitStr, Token, bracketed, parse::Parser, punctuated::Punctuated};

/// 保存一次方法拦截声明中的稳定操作身份与静态元数据。
///
/// 该对象只存在于过程宏执行期。标签和限定符在生成 Rust 代码前完成校验，避免把
/// 空字符串、空白或控制字符带入应用启动阶段。标签保留声明顺序；运行时
/// `OperationMetadata` 会负责确定性排序与去重。
#[derive(Default)]
pub(crate) struct InterceptOptions {
    component: Option<LitStr>,
    method: Option<LitStr>,
    tags: Vec<LitStr>,
    qualifier: Option<LitStr>,
}

impl InterceptOptions {
    /// 解析属性参数并拒绝重复的单值选项或非法静态名称。
    pub(crate) fn parse(attributes: TokenStream) -> syn::Result<Self> {
        let mut options = Self::default();
        let parser = syn::meta::parser(|metadata| {
            if metadata.path.is_ident("component") {
                let component = metadata.value()?.parse::<LitStr>()?;
                Self::set_once(
                    &mut options.component,
                    component,
                    "intercept component 只能声明一次",
                )?;
                return Ok(());
            }
            if metadata.path.is_ident("method") {
                let method = metadata.value()?.parse::<LitStr>()?;
                Self::set_once(&mut options.method, method, "intercept method 只能声明一次")?;
                return Ok(());
            }
            if metadata.path.is_ident("tags") {
                if !options.tags.is_empty() {
                    return Err(metadata.error("intercept tags 只能声明一次"));
                }
                let values = metadata.value()?;
                let content;
                bracketed!(content in values);
                let tags = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                if tags.is_empty() {
                    return Err(metadata.error("intercept tags 至少需要一个标签"));
                }
                for tag in tags {
                    Self::validate_static_name(&tag, "标签")?;
                    options.tags.push(tag);
                }
                return Ok(());
            }
            if metadata.path.is_ident("qualifier") {
                let qualifier = metadata.value()?.parse::<LitStr>()?;
                Self::validate_static_name(&qualifier, "qualifier")?;
                Self::set_once(
                    &mut options.qualifier,
                    qualifier,
                    "intercept qualifier 只能声明一次",
                )?;
                return Ok(());
            }
            Err(metadata.error("intercept 属性只支持 component、method、tags 或 qualifier"))
        });
        parser.parse2(attributes)?;
        Ok(options)
    }

    /// 取出可选逻辑组件名称。
    pub(crate) fn component(&self) -> Option<&LitStr> {
        self.component.as_ref()
    }

    /// 取出可选逻辑方法名称。
    pub(crate) fn method(&self) -> Option<&LitStr> {
        self.method.as_ref()
    }

    /// 返回声明顺序下的静态标签。
    pub(crate) fn tags(&self) -> &[LitStr] {
        &self.tags
    }

    /// 取出可选静态限定符。
    pub(crate) fn qualifier(&self) -> Option<&LitStr> {
        self.qualifier.as_ref()
    }

    /// 为只能出现一次的选项赋值。
    fn set_once(
        slot: &mut Option<LitStr>,
        value: LitStr,
        message: &'static str,
    ) -> syn::Result<()> {
        if slot.is_some() {
            return Err(syn::Error::new(value.span(), message));
        }
        *slot = Some(value);
        Ok(())
    }

    /// 校验标签或限定符可以作为低基数、稳定声明名称。
    fn validate_static_name(value: &LitStr, kind: &'static str) -> syn::Result<()> {
        let value_text = value.value();
        if value_text.is_empty()
            || value_text
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(syn::Error::new(
                value.span(),
                format!("intercept {kind}不能为空，也不能包含空白或控制字符"),
            ));
        }
        Ok(())
    }
}

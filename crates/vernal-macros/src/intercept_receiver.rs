//! 被拦截方法接收器分类对象。

use syn::{FnArg, GenericArgument, ImplItemFn, PathArguments, Type};

/// 描述 `#[intercept]` 支持的三种异步方法所有权模型。
///
/// `OwnedArc` 可以生成 `'static` 目标，适合后台任务或需要移动组件所有权的调用；
/// 两种引用接收器都把业务 Future 约束在当前 `.await`，其中可变引用允许状态型
/// Transient 组件在完整 Around 链内安全修改自身。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InterceptReceiver {
    /// 显式消费一个 `Arc<Self>` 克隆。
    OwnedArc,
    /// 在当前异步调用期间共享借用组件。
    SharedReference,
    /// 在当前异步调用期间独占借用组件。
    MutableReference,
}

impl InterceptReceiver {
    /// 校验并分类方法的第一个参数。
    pub(crate) fn parse(method: &ImplItemFn) -> syn::Result<Self> {
        let Some(FnArg::Receiver(receiver)) = method.sig.inputs.first() else {
            return Err(syn::Error::new_spanned(
                &method.sig.inputs,
                "#[intercept] 必须用于具有 self: Arc<Self>、&self 或 &mut self 接收器的方法",
            ));
        };
        if receiver.reference.is_some() {
            return if receiver.mutability.is_some() {
                Ok(Self::MutableReference)
            } else {
                Ok(Self::SharedReference)
            };
        }
        if is_arc_self(&receiver.ty) {
            return Ok(Self::OwnedArc);
        }
        Err(syn::Error::new_spanned(
            receiver,
            "#[intercept] 的接收器必须写成 self: Arc<Self>、&self 或 &mut self",
        ))
    }

    /// 返回该接收器是否允许参数继续借用调用方数据。
    ///
    /// 借用接收器生成的目标不会逃出当前方法 Future，因此引用参数与共享/独占
    /// receiver 可以处于同一调用生命周期；owned `Arc<Self>` 路径仍保持完整
    /// `'static` 合同。
    pub(crate) const fn allows_borrowed_arguments(self) -> bool {
        matches!(self, Self::SharedReference | Self::MutableReference)
    }
}

/// 判断显式接收器类型是否为 `Arc<Self>`。
fn is_arc_self(receiver_type: &Type) -> bool {
    let Type::Path(type_path) = receiver_type else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Arc" {
        return false;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    matches!(
        arguments.args.first(),
        Some(GenericArgument::Type(Type::Path(inner)))
            if inner.qself.is_none() && inner.path.is_ident("Self")
    )
}

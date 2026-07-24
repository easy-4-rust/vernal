//! 方法体 `self` 引用改写器对象。

use syn::{ExprPath, Ident, visit_mut::VisitMut};

/// 将业务方法体中的 `self` 表达式改写为宏生成的 owned `Arc<Self>` 变量。
///
/// 被拦截目标必须返回 `'static` Future，不能继续借用外层方法接收器。改写只处理
/// Rust 表达式语法树中的 `self` 路径，不修改字符串、文档或其他标识符。
pub(crate) struct SelfReferenceRewriter {
    replacement: Ident,
}

impl SelfReferenceRewriter {
    /// 创建使用指定标识符替换 `self` 的改写器。
    pub(crate) fn new(replacement: Ident) -> Self {
        Self { replacement }
    }
}

impl VisitMut for SelfReferenceRewriter {
    fn visit_expr_path_mut(&mut self, expression: &mut ExprPath) {
        if expression.qself.is_none() && expression.path.is_ident("self") {
            expression.path = self.replacement.clone().into();
            return;
        }
        syn::visit_mut::visit_expr_path_mut(self, expression);
    }
}

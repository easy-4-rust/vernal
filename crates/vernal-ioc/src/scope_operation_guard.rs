//! 自定义作用域活动操作守卫对象。

use crate::ScopeContext;

/// 确保组件工厂 panic 或提前返回时也能归还活动操作计数。
pub(crate) struct ScopeOperationGuard<'a> {
    scope: &'a ScopeContext,
}

impl<'a> ScopeOperationGuard<'a> {
    /// 绑定已经成功登记的作用域操作。
    pub(crate) const fn new(scope: &'a ScopeContext) -> Self {
        Self { scope }
    }
}

impl Drop for ScopeOperationGuard<'_> {
    fn drop(&mut self) {
        self.scope.finish_operation();
    }
}

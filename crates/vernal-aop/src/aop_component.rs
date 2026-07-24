//! AOP 组件运行资源访问契约。

use tokio_util::sync::CancellationToken;

use crate::InvocationPlanCatalog;

/// 为方法拦截宏提供 Context-local AOP 计划和取消令牌。
///
/// `#[derive(vernal_macros::Component)]` 配合 `#[component(aop)]` 会根据组件中的
/// `Arc<InvocationPlanCatalog>` 与 `Arc<CancellationToken>` 字段生成该实现。
/// 宏只读取组件自身持有的资源，不访问进程级全局表或实例指针 Map。
pub trait AopComponent: Send + Sync + 'static {
    /// 返回应用构建阶段预编译的调用计划目录。
    fn invocation_plans(&self) -> &InvocationPlanCatalog;

    /// 克隆当前应用的协作取消令牌。
    fn invocation_cancellation(&self) -> CancellationToken;
}

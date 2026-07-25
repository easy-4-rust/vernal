//! `IoC` 托管应用事件监听器契约。

use std::{any::Any, error::Error, future::Future, sync::Arc};

/// 处理一种 Context-local 强类型应用事件的组件契约。
///
/// 实现类型必须作为 Singleton 注册到 Vernal IoC，并通过
/// [`crate::VernalApplicationBuilder::event_listener`] 或应用模块声明监听关系。
/// Vernal 在 `refresh()` 阶段解析同一个组件实例，先建立 Tokio broadcast 订阅，
/// 再初始化任何生命周期组件，因此初始化阶段发布的事件也不会错过。
///
/// 本 Trait 保持泛型而不支持 `dyn ApplicationEventListener<_>`。类型擦除只发生在
/// 应用构建阶段的启动声明中，事件分发热路径仍调用具体实现，消费方可以让同一组件
/// 分别实现多个事件类型的监听契约。
pub trait ApplicationEventListener<E>: Send + Sync + 'static
where
    E: Any + Send + Sync + 'static,
{
    /// 监听器返回的结构化错误类型。
    type Error: Error + Send + Sync + 'static;

    /// 异步处理一个由所有同类型订阅方共享的只读事件。
    ///
    /// 返回错误会被视为关键后台任务失败：Vernal 保存原始错误链、取消应用并通过
    /// Context 关闭结果暴露失败。实现方不应把凭证、Token 或请求正文写入
    /// [`std::fmt::Display`] 输出。
    fn on_event(&self, event: Arc<E>) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// 返回低基数、无敏感数据的静态监听器名称。
    ///
    /// 默认使用实现类型全名；覆盖值会作为受管 Tokio 任务身份进入错误诊断。
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

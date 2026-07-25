//! 应用事件监听器注册回调类型。

use crate::ApplicationContextBuilder;

/// 在依赖图冻结后把已声明的强类型监听关系登记到 Context 建造器。
///
/// 回调只保存组件、事件类型和可选限定符，不持有业务实例；真正解析、订阅和任务
/// 提交发生在 `refresh()`，并受同一 Context 的 Tokio 监督器管理。
pub(crate) type EventListenerRegistrar =
    dyn FnOnce(&mut ApplicationContextBuilder) + Send + Sync + 'static;

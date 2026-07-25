//! 周期任务延迟登记函数类型。

use crate::ApplicationContextBuilder;

/// 把具体周期任务类型延迟登记到最终 Context 建造器。
///
/// 应用模块和条件模块共享该回调类型，避免在暂存阶段提前解析组件实例。
pub(crate) type ScheduledTaskRegistrar =
    dyn FnOnce(&mut ApplicationContextBuilder) + Send + 'static;

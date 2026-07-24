//! 受管 Tokio 任务运行表对象。

use std::collections::BTreeMap;

use tokio::task::AbortHandle;

use crate::{ManagedTaskError, ManagedTaskId};

/// 保存一个任务监督器的可变运行状态。
///
/// 该对象始终位于短临界区的同步互斥锁内；任何用户 Future 和异步等待都不会在
/// 持锁期间执行。任务表只保存静态名称与 Tokio AbortHandle，不保存业务数据。
pub(crate) struct ManagedTaskRegistry {
    /// 是否仍接受新任务。
    pub(crate) accepting: bool,
    /// 是否已经提交唯一停机协调任务。
    pub(crate) shutdown_started: bool,
    /// 下一个 Context-local 任务标识。
    pub(crate) next_id: u64,
    /// 当前尚未由观察器收口的任务。
    pub(crate) tasks: BTreeMap<ManagedTaskId, (&'static str, AbortHandle)>,
    /// 运行期间观察到的第一个失败。
    pub(crate) first_failure: Option<ManagedTaskError>,
    /// 已完成并从活动表移除的任务总数。
    pub(crate) completed_count: u64,
}

impl Default for ManagedTaskRegistry {
    fn default() -> Self {
        Self {
            accepting: true,
            shutdown_started: false,
            next_id: 1,
            tasks: BTreeMap::new(),
            first_failure: None,
            completed_count: 0,
        }
    }
}

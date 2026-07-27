//! WebSocket 生命周期控制。

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// WebSocket 服务生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// 已创建但未启动。
    Created,
    /// 正在运行。
    Running,
    /// 正在停止并清空连接。
    Draining,
    /// 已停止。
    Stopped,
}

/// 可共享的 WebSocket 生命周期控制器。
#[derive(Debug, Clone)]
pub struct LifecycleController {
    state: Arc<RwLock<LifecycleState>>,
    cancellation: CancellationToken,
}

impl LifecycleController {
    /// 创建生命周期控制器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(LifecycleState::Created)),
            cancellation: CancellationToken::new(),
        }
    }

    /// 标记为运行状态。
    pub async fn start(&self) {
        *self.state.write().await = LifecycleState::Running;
    }

    /// 开始排空并广播取消。
    pub async fn drain(&self) {
        *self.state.write().await = LifecycleState::Draining;
        self.cancellation.cancel();
    }

    /// 标记为已停止。
    pub async fn stop(&self) {
        *self.state.write().await = LifecycleState::Stopped;
    }

    /// 返回状态快照。
    pub async fn state(&self) -> LifecycleState {
        *self.state.read().await
    }

    /// 返回取消令牌。
    #[must_use]
    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
}

impl Default for LifecycleController {
    fn default() -> Self {
        Self::new()
    }
}

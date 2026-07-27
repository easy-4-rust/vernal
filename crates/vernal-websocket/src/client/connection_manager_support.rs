//! 对应 Java 类：org.springframework.web.socket.client.ConnectionManagerSupport
//!
//! 生命周期管理：start/stop/phase/autoStartup/running/isConnected。
//! Spring 用 `SmartLifecycle` + synchronized；Vernal 用 `LifecycleController`。

use std::sync::Arc;
use std::time::Duration;

use http::Uri;
use tokio::sync::Mutex;

use crate::LifecycleController;
use crate::WebSocketError;
use crate::client::WebSocketClient;

/// 客户端连接管理配置。
#[derive(Debug, Clone)]
pub struct WebSocketClientConfig {
    /// 连接失败后的最大重试次数。
    pub max_retries: usize,
    /// 重试之间的等待时间。
    pub retry_delay: Duration,
}

impl Default for WebSocketClientConfig {
    fn default() -> Self {
        Self {
            max_retries: 0,
            retry_delay: Duration::from_secs(1),
        }
    }
}

/// 连接管理基类。
pub struct ConnectionManagerSupport {
    uri: Uri,
    auto_startup: bool,
    phase: i32,
    lifecycle: LifecycleController,
    running: Mutex<bool>,
    config: WebSocketClientConfig,
}

impl ConnectionManagerSupport {
    /// 创建连接管理基类。
    #[must_use]
    pub fn new(uri: Uri) -> Self {
        Self {
            uri,
            auto_startup: false,
            phase: i32::MAX,
            lifecycle: LifecycleController::new(),
            running: Mutex::new(false),
            config: WebSocketClientConfig::default(),
        }
    }

    /// 设置 autoStartup。
    #[must_use]
    pub const fn with_auto_startup(mut self, auto_startup: bool) -> Self {
        self.auto_startup = auto_startup;
        self
    }

    /// 设置 phase。
    #[must_use]
    pub const fn with_phase(mut self, phase: i32) -> Self {
        self.phase = phase;
        self
    }

    /// 设置配置。
    #[must_use]
    pub const fn with_config(mut self, config: WebSocketClientConfig) -> Self {
        self.config = config;
        self
    }

    /// URI。
    #[must_use]
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// autoStartup。
    #[must_use]
    pub const fn auto_startup(&self) -> bool {
        self.auto_startup
    }

    /// phase。
    #[must_use]
    pub const fn phase(&self) -> i32 {
        self.phase
    }

    /// 配置。
    #[must_use]
    pub const fn config(&self) -> &WebSocketClientConfig {
        &self.config
    }

    /// 生命周期控制器。
    #[must_use]
    pub fn lifecycle(&self) -> &LifecycleController {
        &self.lifecycle
    }

    /// 启动。若已运行则无操作。
    pub async fn start(&self) {
        let mut guard = self.running.lock().await;
        if *guard {
            return;
        }
        *guard = true;
        self.lifecycle.start().await;
        self.open_connection().await;
    }

    /// 停止。若未运行则无操作。
    pub async fn stop(&self) {
        let mut guard = self.running.lock().await;
        if !*guard {
            return;
        }
        let _ = self.stop_internal().await;
        *guard = false;
        self.lifecycle.stop().await;
    }

    /// 是否在运行。
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    async fn stop_internal(&self) -> Result<(), WebSocketError> {
        // 子类可重写 close_connection
        Ok(())
    }

    async fn open_connection(&self) {
        // 子类（WebSocketConnectionManager）重写此方法以发起真实握手
    }
}

/// WebSocket 连接管理器。
pub struct WebSocketConnectionManager {
    base: ConnectionManagerSupport,
    /// 客户端实现，供子类型或测试持有以建立连接。
    pub client: Arc<dyn WebSocketClient>,
    /// 关联 handler。
    pub handler: Arc<dyn crate::WebSocketHandler>,
}

impl WebSocketConnectionManager {
    /// 创建连接管理器。
    #[must_use]
    pub fn new(
        client: Arc<dyn WebSocketClient>,
        handler: Arc<dyn crate::WebSocketHandler>,
        uri: Uri,
    ) -> Self {
        Self {
            base: ConnectionManagerSupport::new(uri),
            client,
            handler,
        }
    }

    /// 返回内部连接管理基类。
    #[must_use]
    pub fn base(&self) -> &ConnectionManagerSupport {
        &self.base
    }
}

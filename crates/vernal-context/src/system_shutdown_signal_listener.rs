//! Tokio 操作系统关闭信号监听对象。

use std::io;

use crate::ApplicationShutdownSignal;

/// 使用 Tokio 原生信号 API 等待当前进程的第一个关闭信号。
///
/// 本对象无可变状态，可以安全地作为普通 `IoC` 单例注入。每次 [`Self::wait`]
/// 都创建当前等待所需的 Tokio signal stream，不保存进程级全局 Context，也不
/// 安装 Vernal 自定义操作系统处理器。Unix 覆盖 Ctrl-C、SIGTERM、SIGHUP；
/// Windows 覆盖 Ctrl-C、Ctrl-Break、控制台关闭与系统关机。
#[derive(Debug, Default)]
pub struct SystemShutdownSignalListener;

impl SystemShutdownSignalListener {
    /// 创建一个无状态的系统信号监听对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 等待当前平台支持的第一个关闭信号。
    ///
    /// # Errors
    ///
    /// Tokio 无法注册操作系统信号流，或信号流在收到值前结束时返回 `io::Error`；
    /// 调用方必须决定是否转入保守关闭，不能依赖 panic 处理生产进程生命周期。
    #[cfg(unix)]
    pub async fn wait(&self) -> io::Result<ApplicationShutdownSignal> {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate())?;
        let mut hangup = signal(SignalKind::hangup())?;

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result?;
                Ok(ApplicationShutdownSignal::Interrupt)
            }
            received = terminate.recv() => {
                Self::received(received, ApplicationShutdownSignal::Terminate)
            }
            received = hangup.recv() => {
                Self::received(received, ApplicationShutdownSignal::Hangup)
            }
        }
    }

    /// 等待当前平台支持的第一个关闭信号。
    ///
    /// # Errors
    ///
    /// Tokio 无法注册 Windows 控制台信号流，或信号流在收到值前结束时返回
    /// `io::Error`。
    #[cfg(windows)]
    pub async fn wait(&self) -> io::Result<ApplicationShutdownSignal> {
        use tokio::signal::windows;

        let mut ctrl_break = windows::ctrl_break()?;
        let mut ctrl_close = windows::ctrl_close()?;
        let mut ctrl_shutdown = windows::ctrl_shutdown()?;

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result?;
                Ok(ApplicationShutdownSignal::Interrupt)
            }
            received = ctrl_break.recv() => {
                Self::received(received, ApplicationShutdownSignal::Break)
            }
            received = ctrl_close.recv() => {
                Self::received(received, ApplicationShutdownSignal::Close)
            }
            received = ctrl_shutdown.recv() => {
                Self::received(received, ApplicationShutdownSignal::Shutdown)
            }
        }
    }

    /// 在其他 Tokio 支持平台上至少等待可移植的 Ctrl-C 信号。
    ///
    /// # Errors
    ///
    /// Tokio 无法安装 Ctrl-C 监听器时返回 `io::Error`。
    #[cfg(all(not(unix), not(windows)))]
    pub async fn wait(&self) -> io::Result<ApplicationShutdownSignal> {
        tokio::signal::ctrl_c().await?;
        Ok(ApplicationShutdownSignal::Interrupt)
    }

    /// 把意外结束的操作系统信号流转换为结构化 I/O 错误。
    #[cfg(any(unix, windows))]
    fn received(
        received: Option<()>,
        signal: ApplicationShutdownSignal,
    ) -> io::Result<ApplicationShutdownSignal> {
        received.map(|()| signal).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Tokio operating-system signal stream ended before receiving a signal",
            )
        })
    }
}

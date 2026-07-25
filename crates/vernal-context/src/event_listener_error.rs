//! 应用事件监听任务错误对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

/// 描述 `IoC` 托管事件监听器在后台消费阶段发生的失败。
///
/// 普通 `Display`/`Debug` 只包含静态监听器/事件类型和低基数计数，不展开业务错误正文。
/// 原始处理错误仍可通过标准 [`Error::source`] 链显式取得。
#[derive(Clone)]
#[non_exhaustive]
pub enum EventListenerError {
    /// 监听器实现返回错误。
    HandlerFailed {
        /// 监听器声明的静态诊断名称。
        listener: &'static str,
        /// 被处理事件的 Rust 类型名。
        event: &'static str,
        /// 实现返回的原始错误。
        source: SharedError,
    },
    /// 监听器落后于有界 broadcast 环形缓冲，已经发生事件丢失。
    Lagged {
        /// 监听器声明的静态诊断名称。
        listener: &'static str,
        /// 被处理事件的 Rust 类型名。
        event: &'static str,
        /// Tokio 报告的跳过事件数。
        skipped: u64,
    },
}

impl fmt::Display for EventListenerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HandlerFailed {
                listener, event, ..
            } => write!(
                formatter,
                "application event listener {listener} failed while handling {event}"
            ),
            Self::Lagged {
                listener,
                event,
                skipped,
            } => write!(
                formatter,
                "application event listener {listener} lagged while handling {event}; \
                 skipped {skipped} events"
            ),
        }
    }
}

impl fmt::Debug for EventListenerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HandlerFailed {
                listener, event, ..
            } => formatter
                .debug_struct("HandlerFailed")
                .field("listener", listener)
                .field("event", event)
                .finish_non_exhaustive(),
            Self::Lagged {
                listener,
                event,
                skipped,
            } => formatter
                .debug_struct("Lagged")
                .field("listener", listener)
                .field("event", event)
                .field("skipped", skipped)
                .finish(),
        }
    }
}

impl Error for EventListenerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HandlerFailed { source, .. } => Some(source.as_ref()),
            Self::Lagged { .. } => None,
        }
    }
}

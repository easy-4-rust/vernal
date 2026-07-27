//! 对应 Java 类：org.springframework.web.socket.handler.ConcurrentWebSocketSessionDecorator
//!
//! 包装 `WebSocketSession`，保证同一时刻只有一个发送任务在执行。
//!
//! 行为对齐 Spring `ConcurrentWebSocketSessionDecorator`：
//! - `send_message` 先把消息加入 buffer；
//! - 尝试 `try_lock` 获取 flush lock（非阻塞）；
//! - 获取成功：排空 buffer 中的所有消息（含刚入队的），释放 flush lock；
//! - 获取失败（另一个发送仍在进行）：调用 `check_session_limits`，
//!   按 `OverflowStrategy` 处理（Terminate 抛错或 Drop 丢弃最旧消息），
//!   消息保留在 buffer 中等待持有 flush lock 的任务排空；
//! - `close` 时再次检查限制，若已超限则升级关闭状态为 SESSION_NOT_RELIABLE。

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{
    CloseCode, CloseStatus, WebSocketError, WebSocketMessage, WebSocketSession,
    handler::session_limit_exceeded_error::{SessionLimitExceededError, safe_status},
};

/// Spring `OverflowStrategy` 等价：buffer 满时的策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStrategy {
    /// 抛出 `SessionLimitExceededError`，关闭连接。
    Terminate,
    /// 丢弃最旧消息。
    Drop,
}

/// Spring `SESSION_NOT_RELIABLE` (4500) 关闭状态。
#[must_use]
pub fn session_not_reliable() -> CloseStatus {
    CloseStatus::new(CloseCode::Custom(4500), "Session not reliable")
        .unwrap_or_else(|_| CloseStatus::normal())
}

/// 并发安全 session 装饰器。
///
/// 使用两个锁分别对标 Spring 的 `flushLock` 和 `closeLock`：
/// - `flush_lock`：保证同一时刻只有一个任务在执行底层 `delegate.send`；
///   其他调用者 `try_lock` 失败后把消息留在 buffer 等待排空。
/// - `buffer_lock`：保护 buffer 与 buffer_size 的读写一致性。
pub struct ConcurrentWebSocketSessionDecorator {
    delegate: Arc<dyn WebSocketSession>,
    send_time_limit: Duration,
    buffer_size_limit: usize,
    overflow_strategy: OverflowStrategy,
    /// flush lock（对标 Spring `flushLock`）。
    flush_lock: Mutex<()>,
    /// buffer 状态（对标 Spring `buffer` + `bufferSize` + `sendStartTime` + flags）。
    buffer_state: Mutex<BufferState>,
}

#[derive(Debug)]
struct BufferState {
    buffer: VecDeque<WebSocketMessage>,
    buffer_size: usize,
    send_start: Option<Instant>,
    limit_exceeded: bool,
    close_in_progress: bool,
}

impl BufferState {
    fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            buffer_size: 0,
            send_start: None,
            limit_exceeded: false,
            close_in_progress: false,
        }
    }

    fn time_since_send_started(&self) -> Duration {
        self.send_start
            .map_or(Duration::ZERO, |start| start.elapsed())
    }

    fn should_not_send(&self) -> bool {
        self.limit_exceeded || self.close_in_progress
    }
}

impl ConcurrentWebSocketSessionDecorator {
    /// 创建装饰器，默认 `OverflowStrategy::Terminate`。
    #[must_use]
    pub fn new(
        delegate: Arc<dyn WebSocketSession>,
        send_time_limit: Duration,
        buffer_size_limit: usize,
    ) -> Self {
        Self::with_strategy(
            delegate,
            send_time_limit,
            buffer_size_limit,
            OverflowStrategy::Terminate,
        )
    }

    /// 创建指定 OverflowStrategy 的装饰器。
    #[must_use]
    pub fn with_strategy(
        delegate: Arc<dyn WebSocketSession>,
        send_time_limit: Duration,
        buffer_size_limit: usize,
        overflow_strategy: OverflowStrategy,
    ) -> Self {
        Self {
            delegate,
            send_time_limit,
            buffer_size_limit,
            overflow_strategy,
            flush_lock: Mutex::new(()),
            buffer_state: Mutex::new(BufferState::new()),
        }
    }

    /// 当前 buffer 字节数。
    pub async fn buffer_size(&self) -> usize {
        self.buffer_state.lock().await.buffer_size
    }

    /// 距离当前发送开始经过的时间。
    pub async fn time_since_send_started(&self) -> Duration {
        self.buffer_state.lock().await.time_since_send_started()
    }

    /// 发送时间限制。
    #[must_use]
    pub const fn send_time_limit(&self) -> Duration {
        self.send_time_limit
    }

    /// buffer 大小限制。
    #[must_use]
    pub const fn buffer_size_limit(&self) -> usize {
        self.buffer_size_limit
    }

    /// Overflow 策略。
    #[must_use]
    pub const fn overflow_strategy(&self) -> OverflowStrategy {
        self.overflow_strategy
    }

    /// 入队并发送消息，对齐 Spring `sendMessage` 行为。
    ///
    /// # Errors
    ///
    /// 当 buffer/send 时间超限且策略为 `Terminate` 时返回 `WebSocketError::LimitExceeded`。
    pub async fn send_message(&self, message: WebSocketMessage) -> Result<(), WebSocketError> {
        // Step 1: 入 buffer
        {
            let mut state = self.buffer_state.lock().await;
            if state.should_not_send() {
                return Ok(());
            }
            state.buffer_size = state.buffer_size.saturating_add(message.payload_len());
            state.buffer.push_back(message);
        }

        // Step 2: 尝试获取 flush lock（非阻塞 tryLock，对标 Spring `flushLock.tryLock()`）
        loop {
            let flush_guard = self.flush_lock.try_lock();
            if flush_guard.is_err() {
                // tryLock 失败：另一个发送正在进行，检查限制后返回。
                // 消息已在 buffer 中，由持有 flush lock 的任务排空。
                let mut state = self.buffer_state.lock().await;
                self.check_session_limits(&mut state)?;
                return Ok(());
            }

            // tryLock 成功：排空 buffer（对标 Spring `tryFlushMessageBuffer`）
            let _flush_guard = flush_guard.unwrap();
            loop {
                let front = {
                    let mut state = self.buffer_state.lock().await;
                    if state.should_not_send() {
                        break;
                    }
                    match state.buffer.pop_front() {
                        Some(msg) => {
                            state.buffer_size = state.buffer_size.saturating_sub(msg.payload_len());
                            state.send_start = Some(Instant::now());
                            msg
                        }
                        None => break,
                    }
                };

                let send_result = self.delegate.send(front).await;

                {
                    let mut state = self.buffer_state.lock().await;
                    state.send_start = None;
                }

                if let Err(error) = send_result {
                    return Err(error);
                }
            }
            // flush lock 释放（_flush_guard drop）

            // 检查是否有新消息在 flush 期间入队
            let remaining = {
                let state = self.buffer_state.lock().await;
                state.buffer.is_empty() || state.should_not_send()
            };
            if remaining {
                return Ok(());
            }
            // 还有消息：继续循环尝试再次获取 flush lock
        }
    }

    fn check_session_limits(&self, state: &mut BufferState) -> Result<(), WebSocketError> {
        if state.should_not_send() {
            return Ok(());
        }
        if state.time_since_send_started() > self.send_time_limit {
            let reason = format!(
                "Send time {} (ms) for session '{}' exceeded the allowed limit {}",
                state.time_since_send_started().as_millis(),
                self.delegate.id(),
                self.send_time_limit.as_millis()
            );
            self.limit_exceeded(state, reason)?;
        } else if state.buffer_size > self.buffer_size_limit {
            match self.overflow_strategy {
                OverflowStrategy::Terminate => {
                    let reason = format!(
                        "Buffer size {} bytes for session '{}' exceeds the allowed limit {}",
                        state.buffer_size,
                        self.delegate.id(),
                        self.buffer_size_limit
                    );
                    self.limit_exceeded(state, reason)?;
                }
                OverflowStrategy::Drop => {
                    while state.buffer_size > self.buffer_size_limit {
                        let Some(dropped) = state.buffer.pop_front() else {
                            break;
                        };
                        state.buffer_size = state.buffer_size.saturating_sub(dropped.payload_len());
                    }
                }
            }
        }
        Ok(())
    }

    fn limit_exceeded(
        &self,
        state: &mut BufferState,
        reason: String,
    ) -> Result<(), WebSocketError> {
        state.limit_exceeded = true;
        let _error = SessionLimitExceededError::new(reason, Some(session_not_reliable()));
        Err(WebSocketError::LimitExceeded {
            kind: "session",
            limit: self.buffer_size_limit,
            observed: state.buffer_size,
        })
    }

    /// 关闭，对齐 Spring `close(status)` 行为：在未标记 SESSION_NOT_RELIABLE 时
    /// 再次检查限制，若已超限则升级为 SESSION_NOT_RELIABLE。
    pub async fn close(&self, status: CloseStatus) -> Result<(), WebSocketError> {
        let mut state = self.buffer_state.lock().await;
        if state.close_in_progress {
            return Ok(());
        }
        let mut effective = status;
        if effective.code().as_u16() != session_not_reliable().code().as_u16() {
            let _ = self.check_session_limits(&mut state);
            if state.limit_exceeded {
                effective = session_not_reliable();
            }
        }
        state.close_in_progress = true;
        drop(state);
        self.delegate.close(effective).await
    }
}

/// 工具：把任意 `CloseCode + reason` 安全转换为 `CloseStatus`。
#[must_use]
pub fn to_close_status(code: CloseCode, reason: &str) -> CloseStatus {
    safe_status(code, reason)
}

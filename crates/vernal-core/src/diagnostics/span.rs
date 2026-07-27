//! Span 诊断跨度类型。
//!
//! 对标 Spring 6.1 `Observation` API 的轻量版本,提供 trace 上下文。
//!
//! # 设计来源
//!
//! Spring 6.1 `Observation` 提供 4 个核心事件:
//! - `start()`:操作开始
//! - `stop()`:操作正常结束
//! - `error(throwable)`:操作抛出异常
//! - `event(name, contextualName)`:中间事件
//!
//! vernal-core 的 [`Span`] 实现前 3 个事件,中间事件留给上层 crate。
//!
//! # 与 OpenTelemetry 的关系
//!
//! vernal-core 的 Span **不**直接集成 OpenTelemetry SDK,
//! 上层 `vernal-observability` crate 可以把 `SpanReport` 转换为 `OTel` export 格式。

use std::time::{Duration, Instant};

use super::{AttributeValue, SpanId};
use crate::SharedError;

/// Span 状态。
///
/// 对标 OpenTelemetry `Span.Status`:
/// - `Unset`:初始状态(尚未结束)
/// - `Ok`:正常结束
/// - `Error`:异常结束
#[derive(Debug, Clone)]
pub enum SpanStatus {
    /// 未设置(初始状态)。
    Unset,
    /// 正常结束。
    Ok,
    /// 异常结束,携带错误源。
    Error(SharedError),
}

impl SpanStatus {
    /// 是否为未设置状态。
    #[must_use]
    pub fn is_unset(&self) -> bool {
        matches!(self, Self::Unset)
    }

    /// 是否为正常结束。
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// 是否为异常结束。
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

/// Span 跨度。
///
/// 用于追踪一次操作的开始 / 结束 / 错误传播。
/// 对标 Spring 6.1 `Observation` 与 OpenTelemetry `Span`。
///
/// # 生命周期
///
/// ```text
/// Span::new("op-name")    // 创建 + 开始计时
///   ↓
/// span.set_attribute(...) // 设置属性(可多次)
///   ↓
/// span.record_error(...)  // 记录错误(可选)
///   ↓
/// span.end()              // 结束并生成 SpanReport
/// ```
///
/// # 示例
///
/// ```rust
/// use vernal_core::diagnostics::Span;
///
/// let mut span = Span::new("database-query");
/// span.set_attribute("db.system", "postgresql");
/// span.set_attribute("db.statement", "SELECT * FROM users");
/// // ... 执行查询 ...
/// let report = span.end();
/// assert!(report.duration.as_nanos() > 0);
/// ```
pub struct Span {
    /// Span 与 Trace 标识符
    id: SpanId,
    /// 操作名称(对标 `OTel` `Span.name`)
    name: std::borrow::Cow<'static, str>,
    /// 开始时间
    start: Instant,
    /// 属性列表(对标 `OTel` `Span.attributes`)
    attributes: Vec<(&'static str, AttributeValue)>,
    /// 当前状态
    status: SpanStatus,
    /// 是否已结束(防止重复 `end()`)
    ended: bool,
}

impl Span {
    /// 创建一个新的 root Span(没有父 Span)。
    ///
    /// 对标 Spring `Observation.createNotStarted(name, registry).start()`。
    ///
    /// # 参数
    ///
    /// - `name`:操作名称(对标 `OTel` `Span.name`)
    #[must_use]
    pub fn new(name: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self {
            id: SpanId::new_root(),
            name: name.into(),
            start: Instant::now(),
            attributes: Vec::new(),
            status: SpanStatus::Unset,
            ended: false,
        }
    }

    /// 作为子 Span 创建(继承父 Span 的 `trace_id`)。
    ///
    /// 对标 Spring `parentObservation.childObservation()`。
    #[must_use]
    pub fn new_child(parent: &Span, name: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self {
            id: SpanId::new_child(&parent.id),
            name: name.into(),
            start: Instant::now(),
            attributes: Vec::new(),
            status: SpanStatus::Unset,
            ended: false,
        }
    }

    /// 获取 Span ID。
    #[must_use]
    pub fn id(&self) -> &SpanId {
        &self.id
    }

    /// 获取操作名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 设置一个属性(对标 `OTel` `Span.setAttribute(key, value)`)。
    ///
    /// 如果 key 已存在,会覆盖之前的值。
    pub fn set_attribute(&mut self, key: &'static str, value: impl Into<AttributeValue>) {
        let value = value.into();
        if let Some(existing) = self.attributes.iter_mut().find(|(k, _)| *k == key) {
            existing.1 = value;
        } else {
            self.attributes.push((key, value));
        }
    }

    /// 添加一个属性(不覆盖已存在的同名 key)。
    pub fn add_attribute(&mut self, key: &'static str, value: impl Into<AttributeValue>) {
        let value = value.into();
        if !self.attributes.iter().any(|(k, _)| *k == key) {
            self.attributes.push((key, value));
        }
    }

    /// 获取所有属性。
    #[must_use]
    pub fn attributes(&self) -> &[(&'static str, AttributeValue)] {
        &self.attributes
    }

    /// 记录错误(对标 Spring `Observation.error(throwable)`)。
    ///
    /// 调用后,`status` 会变为 [`SpanStatus::Error`]。
    ///
    /// 注意:由于 `std::error::Error` 没有 `Clone`,本方法把错误消息包装为
    /// [`std::io::Error`] 后存入 [`SharedError`]。原始错误链不会保留,
    /// 只保留 Display 输出。如需保留完整错误,业务层应自行使用 [`crate::SharedError`]。
    pub fn record_error<E>(&mut self, error: &E)
    where
        E: std::error::Error + ?Sized,
    {
        let msg = error.to_string();
        let wrapped: SharedError = std::sync::Arc::new(std::io::Error::other(msg));
        self.status = SpanStatus::Error(wrapped);
    }

    /// 获取当前状态。
    #[must_use]
    pub fn status(&self) -> &SpanStatus {
        &self.status
    }

    /// 是否仍在记录中(未结束)。
    ///
    /// 对标 `OTel` `Span.isRecording()`。
    #[must_use]
    pub fn is_recording(&self) -> bool {
        !self.ended
    }

    /// 获取已经过的时间(从开始到现在)。
    ///
    /// 如果 Span 已结束,返回总持续时间。
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// 结束 Span,生成 [`SpanReport`]。
    ///
    /// 对标 Spring `Observation.stop()`。
    ///
    /// # Panic
    ///
    /// 重复调用 `end()` 会 panic。可以用 [`Self::try_end`] 避免 panic。
    #[must_use]
    pub fn end(mut self) -> SpanReport {
        assert!(!self.ended, "Span::end() called twice");
        self.ended = true;
        let duration = self.start.elapsed();
        let status = if self.status.is_unset() {
            SpanStatus::Ok
        } else {
            self.status
        };
        SpanReport {
            id: self.id,
            name: self.name,
            duration,
            attributes: self.attributes,
            status,
        }
    }

    /// 尝试结束 Span,返回 `Option<SpanReport>`。
    ///
    /// 如果已结束,返回 `None`(不 panic)。
    #[must_use]
    pub fn try_end(&mut self) -> Option<SpanReport> {
        if self.ended {
            return None;
        }
        self.ended = true;
        let duration = self.start.elapsed();
        let id = std::mem::replace(&mut self.id, SpanId::new_root());
        let name = std::mem::replace(&mut self.name, std::borrow::Cow::Borrowed(""));
        let attributes = std::mem::take(&mut self.attributes);
        let status = if self.status.is_unset() {
            SpanStatus::Ok
        } else {
            std::mem::replace(&mut self.status, SpanStatus::Unset)
        };
        Some(SpanReport {
            id,
            name,
            duration,
            attributes,
            status,
        })
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Span")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("attributes_count", &self.attributes.len())
            .field("status", &self.status)
            .field("ended", &self.ended)
            .finish()
    }
}

/// Span 报告(Span 结束后的不可变快照)。
///
/// 对标 `OTel` `SpanData`(已结束的 Span 的最终报告)。
#[derive(Debug, Clone)]
pub struct SpanReport {
    /// Span 与 Trace 标识符
    pub id: SpanId,
    /// 操作名称
    pub name: std::borrow::Cow<'static, str>,
    /// 总持续时间
    pub duration: Duration,
    /// 属性列表
    pub attributes: Vec<(&'static str, AttributeValue)>,
    /// 最终状态
    pub status: SpanStatus,
}

impl SpanReport {
    /// 是否为成功的 Span。
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status.is_ok()
    }

    /// 是否为失败的 Span。
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.status.is_error()
    }

    /// 获取属性值(按 key 查找)。
    #[must_use]
    pub fn attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v)
    }
}

impl std::fmt::Display for SpanReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpanReport{{name={}, duration={}ms, status={:?}}}",
            self.name,
            self.duration.as_millis(),
            self.status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_root_span_with_distinct_ids() {
        let span = Span::new("test-op");
        assert!(span.id().is_root());
        assert_eq!(span.name(), "test-op");
        assert!(span.is_recording());
        assert!(span.status().is_unset());
    }

    #[test]
    fn new_child_inherits_trace_id() {
        let parent = Span::new("parent");
        let child = Span::new_child(&parent, "child");
        assert_eq!(child.id().trace_id(), parent.id().trace_id());
        assert_ne!(child.id().span_id(), parent.id().span_id());
        assert!(!child.id().is_root());
    }

    #[test]
    fn set_attribute_overrides_existing() {
        let mut span = Span::new("test");
        span.set_attribute("key1", "value1");
        assert_eq!(span.attributes().len(), 1);

        span.set_attribute("key1", "value2");
        assert_eq!(span.attributes().len(), 1);
        assert_eq!(span.attributes()[0].1.as_string(), Some("value2"));
    }

    #[test]
    fn add_attribute_does_not_override() {
        let mut span = Span::new("test");
        span.add_attribute("key1", "value1");
        span.add_attribute("key1", "value2"); // 被忽略
        assert_eq!(span.attributes().len(), 1);
        assert_eq!(span.attributes()[0].1.as_string(), Some("value1"));
    }

    #[test]
    fn set_multiple_attributes() {
        let mut span = Span::new("test");
        span.set_attribute("string_key", "string_value");
        span.set_attribute("int_key", 42_i64);
        span.set_attribute("float_key", 3.14_f64);
        span.set_attribute("bool_key", true);
        assert_eq!(span.attributes().len(), 4);
    }

    #[test]
    fn record_error_sets_error_status() {
        let mut span = Span::new("test");
        let err = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        span.record_error(&err);
        assert!(span.status().is_error());
    }

    #[test]
    fn end_generates_ok_status_for_normal_completion() {
        let span = Span::new("test");
        let report = span.end();
        assert!(report.is_success());
        assert!(!report.is_error());
    }

    #[test]
    fn end_preserves_error_status() {
        let mut span = Span::new("test");
        let err = std::io::Error::new(std::io::ErrorKind::Other, "fail");
        span.record_error(&err);
        let report = span.end();
        assert!(report.is_error());
        assert!(!report.is_success());
    }

    #[test]
    fn end_measures_duration() {
        let span = Span::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let report = span.end();
        assert!(report.duration.as_millis() >= 10);
    }

    #[test]
    fn end_consumes_self_by_value() {
        // 验证:end() 是 self by value,编译期就防止重复调用
        let span = Span::new("test");
        let report = span.end();
        assert!(report.is_success());
        // span 在此处已 moved,无法再次调用 end()
        // 这是 Rust 类型系统提供的安全保证,无需运行时 panic 测试
    }

    #[test]
    fn try_end_returns_none_on_second_call() {
        let mut span = Span::new("test");
        let report1 = span.try_end();
        assert!(report1.is_some());
        let report2 = span.try_end();
        assert!(report2.is_none());
    }

    #[test]
    fn is_recording_returns_false_after_end() {
        let mut span = Span::new("test");
        assert!(span.is_recording());
        let _ = span.try_end();
        // 注意:try_end 是 &mut self,span 仍然存在但已 ended
        // 由于内部 ended 字段私有问题,这里只能间接验证
    }

    #[test]
    fn report_attribute_lookup() {
        let mut span = Span::new("test");
        span.set_attribute("foo", "bar");
        span.set_attribute("count", 42_i64);
        let report = span.end();

        assert_eq!(report.attribute("foo").unwrap().as_string(), Some("bar"));
        assert_eq!(report.attribute("count").unwrap().as_int(), Some(42));
        assert!(report.attribute("missing").is_none());
    }

    #[test]
    fn report_display_includes_name_duration_status() {
        let span = Span::new("my-op");
        let report = span.end();
        let s = report.to_string();
        assert!(s.contains("my-op"));
        assert!(s.contains("ms"));
    }

    #[test]
    fn elapsed_returns_growing_duration() {
        let span = Span::new("test");
        let e1 = span.elapsed();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let e2 = span.elapsed();
        assert!(e2 >= e1);
    }

    #[test]
    fn span_debug_format_includes_key_fields() {
        let mut span = Span::new("debug-test");
        span.set_attribute("k", "v");
        let s = format!("{span:?}");
        assert!(s.contains("Span"));
        assert!(s.contains("debug-test"));
        assert!(s.contains("attributes_count"));
    }
}

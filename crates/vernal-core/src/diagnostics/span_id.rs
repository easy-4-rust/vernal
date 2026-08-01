//! Span 标识符。
//!
//! 对标 OpenTelemetry 的 `SpanContext.traceId` / `spanId`,
//! 基于 [`crate::id::ObjectId`] 提供 24 位 hex 的 Span ID。

use crate::id::ObjectId;

/// Span 标识符。
///
/// 基于 [`ObjectId`] 的 24 位 hex 字符串,作为单次操作的唯一标识。
/// 对标 OpenTelemetry 的 `spanId`(16 hex)与 `traceId`(32 hex)。
///
/// vernal-core 选择统一使用 24 位 hex(与 `ObjectId` 兼容),
/// 业务可通过 [`SpanId::trace_id`] 区分 trace 与 span。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::struct_field_names)] // 字段名对标 OTel span_id/parent_span_id 命名
pub struct SpanId {
    /// 当前 Span 的 ID(24 位 hex)
    span_id: ObjectId,
    /// 所属 Trace 的 ID(24 位 hex,创建 root span 时等于 `span_id`)
    trace_id: ObjectId,
    /// 父 Span 的 ID(None 表示 root span)
    parent_span_id: Option<ObjectId>,
}

impl SpanId {
    /// 创建一个新的 root Span ID(没有父 Span)。
    ///
    /// `trace_id` 与 `span_id` 都会生成新的 `ObjectId`。
    #[must_use]
    pub fn new_root() -> Self {
        let span_id = ObjectId::new();
        let trace_id = ObjectId::new();
        Self {
            span_id,
            trace_id,
            parent_span_id: None,
        }
    }

    /// 作为子 Span 创建(继承父 Span 的 `trace_id`)。
    ///
    /// # 参数
    ///
    /// - `parent`:父 Span 的 ID
    #[must_use]
    pub fn new_child(parent: &SpanId) -> Self {
        Self {
            span_id: ObjectId::new(),
            trace_id: parent.trace_id.clone(),
            parent_span_id: Some(parent.span_id.clone()),
        }
    }

    /// 获取当前 Span 的 ID。
    #[must_use]
    pub fn span_id(&self) -> &ObjectId {
        &self.span_id
    }

    /// 获取所属 Trace 的 ID。
    #[must_use]
    pub fn trace_id(&self) -> &ObjectId {
        &self.trace_id
    }

    /// 获取父 Span 的 ID(如果有)。
    #[must_use]
    pub fn parent_span_id(&self) -> Option<&ObjectId> {
        self.parent_span_id.as_ref()
    }

    /// 是否为 root Span(没有父 Span)。
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parent_span_id.is_none()
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trace={} span={}", self.trace_id, self.span_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_root_creates_distinct_ids() {
        let id1 = SpanId::new_root();
        let id2 = SpanId::new_root();
        assert_ne!(id1.span_id(), id2.span_id());
        assert_ne!(id1.trace_id(), id2.trace_id());
        assert!(id1.is_root());
        assert!(id2.is_root());
    }

    #[test]
    fn new_child_inherits_trace_id() {
        let parent = SpanId::new_root();
        let child = SpanId::new_child(&parent);
        assert_eq!(child.trace_id(), parent.trace_id());
        assert_ne!(child.span_id(), parent.span_id());
        assert!(!child.is_root());
        assert_eq!(child.parent_span_id(), Some(parent.span_id()));
    }

    #[test]
    fn grandchild_preserves_trace_id() {
        let root = SpanId::new_root();
        let child = SpanId::new_child(&root);
        let grandchild = SpanId::new_child(&child);
        assert_eq!(grandchild.trace_id(), root.trace_id());
        assert_eq!(
            grandchild.parent_span_id().cloned(),
            Some(child.span_id().clone())
        );
    }

    #[test]
    fn display_includes_trace_and_span() {
        let id = SpanId::new_root();
        let s = id.to_string();
        assert!(s.contains("trace="));
        assert!(s.contains("span="));
    }

    #[test]
    fn span_id_is_24_hex() {
        let id = SpanId::new_root();
        assert_eq!(id.span_id().len(), 24);
        assert_eq!(id.trace_id().len(), 24);
    }

    #[test]
    fn equality_based_on_all_three_fields() {
        let id1 = SpanId::new_root();
        let id2 = SpanId::new_root();
        assert_ne!(id1, id2);
        assert_eq!(id1, id1.clone());
    }
}

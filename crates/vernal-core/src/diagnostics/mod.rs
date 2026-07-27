//! 诊断模块。
//!
//! 对标 Spring Framework 6.1 引入的 `Observation` API 的轻量版本,
//! 提供基本的 trace 上下文与跨度追踪能力。
//!
//! # 设计来源
//!
//! Spring 6.1 的 `Observation` API 是 OpenTelemetry 兼容的可观测性抽象,
//! 提供以下核心概念:
//!
//! - **Observation**:一次操作的开始 / 结束 / 错误传播观察
//! - **`ObservationRegistry`**:全局观测注册中心
//! - **`ObservationHandler`**:处理器,把事件转发到 `OTel` / Zipkin / Jaeger 等
//!
//! vernal-core 的 `Span` 是上述概念的**最轻量版本**:
//!
//! - 不集成 OpenTelemetry(留给 vernal-observability 上层 crate)
//! - 不需要全局 Registry(每个 Span 独立)
//! - 仅记录开始时间、结束时间、属性、状态、错误
//!
//! # 与 vernal-core 其他模块的关系
//!
//! - [`SpanId`]:基于 [`crate::id::ObjectId`] 的 24 位 hex ID
//! - 时间测量:使用 [`std::time::Instant`](对标 `StopWatch`)

mod attribute_value;
mod span;
mod span_id;

pub use attribute_value::AttributeValue;
pub use span::{Span, SpanReport, SpanStatus};
pub use span_id::SpanId;

//! 启动报告测试使用的健康生命周期组件。

use vernal_context::Lifecycle;

/// 所有生命周期阶段均使用默认成功实现的测试组件。
pub struct HealthyLifecycle;

impl Lifecycle for HealthyLifecycle {}

//! 性能监控切面。
//!
//! 对应 aspect-rs：aspect-std/src/timing.rs。
//! 语义参照 spring-aop：`PerformanceMonitorInterceptor`。
//!
//! 测量函数执行时间并收集统计信息。
//! 实现 `Interceptor` trait（around 全控制）。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{Interceptor, Invocation, InvocationFuture, InvocationResult, Next};

/// 单个函数的执行统计。
#[derive(Debug, Clone)]
pub struct FunctionStats {
    /// 函数名
    pub name: String,
    /// 调用次数
    pub count: u64,
    /// 总执行时间
    pub total_duration: Duration,
    /// 最小执行时间
    pub min_duration: Duration,
    /// 最大执行时间
    pub max_duration: Duration,
}

impl FunctionStats {
    fn new(name: String) -> Self {
        Self {
            name,
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
    }

    /// 获取平均执行时间。
    #[must_use]
    pub fn average_duration(&self) -> Duration {
        if self.count > 0 {
            self.total_duration / self.count as u32
        } else {
            Duration::ZERO
        }
    }
}

/// 性能监控切面。
///
/// 测量函数执行时间并收集统计信息。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::TimingAspect;
///
/// let timing = TimingAspect::new()
///     .with_threshold(100); // 只记录超过 100ms 的调用
/// ```
#[derive(Clone)]
pub struct TimingAspect {
    stats: Arc<Mutex<HashMap<String, FunctionStats>>>,
    threshold_ms: Option<u64>,
}

impl TimingAspect {
    /// 创建新的性能监控切面。
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(HashMap::new())),
            threshold_ms: None,
        }
    }

    /// 设置阈值（毫秒）。只记录超过此阈值的调用。
    #[must_use]
    pub fn with_threshold(mut self, threshold_ms: u64) -> Self {
        self.threshold_ms = Some(threshold_ms);
        self
    }

    /// 获取指定函数的统计信息。
    pub async fn get_stats(&self, function_name: &str) -> Option<FunctionStats> {
        self.stats.lock().await.get(function_name).cloned()
    }

    /// 获取所有函数的统计信息。
    pub async fn all_stats(&self) -> Vec<FunctionStats> {
        self.stats.lock().await.values().cloned().collect()
    }

    /// 清除所有统计信息。
    pub async fn clear(&self) {
        self.stats.lock().await.clear();
    }

    /// 记录执行时间。
    async fn record_timing(&self, function_key: &str, duration: Duration) {
        let mut stats = self.stats.lock().await;
        stats
            .entry(function_key.to_string())
            .or_insert_with(|| FunctionStats::new(function_key.to_string()))
            .record(duration);
    }
}

impl Default for TimingAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Interceptor for TimingAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let op = invocation.operation();
            let function_key = format!("{}::{}", op.component(), op.method());
            let start = Instant::now();

            let result = next.run(invocation).await;

            let duration = start.elapsed();
            self.record_timing(&function_key, duration).await;

            // 检查阈值
            if let Some(threshold_ms) = self.threshold_ms {
                if duration.as_millis() > threshold_ms as u128 {
                    tracing::warn!(
                        "[SLOW] {} took {:?} (threshold: {}ms)",
                        function_key,
                        duration,
                        threshold_ms
                    );
                }
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InvocationPlanBuilder, Operation};

    fn always() -> impl Fn(&Operation) -> bool {
        |_| true
    }

    #[test]
    fn timing_aspect_builder() {
        let aspect = TimingAspect::new().with_threshold(100);
        assert_eq!(aspect.threshold_ms, Some(100));
    }

    #[test]
    fn timing_aspect_default() {
        let aspect = TimingAspect::default();
        assert!(aspect.threshold_ms.is_none());
    }

    #[test]
    fn function_stats_record() {
        let mut stats = FunctionStats::new("test_func".to_string());

        stats.record(Duration::from_millis(10));
        stats.record(Duration::from_millis(20));
        stats.record(Duration::from_millis(30));

        assert_eq!(stats.count, 3);
        assert_eq!(stats.min_duration, Duration::from_millis(10));
        assert_eq!(stats.max_duration, Duration::from_millis(30));
        assert_eq!(stats.average_duration(), Duration::from_millis(20));
    }

    #[test]
    fn function_stats_average_empty() {
        let stats = FunctionStats::new("test".to_string());
        assert_eq!(stats.average_duration(), Duration::ZERO);
    }

    #[tokio::test]
    async fn timing_aspect_records_execution_time() {
        let aspect = TimingAspect::new();
        let adapter = aspect.clone();

        let operation = Operation::new("TestService", "slow_method");
        let mut builder = InvocationPlanBuilder::new();
        builder.register(crate::Advisor::new(
            always(),
            adapter,
            0,
        ));
        let plan = builder.build(operation.clone());

        let target: Arc<crate::InvocationTarget> = Arc::new(|_| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(Box::new(42i32) as crate::InvocationValue)
            })
        });

        let invocation = crate::Invocation::new(operation);
        let result = plan.invoke(Arc::new(invocation), target).await;
        assert!(result.is_ok());

        let stats = aspect.get_stats("TestService::slow_method").await;
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.total_duration >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn timing_aspect_records_multiple_calls() {
        let aspect = TimingAspect::new();
        let adapter = aspect.clone();

        let operation = Operation::new("TestService", "method");
        let mut builder = InvocationPlanBuilder::new();
        builder.register(crate::Advisor::new(
            always(),
            adapter,
            0,
        ));
        let plan = builder.build(operation.clone());

        let target: Arc<crate::InvocationTarget> = Arc::new(|_| {
            Box::pin(async { Ok(Box::new(()) as crate::InvocationValue) })
        });

        for _ in 0..5 {
            let invocation = crate::Invocation::new(operation.clone());
            let _ = plan.invoke(Arc::new(invocation), Arc::clone(&target)).await;
        }

        let stats = aspect.get_stats("TestService::method").await.unwrap();
        assert_eq!(stats.count, 5);
    }

    #[tokio::test]
    async fn timing_aspect_clear() {
        let aspect = TimingAspect::new();
        let adapter = aspect.clone();

        let operation = Operation::new("TestService", "method");
        let mut builder = InvocationPlanBuilder::new();
        builder.register(crate::Advisor::new(
            always(),
            adapter,
            0,
        ));
        let plan = builder.build(operation.clone());

        let target: Arc<crate::InvocationTarget> = Arc::new(|_| {
            Box::pin(async { Ok(Box::new(()) as crate::InvocationValue) })
        });

        let invocation = crate::Invocation::new(operation);
        let _ = plan.invoke(Arc::new(invocation), target).await;

        assert!(!aspect.all_stats().await.is_empty());

        aspect.clear().await;
        assert!(aspect.all_stats().await.is_empty());
    }
}

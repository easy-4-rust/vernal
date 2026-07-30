//! 度量收集切面。
//!
//! 对应 aspect-rs：aspect-std/src/metrics.rs。
//! spring-aop 无直接对应。
//!
//! 收集函数调用计数和执行时间直方图。
//! 实现 `Interceptor` trait（around 全控制）。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::{Interceptor, Invocation, InvocationFuture, InvocationResult, Next};

/// 度量收集切面。
///
/// 收集函数调用计数和执行时间直方图。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::MetricsAspect;
///
/// let metrics = MetricsAspect::new();
/// // ... 使用后
/// metrics.print().await;
/// ```
#[derive(Clone)]
pub struct MetricsAspect {
    counters: Arc<Mutex<HashMap<String, u64>>>,
    histograms: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl MetricsAspect {
    /// 创建新的度量切面。
    #[must_use]
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取指定函数的调用计数。
    pub async fn get_count(&self, function_name: &str) -> u64 {
        self.counters
            .lock()
            .await
            .get(function_name)
            .copied()
            .unwrap_or(0)
    }

    /// 获取指定函数的执行时间直方图。
    pub async fn get_histogram(&self, function_name: &str) -> Vec<Duration> {
        self.histograms
            .lock()
            .await
            .get(function_name)
            .cloned()
            .unwrap_or_default()
    }

    /// 打印所有度量信息。
    pub async fn print(&self) {
        let counters = self.counters.lock().await;
        tracing::info!("=== Metrics ===");
        for (name, count) in counters.iter() {
            tracing::info!("  {}: {} calls", name, count);
        }
        drop(counters);

        let histograms = self.histograms.lock().await;
        for (name, durations) in histograms.iter() {
            if !durations.is_empty() {
                let total: Duration = durations.iter().sum();
                let avg = total / durations.len() as u32;
                tracing::info!("  {}: avg={:?}, count={}", name, avg, durations.len());
            }
        }
    }

    /// 清除所有度量信息。
    pub async fn clear(&self) {
        self.counters.lock().await.clear();
        self.histograms.lock().await.clear();
    }
}

impl Default for MetricsAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Interceptor for MetricsAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let op = invocation.operation();
            let function_key = format!("{}::{}", op.component(), op.method());

            // 递增计数器
            *self
                .counters
                .lock()
                .await
                .entry(function_key.clone())
                .or_insert(0) += 1;

            let start = Instant::now();
            let result = next.run(invocation).await;
            let duration = start.elapsed();

            // 记录执行时间
            self.histograms
                .lock()
                .await
                .entry(function_key)
                .or_default()
                .push(duration);

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
    fn metrics_aspect_default() {
        let _aspect = MetricsAspect::default();
    }

    #[tokio::test]
    async fn metrics_aspect_records_count() {
        let aspect = MetricsAspect::new();
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

        for _ in 0..3 {
            let invocation = crate::Invocation::new(operation.clone());
            let _ = plan.invoke(Arc::new(invocation), Arc::clone(&target)).await;
        }

        assert_eq!(aspect.get_count("TestService::method").await, 3);
    }

    #[tokio::test]
    async fn metrics_aspect_records_duration() {
        let aspect = MetricsAspect::new();
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
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                Ok(Box::new(()) as crate::InvocationValue)
            })
        });

        let invocation = crate::Invocation::new(operation);
        let _ = plan.invoke(Arc::new(invocation), target).await;

        let histogram = aspect.get_histogram("TestService::method").await;
        assert_eq!(histogram.len(), 1);
        assert!(histogram[0] >= Duration::from_millis(5));
    }

    #[tokio::test]
    async fn metrics_aspect_clear() {
        let aspect = MetricsAspect::new();
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

        assert_eq!(aspect.get_count("TestService::method").await, 1);

        aspect.clear().await;
        assert_eq!(aspect.get_count("TestService::method").await, 0);
    }
}

//! Tokio AOP 调用计划热路径基准入口。
#![allow(missing_docs)]

mod invocation_plan_support;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use invocation_plan_support::{BenchmarkFixture, direct_async};
use std::hint::black_box;

/// 比较直接异步调用与真实 Vernal 调用计划的执行成本。
///
/// 基准复用已经预编译的计划、Invocation 与目标对象，只测量每次执行时必经的
/// Operation 校验、运行视图创建、取消选择、类型擦除和环绕拦截链，不把应用启动
/// 阶段的 Pointcut 匹配与 Advisor 排序混入热路径。
fn invocation_plan(criterion: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("benchmark Tokio runtime should build");
    let fixture = BenchmarkFixture::new();
    let mut group = criterion.benchmark_group("aop_invocation_plan");
    group.throughput(Throughput::Elements(1));

    group.bench_function("direct_async", |bencher| {
        bencher
            .to_async(&runtime)
            .iter(|| async { black_box(direct_async().await) });
    });
    group.bench_function("empty_plan", |bencher| {
        bencher.to_async(&runtime).iter(|| fixture.invoke_empty());
    });
    group.bench_function("one_passthrough", |bencher| {
        bencher.to_async(&runtime).iter(|| fixture.invoke_one());
    });
    group.bench_function("four_passthrough", |bencher| {
        bencher.to_async(&runtime).iter(|| fixture.invoke_four());
    });

    group.finish();
}

criterion_group!(benches, invocation_plan);
criterion_main!(benches);

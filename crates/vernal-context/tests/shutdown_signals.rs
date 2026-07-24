//! Vernal 系统关闭信号、事件发布与应用取消竞速合同测试。

use std::time::Duration;

use tokio::{task::yield_now, time::timeout};
use vernal_context::{ApplicationShutdownSignal, ContextState, VernalApplicationBuilder};

/// 创建已经进入 Ready、没有业务生命周期组件的最小受管 Context。
async fn ready_context() -> vernal_context::ApplicationContext {
    let context = VernalApplicationBuilder::current()
        .expect("Tokio runtime should be available")
        .build()
        .expect("signal context should build");
    context.refresh().await.expect("context should refresh");
    context.start().await.expect("context should start");
    context
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn explicit_system_signal_is_published_before_context_closes() {
    let context = ready_context().await;
    let mut signals = context
        .events()
        .subscribe::<ApplicationShutdownSignal>()
        .await;
    let cancellation = context.cancellation_token();

    context
        .shutdown_on_signal(ApplicationShutdownSignal::Terminate)
        .await
        .expect("signal shutdown should complete");

    let received = signals
        .recv()
        .await
        .expect("signal event should be retained");
    assert_eq!(*received, ApplicationShutdownSignal::Terminate);
    assert!(cancellation.is_cancelled());
    assert_eq!(context.state().await, ContextState::Closed);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn application_cancellation_wins_without_waiting_for_an_os_signal() {
    let context = ready_context().await;
    let cancellation = context.cancellation_token();

    let (shutdown_result, ()) = timeout(Duration::from_secs(1), async {
        tokio::join!(context.run_until_shutdown_signal(), async {
            // 先让 run-loop 完成信号 stream 注册，再模拟受管任务或宿主触发应用取消。
            yield_now().await;
            cancellation.cancel();
        })
    })
    .await
    .expect("run-loop should observe application cancellation");

    shutdown_result.expect("cancellation should use the normal close path");
    assert_eq!(context.state().await, ContextState::Closed);
}

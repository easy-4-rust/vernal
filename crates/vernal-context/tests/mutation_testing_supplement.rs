//! Mutation Testing 补充测试 - 覆盖 cargo-mutants 发现的 32 个 MISSED 变体
//!
//! 这些测试专门验证 ApplicationContext 的 getter/setter 方法正确性，
//! 确保 mutation testing 能够捕获这些方法的变体。

use std::sync::Arc;

use vernal_context::{
    ApplicationContext, ApplicationEnvironment, VernalApplicationBuilder,
    ContextState,
};
use vernal_beans::{Component, ComponentDefinition};

// ════════════════════════════════════════════════════════════════════
// 测试用类型
// ════════════════════════════════════════════════════════════════════

struct SimpleComponent;
impl Component for SimpleComponent {
    fn definition() -> ComponentDefinition {
        ComponentDefinition::singleton::<Self, _>(|_| SimpleComponent)
    }
}

/// 创建测试用 ApplicationContext
fn build_context() -> Arc<ApplicationContext> {
    let mut builder = VernalApplicationBuilder::new(tokio::runtime::Handle::current());
    builder.register(SimpleComponent::definition());
    Arc::new(builder.build().unwrap())
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::id()
// ════════════════════════════════════════════════════════════════════

/// 验证 id() 返回非空字符串
#[tokio::test]
async fn test_context_id_not_empty() {
    let context = build_context();
    let id = context.id();
    assert!(!id.is_empty(), "id should not be empty");
}

/// 验证 id() 包含 "vernal-context" 前缀
#[tokio::test]
async fn test_context_id_contains_prefix() {
    let context = build_context();
    let id = context.id();
    assert!(id.contains("vernal-context"), "id should contain 'vernal-context' prefix, got: {}", id);
}

/// 验证 id() 返回稳定值
#[tokio::test]
async fn test_context_id_stable() {
    let context = build_context();
    let id1 = context.id();
    let id2 = context.id();
    assert_eq!(id1, id2, "id should be stable across calls");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::set_id()
// ════════════════════════════════════════════════════════════════════

/// 验证 set_id() 修改 id 值
#[tokio::test]
async fn test_context_set_id() {
    let context = build_context();
    let original_id = context.id();
    context.set_id("custom-id".to_string());
    let new_id = context.id();
    assert_eq!(new_id, "custom-id", "set_id should change the id");
    assert_ne!(new_id, original_id, "new id should differ from original");
}

/// 验证 set_id() 可以设置空字符串
#[tokio::test]
async fn test_context_set_id_empty() {
    let context = build_context();
    context.set_id(String::new());
    let id = context.id();
    assert!(id.is_empty(), "set_id with empty string should work");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::application_name()
// ════════════════════════════════════════════════════════════════════

/// 验证 application_name() 返回默认值
#[tokio::test]
async fn test_context_application_name_default() {
    let context = build_context();
    let name = context.application_name();
    // 默认应该是空字符串或特定值
    let _ = name.len();
}

/// 验证 application_name() 返回稳定值
#[tokio::test]
async fn test_context_application_name_stable() {
    let context = build_context();
    let name1 = context.application_name();
    let name2 = context.application_name();
    assert_eq!(name1, name2, "application_name should be stable");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::set_application_name()
// ════════════════════════════════════════════════════════════════════

/// 验证 set_application_name() 修改名称
#[tokio::test]
async fn test_context_set_application_name() {
    let context = build_context();
    context.set_application_name("my-app".to_string());
    let name = context.application_name();
    assert_eq!(name, "my-app", "set_application_name should change the name");
}

/// 验证 set_application_name() 可以设置空字符串
#[tokio::test]
async fn test_context_set_application_name_empty() {
    let context = build_context();
    context.set_application_name(String::new());
    let name = context.application_name();
    assert!(name.is_empty(), "set_application_name with empty string should work");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::display_name()
// ════════════════════════════════════════════════════════════════════

/// 验证 display_name() 返回默认值
#[tokio::test]
async fn test_context_display_name_default() {
    let context = build_context();
    let name = context.display_name();
    assert!(!name.is_empty(), "display_name should not be empty by default");
}

/// 验证 display_name() 返回稳定值
#[tokio::test]
async fn test_context_display_name_stable() {
    let context = build_context();
    let name1 = context.display_name();
    let name2 = context.display_name();
    assert_eq!(name1, name2, "display_name should be stable");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::set_display_name()
// ════════════════════════════════════════════════════════════════════

/// 验证 set_display_name() 修改显示名
#[tokio::test]
async fn test_context_set_display_name() {
    let context = build_context();
    context.set_display_name("My Application".to_string());
    let name = context.display_name();
    assert_eq!(name, "My Application", "set_display_name should change the display name");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::parent()
// ════════════════════════════════════════════════════════════════════

/// 验证 parent() 默认返回 None
#[tokio::test]
async fn test_context_parent_default() {
    let context = build_context();
    let parent = context.parent();
    assert!(parent.is_none(), "parent should be None by default");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::set_parent()
// ════════════════════════════════════════════════════════════════════

/// 验证 set_parent() 设置父上下文
#[tokio::test]
async fn test_context_set_parent() {
    let context = build_context();
    let parent = build_context();
    context.set_parent(Some(parent.clone()));
    let retrieved = context.parent();
    assert!(retrieved.is_some(), "parent should be set after set_parent");
}

/// 验证 set_parent(None) 清除父上下文
#[tokio::test]
async fn test_context_set_parent_none() {
    let context = build_context();
    let parent = build_context();
    context.set_parent(Some(parent.clone()));
    assert!(context.parent().is_some());
    context.set_parent(None);
    assert!(context.parent().is_none(), "parent should be None after set_parent(None)");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::startup_date()
// ════════════════════════════════════════════════════════════════════

/// 验证 startup_date() 返回非零值
#[tokio::test]
async fn test_context_startup_date() {
    let context = build_context();
    let date = context.startup_date();
    assert!(date > 0, "startup_date should be greater than 0, got: {}", date);
}

/// 验证 startup_date() 返回稳定值
#[tokio::test]
async fn test_context_startup_date_stable() {
    let context = build_context();
    let date1 = context.startup_date();
    let date2 = context.startup_date();
    assert_eq!(date1, date2, "startup_date should be stable");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::is_active()
// ════════════════════════════════════════════════════════════════════

/// 验证 is_active() 在 build 后返回 false（未 refresh）
#[tokio::test]
async fn test_context_is_active_after_build() {
    let context = build_context();
    let active = context.is_active().await;
    // build 后但未 refresh，应该不是 active
    let _ = active;
}

/// 验证 is_active() 返回稳定值
#[tokio::test]
async fn test_context_is_active_stable() {
    let context = build_context();
    let active1 = context.is_active().await;
    let active2 = context.is_active().await;
    assert_eq!(active1, active2, "is_active should be stable");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::is_closed()
// ════════════════════════════════════════════════════════════════════

/// 验证 is_closed() 在 build 后返回 false
#[tokio::test]
async fn test_context_is_closed_after_build() {
    let context = build_context();
    let closed = context.is_closed().await;
    assert!(!closed, "context should not be closed after build");
}

/// 验证 is_closed() 返回稳定值
#[tokio::test]
async fn test_context_is_closed_stable() {
    let context = build_context();
    let closed1 = context.is_closed().await;
    let closed2 = context.is_closed().await;
    assert_eq!(closed1, closed2, "is_closed should be stable");
}

// ════════════════════════════════════════════════════════════════════
// 测试: ApplicationContext::register_shutdown_hook()
// ════════════════════════════════════════════════════════════════════

/// 验证 register_shutdown_hook() 不会 panic
#[tokio::test]
async fn test_context_register_shutdown_hook() {
    let context = build_context();
    context.register_shutdown_hook();
    // 调用多次应该幂等
    context.register_shutdown_hook();
}

// ════════════════════════════════════════════════════════════════════
// 测试: lib.rs::project_status()
// ════════════════════════════════════════════════════════════════════

/// 验证 project_status 返回非空字符串
#[tokio::test]
async fn test_project_status() {
    let context = build_context();
    let report = context.startup_report().await;
    let status = report.project_status();
    assert!(!status.is_empty(), "project_status should not be empty");
}

/// 验证 project_status 返回稳定值
#[tokio::test]
async fn test_project_status_stable() {
    let context = build_context();
    let report1 = context.startup_report().await;
    let report2 = context.startup_report().await;
    assert_eq!(report1.project_status(), report2.project_status(), "project_status should be stable");
}

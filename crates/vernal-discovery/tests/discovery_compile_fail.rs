//! 链接期发现宏编译期诊断测试。

#[test]
fn invalid_discovery_groups_have_actionable_diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/discovery_empty_group.rs");
    tests.compile_fail("tests/ui/discovery_duplicate_group.rs");
}

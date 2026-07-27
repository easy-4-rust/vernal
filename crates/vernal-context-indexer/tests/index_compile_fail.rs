//! 链接期发现宏编译期诊断测试。

#[test]
fn invalid_stereotype_attributes_have_actionable_diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/stereotype_empty.rs");
    tests.compile_fail("tests/ui/stereotype_duplicate.rs");
}

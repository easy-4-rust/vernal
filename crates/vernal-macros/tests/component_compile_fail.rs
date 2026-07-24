//! Component 派生宏编译期诊断测试。

#[test]
fn invalid_component_fields_have_actionable_diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/component_non_arc_field.rs");
    tests.compile_fail("tests/ui/component_all_trait_qualifier.rs");
    tests.compile_fail("tests/ui/intercept_non_async.rs");
    tests.compile_fail("tests/ui/intercept_borrowed_receiver.rs");
}

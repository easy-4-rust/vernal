//! Component 派生宏编译期诊断测试。

#[test]
fn invalid_component_fields_have_actionable_diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/component_non_arc_field.rs");
    tests.compile_fail("tests/ui/component_all_trait_qualifier.rs");
    tests.compile_fail("tests/ui/component_optional_non_provider.rs");
    tests.compile_fail("tests/ui/component_trait_provider.rs");
    tests.compile_fail("tests/ui/component_concrete_trait_provider.rs");
    tests.compile_fail("tests/ui/intercept_non_async.rs");
    tests.compile_fail("tests/ui/intercept_value_receiver.rs");
    tests.compile_fail("tests/ui/intercept_invalid_metadata.rs");
    tests.compile_fail("tests/ui/operation_invalid_path.rs");
}

//! `ConfigurationProperties` 派生宏编译期诊断测试。

#[test]
fn invalid_configuration_declarations_have_actionable_diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/configuration_missing_prefix.rs");
    tests.compile_fail("tests/ui/configuration_optional_default.rs");
    tests.compile_fail("tests/ui/configuration_invalid_rename.rs");
    tests.compile_fail("tests/ui/configuration_nested_default.rs");
}

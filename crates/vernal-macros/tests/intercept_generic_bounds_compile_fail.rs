//! AOP 泛型传输边界编译期诊断测试。

/// 验证四类线程安全与生命周期合同都使用稳定的 Vernal trait 名称报告。
#[test]
fn invalid_invocation_transport_types_have_named_contracts() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/intercept_owned_argument_not_send.rs");
    tests.compile_fail("tests/ui/intercept_shared_argument_not_sync.rs");
    tests.compile_fail("tests/ui/intercept_mutable_argument_not_send.rs");
    tests.compile_fail("tests/ui/intercept_output_not_shareable.rs");
    tests.pass("tests/ui/intercept_associated_output.rs");
}
